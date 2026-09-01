# WeakMap-entry heap-slot identity authority

## Closed layout identities

The passive WeakMap entry layout contains exactly five capability-free
`WeakMapEntryHeapSlot` identities, in present-key-tag-key-payload-value-tag-
value-payload order:

- `Present`;
- `KeyTag`;
- `KeyPayload`;
- `ValueTag`;
- `ValuePayload`.

One private exhaustive `metadata()` projection is the sole authority for all
five identities' record names, slot names, offsets, widths and pointer
classifications. All five remain non-pointer 8-byte words at their existing
offsets. An arbitrary row cannot turn either weak payload—or either tag or the
presence word—into a strong tracing edge.

The independent `HeapWeakEdge::{WeakMapKey, WeakMapValue}` identities remain
the semantic retention authority. They project `EphemeronKey` and
`EphemeronValue`; their exhaustive retention projection respectively does not
retain the key and retains the value only when its ephemeron key is reachable.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form WeakMap entry rows. The bounded
heap owner witness asserts every projected field and retains the existing
collision, record-size and ephemeron checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm, WeakMap behavior or
collector execution. It does not implement ephemeron processing or make
current weak records weak in practice.

```sh
cargo test -p lila-aot-wasm --test weak_map_entry_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::weak_map_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::weak_map_entries_are_ephemerons_not_strong_heap_edges -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_has_no_slot_collisions -- --exact --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_weak_map_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/weak_map_entry_heap_slot_structure.rs
git diff --check
```

The structure regression passes `4/4`; the exact layout owner, ephemeron
authority and collision witnesses each pass `1/1`, with only existing workspace
warnings. The shared `cargo xc`, workspace formatting, diff-hygiene,
module-boundary and task-plan checks pass. Golden and conformance execution do
not apply to this passive metadata-only change.
