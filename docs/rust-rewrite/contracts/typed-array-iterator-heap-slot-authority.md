# TypedArray iterator heap-slot identity authority

## Closed layout identities

The passive TypedArray iterator record contains exactly four capability-free
`TypedArrayIteratorHeapSlot` identities in typed-array-payload, next-index,
kind and done order:

- `TypedArrayPayload`;
- `NextIndex`;
- `Kind`;
- `Done`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The TypedArray payload remains the sole pointer-classified
8-byte word at offset zero. The next index, iterator kind and done state remain
scalar 8-byte words at offsets 8, 16 and 24. An arbitrary row cannot omit the
iterated TypedArray from the pointer census or trace iterator scalar state.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form TypedArray iterator rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change TypedArray iterator allocation, emitted Wasm, iterator stepping,
detachment or resizable-buffer semantics, or collector execution.

```sh
cargo test -p lila-aot-wasm --test typed_array_iterator_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::typed_array_iterator_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_typed_array_iterator_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/typed_array_iterator_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact four rows, the one-pointer/three-scalar census,
typed registry order and unchanged runtime offset consumers. The recursive
structure target passes `4/4`, the exact heap owner witness passes `1/1`, the
collision/pointer registry filter passes `2/2`, and the shared `cargo xc`
checkpoint is green. No semantic golden or Test262 rerun applies to this passive
layout-only migration.
