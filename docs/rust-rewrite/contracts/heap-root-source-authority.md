# Heap root-source authority

## Closed source identity

The passive collector inventory names exactly seven `HeapRootSource` variants:
Realm globals, active-frame locals, lexical environments, completion records,
the function table, host-borrowed values and pending jobs. Each source selects
one of three `HeapRootKind` variants:

- `PersistentNonTagged`;
- `PersistentTaggedValues`;
- `TransientTaggedValues`.

One private exhaustive `metadata()` projection is the sole authority for each
source's diagnostic name, owner and root kind. Public crate-local accessors only
read that metadata. The inventory therefore cannot independently combine
tagged-value and transient Booleans, and adding a source requires an explicit
metadata arm.

`HeapHostBoundaryPolicy` projects the typed
`HeapRootSource::HostBorrowedValues` identity. An arbitrary string can no longer
silently misspell or drift away from the registered host root.

The focused structure regression pins the two closed domains, all seven exact
metadata meanings, the exhaustive projection, one occurrence of every source
in the registry and the typed host-boundary producer.

## Inventory boundary

This is passive Rust metadata only. It does not trace a root, add a safepoint,
change emitted Wasm, make the collector executable or establish semantic roots
for active calls, exceptions, suspended frames or jobs.

```sh
cargo test -p lila-aot-wasm --test heap_root_source_structure
cargo test -p lila-aot-wasm --test heap_named_slot_storage_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_root_registry_covers_gc_safepoint_sources -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_host_boundary_is_call_scoped_and_transiently_rooted -- --exact --test-threads=1
cargo check -p lila-aot-wasm --lib
git diff --check
```

The standalone root-source structure guard passes `4/4`, the adjusted
named-slot structure guard remains green at `3/3`, and targeted formatting and
diff checks pass. Package owner witnesses and compilation remain deferred to
the shared batch checkpoint. Broad workspace, golden and Test262 verification
remain batch-level work.
