# NativeError constructor realm prototypes

The six ECMA-262 NativeError constructors — `EvalError`, `RangeError`,
`ReferenceError`, `SyntaxError`, `TypeError` and `URIError` — share the
`(message, options)` constructor algorithm with `Error`, but each names its own
intrinsic default prototype. `Get(NewTarget, "prototype")` is observable and
occurs exactly once. If it produces a primitive, the fallback is that native
prototype from `GetFunctionRealm(NewTarget)`, not `%Error.prototype%`, an
entry-realm global or a per-function cache.

The normative algorithms are
[`NativeError ( message [ , options ] )`](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-nativeerror-constructors)
and
[`GetPrototypeFromConstructor`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor).
The pinned regressions are the six
`built-ins/NativeErrors/*/proto-from-ctor-realm.js` files.

The 2026-08-13 low-RAM Wasm-AOT baseline was produced by an older compiler
artifact while this change was dry-written. It reported 88 of 94 cases passing
in `built-ins/NativeErrors`; its exact six failures were those six constructor
realm files, all at their first `undefined` fallback assertion. That artifact
selects and bounds this seam. It is not evidence about this source revision.

## One seven-kind constructor authority

`ErrorMessageConstructorKind` is the exact closed set of constructors that use
the shared message/options body: `Error` plus the six NativeErrors. One macro
row owns, for each kind:

- the `NativeErrorKind` identity;
- the `StandardBuiltinId` body identity;
- the entry-realm constructor global;
- the entry-realm prototype global; and
- the realm-intrinsics prototype slot.

`AggregateError` and `SuppressedError` cannot enter this type. Their distinct
argument processing remains on their existing paths. Adding a row changes the
fixed-size `ALL` array and every exhaustive map together.

The shared `(message, options)` algorithm is emitted into each of the seven
builtin bodies. The former NativeError wrapper called the `Error` body, which
made `%Error.prototype%` the only required fallback and could not retain the
NativeError kind. Emitting one parameterized Rust coordinator keeps the
algorithm shared without erasing that compile-time identity. Message coercion,
the absent-message branch and `InstallErrorCause(options)` retain the existing
order.

## Active function and the sole prototype Get

`RequiredResolvedRealmMessageErrorActive(kind)` owns the undefined-NewTarget
transition. For an entry builtin it selects `kind`'s constructor global. For a
created-realm builtin it selects the function object's self-backed environment
slot. It then rejoins the same common `Get(effectiveNewTarget, "prototype")`
used by an explicit NewTarget.

If that Get returns an object-like value, its payload and Object, Function,
Array or Arguments representation tag move together in the private,
non-`Copy`, `must_use` `ErrorConstructorPrototypeLocals` witness. No function
realm is resolved on this arm, including when the Get trap revokes a Proxy.

If the Get returns a primitive, `GetFunctionRealm` is performed only after the
observable Get. Revoked and invalid outcomes are routed before the realm can be
used. The required `OrdinaryDefaultPrototype::MessageError(kind)` slot is then
loaded; missing realm or slot state is an internal bootstrap invariant failure,
never permission to use an entry global.

The generic construct dispatcher derives all seven direct-return entries from
`ErrorMessageConstructorKind::ALL`. Their bodies allocate and return the Error
object, so generic `%Object.prototype%` preallocation would be discarded and,
more importantly, would perform a first observable prototype Get before the
body's second one. The derived direct route makes the typed body the sole owner
of `OrdinaryCreateFromConstructor`.

## Realm storage and materialization

The realm-intrinsics record has ordinary-object slots for all seven prototypes.
The existing `%Error.prototype%` and `%TypeError.prototype%` slots are reused;
five appended slots hold EvalError, RangeError, ReferenceError, SyntaxError and
URIError. Entry bootstrap publishes each already-initialized global through the
typed kind. `$262.createRealm()` publishes the corresponding seven locals
through the same kind-to-slot map.

Created-realm constructor metadata is derived from
`ErrorMessageConstructorKind::ALL`. Each function is materialized with that
realm as `[[Realm]]` and stores its own function object in the environment slot,
so a call without `new` can recover the active intrinsic identity. The
constructor's public `prototype` data and internal function inheritance remain
the corresponding native prototype and created-realm `Error` constructor.

## Durable regression

The CLI fixture covers all six NativeErrors and checks:

- all six primitive fallback values against the matching other-realm native
  prototype;
- direct call through each created-realm active function;
- direct call through each entry-realm active function;
- message and `cause` installation;
- Object, Function, Array and Arguments custom prototype identity/tags;
- `prototype` Get before message coercion and `cause` access;
- one Proxy prototype Get and exact abrupt propagation;
- no realm resolution for an object result after Proxy revocation; and
- required TypeError routing for a primitive result after revocation.

The AOT structural test pins the seven-row authority, record layout, entry and
created-realm publication, self-backing, sole shared Get, per-kind active
selection, ALL-derived direct-return dispatch, absence of the old wrapper call,
and tagged witness allocation.

## Deferred gates

This batch is frozen with source-format, syntax and diff checks only while the
low-RAM matrix owns compilation and Test262 resources. Once it releases them,
verification must include:

```sh
cargo test -p lila-aot-wasm error_message_constructors_are_realm_typed_direct_and_tagged --quiet
cargo test -p lila-cli run_wasm_backend_uses_new_target_realms_for_native_error_prototypes --quiet
./target/debug/lila test262 run built-ins/NativeErrors/EvalError/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/NativeErrors/RangeError/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/NativeErrors/ReferenceError/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/NativeErrors/SyntaxError/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/NativeErrors/TypeError/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/NativeErrors/URIError/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/NativeErrors --execution-backend wasm --timeout-ms 180000 --threads 1
```

The final current-SHA evidence remains the complete T24 ladder and full
low-RAM Wasm-AOT publication run.

## Non-claims

This seam does not migrate `AggregateError` or `SuppressedError`, change
runtime-created throw helpers, add `stack`, alter Error descriptors, close
cross-realm thrown-error identities, complete the NativeErrors tree, complete
T24, refresh snapshots or update README status. No runtime or current-SHA
conformance result is claimed until the deferred gates actually run.
