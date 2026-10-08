# Linear side-storage identity and element authority

Current dry source — 2026-10-05: the passive raw-representation source
mirror below is retired from the atomic GC draft. Its registry spelling,
offset descriptions and historical checks do not verify actual GC references,
roots, host values or weak reachability. Current semantic GC source owns those
operations through typed fields and complete value/Realm/completion owners.
Final provider retirement and the remaining native source families are still
in progress; the earlier descriptions, commands and results are historical.

Compilation and behavioral verification remain deferred to the complete task
batch. Weak reachability keeps its explicit [unavailable facility boundary](weak-unavailable-runtime-boundary.md). No current green result or weak
retention facility is supplied by this source retirement.

## Closed storage identities

The retained linear-memory side-storage registry contains exactly three
capability-free `LinearSideStorage` identities:

- `ArrayBufferBackingStore`;
- `StringCodeUnits`;
- `BigIntLimbs`.

One private exhaustive `metadata()` projection is the sole authority for each
identity's record name, length source and `LinearSideStorageElement`. An
ArrayBuffer backing store therefore cannot compile with the BigInt length
source or string element kind, and an arbitrary fourth row cannot bypass the
closed domain.

The element mappings remain:

- `Byte` for ArrayBuffer backing stores;
- `Utf16CodeUnit` for JavaScript string storage;
- `BigIntLimb` for arbitrary-precision magnitude storage.

One exhaustive projection derives byte widths `1`, `2` and `8`. A second
exhaustive projection classifies every variant as non-reference storage. Adding
an element kind requires an explicit choice in both projections.

The focused recursive structure regression pins both exact domains, the sole
metadata projection, both element projections, every exact mapping, registry
order, retired-row absence and the heap owner import. The heap owner witness
exercises all identities through their typed projections.

## Side-storage boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change any linear-memory address, allocation, emitted Wasm, string encoding,
ArrayBuffer behavior or BigInt representation. It does not migrate semantic
objects to Wasm GC or implement reclamation, cyclic collection or weak
reachability.

```sh
cargo test -p lila-aot-wasm --test heap_side_storage_element_structure
cargo test -p lila-aot-wasm --test heap_value_encoding_structure
cargo test -p lila-aot-wasm --lib heap::tests::linear_side_storage_identities_own_metadata_and_element_semantics -- --exact --test-threads=1
git diff --check
```

The strengthened side-storage and adjusted value-encoding guards each pass
`4/4`. The exact heap owner witness passes `1/1` with only the workspace's
existing warnings, and targeted formatting and diff checks pass. The shared
`cargo xc` checkpoint is green. Golden and Test262 verification do not apply to
this passive metadata-only closure.
