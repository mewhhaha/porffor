# Required heap-collector phase authority

## Closed required-phase domain

The passive collector contract accepts exactly eight
`RequiredHeapCollectorPhase` variants, in execution order: stop the world, scan
roots, mark the strong graph, process ephemerons, clear weak references, queue
finalizers, sweep and resume.

One exhaustive `name()` projection is the sole authority for the eight
diagnostic names. `HeapCollectorPolicy::required_phases()` returns only a slice
of this required-phase type. A phase can therefore no longer combine an
arbitrary name with a different kind or independently set
`required_for_gc_builtin` to false. Adding a phase requires an explicit
diagnostic-name arm.

The focused structure regression pins the exact domain, exhaustive name
projection, exact ordered registry, absence of the retired phase struct and
Boolean, and the collector contract's typed field and producer. The neighboring
weak-edge guard retains its exact seven-row inventory with only its ending
source marker updated.

## Metadata boundary

This transition changes passive Rust metadata only. It does not implement any
phase, trace a root, clear a weak reference, queue a finalizer, expose `gc()` or
change emitted Wasm.

```sh
cargo test -p lila-aot-wasm --test heap_collector_phase_structure
cargo test -p lila-aot-wasm --test weak_edge_retention_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_collector_policy_requires_all_gc_builtin_phases -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_collector_policy_keeps_gc_builtin_unsupported_until_executable -- --exact --test-threads=1
cargo check -p lila-aot-wasm --lib
git diff --check
```

The standalone phase and adjusted weak-edge guards each pass `4/4`, and
targeted formatting and diff checks pass. The exact phase-inventory and
unsupported-policy owner witnesses each pass `1/1` with only the workspace's
existing warnings. Broad workspace, golden and Test262 verification remain
batch-level work.
