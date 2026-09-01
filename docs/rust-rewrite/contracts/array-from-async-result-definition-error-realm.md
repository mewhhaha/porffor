# Array.fromAsync result-definition error Realm

Status: implemented on 2026-08-26; focused verification is recorded below.

## Authority

`Array.fromAsync` uses constructor `C` to create result object `A`, but `C`
does not own errors synthesized later by `CreateDataPropertyOrThrow(A, Pk,
value)` or `Set(A, "length", len, true)`. Those operations execute in the
method-owned continuation. A false ordinary or Proxy internal-method result is
therefore converted to a `TypeError` from the executing `Array.fromAsync`
method's Realm.

Constructor and result-object execution remain independent. A value thrown by
the constructor, a result setter or a Proxy trap is propagated unchanged.

## Typed boundary

`ObjectMutationErrorRealmSource` is the closed three-case body authority:

- `GlobalFallback` for bodies whose environment can be an ordinary lexical
  environment;
- `StandardBuiltinEnvironment` for a self-backed builtin; and
- `SetPathHelperArgument` for the five compiler-owned outlined Set helpers.

Its exhaustive projection produces only
`ObjectMutationErrorRealm::{TrustedCurrentEnvironment, MainRealmFallback}`.
The mutation TypeError emitters consume that projection directly. No caller
can select Realm authority with a Boolean, raw environment test or wildcard
arm.

The boundary owns all eight ordinary property-descriptor rejection producers:
six attribute/value/accessor comparisons, the shared kind-change rejection and
the non-extensible new-property rejection. The last path retains the
specification's no-message error shape while selecting its prototype through
the same authority. Both ordinary `CreateDataPropertyOrThrow` rejection sites
and ordinary non-writable/non-extensible Set failures also use the boundary.
Proxy-false errors share the same projection; values thrown by setters or
Proxy traps bypass it.

## Array.fromAsync routes

The fulfilled continuation has one result-element definition producer. Length
publication has two producers: the zero-length array-like fast path and the
common completion callback used after iterable or nonempty array-like
processing. Their existing rejection and iterator-closing routes are unchanged.

The finite CLI fixture separates method, constructor and result authority in
both Realm directions using Proxy constructors whose targets belong to the
opposite Realm. It covers incompatible index replacement, non-extensible index
creation, non-writable length on both the immediate and continuation paths, and
identity preservation for a throwing length setter. The non-extensible control
also asserts that the synthesized error has no own `message` property.

The bounded structure target pins the three-source/two-authority domain, all
eight descriptor producers, both `CreateDataPropertyOrThrow` rejection sites,
both ordinary Set failure owners, the Array.fromAsync producer census and the
fixture registration.

Both bounded structure targets pass `4/4`, and the seven-control CLI fixture
passes `1/1`. The six directly relevant pinned Test262 files pass all twelve
sloppy/strict Wasm-AOT executions (`12/12`), with every failure bucket at zero.

```sh
cargo test -p lila-aot-wasm --test object_write_proxy_realm_structure --quiet
cargo test -p lila-aot-wasm --test array_from_async_result_definition_error_realm_structure --quiet
cargo test -p lila-cli --test cli array_from_async_result_definition_errors_use_the_method_realm --quiet
```

The pinned controls are `this-constructor-with-unsettable-element.js`, both
`this-constructor-with-unsettable-element-closes-*-iterator.js` files,
`this-constructor-with-readonly-length.js`,
`this-constructor-with-bad-length-setter.js` and
`this-constructor-with-readonly-elements.js`.

The shared workspace semantic golden passes `2/2` in 800.46 seconds with 679
dumps. It adds only the focused fixture and removes none. Of 678 retained
dumps, 677 are equal after accounting normalization; the independently
expanded Promise internal-callback Realm fixture is the sole structural
change. The Realm-aware shared ObjectWrite and descriptor paths account for
the broad emitted-byte deltas. The complete 95-file Array.fromAsync leaf
remains deferred.

This boundary does not change result construction, callback materialization,
Promise capability ownership, iterator closing, Proxy trap semantics or
general user-function Realm inference.
