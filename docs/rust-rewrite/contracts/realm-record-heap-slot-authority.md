# Realm-record heap-slot identity authority

## Closed layout identities

The passive Realm record contains exactly nine capability-free
`RealmRecordHeapSlot` identities in Realm-id, Agent-id, global-object,
global-this, global-environment, intrinsics, host-hooks, module-registry and
private-elements order.

One private exhaustive `metadata()` projection is the sole authority for all
nine identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Realm and Agent ids
occupy offsets 0 and 8. Global object, global this, global environment,
intrinsics, host hooks, module registry and private elements occupy offsets 16,
24, 32, 40, 48, 56 and 64. The two ids remain scalar, while all seven Realm
ownership edges remain pointer-classified.

This two-scalar/seven-pointer census is a retention invariant. A Realm must
keep its global state, intrinsic table, host state, module registry and private
element list visible to tracing, while neither identity word may be scanned as
an address. An arbitrary row can no longer reverse either side of that
relation or reorder one field independently of the closed identity registry.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Realm rows. The bounded heap
owner witness asserts every projected field. `RealmRecordLocal`, Realm-id
allocation and created-Realm publication policies remain independent lifetime
and semantic authorities.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change Realm allocation, initialization, lookup, intrinsic publication,
global-environment behavior, host hooks, module loading, private elements,
emitted Wasm, root scanning or collector execution. All Realm runtime offset
and size consumers remain unchanged.

```sh
cargo test -p lila-aot-wasm --test realm_record_heap_slot_structure
cargo test -p lila-aot-wasm --test created_realm_array_prototype_structure
cargo test -p lila-aot-wasm --test created_realm_promise_publication_structure
cargo test -p lila-aot-wasm --lib heap::tests::realm_record_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_realm_record_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/realm_record_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact nine rows, offsets 0, 8, 16, 24, 32, 40, 48,
56 and 64, the two-scalar/seven-pointer census, typed registry order and
unchanged runtime offset consumers. At the Batch AN checkpoint, `cargo xc` is
green, the new structure target passes `4/4`, the Array and Promise Realm
neighbors pass `3/3` and `5/5`, the bounded heap owner passes `1/1`, and the
registry checks pass `2/2`. No runtime CLI, Test262 leaf or semantic golden was
required or run for this passive metadata change.
