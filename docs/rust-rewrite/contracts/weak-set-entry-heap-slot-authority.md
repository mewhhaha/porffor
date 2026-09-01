# WeakSet-entry heap-slot identity authority

## Closed layout identities

The passive WeakSet entry layout contains exactly three capability-free
`WeakSetEntryHeapSlot` identities, in present-tag-payload order:

- `Present`;
- `ValueTag`;
- `ValuePayload`.

One private exhaustive `metadata()` projection is the sole authority for all
three identities' record names, slot names, offsets, widths and pointer
classifications. All three remain non-pointer 8-byte words at their existing
offsets. An arbitrary row cannot mark the weak value payload—or either scalar
word—as a strong tracing edge.

The independent `HeapWeakEdge::WeakSetValue` identity remains the semantic
retention authority. It projects `HeapWeakEdgeKind::EphemeronKey`, whose
exhaustive retention projection is `DoesNotRetain`. The layout therefore
contains no strong pointer while the weak-edge domain records why the value
does not retain its referent.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form WeakSet entry rows. The
bounded heap owner witness exercises all three projected rows through the
existing collision and record-size checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm or WeakSet behavior. It
does not execute tracing, clear a weak target, reclaim an object, collect a
cycle or implement `gc()`.

```sh
cargo test -p lila-aot-wasm --test weak_set_entry_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::weak_set_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_weak_edge_registry_models_ephemerons_and_finalizers -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_weak_set_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/weak_set_entry_heap_slot_structure.rs
git diff --check
```

The structure regression passes `4/4`; the exact layout owner and weak-edge
owner witnesses each pass `1/1`; and the adjusted collision and pointer
registry witnesses pass `2/2`, with only existing workspace warnings. The
shared `cargo xc` checkpoint, workspace formatting, diff hygiene,
module-boundary audit and task-plan audit pass. Golden and conformance
execution do not apply to this passive metadata-only closure.
