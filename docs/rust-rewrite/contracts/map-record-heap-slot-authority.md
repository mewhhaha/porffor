# Map-record heap-slot identity authority

## Closed layout identities

The passive Map record layout contains exactly four capability-free
`MapRecordHeapSlot` identities, in entries-pointer, entries-length,
entries-capacity and live-count order:

- `EntriesPointer`;
- `EntriesLength`;
- `EntriesCapacity`;
- `LiveCount`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The entries pointer remains the sole traced 8-byte word at
offset zero. The entries length, capacity and live count remain scalar 8-byte
words at offsets 8, 16 and 24. The closed Map entry layout independently keeps
its key and value payloads as the two traced entry edges.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Map record rows. The bounded
heap owner witness asserts every projected field and retains the existing
collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm, Map behavior or
collector execution.

```sh
cargo test -p lila-aot-wasm --test map_record_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::map_record_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_map_record_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/map_record_heap_slot_structure.rs
git diff --check
```

The recursive structure target passes `4/4`, the exact identity owner witness
passes `1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`
with only existing workspace warnings. Targeted formatting and diff checks
pass. The shared `cargo xc`, formatting, diff, module-boundary and task-plan
checks are green. Golden and conformance execution do not apply to this passive
metadata-only closure.
