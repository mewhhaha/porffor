# `Array.prototype.pop` algorithm and dispatch ownership

Status: routing invariant implemented and verified for the Wasm-AOT compiler
on 2026-08-28.

## Semantic boundary

ECMA-262 defines `Array.prototype.pop` as one generic sequence of observable
operations:

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
owner. Its body performs receiver conversion, length observation, last-property
read, deletion, current-function-realm deletion error, and strict length write
in specification order.

The canonical Pop arm remains pinned at
`240862a71152eef7a1373e0bfd98b928ccad87dd1daf057851c099e437c91038`.

## Closed static dispatch

The `ExprIr::CallMethod` lowering in `functions.rs` is only a dispatch
optimization. Its static `pop()` seam locally owns the capability-free
`PopMethodDispatch::{ArrayCanonical, GenericGetCall}` authority. A data property
whose function-target set contains exactly the one
`StandardBuiltinId::ArrayPrototypePop` target constructs `ArrayCanonical`.
Every absent, accessor or ambiguous target constructs `GenericGetCall`.

One exhaustive match owns the sole canonical direct call and ordinary property
Get/Call fallthrough. The authority has no derives, wildcard, default,
kind-only shortcut or independently reusable target Boolean. Array kind and
Array heap shape therefore cannot bypass an own `pop` method or accessor.

The canonical arm delegates the complete source argument list through
`emit_array_direct_builtin_method_call`. It does not read or write Array heap
length fields, read dense slots, or implement a second partial Pop algorithm.
The generic state emits nothing in the optimized seam, allowing the shared
EvaluateCall tail to acquire and call the actual property.

## Durable evidence

`array_pop_algorithm_owner_structure.rs` recursively pins the exact
capability-free two-state authority, one producer and one exhaustive consumer
per state, one singleton shape-target proof, one direct canonical call, generic
fallthrough, and zero parallel Pop emitters. It also fixes the canonical
standard body's operation order: receiver conversion, `length` read and
`ToLength`, last-element `Get`, delete, current-function-realm deletion
TypeError, then strict `length` write.

The existing `wasm_array_pop_algorithm_owner.js` fixture covers the observable
failures of a second algorithm owner: dense elements cannot reappear after
length regrowth, accessors are read and deleted, non-configurable properties
throw without mutation, deletion precedes a non-writable-length throw, and an
empty non-writable `+0` length still throws.

`wasm_array_pop_own_method_dispatch.js` installs an own `pop` method on an
Array, observes its receiver and ordinary/spread arguments, returns a custom
result and requires the elements and length to remain unchanged. It fails if a
kind or heap-shape heuristic restores unconditional canonical routing.

## Verification boundary and nonclaims

The recursive `array_pop_algorithm_owner_structure` target passes `5/5`. The
exact own-method dispatch and canonical-algorithm CLI controls each pass `1/1`.
The shared `cargo xc` checkpoint is green.

Pinned Test262 controls are `set-length-array-length-is-non-writable.js`,
`set-length-zero-array-length-is-non-writable.js` and
`throws-with-string-receiver.js`. Each leaf passes both ordinary Wasm-AOT
executions, for `6/6` total.

This closure removes no Test262 materializer, changes no published conformance
count, and does not claim a green Array subtree. It does not complete generic
primitive receivers, Proxy observation, Array exotic descriptors, or any other
Array mutator.
