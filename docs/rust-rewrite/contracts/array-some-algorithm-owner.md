# `Array.prototype.some` direct-entry ownership

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

Status: implemented and shared-checkpoint verified for the Wasm-AOT compiler
on 2026-08-28.

## Single Array direct entry

The Array arm of static `some()` lowering delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeSome`. The shared call boundary evaluates the
receiver once, propagates abrupt completion, evaluates and expands the complete
argument list from left to right, and only then enters the standard builtin.

The deleted `emit_array_some_method_call` targeted that same standard builtin
but compiled every argument as a standalone expression before constructing its
call. A `SpreadArgument` therefore reached the expression compiler outside a
call and was rejected instead of invoking the iterator protocol. With that
owner absent, a stale direct call fails to compile and every Array Some call
uses the complete argument-list boundary.

The canonical compiler requires argument zero as the predicate and projects
argument one as optional `thisArg`. Later values are evaluated by the shared
boundary but do not affect Some semantics.

## Preserved dispatch and algorithm

The static Some branch retains its exact Array, heap-shape and Iterator
receiver classification. Only the existing Array/known-Array-some arm changes
its call boundary. The earlier Iterator arm and later dynamic Iterator fallback
remain unchanged. Strict TypedArray Some routing and the generic-versus-strict
entry split remain separate compiler families.

`compile_array_prototype_some_builtin` remains the sole standard Array Some
entry and does not change in this closure. It still owns callback validation,
optional `thisArg`, generic receiver conversion and length observation, sparse
`HasProperty` and indexed `Get`, predicate `Call`, truthiness and true-first
short-circuiting.

## Durable evidence

`array_some_algorithm_owner_structure.rs` recursively pins:

- the unchanged receiver classification and both Iterator destinations;
- the exact Array standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler and one standard dispatcher consumer;
- receiver-before-arguments-before-call ordering in the shared boundary;
- predicate and optional `thisArg` projection; and
- receiver conversion, `HasProperty`, indexed `Get`, predicate `Call` and
  truthiness order in the unchanged canonical compiler.

The focused fixture `wasm_array_some_argument_evaluation.js` records predicate,
`thisArg`, an ignored third argument and a custom iterable spread before an
indexed getter and predicate record the start of Some iteration. The existing
Some core fixture remains the generic receiver, sparse-array, callback and
short-circuit control.

Existing quantifier-family guards bound the canonical compiler rather than the
deleted wrapper, so this closure changes no marker file.

## Verification

On 2026-08-28, the recursive owner target passed `4/4`, and the exact new
argument-evaluation witness and existing Some core control each passed `1/1`
against the Wasm backend. The canonical compiler's source hash remained
`5301cd10772a6e9b71783b283533b5cb77889d84b87f069b61d4fb113cda0b7d`, and the
complete Rust source census contains no `emit_array_some_method_call`. Targeted
Rust formatting and the scoped diff check passed. The shared `cargo xc`,
workspace formatting, diff, module-boundary and task-plan checks are green.
The pinned `callbackfn-resize-arraybuffer.js`,
`resizable-buffer-shrink-mid-iteration.js` and `spread-mult-iter.js` controls
pass all `6/6` sloppy/strict Wasm-AOT executions with every failure bucket at
zero.

## Nonclaims

This closure does not change the canonical Some algorithm, merge generic Array
and strict TypedArray entries, remove a Test262 materializer, change a published
conformance count or claim the Array subtree green. It does not change Iterator
dispatch, repair receiver classification or ordinary property lookup, or
canonicalize `filter`, `concat`, `push` or another method.
