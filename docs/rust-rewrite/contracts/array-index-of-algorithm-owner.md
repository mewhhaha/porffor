# `Array.prototype.indexOf` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused
verification is recorded below; broader Array and Test262 verification remain
at the shared checkpoint.

## Single direct entry

Static direct `indexOf()` syntax delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeIndexOf`. The shared direct-call boundary
evaluates the receiver once, propagates its abrupt completion, evaluates and
expands the complete argument list from left to right, and only then enters the
standard builtin.

The deleted `emit_array_index_of_method_call` was a second entry owner. It
compiled only `args.first()` and `args.get(1)`. A third argument was therefore
not evaluated, and a spread argument could bypass the call-argument iterator
protocol. With that owner absent, a stale direct call fails to compile and the
complete source argument list cannot be replaced by two optional expressions.

The canonical builtin reads argument zero and reads argument one only when
`argc` proves it present. Values after the second are evaluated by the call
boundary but do not affect IndexOf semantics.

## Preserved IndexOf algorithm

`compile_array_prototype_index_of_builtin` remains the sole standard entry and
sole caller of `emit_array_index_of_from_locals`. Neither body changes in this
closure. The shared algorithm still owns:

1. `ToObject` and one `LengthOfArrayLike` snapshot;
2. the zero-length return before `fromIndex` coercion;
3. optional `fromIndex` coercion and the forward starting index;
4. `HasProperty` before indexed `Get`, so holes are skipped;
5. the existing borrowed-TypedArray live witness path; and
6. strict equality rather than Includes' `SameValueZero` comparison.

The earlier custom Array named-property path remains ahead of the direct
builtin branch. The static `indexOf` branch retains its existing receiver
classification and direct Array-builtin policy. Strict
`%TypedArray%.prototype.indexOf` remains a separate builtin owner.

## Durable evidence

`crates/lila-aot-wasm/tests/array_index_of_algorithm_owner_structure.rs`
recursively pins:

- the exact direct standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler, one standard dispatcher consumer and one remaining
  caller of the shared IndexOf algorithm;
- receiver-before-arguments-before-call ordering in the shared direct-call
  boundary;
- the canonical first/optional-second argument projection; and
- `ToObject`, length observation, `HasProperty`, indexed `Get` and strict
  equality order in the unchanged inner algorithm.

The focused CLI fixture `wasm_array_index_of_argument_evaluation.js` uses one
direct call whose first two arguments find an accessor-backed element. A third
argument and a trailing custom iterable spread record their complete
left-to-right evaluation before the indexed getter records the start of the
search. The existing TypedArray search fixture remains the control for the
unchanged borrowed Array algorithm and distinct strict TypedArray entry.

The neighboring Array `at` policy guard changes only its end marker because the
deleted IndexOf entry previously followed the shared `at` consumer.

## Verification

The focused owner structure target passes `4/4`, and the neighboring `at`
policy target remains green at `3/3`. The new argument-evaluation CLI witness
and existing TypedArray search control pass `2/2`. The pre/post hashes for the
canonical compiler and inner algorithm are identical, the removed source owner
has zero matches, direct Rust formatting for the five touched Rust files is
green, and the scoped diff check is clean. The completed builds retain the
repository's existing compiler warnings. No broad workspace compile or
Test262 run was performed.

## Nonclaims

This closure does not change the inner IndexOf algorithm, remove a Test262
materializer, change a published conformance count or claim the Array subtree
green. It does not repair the existing direct branch's receiver classification
or ordinary property-lookup policy, merge generic Array entry with strict
TypedArray IndexOf, or canonicalize `lastIndexOf`, `push` or another method.
