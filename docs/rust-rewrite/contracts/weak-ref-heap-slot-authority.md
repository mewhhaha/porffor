# WeakRef heap-slot identity authority

## Closed layout identities

The passive WeakRef record layout contains exactly two capability-free
`WeakRefHeapSlot` identities, in tag-then-payload order:

- `TargetTag`;
- `TargetPayload`.

One private exhaustive `metadata()` projection is the sole authority for both
identities' record names, slot names, offsets, widths and pointer
classifications. Both remain non-pointer 8-byte words at
`HEAP_WEAK_REF_TARGET_TAG_OFFSET` and `HEAP_WEAK_REF_TARGET_PAYLOAD_OFFSET`.
An arbitrary row cannot mark either physical word as a strong tracing edge or
enter the typed registry.

The independent `HeapWeakEdge::WeakRefTarget` identity remains the semantic
retention authority. It projects `HeapWeakEdgeKind::WeakTarget`, whose
exhaustive retention projection is `DoesNotRetain`. The layout therefore
contains no strong pointer while the weak-edge domain records why the target
does not retain its referent.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form WeakRef layout rows. The
bounded heap owner witness exercises both projected rows through the existing
collision and record-size checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm or WeakRef behavior. It
does not execute tracing, clear a weak target, reclaim an object, collect a
cycle, schedule cleanup or implement `gc()`.

```sh
cargo test -p lila-aot-wasm --test weak_ref_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::weak_ref_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::weak_ref_target_is_not_a_strong_heap_edge -- --exact --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_weak_ref_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/src/lib.rs crates/lila-aot-wasm/tests/weak_ref_heap_slot_structure.rs
git diff --check
```

The structure target passes `4/4`, the exact identity/non-retention owner
witnesses pass `2/2`, and the adjusted collision/pointer registry witnesses
pass `2/2`. Only the workspace's existing warnings are emitted. Targeted
formatting and diff checks pass, and the shared `cargo xc` checkpoint is green.
Golden and conformance execution do not apply to this passive metadata-only
closure.
