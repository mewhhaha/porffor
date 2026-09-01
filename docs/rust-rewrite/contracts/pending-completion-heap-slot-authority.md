# Pending-completion heap-slot identity authority

## Closed layout identities

The passive pending-completion record contains exactly five capability-free
`PendingCompletionHeapSlot` identities in next, payload, tag, kind and auxiliary
order:

- `Next`;
- `Payload`;
- `Tag`;
- `Kind`;
- `Aux`.

One private exhaustive `metadata()` projection is the sole authority for all
five identities' record names, slot names, offsets, widths and pointer
classifications. The next link and completion payload remain pointer-classified
8-byte words at offsets 0 and 8. The tag, kind and auxiliary completion state
remain scalar 8-byte words at offsets 16, 24 and 32. An arbitrary row cannot
omit either retained edge or trace one of the scalar completion words.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form pending-completion rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change pending-completion allocation, finally restoration, async disposal,
emitted Wasm, root scanning or collector execution.

```sh
cargo test -p lila-aot-wasm --test pending_completion_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::pending_completion_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_pending_completion_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/pending_completion_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact five rows, the three-scalar/two-pointer census,
typed registry order and unchanged runtime offset consumers. At the 2026-08-28
Batch Z checkpoint, `cargo xc` is green, the recursive structure target passes
`4/4`, the exact heap owner witness passes `1/1`, and the collision/pointer
registry filter passes `2/2`. Runtime execution and semantic goldens were not
rerun; no Test262 claim is made by this passive layout-only migration.
