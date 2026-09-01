# Set-iterator heap-slot identity authority

## Closed layout identities

The passive Set iterator layout contains exactly four capability-free
`SetIteratorHeapSlot` identities, in payload-index-kind-cursor order:

- `SetPayload`;
- `NextIndex`;
- `Kind`;
- `CursorState`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The Set payload remains the sole traced 8-byte word at offset
zero. The next index, iteration kind and cursor state remain scalar 8-byte
words at offsets 8, 16 and 24. An arbitrary row cannot omit the Set edge or
trace iterator control state.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Set iterator rows. The bounded
heap owner witness asserts every projected field and retains the existing
collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm, Set iteration behavior
or collector execution.

```sh
cargo test -p lila-aot-wasm --test set_iterator_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::set_iterator_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_set_iterator_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/set_iterator_heap_slot_structure.rs
git diff --check
```

The structure regression passes `4/4`, the exact owner witness passes `1/1`
and the adjusted collision and pointer registry witnesses pass `2/2`, with only
existing workspace warnings. The shared `cargo xc`, workspace formatting,
diff, module-boundary and task-plan checks are green. Golden and conformance
execution do not apply to this passive metadata-only closure.
