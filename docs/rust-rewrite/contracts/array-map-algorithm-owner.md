# `Array.prototype.map` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused and
shared-checkpoint verification is recorded below.

## Single Array direct entry

The Array arm of static `map()` lowering delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeMap`. The shared call boundary evaluates the
receiver once, propagates abrupt completion, evaluates and expands the complete
argument list from left to right, and only then enters the standard builtin.

The deleted `emit_array_map_method_call` targeted that same standard builtin
but compiled every argument as a standalone expression before constructing its
call. A `SpreadArgument` therefore reached the expression compiler outside a
call and was rejected instead of invoking the iterator protocol. With that
owner absent, a stale direct call fails to compile and every Array Map call uses
the complete argument-list boundary.

The canonical compiler requires argument zero as the mapper and projects
argument one as optional `thisArg`. Later values are evaluated by the shared
boundary but do not affect Map semantics.

## Preserved dispatch and algorithm

The static Map branch retains its exact Array, heap-shape and Iterator receiver
classification. Only the existing Array/known-Array-map arm changes its call
boundary. The earlier Iterator arm and later dynamic Iterator fallback remain
unchanged. Strict TypedArray Map routing and custom Array named-property lookup
remain separate paths.

`compile_array_prototype_map_builtin` remains the sole standard Array Map entry
and does not change in this closure. It still owns callback validation, optional
`thisArg`, receiver conversion and length observation, Array species creation,
sparse `HasProperty` and indexed `Get`, mapper `Call`, target writes and result
publication.

## Durable evidence

`array_map_algorithm_owner_structure.rs` recursively pins:

- the unchanged receiver classification and both Iterator destinations;
- the exact Array standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler and one standard dispatcher consumer;
- receiver-before-arguments-before-call ordering in the shared boundary;
- mapper and optional `thisArg` projection; and
- receiver conversion, `HasProperty`, indexed `Get` and mapper `Call` order in
  the unchanged canonical compiler.

The focused fixture `wasm_array_map_argument_evaluation.js` records mapper,
`thisArg`, an ignored third argument and a custom iterable spread before an
indexed getter and mapper record the start of Map execution. The existing Map
core fixture remains the callback, sparse-array and generic-receiver control.

Existing neighboring guards bound the canonical compiler rather than the
deleted wrapper, so this closure changes no marker file.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`, and the exact new
argument-evaluation witness and existing Map core control each passed `1/1`
against the Wasm backend. The canonical compiler's source hash remained
`6aab327d7a4ae85907a93eebe0acd0b7c88529f0114b1197db526633d9b72b32`, and the
complete Rust source census contains no `emit_array_map_method_call`. Targeted
Rust formatting and the scoped diff check passed. The shared `cargo xc`
checkpoint is green. The pinned `create-proxy.js`,
`callbackfn-resize-arraybuffer.js` and `spread-mult-iter.js` controls pass all
six sloppy/strict Wasm-AOT executions with every failure bucket at zero. No
broader Array or Test262 refresh was performed.

## Nonclaims

This closure does not change the canonical Map algorithm, remove a Test262
materializer, change a published conformance count or claim the Array subtree
green. It does not change Iterator or TypedArray dispatch, repair receiver
classification or ordinary property-lookup policy, or canonicalize `filter`,
`every`, `some`, `concat`, `push` or another method.
