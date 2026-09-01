# Heap weak-edge identity authority

## Closed edge identities

The passive weak-edge inventory contains exactly seven `HeapWeakEdge`
identities: the WeakMap key and value, the WeakSet value, the WeakRef target,
and the FinalizationRegistry target, holdings and unregister token.

One private exhaustive `metadata()` projection is the sole authority for each
identity's record name, slot name and `HeapWeakEdgeKind`. The ordered
`HEAP_WEAK_EDGES` registry stores only those identities. It no longer accepts a
row that can independently combine arbitrary strings with an unrelated edge
kind.

`HeapWeakEdgeKind::retention()` remains the sole retention authority. WeakMap
and WeakSet keys use `EphemeronKey`, the WeakMap value uses `EphemeronValue`,
WeakRef and FinalizationRegistry targets use `WeakTarget`, holdings use
`FinalizerHoldings`, and the unregister token uses `FinalizerToken`.

The focused recursive structure guard pins both closed domains, the exhaustive
retention and metadata projections, all seven exact mappings, registry order,
retired-row absence and the typed collector consumer.

## Inventory boundary

This changes passive Rust metadata only. It does not trace or clear a weak
target, process an ephemeron fixpoint, queue finalization cleanup, make `gc()`
executable or give current linear-memory records weak semantics.

```sh
cargo test -p lila-aot-wasm --test weak_edge_retention_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_weak_edge_registry_models_ephemerons_and_finalizers -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::weak_map_entries_are_ephemerons_not_strong_heap_edges -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::weak_ref_target_is_not_a_strong_heap_edge -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::finalization_registry_cells_keep_only_holdings_strongly_reachable -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_collector_policy_keeps_gc_builtin_unsupported_until_executable -- --exact --test-threads=1
git diff --check
```

The recursive structure guard passes `4/4`, and the adjusted named-slot guard
remains green at `3/3`. The exact weak-edge registry, WeakMap, WeakRef,
FinalizationRegistry and unsupported-collector owner witnesses each pass
`1/1`, with only the workspace's existing warnings. Targeted formatting and
diff checks pass. This invariant does not require a broad workspace or
conformance run.
