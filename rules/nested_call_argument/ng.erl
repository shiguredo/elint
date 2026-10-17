-module(ng).

-export([local/1, remote/1, later/1, nested/1]).


local(Tuple) ->
    first(element(1, Tuple)).


remote(Item) ->
    first(erlang:element(1, Item)).


later(Item) ->
    combine(Item, length(Item)).


nested(Item) ->
    outer(inner(Item)).
