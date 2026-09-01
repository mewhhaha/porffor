# Created-Realm weak-collection publication

## Scope

This boundary publishes the implemented `WeakMap` and `WeakSet` constructors,
prototypes and methods in realms created by the Test262 host capability. It
closes intrinsic identity, exact descriptors, prototype-key order,
constructor-prototype fallback and defining-Realm TypeError ownership. It does
not make the collector's weak and ephemeron metadata executable.

The entry-Realm algorithms remain in
`crates/lila-aot-wasm/src/builtins/collections.rs`, and their entry-Realm
properties remain in `crates/lila-aot-wasm/src/intrinsics/collections.rs`.
Created-Realm ownership lives in the private
`crates/lila-aot-wasm/src/builtins/host/created_realm_weak_collection_intrinsics.rs`
child.

## Publication lifecycle

`CreatedRealmWeakCollectionIntrinsics` is a non-`Copy`, `must_use` token with
child-private prototype and constructor locals for both families. Its
materializer completes these operations before the token can exist:

1. allocate each prototype under the created Realm's `%Object.prototype%`;
2. store each identity in the closed `WeakMapPrototype` or `WeakSetPrototype`
   Realm slot;
3. materialize and link each constructor before installing prototype methods;
4. install fresh WeakMap `delete`, `get`, `getOrInsert`,
   `getOrInsertComputed`, `has` and `set` functions;
5. install fresh WeakSet `add`, `delete` and `has` functions; and
6. move the matching `CollectionPrototypeIntrinsic` into the sole shared
   `@@toStringTag` descriptor emitter.

The constructor-first link preserves the exact own-key order:

- WeakMap: `constructor`, `delete`, `get`, `getOrInsert`,
  `getOrInsertComputed`, `has`, `set`, `Symbol.toStringTag`;
- WeakSet: `constructor`, `add`, `delete`, `has`, `Symbol.toStringTag`.

The constructor's `prototype` property is non-writable, non-enumerable and
non-configurable. Prototype constructors and methods are writable,
non-enumerable and configurable. `@@toStringTag` is non-writable,
non-enumerable and configurable. The created global bindings are writable,
non-enumerable and configurable.

Temporary locals are stack-shaped. The child therefore materializes WeakSet
before WeakMap, then its consuming publisher exposes and releases WeakMap
before WeakSet. The parent materializes FinalizationRegistry, WeakRef and the
weak-collection pair in that order, then publishes the pair, WeakRef and
FinalizationRegistry in reverse. Among the globals present on created realms,
the resulting observable subsequence follows `GlobalOrdinal`:
`Map`, `WeakMap`, `WeakSet`, `WeakRef`, `FinalizationRegistry`, `Set`.

## Typed family authority

`CollectionPrototypeIntrinsic` remains the sole four-family authority for
collection prototype tags. Its exhaustive projections select the entry-Realm
prototype global, the matching Realm intrinsic slot and the tag spelling. The
created-Realm path borrows `realm_slot()` before moving the same value into the
shared descriptor emitter. No raw created-Realm `Symbol.toStringTag` key,
family String or descriptor flags are duplicated.

## Realm ownership

Both constructors and all nine prototype methods are materialized through the
created Realm's `RealmFunctionMaterializationContext`. Every callable receives
the created `%Function.prototype%`, a self-backed environment handle and the
created `%TypeError.prototype%` before publication. Requires-`new`, invalid
weak key/value and borrowed-method receiver failures therefore use the
executing builtin's defining Realm.

Both constructors already select
`NewTargetPrototypeFallback::RealmIntrinsic`. When a foreign NewTarget has a
primitive `prototype`, `GetFunctionRealm` reaches the private created-Realm
slot even after the mutable `global.WeakMap` and `global.WeakSet` properties
are overwritten.

## Evidence

`crates/lila-aot-wasm/tests/created_realm_weak_collection_publication_structure.rs`
pins the one-shot token, both Realm-slot writes, typed tag authority, method
parity with entry installers, constructor-first key order, internal and parent
LIFO lifecycles, and the filtered global catalog order. It passes `6/6`.

`crates/lila-cli/tests/fixtures/wasm_weak_collections_created_realm.js` is the
source-free runtime witness. It covers fresh identities, parent prototypes,
descriptors, all method names and lengths, constructability, iterable
construction, core operations, bidirectional method borrowing, six
defining-Realm TypeError paths and all seven primitive foreign-NewTarget
fallback types after overwriting both created global bindings. Its exact CLI
owner passes `1/1`; `node --check` also accepts the fixture.

On 2026-08-31, these sixteen pinned files passed all `32/32` sloppy/strict
Wasm-AOT executions with every failure and non-success bucket at zero:

- `built-ins/WeakMap/proto-from-ctor-realm.js`;
- `built-ins/WeakMap/undefined-newtarget.js`;
- `built-ins/WeakMap/prototype/prototype-attributes.js`;
- `built-ins/WeakMap/prototype/constructor.js`;
- `built-ins/WeakMap/prototype/get/get.js`;
- `built-ins/WeakMap/prototype/set/returns-this.js`;
- `built-ins/WeakMap/prototype-of-weakmap.js`;
- `built-ins/WeakMap/properties-of-the-weakmap-prototype-object.js`;
- `built-ins/WeakSet/proto-from-ctor-realm.js`;
- `built-ins/WeakSet/undefined-newtarget.js`;
- `built-ins/WeakSet/prototype/prototype-attributes.js`;
- `built-ins/WeakSet/prototype/constructor/weakset-prototype-constructor.js`;
- `built-ins/WeakSet/prototype/add/add.js`;
- `built-ins/WeakSet/prototype/has/returns-true-when-object-value-present.js`;
- `built-ins/WeakSet/prototype-of-weakset.js`; and
- `built-ins/WeakSet/properties-of-the-weakset-prototype-object.js`.

`cargo xc` is green. The broad `lila-aot-wasm` library target remains
`367/374`; its seven failures are the unchanged runtime-message ordering,
async-resume ownership, Promise-state ownership, suspending async-generator,
static-planning and string/RegExp layout baseline failures. No failure belongs
to this boundary. Formatting, diff hygiene, task-plan, README-status and
Test262-host-ABI checks are green. The module-boundary check remains red only
at the pre-existing raw-line caps for `lila-ir/src/builtins.rs` (`1769 > 1760`)
and `lila-ir/src/lowering/builtin_call_info.rs` (`2291 > 2250`); neither file is
part of this boundary.

The focused verification commands are:

```sh
cargo test -p lila-aot-wasm --test created_realm_weak_collection_publication_structure
cargo test -p lila-cli --test cli \
  iterator::run_wasm_backend_succeeds_for_created_realm_weak_collection_publication \
  -- --exact
```

## Nonclaims

The collector still cannot clear weak entries, enforce ephemeron liveness or
schedule cleanup. This boundary does not publish AsyncDisposableStack in
created realms, repair the pre-existing full created-global ordering debt,
claim complete WeakMap/WeakSet trees, refresh aggregate conformance status or
complete T21.
