# Temporal.PlainDateTime heap-slot identity authority

## Closed passive layout

The passive Temporal.PlainDateTime record has exactly ten capability-free
`TemporalPlainDateTimeHeapSlot` identities. Nine scalar ISO date/time fields
occupy offsets 0 through 64 in 8-byte steps; the traced calendar payload is at
offset 72. One private exhaustive metadata projection owns every record name,
slot name, offset, width and pointer classification, and the typed registry
fixes their order.

The former free-form layout could pair an arbitrary field name with the wrong
offset or pointer bit. The typed identity makes those combinations unavailable
outside its sole exhaustive projection. The domain derives and implements no
clone, copy, debug, equality, ordering, hashing or default capability.

## Passive boundary

This is a source-equivalent passive metadata migration. The former ten-row
layout has SHA-256
`35c1a0b84dcfe732f5b97cb89f0c728d1e5bb23c833d6f7b7fd732ffd778cd0d`.
The 135-line typed owner has SHA-256
`d02da5883f159b9e2379ca6327fd1b998c024e2b024b83f771e95a61148b647d`.
It does not change Temporal.PlainDateTime allocation, field reads, arithmetic,
calendar behavior, emitted Wasm, root scanning or collector execution. It
claims no new Temporal behavior, Test262 pass or published conformance change.

## Verification

```sh
cargo test -p lila-aot-wasm --test temporal_plain_date_time_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::temporal_plain_date_time_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Batch AP verification is green on 2026-08-28: the recursive structure target
passes `4/4`, the exact heap-slot identity unit passes `1/1`, both heap-layout
registry controls pass `2/2`, and `cargo xc` is green. No runtime CLI, Test262
leaf or semantic golden is required for this passive migration.
