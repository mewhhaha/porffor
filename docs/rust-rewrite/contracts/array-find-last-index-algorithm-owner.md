# `Array.prototype.findLastIndex` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused
verification is recorded below; broader Array and Test262 verification remain
at the shared checkpoint.

## Single generic direct entry

Static `findLastIndex()` lowering first recognizes the strict TypedArray
builtin from the receiver shape. That branch continues to select
`StandardBuiltinId::TypedArrayPrototypeFindLastIndex`. Every remaining receiver
delegates directly through `emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeFindLastIndex`, preserving the receiver and
complete source argument list.

The deleted `emit_array_find_last_index_method_call` had one caller and only
forwarded the same builtin, label, receiver, arguments and destination to the
shared direct-call boundary. Removing it closes the final redundant direct
owner in the four-method FindViaPredicate family.

## Preserved reverse-index algorithm

The shared boundary remains responsible for receiver evaluation, abrupt
completion and left-to-right evaluation and spread expansion of every argument
before the standard builtin call. The closed
`FindViaPredicateKind::FindLastIndex` selection still enters the unchanged
`compile_array_prototype_find_builtin` compiler. That owner performs generic
receiver conversion and length observation, predicate validation, optional
`thisArg`, reverse-index initialization, indexed `Get`, Proxy-aware predicate
`Call`, truthiness, index projection and reverse advancement in specification
order.

This closure changes no other Find-family route. Iterator helpers have neither
`findIndex` nor `findLastIndex` methods in this dispatcher.

## Durable evidence

`array_find_last_index_algorithm_owner_structure.rs` recursively pins:

- the strict TypedArray-first and generic Array fallback split;
- complete receiver, argument and destination forwarding in both branches;
- absence of the deleted wrapper and one canonical Array Find compiler;
- the exact `FindViaPredicateKind::FindLastIndex` standard consumer;
- receiver-before-arguments-before-call ordering at the shared boundary;
- receiver conversion, predicate validation, optional `thisArg`, reverse-index
  initialization, indexed Get, predicate Call, truthiness, index projection
  and reverse advancement order; and
- the existing reverse-index, `thisArg`, callable-Proxy and Proxy-error CLI
  control.

The pre-existing FindViaPredicate guard now retains only the shared direct-call
boundary and Array receiver-conversion helper from its parent-method inventory.
All four method-specific forwarding wrappers are absent.

## Verification

On 2026-08-28, the recursive ownership target passed `4/4`, and the existing
FindViaPredicate structure target passed `5/5`. Targeted Rust formatting and
the scoped diff check passed. The canonical FindViaPredicate module remained
`f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`, and the
deleted wrapper has zero Rust source occurrences.

The exact existing CLI control
`array::run_wasm_backend_succeeds_for_supported_array_find_last_core_fixture`
passes `1/1`. The pinned `callbackfn-resize-arraybuffer.js`,
`predicate-call-this-strict.js` and `return-abrupt-from-this-length.js`
FindLastIndex controls pass all `5/5` Wasm-AOT executions with every failure
bucket at zero. The shared `cargo xc`, formatting, diff, module-boundary and
task-plan checks are green.

## Nonclaims

This closure changes no JavaScript behavior, receiver classification,
canonical FindViaPredicate compiler, strict TypedArray entry, published
conformance status or another Array method. It does not claim the Array subtree
green.
