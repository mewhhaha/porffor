# FinalizationRegistry-cell heap-slot identity authority

Current dry source — 2026-10-05: the raw-offset source-mirror target below
is retired from the atomic semantic GC draft. It checked passive layout
spelling, offsets and source occurrence counts. The current GC authoring uses
typed registry fields and complete semantic value owners; these earlier
mirrors cannot verify that representation or its behavior. Historical source
descriptions, commands and results below retain their original scope.

The full GC source cutover, remaining Temporal and AsyncDisposableStack
authoring, compiler checks and runtime verification remain pending. See the
[current value/heap architecture](../value-heap-gc.md). Weak reachability retains
its explicit [unavailable facility boundary](weak-unavailable-runtime-boundary.md).
No verification or conformance result is inferred from this retirement.

Current source — 2026-10-03: actual weak operations follow the
[unavailable facility boundary](weak-unavailable-runtime-boundary.md). Active
strong-retaining weak producers are retired. Installed intrinsic publication
and earlier JavaScript validation remain observable; passive layout/edge rows
are inventory only. Producer, valid-instance and verification descriptions
below retain their earlier source scope and do not verify the current batch.
Real weak reachability and semantic GC remain open; current controls are unexecuted.

## Closed layout identities

The passive FinalizationRegistry cell contains exactly seven capability-free
`FinalizationRegistryCellHeapSlot` identities in state, target-tag,
target-payload, holdings-tag, holdings-payload, unregister-token-tag and
unregister-token-payload order.

One private exhaustive `metadata()` projection is the sole authority for all
seven identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. State is scalar at offset
zero. Target tag and payload are scalar at offsets 8 and 16. Holdings tag is
scalar at offset 24, while holdings payload is the sole pointer-classified word
at offset 32. Unregister-token tag and payload are scalar at offsets 40 and 48.

This pointer census is a retention invariant. The closed weak-edge registry
owns the target and unregister token as non-retaining edges, so their payload
words must not become strong layout pointers. Holdings alone remains strongly
reachable until cleanup. An arbitrary row can no longer reverse those roles,
trace a tag or omit the holdings edge from the strong pointer census.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form FinalizationRegistry cell rows.
The bounded heap owner witness asserts every projected field. The existing
retention witness keeps the layout classifications aligned with the three
closed FinalizationRegistry weak-edge identities.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change cell allocation or growth, registration or unregistration, persisted
cell-state admission, weak reachability, cleanup scheduling, emitted Wasm or
collector execution.

```sh
cargo test -p lila-aot-wasm --test finalization_registry_cell_heap_slot_structure
cargo test -p lila-aot-wasm --test finalization_registry_cell_state_structure
cargo test -p lila-aot-wasm --lib heap::tests::finalization_registry_cell_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::finalization_registry_cells_keep_only_holdings_strongly_reachable -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_finalization_registry_cell_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/finalization_registry_cell_heap_slot_structure.rs crates/lila-aot-wasm/tests/finalization_registry_cell_state_structure.rs
git diff --check
```

Dry source review pins the exact seven rows, the six-scalar/one-pointer census,
typed registry order, weak-edge relation and unchanged runtime offset consumers.
At the shared Batch AE checkpoint, `cargo xc` is green. The cell-layout, cell-
state and weak-edge-retention structure targets pass `4/4` each (`12/12`
combined). The exact typed-owner witness passes `1/1`, the
`heap_layout_registry_` filter passes `2/2`, and the only-holdings retention
witness passes `1/1` (`4/4` combined). The FinalizationRegistry runtime builtin
remains unchanged. No CLI, Test262 or semantic-golden verification applies to
this passive layout-only migration, and none was run.
