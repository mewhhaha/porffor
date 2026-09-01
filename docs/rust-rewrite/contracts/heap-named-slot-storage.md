# Heap named-slot family and storage authority

## Closed layout families

The passive inventory contains exactly six capability-free
`HeapNamedSlotFamily` identities: ArrayIterator, StringIterator,
RegExpStringIterator, IteratorHelper, IteratorConcatState and IteratorZipState.
One exhaustive `slots()` projection is the sole authority mapping each family
to its named-slot slice.

The typed `HEAP_NAMED_SLOT_FAMILIES` registry preserves the exact Array,
String, RegExp, Helper, Concat, Zip order. It no longer accepts an arbitrary or
ad-hoc `&[HeapNamedSlot]` slice.

## Closed storage class

Every registered named iterator slot chooses exactly one
`HeapNamedSlotStorage` variant:

- `StrongReference` means the slot owns a strong target and tracing scans it;
- `Scalar` means the slot has no reference target and tracing does not scan it.

`HeapNamedSlot` stores that enum rather than independently writable strength
and tracing booleans. Two exhaustive projections derive both meanings from the
same value. A row therefore cannot claim that a scalar or weak target is
scanned, or that a strong target is omitted from tracing. Adding another
storage class requires an explicit choice in both projections.

The six registered families contain 50 rows: 30 strong references and 20
scalar slots. The focused structure regression pins both closed domains, all
three exhaustive projections, the enum-owned record shape, complete producer
census, exact family mappings and registry order.

## Inventory boundary

This changes passive Rust heap metadata only. It does not emit tracing code,
implement collection, introduce weak reachability, alter an object layout or
change emitted Wasm. The existing semantic unit witnesses continue to verify
the iterator, zip-state and concat-state GC-edge classifications.

```sh
cargo test -p lila-aot-wasm --test heap_named_slot_storage_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_named_slot_registry_marks_iterator_references -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::iterator_zip_state_slots_have_expected_gc_edges -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::iterator_concat_state_slots_have_expected_gc_edges -- --exact --test-threads=1
git diff --check
```

The strengthened structure guard passes `4/4`, and the exact named-slot
registry, zip-state and concat-state owner witnesses each pass `1/1` with only
the workspace's existing warnings. Targeted formatting and diff checks pass,
and the shared `cargo xc` checkpoint is green. Golden and conformance
verification do not apply to this passive metadata-only closure.

## Non-claims

This invariant does not close T05's collector, root-lifecycle, heap migration,
weak collection, `WeakRef` or `FinalizationRegistry` work. Broad workspace,
golden and conformance verification remain batch-level work.
