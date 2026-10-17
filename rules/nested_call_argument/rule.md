# `nested_call_argument`

Disallow a function call passed directly as an argument of another call.

## What it does

This rule reports a function call that appears directly as an argument of
another function call. It reports `g(A)` in `f(g(A))` and in `f(A, g(A))`, and
it reports every argument-position call in a nesting such as `f(g(h(A)))`.

It does not report:

- a call in any other position, such as the subject of a `case`, the
  right-hand side of `=`, an element of a list / tuple / map / binary, or an
  operand of an operator
- a call used as the target of another call (`(f())(A)`)
- an unknown macro invocation `?M(...)`, for which the preprocessor stands in
  a synthetic call

## Why restrict this?

Passing a call result straight into another call hides the intermediate value
inside a nested expression, so a reader cannot name it or check it at the call
site. Assigning the result to a variable first makes the data flow explicit.

## Example

```erlang
-module(example).

average(Values) ->
    divide(sum(Values), count(Values)).
```

Use instead:

```erlang
-module(example).

average(Values) ->
    Total = sum(Values),
    Length = count(Values),
    divide(Total, Length).
```

## Known limitations

The rule does not judge whether an intermediate value deserves a name. A short,
obviously pure call such as `length(List)` is still reported when it is passed
straight to another call. Suppress such a use with an `-elint_expect` attribute
(see `elint --explain elint_expect_attr`) when the nested form is clearer.

A macro body is linted through each of its call sites, so a nested call inside
a `-define` body is reported once per expansion.
