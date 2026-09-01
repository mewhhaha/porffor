# `Array.prototype.flatMap` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused
verification is recorded below; broader Array and Test262 verification remain
at the shared checkpoint.

## Single Array direct entry

The Array arm of static `flatMap()` syntax delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeFlatMap`. The shared direct-call boundary
evaluates the receiver once, propagates its abrupt completion, evaluates and
expands the complete argument list from left to right, and only then enters the
standard builtin.

The deleted `emit_array_flat_map_method_call` was a second Array entry owner. It
compiled each argument as an ordinary expression and then built an argv from
those locals. A `SpreadArgument` therefore reached the expression compiler
outside a call and was rejected instead of invoking the iterator protocol. The
wrapper also lacked the shared boundary's explicit abrupt-completion checks
between the receiver and argument expressions. With that owner absent, a stale
Array call fails to compile and every Array FlatMap call uses the complete
argument-list boundary.

The canonical compiler requires argument zero as the mapper, reads argument one
as optional `thisArg`, and ignores later values after the call boundary has
evaluated them. Runtime argc remains unrestricted.

## Preserved dispatch and algorithm

`compile_array_prototype_flat_map_builtin` remains the sole standard entry and
sole Array FlatMap algorithm owner. Its body does not change in this closure.
It still owns mapper validation, optional `thisArg`, receiver conversion and
source-length observation, Array species construction, sparse indexed access,
mapper Call, one-level flattening and target publication.

The static branch retains its exact `receiver.kind` or Array heap-shape test.
Only the true Array arm changes its call boundary. The false arm still performs
the ordinary Iterator-helper property read and calls `IteratorHelper::FlatMap`.
The earlier custom Array named-property path and all neighboring receiver
dispatch remain unchanged.

## Durable evidence

`crates/lila-aot-wasm/tests/array_flat_map_algorithm_owner_structure.rs`
recursively pins:

- the unchanged Array/Iterator classification and both destinations;
- the Array arm's exact standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler and one standard dispatcher consumer;
- receiver-before-arguments-before-call ordering in the shared direct-call
  boundary;
- mapper and optional-`thisArg` projection; and
- receiver conversion, length observation, `HasProperty`, indexed `Get` and
  mapper Call order in the unchanged canonical compiler.

The focused CLI fixture `wasm_array_flat_map_argument_evaluation.js` uses an
accessor-backed source element. Its mapper, `thisArg`, ignored third argument
and trailing custom iterable spread record complete left-to-right evaluation
before the indexed getter and mapper record the start of FlatMap semantics. The
existing core and Proxy access-count fixtures remain algorithm controls.

Existing FlatMap TypedArray-witness and neighboring Concat structure guards
bound the canonical compiler, not the deleted wrapper, so this closure changes
no marker file.

## Verification

On 2026-08-28, the focused recursive owner target passed `4/4`. The exact new
argument-evaluation CLI witness and the existing core and Proxy access-count
controls each passed `1/1` against the Wasm backend. The canonical compiler's
source hash remained
`009ab7510a4d965f1db3ff83df63ed3b1739ae9c137d878c9148ee68801c5761`, and the
complete Rust source census contains no `emit_array_flat_map_method_call`.
Scoped formatting and diff checks passed for this lane, and the shared
`cargo xc` checkpoint is green. Pinned Proxy access-count, `thisArg`, and
generic spread controls pass all five generated variants (`5/5`) on Wasm-AOT.

## Nonclaims

This closure does not change the canonical FlatMap algorithm, remove a Test262
materializer, change a published conformance count or claim the Array subtree
green. It does not alter Iterator helper dispatch, repair the existing direct
branch's receiver classification or ordinary property-lookup policy, or
canonicalize `map`, `concat`, `push` or another method.
