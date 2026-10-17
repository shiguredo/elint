-module(ok).

-export([assigned/1, operator/1, container/1, target/1, no_call/1]).


assigned(Item) ->
    Length = length(Item),
    first(Length).


operator(Item) ->
    first(1 + length(Item)).


container(Item) ->
    first([length(Item)]).


target(Item) ->
    (first())(Item).


no_call(Item) ->
    first(Item).
