# `Array.prototype.filter` direct-entry ownership

## Current algorithm ownership (2026-09-06)

The direct-call argument boundary and standard-builtin entry below remain.
The entry now delegates to the closed result policy in
`builtins/array/callback_iteration.rs`, rather than retaining a copied algorithm.
ToObject and captured LengthOfArrayLike precede callback validation; live
HasProperty/Get precede Proxy-aware Call. The revised structural guard pins this
shared ordering and forbids the private-length/function-only bypasses.
See [the shared callback iteration contract](../aot-array-callback-iteration.md)
for result semantics, exact commands and evidence limits.

## Historical direct-entry checkpoint

The dated implementation hashes, counts and unchanged-algorithm statements below
describe the original direct-entry refactor, not the current shared algorithm.

Status: implemented and verified for the bounded Wasm-AOT owner closure on
2026-08-28.

## Single Array direct entry

The Array arm of static `filter()` lowering delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeFilter`. The shared call boundary evaluates
the receiver once, propagates abrupt completion, evaluates and expands the
complete argument list from left to right, and only then enters the standard
builtin.

The deleted `emit_array_filter_method_call` was a second source-equivalent call
owner. It selected the same builtin metadata and performed the same receiver,
complete-argv and direct-call sequence as the shared boundary. With that owner
absent, a stale direct call fails to compile and later call-boundary changes
cannot drift between Filter and the other canonical Array entries.

The canonical compiler requires argument zero as the predicate and projects
argument one as optional `thisArg`. Later values are evaluated by the shared
boundary but do not affect Filter semantics.

## Preserved dispatch and algorithm

The static Filter branch retains its exact Array, heap-shape and Iterator
receiver classification. Only the existing Array/known-Array-filter arm changes
its source owner. The earlier Iterator arm and later dynamic Iterator fallback
remain unchanged. Strict TypedArray Filter routing remains a separate compiler
family.

`compile_array_prototype_filter_builtin` remains the sole standard Array Filter
entry and does not change in this closure. It still owns callback validation,
optional `thisArg`, receiver conversion and length observation, Array species
creation, sparse `HasProperty` and indexed `Get`, predicate `Call`, truthiness,
target writes and result publication.

## Durable evidence

`array_filter_algorithm_owner_structure.rs` recursively pins:

- the unchanged receiver classification and both Iterator destinations;
- the exact Array standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler and one standard dispatcher consumer;
- receiver-before-arguments-before-call ordering in the shared boundary;
- predicate and optional `thisArg` projection; and
- receiver conversion, `HasProperty`, indexed `Get`, predicate `Call`,
  truthiness and target-write order in the unchanged canonical compiler.

The existing `wasm_array_filter_core.js` fixture remains the finite generic
receiver, sparse-array, `thisArg`, callback, target and abrupt-completion
control. This source-equivalent owner deletion adds no new runtime scenario.

Existing guards bound the canonical compiler rather than the deleted wrapper,
so this closure changes no marker file. The unrelated dead
`emit_array_for_each_method_call` remains outside this direct-call lane.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`, and the exact existing
Filter core CLI control passed `1/1` against the Wasm backend. The canonical
compiler's source hash remained
`1f76b4049e22ebd399898021a726a09231429085da68490dc69b5e4339349edd`, and the
complete Rust source census contains no `emit_array_filter_method_call`.
Targeted Rust formatting and the scoped diff check passed. No broad workspace
compile or Test262 run belonged to the implementation lane. At the shared
checkpoint, `cargo xc` and all workspace hygiene gates passed. The pinned
`create-proxy.js`, `target-array-non-extensible.js` and generic
`spread-mult-iter.js` controls passed all `6/6` sloppy/strict Wasm-AOT
executions with every failure bucket at zero.

## Nonclaims

This closure changes no JavaScript behavior. It does not change the canonical
Filter algorithm, remove the dead ForEach emitter, merge generic Array and
strict TypedArray entries, remove a Test262 materializer, change a published
conformance count or claim the Array subtree green. It does not change Iterator
dispatch, receiver classification or ordinary property lookup, or canonicalize
`concat`, `push` or another method.
