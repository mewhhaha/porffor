# `Array.prototype.pop` algorithm ownership

## Semantic boundary

ECMA-262 defines
[`Array.prototype.pop`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.pop)
as one generic sequence of observable operations:

1. apply `ToObject` to the receiver;
2. obtain one `LengthOfArrayLike` snapshot;
3. for a non-empty receiver, `Get` the last property and then perform
   `DeletePropertyOrThrow` on it;
4. strictly `Set` `length` to the new length, including setting `+0` on an
   already-empty receiver; and
5. return the saved element, or `undefined` for the empty case.

The delete must happen before the strict length write. Consequently, a
non-configurable last property throws without changing the old length, while a
writable last property paired with a non-writable `length` is deleted before
the later TypeError. The empty case must still attempt the strict write, so an
empty object with a non-writable `+0` length throws.

## Single compiler owner

`StandardBuiltinId::ArrayPrototypePop` in
`crates/lila-aot-wasm/src/builtins/standard.rs` is the sole product algorithm
owner. Its body performs the receiver conversion, length observation, last
property read, deletion, current-function-realm deletion error, and strict
length write in specification order.

The `ExprIr::CallMethod` lowering in `functions.rs` is only a dispatch
optimization. A statically named `pop` call delegates through
`emit_array_direct_builtin_method_call` to that standard built-in. It must not
read or write Array heap length fields, read dense slots, or implement a second
partial `pop` algorithm. This makes later corrections to the standard body
apply equally to direct method syntax and first-class built-in calls.

## Durable evidence

A bounded source-structure test proves that the direct `pop` branch delegates
exactly once and contains none of the former raw heap-length or dense-slot
operations. The same test fixes the canonical standard body's operation order:
receiver conversion, `length` read and `ToLength`, last-element `Get`, delete,
current-function-realm deletion TypeError, then strict `length` write.

The focused CLI fixture covers the observable failures of a second algorithm
owner: a dense element cannot reappear after `pop` followed by length regrowth;
a configurable accessor is read and deleted; a non-configurable last property
throws without mutation; a non-writable length observes deletion before its
throw; and even an empty non-writable `+0` length throws.

## Verification boundary and nonclaims

The focused structure test and CLI fixture pass on the current working tree,
alongside scoped formatting, JavaScript syntax and task/module bookkeeping
checks. The pinned `Array.prototype.pop` leaf and broader Array checkpoint
remain centralized verification obligations.

It removes no Test262 materializer, changes no published conformance count, and
does not claim a current-SHA snapshot delta or a green Array subtree. It does
not by itself complete generic primitive receivers, Proxy observation, Array
exotic descriptors, or any other Array mutator.
