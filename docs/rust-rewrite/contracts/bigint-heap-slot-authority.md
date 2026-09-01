# BigInt heap-slot identity authority

## Closed layout identities

The passive BigInt record layout contains exactly four capability-free
`BigIntHeapSlot` identities in sign, limbs-pointer, limbs-length and
limbs-capacity order:

- `Sign`;
- `LimbsPointer`;
- `LimbsLength`;
- `LimbsCapacity`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The sign remains a scalar 8-byte word at offset zero. The
limbs address remains the sole pointer-classified word at offset 8, followed
by scalar length and capacity words at offsets 16 and 24. The existing
`LinearSideStorage::BigIntLimbs` identity remains the separate authority for
the retained non-reference limb element representation.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form BigInt record rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change BigInt allocation, emitted Wasm, sign decoding, limb storage,
arbitrary-precision readiness or collector execution.

```sh
cargo test -p lila-aot-wasm --test bigint_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::bigint_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_bigint_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/bigint_heap_slot_structure.rs
git diff --check
```

The recursive structure target passes `4/4`, the exact heap owner witness
passes `1/1`, and the collision/pointer registry filter passes `2/2`. The
shared `cargo xc` checkpoint is green. This is a passive layout migration, so
no semantic golden or Test262 rerun was performed.
