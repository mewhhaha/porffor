# Temporal.Duration heap-slot identity authority

## Closed passive layout

The passive Temporal.Duration record has exactly ten capability-free
`TemporalDurationHeapSlot` identities in years-through-nanoseconds order. All
ten fields are untraced 8-byte scalars at offsets 0 through 72. One private
exhaustive metadata projection owns every record name, slot name, offset,
width and pointer classification, and the typed registry fixes their order.

The former free-form layout could pair an arbitrary field name with the wrong
offset or pointer bit. The typed identity makes those combinations unavailable
outside its sole exhaustive projection. The domain derives and implements no
clone, copy, debug, equality, ordering, hashing or default capability.

## Passive boundary

This is a source-equivalent passive metadata migration. The former ten-row,
72-line layout has SHA-256
`a861ae7d97005dc2c761fcc16dcd56cd57bf671205b9fefaa806b04c891e4d35`.
The 134-line typed owner has SHA-256
`47ead73fde0338f65387f07bae6812c05cd049e1c8b3c14d4d93fb33ae20967f`.
It does not change Temporal.Duration allocation, field reads, arithmetic,
emitted Wasm, root scanning or collector execution. It claims
no new Temporal behavior, Test262 pass or published conformance change.

## Verification

```sh
cargo test -p lila-aot-wasm --test temporal_duration_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::temporal_duration_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the recursive
structure target passes `4/4`, the exact heap-slot identity unit passes `1/1`,
and both heap-layout registry controls pass `2/2`. No runtime CLI, Test262 leaf
or semantic golden is required for this passive metadata migration, which
claims no new Temporal behavior.
