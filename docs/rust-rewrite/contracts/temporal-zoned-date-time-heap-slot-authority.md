# Temporal.ZonedDateTime heap-slot identity authority

## Closed layout identities

The passive Temporal.ZonedDateTime record contains exactly six capability-free
`TemporalZonedDateTimeHeapSlot` identities in epoch-nanoseconds-tag,
epoch-nanoseconds-payload, time-zone-tag, time-zone-payload, calendar-tag and
calendar-payload order.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Epoch nanoseconds tag and
payload occupy offsets 0 and 8, time-zone tag and payload occupy offsets 16 and
24, and calendar tag and payload occupy offsets 32 and 40. The three tags
remain scalar while all three payloads remain pointer-classified.

This three-scalar/three-pointer census is a retention invariant. A
ZonedDateTime record must keep its epoch-nanoseconds, time-zone and calendar
payloads visible to tracing while none of their tags may be scanned as an
address. An arbitrary row can no longer reverse either side of that relation or
reorder a tag independently of its payload.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form ZonedDateTime rows. The bounded
heap owner witness asserts every projected field. The existing
`TemporalInstantHeapSlot`, `TemporalPlainDateHeapSlot` and ZonedDateTime
algorithm enums remain independent semantic authorities rather than duplicate
layout owners.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change ZonedDateTime allocation or access, epoch-nanoseconds representation,
time-zone or calendar semantics, emitted Wasm, root scanning or collector
execution. All Temporal runtime offset consumers remain unchanged.

```sh
cargo test -p lila-aot-wasm --test temporal_zoned_date_time_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::temporal_zoned_date_time_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_temporal_zoned_date_time_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/temporal_zoned_date_time_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, offsets 0, 8, 16, 24, 32 and 40,
the three-scalar/three-pointer census, typed registry order and unchanged
runtime offset consumers. Shared `cargo xc` passes, the recursive structure
target passes `4/4`, the exact heap owner passes `1/1`, and the registry
filters pass `2/2`. No runtime CLI, Test262 or semantic-golden verification is
needed for this passive metadata change with byte-untouched Temporal consumers.
