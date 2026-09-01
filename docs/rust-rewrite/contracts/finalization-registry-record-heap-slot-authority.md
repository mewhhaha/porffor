# FinalizationRegistry-record heap-slot identity authority

## Closed layout identities

The passive FinalizationRegistry record contains exactly five capability-free
`FinalizationRegistryRecordHeapSlot` identities in cleanup-callback-tag,
cleanup-callback-payload, cells-pointer, cells-length and cells-capacity order:

- `CleanupCallbackTag`;
- `CleanupCallbackPayload`;
- `CellsPointer`;
- `CellsLength`;
- `CellsCapacity`.

One private exhaustive `metadata()` projection is the sole authority for all
five identities' record names, slot names, offsets, widths and pointer
classifications. The cleanup callback tag remains a scalar 8-byte word at
offset zero. Its payload and the cells-storage pointer remain pointer-classified
8-byte words at offsets 8 and 16. The cells length and capacity remain scalar
8-byte words at offsets 24 and 32. An arbitrary row cannot trace the tag or
accounting state or omit either strong record edge from the pointer census.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form FinalizationRegistry record
rows. The bounded heap owner witness asserts every projected field and retains
the existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change allocation, cleanup callback invocation, cell registration or removal,
weak-edge retention, cleanup scheduling, emitted Wasm or collector execution.

```sh
cargo test -p lila-aot-wasm --test finalization_registry_record_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::finalization_registry_record_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_finalization_registry_record_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/finalization_registry_record_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact five rows, the three-scalar/two-pointer census,
typed registry order and unchanged runtime offset consumers. At the shared
Batch AD checkpoint, `cargo xc` is green, the recursive structure target passes
`4/4`, the exact heap-owner witness passes `1/1`, the `heap_layout_registry_`
filter passes `2/2`, and
`finalization_registry_cells_keep_only_holdings_strongly_reachable` passes
`1/1`. The FinalizationRegistry runtime builtin remains unchanged. No CLI,
Test262 or semantic-golden verification applies to this passive layout-only
migration, and none was run.
