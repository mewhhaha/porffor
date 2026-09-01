# `Array.prototype.forEach` algorithm ownership

Status: implemented and verified for the bounded Wasm-AOT owner closure on
2026-08-28.

## One callback algorithm owner

`compile_array_like_for_each_builtin` is the sole Rust source owner of the
Array-like and strict TypedArray ForEach callback algorithm. The standard
builtin dispatcher selects its closed `ArrayCallbackReceiverKind` policy:
`ArrayPrototypeForEach` selects `ArrayLike`, while
`TypedArrayPrototypeForEach` selects `TypedArray`.

The deleted `emit_array_for_each_method_call` had no caller. It independently
compiled only callback and optional `thisArg` expressions, converted the
receiver, walked present indexes and invoked the callback. Its older receiver,
callability and callback-call behavior had already drifted from the live
standard owner. Removing it makes reactivating that stale algorithm a compile
error instead of allowing a second ForEach implementation to diverge silently.

## Preserved live routes

This closure changes no dispatch branch. Ordinary and borrowed Array-like
ForEach calls continue through `StandardBuiltinId::ArrayPrototypeForEach`, and
strict TypedArray calls continue through
`StandardBuiltinId::TypedArrayPrototypeForEach`. The static Iterator
`forEach` branch in `functions.rs` remains a distinct algorithm family and
still delegates to `emit_iterator_prototype_helper_method_call` with its
complete arguments and destination.

The canonical compiler is unchanged. It still projects callback and optional
`thisArg`, applies the selected receiver policy, validates callability, observes
the length snapshot, performs sparse `HasProperty` and indexed `Get`, calls the
callback with value/index/receiver, and returns `undefined`.

## Durable evidence

`array_for_each_algorithm_owner_structure.rs` recursively pins:

- absence of the deleted emitter and exactly one canonical ForEach compiler;
- the exact Array-like and strict TypedArray standard producers;
- the unchanged, distinct Iterator ForEach dispatch owner;
- callback/`thisArg`, receiver conversion, callability, `HasProperty`, indexed
  `Get` and callback `Call` order in the canonical compiler; and
- the existing resizable TypedArray CLI control.

The callback-receiver structure guard now bounds the canonical compiler with
the next live `emit_alloc_array_payload_with_length` function. The deleted
emitter can no longer serve as a stale structural boundary.

`wasm_array_foreach_resizable_typedarray.js` remains the finite behavior
control. It exercises generic `Array.prototype.forEach.call` on fixed and
length-tracking TypedArrays across shrink and grow operations, including a
mid-iteration resize. This dead-owner deletion introduces no new JavaScript
behavior and therefore adds no fixture.

## Verification

On 2026-08-28, the recursive owner target and existing callback-receiver target
each passed `4/4`, and the exact resizable TypedArray ForEach CLI control passed
`1/1` against the Wasm backend. The canonical compiler source is pinned at
`52d8982bbef8b3a99ce51a870919b604394773948aa1944d3f21e939a7aa15fb`, and the
recursive Rust source census contains no `emit_array_for_each_method_call`.
Targeted Rust formatting and the scoped diff check passed. No broad workspace
compile or Test262 run belonged to the implementation lane. At the shared
checkpoint, `cargo xc` and all workspace hygiene gates passed. The pinned
Array `callbackfn-resize-arraybuffer.js`, Array
`resizable-buffer-shrink-mid-iteration.js` and TypedArray
`callbackfn-resize.js` controls passed all `6/6` sloppy/strict Wasm-AOT
executions with every failure bucket at zero.

## Nonclaims

This closure does not change JavaScript behavior, argument evaluation,
receiver classification, the canonical callback algorithm, Iterator helpers,
published conformance status or another Array method. It does not claim the
Array or TypedArray subtree green.
