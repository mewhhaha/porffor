# Temporal.PlainDate heap-slot identity authority

## Closed layout identities

The passive Temporal.PlainDate record contains exactly four capability-free
`TemporalPlainDateHeapSlot` identities in ISO-year, ISO-month, ISO-day and
calendar-payload order:

- `IsoYear`;
- `IsoMonth`;
- `IsoDay`;
- `CalendarPayload`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The ISO year, month and day remain scalar 8-byte words at
offsets 0, 8 and 16. The calendar payload remains the sole pointer-classified
8-byte word at offset 24. An arbitrary row cannot trace a numeric date field or
omit the calendar edge from the pointer census.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Temporal.PlainDate rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change Temporal.PlainDate allocation, emitted Wasm, date or calendar semantics,
or collector execution.

```sh
cargo test -p lila-aot-wasm --test temporal_plain_date_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::temporal_plain_date_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_temporal_plain_date_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/temporal_plain_date_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact four rows, the three-scalar/one-pointer census,
typed registry order and unchanged runtime offset consumers. At the 2026-08-28
Batch U checkpoint, the structure target passed `4/4`, the exact heap owner
witness passed `1/1`, the collision/pointer registry filter passed `2/2`, and
the shared `cargo xc` gate was green. No semantic golden or Test262 rerun
applies to this passive layout-only migration.
