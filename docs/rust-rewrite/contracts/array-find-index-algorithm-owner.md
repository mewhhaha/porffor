# `Array.prototype.findIndex` direct-entry ownership

Status: implemented and verified for the Wasm-AOT compiler on 2026-08-28.

## Single generic direct entry

Static `findIndex()` lowering first recognizes the strict TypedArray builtin
from the receiver shape. That branch continues to select
`StandardBuiltinId::TypedArrayPrototypeFindIndex`. Every remaining receiver
delegates directly through `emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeFindIndex`, preserving the receiver and
complete source argument list.

The deleted `emit_array_find_index_method_call` had one caller and only
forwarded the same builtin, label, receiver, arguments and destination to the
shared direct-call boundary. Removing it makes a second Array FindIndex call
owner impossible to invoke or change independently.

## Preserved algorithm

The shared boundary remains responsible for receiver evaluation, abrupt
completion and left-to-right evaluation and spread expansion of every argument
before the standard builtin call. The closed `FindViaPredicateKind::FindIndex`
selection still enters the unchanged
`compile_array_prototype_find_builtin` compiler. That owner performs generic
receiver conversion and length observation, predicate validation, optional
`thisArg`, indexed `Get`, Proxy-aware predicate `Call`, truthiness and index
projection in specification order.

This closure does not alter `find`, `findLast`, `findLastIndex` or Iterator
helper dispatch. Iterator helpers have no `findIndex` method in this branch.

## Durable evidence

`array_find_index_algorithm_owner_structure.rs` recursively pins:

- the strict TypedArray-first and generic Array fallback split;
- complete receiver, argument and destination forwarding in both branches;
- absence of the deleted wrapper and one canonical Array Find compiler;
- the exact `FindViaPredicateKind::FindIndex` standard consumer;
- receiver-before-arguments-before-call ordering at the shared boundary;
- receiver conversion, predicate validation, optional `thisArg`, indexed Get,
  predicate Call, truthiness and index projection order; and
- the existing generic, `thisArg`, callable-Proxy and Proxy-error CLI control.

The pre-existing FindViaPredicate guard drops only the deleted parent wrapper
from its retained-method inventory. All closed-kind, predicate-witness and
algorithm assertions remain unchanged.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`. The canonical
FindViaPredicate module source remained
`f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`, and the
recursive Rust source census contains no
`emit_array_find_index_method_call`. The existing FindViaPredicate target
passes `5/5`, and the exact Find core CLI control passes `1/1`. The pinned
`callbackfn-resize-arraybuffer.js`, `predicate-call-this-strict.js` and
`return-abrupt-from-this-length.js` controls pass all `5/5` Wasm-AOT
executions with every failure bucket at zero. The shared `cargo xc`, workspace
formatting, diff, module-boundary and task-plan checks are green.

## Nonclaims

This closure changes no JavaScript behavior, receiver classification,
canonical FindViaPredicate compiler, strict TypedArray entry, published
conformance status or another Array method. It does not claim the Array subtree
green.
