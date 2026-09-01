# Temporal.PlainTime heap-slot identity authority

## Closed layout identities

The passive Temporal.PlainTime record contains exactly six capability-free
`TemporalPlainTimeHeapSlot` identities in hour, minute, second, millisecond,
microsecond and nanosecond order.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. The fields remain scalar 8-byte words at offsets 0, 8, 16,
24, 32 and 40. An arbitrary row cannot rename a component, reorder the typed
registry or classify a numeric time field as a traced address.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Temporal.PlainTime rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

`TemporalTimeUnit` remains the runtime authority for field indexes, record
offset selection and component ranges. This invariant reorganizes passive Rust
layout metadata only. It does not change Temporal.PlainTime allocation, emitted
Wasm, time arithmetic, Intl formatting, root scanning or collector execution.

```sh
cargo test -p lila-aot-wasm --test temporal_plain_time_heap_slot_structure
cargo test -p lila-aot-wasm --test temporal_plain_time_field_authority_structure
cargo test -p lila-aot-wasm --lib heap::tests::temporal_plain_time_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_temporal_plain_time_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/temporal_plain_time_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, the six-scalar/zero-pointer census,
typed registry order and byte-untouched runtime consumers. At the Batch AO
checkpoint, `cargo xc` is green, the focused and neighboring structure targets
pass `4/4` each, the bounded owner passes `1/1`, and the registry witnesses
pass `2/2`. Runtime CLI coverage, Test262 and semantic goldens were not required
or run for this passive migration.
