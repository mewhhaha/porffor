# WeakSet-record heap-slot identity authority

## Closed layout identities

The passive WeakSet record layout contains exactly four capability-free
`WeakSetRecordHeapSlot` identities, in entries-pointer, entries-length,
entries-capacity and live-count order:

- `EntriesPointer`;
- `EntriesLength`;
- `EntriesCapacity`;
- `LiveCount`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The entries-storage pointer remains the sole traced 8-byte
word at offset zero. The length, capacity and live count remain scalar 8-byte
words at offsets 8, 16 and 24.

Storage reachability remains distinct from weak value retention: the record
keeps its entries storage reachable, while the closed WeakSet entry layout has
no strong pointer and `HeapWeakEdge::WeakSetValue` remains the semantic weak
edge authority. An arbitrary record row cannot omit the storage edge or trace
collection accounting state.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order, verifies the
storage/weak-value distinction and excludes a second free-form WeakSet record
producer. The bounded heap owner witness asserts every projected field and
retains the existing collision and record-size checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm, WeakSet behavior or
collector execution. It does not implement weak clearing or make current weak
records weak in practice.

```sh
cargo test -p lila-aot-wasm --test weak_set_record_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::weak_set_record_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_has_no_slot_collisions -- --exact --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_weak_set_record_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/weak_set_record_heap_slot_structure.rs
git diff --check
```

The structure regression passes `4/4`, and the exact owner and collision
witnesses each pass `1/1`, with only existing workspace warnings. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.
