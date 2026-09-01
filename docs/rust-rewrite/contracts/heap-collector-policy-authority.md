# Heap collector-policy authority

## Closed passive policy

The passive heap inventory selects exactly one
`HeapCollectorPolicy::NonMovingMetadataChecked` identity. Six exhaustive
no-wildcard projections own its diagnostic name, movement behavior, root
sources, weak edges, required phases and executable state.

The policy remains named `non-moving-tracing-collector`, does not move objects,
uses the exact `HEAP_ROOT_SOURCES`, `HEAP_WEAK_EDGES` and
`REQUIRED_HEAP_COLLECTOR_PHASES` registries, and is not executable. The old
contract could independently combine an arbitrary name, movement Boolean,
capability and registry slices; those fields and the unused capability states
no longer exist.

Advancing collection now requires adding an explicit policy identity and
handling it in every projection. Flipping one capability field cannot make
`gc()` appear executable while roots, weak edges or phases remain disconnected.

The focused recursive structure guard pins the exact capability-free domain,
all six projections, registry identities, heap delegation and the host-GC
unsupported boundary.

## Passive boundary

This changes passive Rust metadata only. It does not implement tracing,
relocation, reclamation, ephemeron processing, weak clearing, finalization
cleanup or executable `gc()`. The host builtin continues to emit its explicit
unsupported throw.

```sh
cargo test -p lila-aot-wasm --test heap_collector_policy_structure
cargo test -p lila-aot-wasm --test heap_collector_phase_structure
cargo test -p lila-aot-wasm --test weak_edge_retention_structure
cargo test -p lila-aot-wasm --test heap_named_slot_storage_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_collector_policy_requires_all_gc_builtin_phases -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_collector_policy_keeps_gc_builtin_unsupported_until_executable -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib tests::supports_host_gc_builtin_as_explicit_unsupported_throw -- --exact --test-threads=1
git diff --check
```

The policy, phase and weak-edge guards each pass `4/4`, and the adjusted
named-slot guard remains green at `3/3`. The exact phase inventory,
unsupported-policy and emitted host-GC throw witnesses each pass `1/1`, with
only the workspace's existing warnings. Targeted formatting and diff checks
pass. No broad workspace or conformance run was performed for this passive
transition.
