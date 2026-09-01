# `Array.prototype.find` direct-entry ownership

Status: implemented and verified for the Wasm-AOT compiler on 2026-08-28.

## Single Array direct entry

The Array arm of static `find()` lowering delegates directly through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeFind`. The shared boundary evaluates the
receiver once, propagates abrupt completion, evaluates and expands the complete
argument list from left to right, and only then enters the standard builtin.

The deleted `emit_array_find_method_call` had one caller and only forwarded the
same builtin, label, receiver, arguments and destination to that boundary.
Removing the redundant owner means a stale Array Find call path cannot be
reintroduced without a compile error.

## Preserved dispatch and algorithm

The complete static Find receiver classification is unchanged. Iterator Find
still owns both its statically known and dynamic-helper destinations. Strict
TypedArray Find still selects `TypedArrayPrototypeFind` before the Array arm.
Only the existing Array/known-Array-Find arm changes its source-level call
owner. `findIndex`, `findLast` and `findLastIndex` remain outside this closure.

The canonical `compile_array_prototype_find_builtin` compiler remains in
`builtins/array/find_via_predicate.rs` and is unchanged. It still owns generic
receiver conversion and length observation, predicate validation, optional
`thisArg`, indexed `Get`, Proxy-aware predicate `Call`, truthiness, projection
and direction through the closed `FindViaPredicateKind::Find` selection.

## Durable evidence

`array_find_algorithm_owner_structure.rs` recursively pins:

- the unchanged Array, strict TypedArray and two Iterator destinations;
- exact Array standard builtin selection, label and complete arguments;
- absence of the deleted wrapper and one canonical Array Find compiler;
- the exact standard dispatcher consumer;
- receiver-before-arguments-before-call ordering at the shared boundary;
- receiver conversion, predicate validation, optional `thisArg`, indexed Get,
  predicate Call, truthiness and result projection order; and
- the existing sparse, borrowed TypedArray and callable-Proxy CLI control.

The pre-existing FindViaPredicate guard no longer lists the deleted parent
wrapper as a retained method. Its closed-kind, predicate-witness, TypedArray
witness and algorithm assertions are otherwise unchanged.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`, the existing
FindViaPredicate target passed `5/5`, and the exact Find core CLI control passed
`1/1` against the Wasm backend. The canonical FindViaPredicate module source
remained `f3785b16f21f209331fdbb16888c6752afdcfda87eb570dbbd3211349b848184`,
and the recursive Rust source census contains no
`emit_array_find_method_call`. The pinned
`callbackfn-resize-arraybuffer.js`, `predicate-call-this-strict.js` and
`return-abrupt-from-this-length.js` controls pass all `5/5` Wasm-AOT
executions with zero failures. The shared `cargo xc`, workspace formatting,
diff, module-boundary and task-plan checks are green.

## Nonclaims

This closure changes no JavaScript behavior, receiver classification,
canonical Find algorithm, Iterator helper, strict TypedArray entry, published
conformance status or another Array method. It does not claim the Array subtree
green.
