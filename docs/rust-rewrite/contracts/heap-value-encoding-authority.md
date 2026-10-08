# Heap value-encoding authority

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

## Closed value identities

The passive value ABI inventory now contains exactly twelve
`HeapValueEncoding` identities: Undefined, Null, Boolean, Number, String,
Symbol, Object, Array, Function, Arguments, BigInt and Dynamic. Four exhaustive
projections derive each identity's `ValueKind`, payload representation, Number
bit-preservation claim and arbitrary-precision readiness.

The ordered `HEAP_VALUE_ENCODINGS` registry stores only those identities. It no
longer accepts a row that independently combines a kind, payload and two
Booleans. Adding an identity requires explicit arms in all four projections,
and no wildcard can silently assign default semantics.

The exact payload mappings are unchanged. Number remains the sole
`Ieee754Bits` identity and the sole identity preserving Number bits. BigInt
remains `I64TemporaryOrHeapPointer` and explicitly not ready for arbitrary
precision. The unused standalone `I64Temporary` payload variant had no producer
or consumer and is removed.

The focused recursive structure guard pins both closed domains, all four
exhaustive projections, every exact mapping, registry order, retired-row
absence and the heap owner's Number and BigInt witnesses.

## Inventory boundary

This changes passive Rust metadata only. It does not change the emitted tagged
value ABI, heap allocation, Number execution, BigInt execution or emitted Wasm.
It does not claim the BigInt storage gap is closed.

```sh
cargo test -p lila-aot-wasm --test heap_value_encoding_structure
cargo test -p lila-aot-wasm --test heap_collector_phase_structure
cargo test -p lila-aot-wasm --test heap_host_boundary_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_value_encoding_registry_covers_ecmascript_language_types -- --exact --test-threads=1
cargo check -p lila-aot-wasm --lib
git diff --check
```

The standalone value-encoding, collector-phase and host-policy guards each pass
`4/4`; the exact package owner witness passes `1/1` with the workspace's
existing warnings; and targeted formatting and diff checks pass. Broad
workspace, golden and Test262 verification remain batch-level work.
