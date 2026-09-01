# `Array.prototype.toString` algorithm ownership

Status: implemented and focused-verified for the Wasm-AOT compiler on
2026-08-28. The exact pinned Test262 leaf and broader Array verification remain
deferred to the shared checkpoint.

## Single product algorithm

The shared standard builtin currently named
`StandardBuiltinId::TypedArrayPrototypeToString` is the sole installed
`Array.prototype.toString` and `%TypedArray%.prototype.toString` algorithm.
Its compiler body performs the required observable sequence:

1. convert the receiver with `ToObject`;
2. `Get` its `join` property;
3. test the result with `IsCallable`;
4. call a callable value with the original object receiver and no arguments;
   or
5. use the intrinsic `Object.prototype.toString` result algorithm when the
   value is not callable.

The fallback retains recursive Proxy-aware `IsArray`, callable classification,
builtin brands, `@@toStringTag` observation and the executing builtin's Realm
for revoked-Proxy failures.

Static direct Array `toString()` syntax now delegates through
`emit_array_direct_builtin_method_call` to that same closed
`StandardBuiltinId`. The former direct array-only join body is deleted. It
could read the raw Array length and join indexed values, but it could not own
the observable `join` lookup, callability decision or intrinsic fallback.
There is therefore no second body that can drift when the standard algorithm
changes, and a stale call to either removed entry fails to compile.

## Preserved `join` ownership

This closure does not merge `Array.prototype.join` with `toString`.
`compile_array_prototype_join_builtin` still owns generic `ToObject`,
`LengthOfArrayLike`, separator conversion and indexed reads. Its generic path
and the strict TypedArray join entry continue to share only the existing
length-bounded join emitter.

Named-property dispatch remains ahead of the direct intrinsic route. Array
subclasses and shapes that expose an overridden named method therefore retain
their ordinary lookup and call behavior.

## Durable evidence

`crates/lila-aot-wasm/tests/array_to_string_algorithm_owner_structure.rs`
recursively guards the Rust source tree and pins:

- the sole direct Array branch and its exact shared builtin selection;
- absence of both removed array-only owners;
- one canonical compiler definition and one standard dispatcher call;
- `ToObject`, `Get("join")`, `IsCallable`, Proxy-aware call and fallback in
  their required order;
- continued ownership of real `Array.prototype.join`; and
- the existing focused runtime fixtures and CLI registrations.

The stale boundary in the Proxy Set Realm structure guard now ends at the real
Array join compiler. No Proxy Set behavior changed.

## Focused verification

The owner structure target passes `4/4`, and the existing direct conversion
CLI regression passes `1/1`. The subclass named-property and Proxy Array
fallback CLI regressions remain at the shared checkpoint because another
source lane was mid-extraction when their focused run began. Direct Rust
formatting checks for the frozen source and the scoped diff check are green.
The completed runs retain the repository's existing compiler warnings.

The unchanged pinned source
`built-ins/Array/prototype/toString/non-callable-join-string-tag.js` remains
the focused Test262 control for non-callable primitive values, Proxy
revocation, builtin tags, `@@toStringTag` and abrupt lookup. Its sloppy and
strict Wasm-AOT executions are deferred to the shared checkpoint.

## Nonclaims

This source ownership closure does not change the shared builtin's public
identity, rename its existing StandardBuiltinId, remove a Test262 materializer,
alter published conformance counts or claim the Array subtree green. It does
not change `join`, element string conversion, generic receiver behavior or
another direct Array optimization.
