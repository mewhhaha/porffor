# `Array.prototype.splice` algorithm and dispatch ownership

Status: dispatch invariant implemented and verified for the Wasm-AOT compiler
on 2026-08-28.

## One standard Splice algorithm

`compile_array_prototype_splice_builtin` is the sole Rust source owner of the
standard Array Splice algorithm. Static `splice()` method lowering delegates
the receiver and complete argument list through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeSplice`; the standard dispatcher then enters
the canonical compiler.

The deleted `emit_array_splice_insert_method_call` had no caller outside its
own unreachable subgraph. Its only edge selected the likewise unreachable
`emit_array_splice_delete_one_method_call` for a static delete count of one.
Both functions restricted receivers and arguments to partial specialized
cases, directly manipulated Array storage, and implemented a second Splice
operation order. Removing the whole subgraph makes any attempt to reactivate
those partial algorithms fail to compile.

## Closed static dispatch

The `ExprIr::CallMethod` lowering in `functions.rs` is only a dispatch
optimization. Its static `splice()` seam locally owns the capability-free
`SpliceMethodDispatch::{ArrayCanonical, GenericGetCall}` authority. A data
property whose function-target set contains exactly the one
`StandardBuiltinId::ArrayPrototypeSplice` target constructs `ArrayCanonical`.
Every absent, accessor, ambiguous or unknown target constructs
`GenericGetCall`.

One exhaustive match owns the sole canonical direct call and ordinary property
Get/Call fallthrough. The authority has no derives, wildcard, default,
kind-only shortcut or Array heap-shape shortcut. An own method, accessor or
Proxy receiver therefore cannot be bypassed by the canonical algorithm.

## Preserved live routes

The direct standard route is unchanged and still forwards all arguments,
including spreads, through the shared call boundary. The canonical compiler is
unchanged: it owns receiver conversion, the length snapshot, start and delete
count coercion, species construction, deleted-element collection, directional
property moves and deletes, item insertion, final length write and result.

The separate `spliceFromArray` extension remains live and unchanged. Its
static branch still delegates to `emit_array_splice_from_array_method_call`.
This closure does not treat that nonstandard array-spread operation as a second
owner of `Array.prototype.splice`.

## Durable evidence

`array_splice_algorithm_owner_structure.rs` recursively pins:

- zero source occurrences of both deleted specialized emitters;
- exactly one canonical standard compiler and one preserved `spliceFromArray`
  emitter;
- the exact capability-free two-state dispatch, one producer and one exhaustive
  consumer per state, singleton target proof and generic Get/Call fallthrough;
- complete receiver and argument forwarding in the proven direct standard
  branch;
- the exact standard dispatcher consumer;
- receiver conversion, length, argument coercion, species, property traversal,
  deletion, insertion and result ordering in the canonical compiler; and
- the existing finite mutation-during-Find CLI control.

`wasm_array_find_core.js` calls `splice(1, 1)` during the first Find callback
and records all three subsequent callback observations. It would expose an
accidental return to the removed static-array-only delete path while adding no
new scenario for a dead-owner deletion.

`wasm_array_splice_own_method_dispatch.js` installs an own `splice` method on
an Array, observes its receiver and ordinary/spread arguments, returns a custom
result and requires the elements and length to remain unchanged. It fails if a
kind or heap-shape heuristic restores unconditional canonical routing.

## Verification

The recursive `array_splice_algorithm_owner_structure` target passes `5/5`.
The exact own-Splice and existing canonical Find-core CLI controls each pass
`1/1`. The shared `cargo xc` checkpoint is green. The canonical compiler source
remains pinned at
`7236e422756416048ad4668122d118af0f83b7a49ed27c913eb4941d29972394`, and the
recursive Rust source census contains neither deleted emitter.

Pinned Test262 controls are `called_with_one_argument.js`,
`property-traps-order-with-species.js` and `create-proxy.js`. Each leaf passes
both ordinary Wasm-AOT executions, for `6/6` total.

## Nonclaims

This closure changes only statically named calls whose actual `splice` property
is not proven to be the intrinsic. It changes no standard Splice operation,
Iterator or TypedArray dispatch, `spliceFromArray`, published conformance
status or another Array method. It does not claim the Array subtree green.
