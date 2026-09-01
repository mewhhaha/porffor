# Temporal.Instant heap-slot identity authority

## Closed layout identities

The passive Temporal.Instant layout registry contains exactly two
capability-free `TemporalInstantHeapSlot` identities, in tag-then-payload
order:

- `EpochNanosecondsTag`;
- `EpochNanosecondsPayload`.

One private exhaustive `metadata()` projection is the sole authority for each
identity's record name, slot name, offset, width and pointer classification.
The tag remains the scalar 8-byte word at
`HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET`. The payload remains the
strong-reference 8-byte word at
`HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET`. An arbitrary row
cannot misspell either slot, reverse its pointer classification or enter the
typed registry.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves registry order and proves that no
second Rust source constructs a free-form `temporal-instant-record` row. The
bounded heap owner witness exercises both identities through the projected
layout and retains the existing collision and record-size checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm or Temporal semantics.
It does not migrate semantic values to Wasm GC, execute tracing, reclaim an
object, collect a cycle or provide weak reachability.

```sh
cargo test -p lila-aot-wasm --test temporal_instant_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::temporal_instant_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_temporal_instant_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/src/lib.rs crates/lila-aot-wasm/tests/temporal_instant_heap_slot_structure.rs
git diff --check
```

The structure target passes `4/4`, the exact identity owner witness passes
`1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`. Only
the workspace's existing warnings are emitted. Targeted formatting and diff
checks pass, and the shared `cargo xc` checkpoint is green. Golden and
conformance execution do not apply to this passive metadata-only closure.
