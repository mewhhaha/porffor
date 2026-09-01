# Created-Realm `FinalizationRegistry` publication

## Scope

This boundary publishes the existing `FinalizationRegistry` constructor,
prototype, `register`, and `unregister` implementation in realms created by the
Test262 host capability. It closes intrinsic identity, descriptors,
constructor prototype fallback, and defining-Realm TypeError ownership. It
does not implement collection-driven cleanup.

The entry-Realm algorithm owners remain
`crates/lila-aot-wasm/src/builtins/finalization_registry.rs` and
`crates/lila-aot-wasm/src/intrinsics/collections.rs`. Created-Realm ownership
lives in the private
`crates/lila-aot-wasm/src/builtins/host/created_realm_finalization_registry_intrinsics.rs`
child.

## Publication lifecycle

`CreatedRealmFinalizationRegistryIntrinsics` is a non-`Copy`, `must_use`
publication token with child-private fields. Its materializer must complete all
of these operations before returning the token:

1. allocate `%FinalizationRegistry.prototype%` under the created Realm's
   `%Object.prototype%`;
2. store it in
   `NonArrayRealmIntrinsicSlot::FinalizationRegistryPrototype`;
3. materialize the fresh constructor and install the reciprocal
   constructor/prototype links with their exact attributes;
4. install fresh `register` and `unregister` functions; and
5. append the exact `@@toStringTag` property.

Only the consuming publisher can expose `global.FinalizationRegistry`, after
the created global exists. The constructor's `prototype` property is
non-writable, non-enumerable, and non-configurable. The prototype's
`constructor`, `register`, and `unregister` properties are writable,
non-enumerable, and configurable. `@@toStringTag` is keyed by the well-known
Symbol and has `{ writable: false, enumerable: false, configurable: true }`.
Constructor-first linking matches entry-Realm bootstrap and preserves the exact
own-key order: `constructor`, `register`, `unregister`, `Symbol.toStringTag`.

The WeakRef and FinalizationRegistry tokens retain backend temporary locals at
the same time. They therefore materialize in FinalizationRegistry-to-WeakRef
order and publish in WeakRef-to-FinalizationRegistry order. This preserves the
observable global property order while satisfying the backend's stack-shaped
temporary-local lifecycle.

## Realm ownership

The constructor and both prototype methods are materialized through the
created Realm's `RealmFunctionMaterializationContext`. Each callable receives
a self-backed environment handle and the created Realm's
`%TypeError.prototype%` before publication. Consequently, requires-`new`,
invalid cleanup-callback, and borrowed-method receiver errors are created in
the builtin function's defining Realm.

The constructor already uses
`NewTargetPrototypeFallback::RealmIntrinsic` with the closed
FinalizationRegistry prototype slot. A construct whose foreign NewTarget has a
primitive `prototype` therefore falls back to that NewTarget function's
Realm-local `%FinalizationRegistry.prototype%`.

## Evidence

The bounded source test
`crates/lila-aot-wasm/tests/created_realm_finalization_registry_publication_structure.rs`
pins the private one-shot token, Realm-slot write, exact callable ownership,
constructor-first own-key order, descriptor installation, global-allocation
boundary, and reverse materialize/publish ordering. It passes `7/7`.

The source-free fixture
`crates/lila-cli/tests/fixtures/wasm_finalization_registry_created_realm.js`
covers fresh identities, function parents and metadata, descriptors, exact
prototype own keys, construction, register/unregister behavior in both
cross-Realm borrowing directions, four defining-Realm TypeError paths, and all
seven primitive foreign-NewTarget fallbacks after replacing the created
Realm's mutable global binding. Its exact CLI test passes `1/1`.

On 2026-08-31, these six pinned files passed all `12/12` sloppy/strict
Wasm-AOT executions with every failure and non-success bucket at zero:

- `built-ins/FinalizationRegistry/proto-from-ctor-realm.js`;
- `built-ins/FinalizationRegistry/newtarget-prototype-is-not-object.js`;
- `built-ins/FinalizationRegistry/prototype/constructor.js`;
- `built-ins/FinalizationRegistry/prototype/prop-desc.js`;
- `built-ins/FinalizationRegistry/prototype/register/this-not-object-throws.js`;
- `built-ins/FinalizationRegistry/prototype/unregister/custom-this.js`.

On 2026-09-01, the focused constructor, method-descriptor and tag cohort passed
all `8/8` sloppy/strict Wasm-AOT executions with every failure and non-success
bucket at zero:

- `built-ins/FinalizationRegistry/prototype/constructor.js`;
- `built-ins/FinalizationRegistry/prototype/register/prop-desc.js`;
- `built-ins/FinalizationRegistry/prototype/unregister/prop-desc.js`; and
- `built-ins/FinalizationRegistry/prototype/Symbol.toStringTag.js`.

At the 2026-09-01 shared checkpoint, `cargo xc`, formatting, diff hygiene,
task-plan, README-status, host-ABI and fixture-syntax gates are green. The broad
`lila-aot-wasm` library target reports `367/374`; its seven failures are the
unchanged pre-existing string-layout, async-resume, Promise-state,
async-generator and static-planning baseline failures, with no failure in this
boundary. The module-boundary gate is red only on the pre-existing raw line
caps in `lila-ir/src/lowering/builtin_call_info.rs` (`2291 > 2250`) and
`lila-ir/src/builtins.rs` (`1769 > 1760`).

The focused verification commands are:

```sh
cargo test -p lila-aot-wasm --test created_realm_finalization_registry_publication_structure
cargo test -p lila-cli --test cli \
  iterator::run_wasm_backend_succeeds_for_created_realm_finalization_registry_publication \
  -- --exact
```

## Nonclaims

The current collector cannot yet clear weak targets or enqueue finalization
cleanup jobs. This boundary does not claim weak reachability, cleanup timing,
cleanup callback delivery, created-Realm WeakMap/WeakSet publication, the full
FinalizationRegistry Test262 tree, or a published conformance-count change.
