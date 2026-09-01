# `Array.prototype.includes` direct-entry ownership

Status: implemented for the Wasm-AOT compiler on 2026-08-28. Focused
verification is recorded below; broader Array and Test262 verification remain
at the shared checkpoint.

## Single direct entry

Static direct `includes()` syntax delegates through
`emit_array_direct_builtin_method_call` to
`StandardBuiltinId::ArrayPrototypeIncludes`. The shared direct-call boundary
evaluates the receiver, propagates its abrupt completion, evaluates and expands
the complete argument list from left to right, and only then enters the
standard builtin.

The deleted `emit_array_includes_method_call` was a second entry owner. It
compiled only `args.first()` and `args.get(1)`. A third argument was therefore
not evaluated, and a spread argument could bypass the call-argument iterator
protocol. With that owner absent, a stale direct call fails to compile and the
complete source argument list cannot be replaced by two optional expressions.

The canonical builtin reads argument zero and reads argument one only when
`argc` proves it present. Values after the second are evaluated by the call
boundary but do not affect Includes semantics. Omitted `fromIndex` remains
distinct from an explicitly supplied `undefined` at that entry boundary.

## Preserved Includes algorithm

`compile_array_prototype_includes_builtin` remains the sole standard entry and
sole caller of `emit_array_includes_from_locals`. Neither body changes in this
closure. The shared algorithm still owns:

1. `ToObject` and the observable `length` lookup;
2. one `ToLength` snapshot;
3. `fromIndex` coercion after a nonzero length has been observed;
4. indexed `Get` for Arrays and generic array-like receivers;
5. the existing borrowed-TypedArray live witness path; and
6. `SameValueZero` comparison.

The early custom Array named-property path remains ahead of the direct builtin
branch, so an overridden `includes` method is still looked up and called.
Strict `%TypedArray%.prototype.includes` remains a separate builtin owner.

## Durable evidence

`crates/lila-aot-wasm/tests/array_includes_algorithm_owner_structure.rs`
recursively pins:

- the exact direct standard builtin selection, label and complete `args`
  forwarding;
- absence of the deleted direct owner;
- one canonical compiler, one standard dispatcher consumer and one remaining
  caller of the shared Includes algorithm;
- receiver-before-arguments-before-call ordering in the shared direct-call
  boundary;
- the canonical first/optional-second argument projection; and
- `ToObject`, length observation, indexed `Get` and `SameValueZero` order in
  the unchanged inner algorithm.

The focused CLI fixture
`wasm_array_includes_argument_evaluation.js` uses one direct call whose first
two arguments find the value while a third argument and a trailing custom
iterable spread record their complete left-to-right evaluation. The existing
symbol-identity fixture remains the direct SameValueZero control. Existing
Proxy, borrowed TypedArray and pinned Test262 controls remain unchanged.

The neighboring Array `at` policy guard changes only its end marker because
the deleted Includes entry previously followed the shared `at` consumer.

The focused owner structure target passes `4/4`, and the neighboring `at`
policy target remains green at `3/3`. The new argument-evaluation CLI witness
and existing symbol-identity CLI control pass `2/2`. Direct Rust formatting for
the five touched Rust files is green, and the scoped diff check is clean. The
completed builds retain the repository's existing compiler warnings. No
Test262 leaf or broad suite was rerun.

## Nonclaims

This closure does not change the inner Includes algorithm, remove a Test262
materializer, change a published conformance count or claim the Array subtree
green. It does not merge the generic Array entry with strict TypedArray
Includes, and it does not canonicalize `indexOf`, `lastIndexOf` or another
direct method.
