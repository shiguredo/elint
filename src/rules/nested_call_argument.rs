//! `nested_call_argument` lint: flag a call passed directly as a call argument.

use super::Rule;
use crate::{BranchContext, Context, Finding};

/// Lint rule that flags a function call used directly as an argument of
/// another function call.
pub const RULE: Rule = Rule::new(
    "nested_call_argument",
    include_str!("../../rules/nested_call_argument/rule.md"),
    check,
);

fn check(_ctx: &Context, branch: &BranchContext) -> Vec<Finding> {
    let mut errors = Vec::new();
    for node in branch.tree.nodes() {
        check_node(branch, node, &mut errors);
    }
    errors
}

fn check_node(branch: &BranchContext, node: erl_parse::NodeView<'_>, errors: &mut Vec<Finding>) {
    if node.kind() != erl_parse::SyntaxKind::CallExpr || is_macro_invocation(branch, node) {
        return;
    }
    let Some(args) = node
        .children()
        .find(|c| c.kind() == erl_parse::SyntaxKind::ArgumentList)
    else {
        return;
    };
    for arg in args.children() {
        if arg.kind() != erl_parse::SyntaxKind::CallExpr || is_macro_invocation(branch, arg) {
            continue;
        }
        if let Some(span) = branch.span_of_range(arg.token_range()) {
            errors.push(Finding {
                span,
                node: arg.node_id(),
            });
        }
    }
}

/// Returns whether `call` is the synthetic call the preprocessor stands in
/// for an unknown macro invocation `?NAME(...)`.
///
/// `erl_pp` replaces an unknown macro name with a single synthesized atom, so
/// the following `(...)` turns into a call whose callee is that atom. Such a
/// callee is an [`erl_pp::Origin::CallerExpansion`] token, unlike the callee of
/// a call written in the source (including one inside a macro body).
fn is_macro_invocation(branch: &BranchContext, call: erl_parse::NodeView<'_>) -> bool {
    let Some(callee) = call.children().next() else {
        return false;
    };
    if callee.kind() != erl_parse::SyntaxKind::AtomExpr {
        return false;
    }
    callee.indexed_tokens().next().is_some_and(|(i, _)| {
        branch
            .source_tokens
            .get(i.get())
            .is_some_and(|token| matches!(token.origin(), erl_pp::Origin::CallerExpansion { .. }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_support::findings;

    #[test]
    fn flags_local_call_argument() {
        let src = "-module(t).\nfoo(A) -> f(g(A)).\n";
        assert_eq!(findings(check, src), ["g(A)"]);
    }

    #[test]
    fn flags_remote_call_argument() {
        let src = "-module(t).\nfoo(A) -> f(lists:reverse(A)).\n";
        assert_eq!(findings(check, src), ["lists:reverse(A)"]);
    }

    #[test]
    fn flags_call_in_later_argument() {
        let src = "-module(t).\nfoo(A) -> f(A, g(A)).\n";
        assert_eq!(findings(check, src), ["g(A)"]);
    }

    #[test]
    fn flags_every_nested_argument_call() {
        let src = "-module(t).\nfoo(A) -> f(g(h(A))).\n";
        assert_eq!(findings(check, src), ["g(h(A))", "h(A)"]);
    }

    #[test]
    fn ignores_call_in_list() {
        let src = "-module(t).\nfoo(A) -> f([g(A)]).\n";
        assert!(findings(check, src).is_empty());
    }

    #[test]
    fn ignores_call_in_tuple() {
        let src = "-module(t).\nfoo(A) -> f({g(A)}).\n";
        assert!(findings(check, src).is_empty());
    }

    #[test]
    fn ignores_operator_argument() {
        let src = "-module(t).\nfoo(A) -> f(1 + g(A)).\n";
        assert!(findings(check, src).is_empty());
    }

    #[test]
    fn ignores_call_used_as_target() {
        let src = "-module(t).\nfoo(A) -> (f())(g(A)).\n";
        // `g(A)` is the argument of the call `(f())(g(A))`, so it is
        // reported; the inner `f()` is the target, not an argument, and is
        // not reported.
        assert_eq!(findings(check, src), ["g(A)"]);
    }

    #[test]
    fn ignores_unknown_macro_invocation_argument() {
        let src = "-module(t).\nfoo(A) -> ?UNKNOWN(g(A)).\n";
        assert!(findings(check, src).is_empty());
    }

    #[test]
    fn flags_nested_call_in_macro_body() {
        let src = "\
-module(t).
-define(M(A), g(h(A))).
foo(A) -> ?M(A).
";
        // The call inside the macro body maps to the macro call site.
        assert_eq!(findings(check, src), ["?M(A)"]);
    }

    #[test]
    fn ng_fixture_has_four_findings() {
        let src = include_str!("../../rules/nested_call_argument/ng.erl");
        assert_eq!(findings(check, src).len(), 4);
    }

    #[test]
    fn ok_fixture_has_no_findings() {
        let src = include_str!("../../rules/nested_call_argument/ok.erl");
        assert!(findings(check, src).is_empty());
    }
}
