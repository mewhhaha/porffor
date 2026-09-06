# `Array.prototype.every` direct-entry ownership

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

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused and
shared-checkpoint verification is recorded below.

## Single Array direct entry

The Array arm of static `every()` lowering delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeEvery`. The shared call boundary evaluates
the receiver once, propagates abrupt completion, evaluates and expands the
complete argument list from left to right, and only then enters the standard
builtin.

The deleted `emit_array_every_method_call` targeted that same standard builtin
but compiled every argument as a standalone expression before constructing its
call. A `SpreadArgument` therefore reached the expression compiler outside a
call and was rejected instead of invoking the iterator protocol. With that
owner absent, a stale direct call fails to compile and every Array Every call
uses the complete argument-list boundary.

The canonical compiler requires argument zero as the predicate and projects
argument one as optional `thisArg`. Later values are evaluated by the shared
boundary but do not affect Every semantics.

## Preserved dispatch and algorithm

The static Every branch retains its exact Array, heap-shape and Iterator
receiver classification. Only the existing Array/known-Array-every arm changes
its call boundary. The earlier Iterator arm and later dynamic Iterator fallback
remain unchanged. Strict TypedArray Every routing and the generic-versus-strict
entry split remain separate compiler families.

`compile_array_prototype_every_builtin` remains the sole standard Array Every
entry and does not change in this closure. It still owns callback validation,
optional `thisArg`, generic receiver conversion and length observation, sparse
`HasProperty` and indexed `Get`, predicate `Call`, truthiness and false-first
short-circuiting.

## Durable evidence

`array_every_algorithm_owner_structure.rs` recursively pins:

- the unchanged receiver classification and both Iterator destinations;
- the exact Array standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler and one standard dispatcher consumer;
- receiver-before-arguments-before-call ordering in the shared boundary;
- predicate and optional `thisArg` projection; and
- receiver conversion, `HasProperty`, indexed `Get`, predicate `Call` and
  truthiness order in the unchanged canonical compiler.

The focused fixture `wasm_array_every_argument_evaluation.js` records
predicate, `thisArg`, an ignored third argument and a custom iterable spread
before an indexed getter and predicate record the start of Every iteration. The
existing Every core fixture remains the generic receiver, sparse-array,
callback and short-circuit control.

Existing quantifier-family guards bound the canonical compiler rather than the
deleted wrapper, so this closure changes no marker file.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`, and the exact new
argument-evaluation witness and existing Every core control each passed `1/1`
against the Wasm backend. The canonical compiler's source hash remained
`806b26541d7a713834383c191ffb5377f3dc43366d87454dcbf9989f6f0b4cff`, and the
complete Rust source census contains no `emit_array_every_method_call`.
Targeted Rust formatting and the scoped diff check passed. The shared `cargo xc`
checkpoint is green. The pinned `callbackfn-resize-arraybuffer.js`,
`resizable-buffer-grow-mid-iteration.js` and `spread-mult-iter.js` controls pass
all six sloppy/strict Wasm-AOT executions with every failure bucket at zero. No
broader Array or Test262 refresh was performed.

## Nonclaims

This closure does not change the canonical Every algorithm, merge generic
Array and strict TypedArray entries, remove a Test262 materializer, change a
published conformance count or claim the Array subtree green. It does not
change Iterator dispatch, repair receiver classification or ordinary property
lookup, or canonicalize `some`, `filter`, `concat`, `push` or another method.
