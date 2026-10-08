# Heap host-boundary policy authority

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

## Closed call-scoped policy

The passive heap inventory exposes exactly one `HeapHostBoundaryPolicy`:
`ImportCallOnlyWithTransientTaggedRoots`. The variant makes both constraints
part of the representable state:

- a host import may borrow linear memory only for the dynamic extent of its
  call; and
- re-entrancy retains the typed transient tagged root source.

Two exhaustive projections derive the diagnostic name
`host-import-memory-borrow` and the typed
`HeapRootSource::HostBorrowedValues` identity. That source's existing metadata
classifies it as `HeapRootKind::TransientTaggedValues`.

The retired contract's arbitrary name, `durable_host_pointers` Boolean,
single-variant borrow-duration field and
`reentrant_imports_require_transient_roots` Boolean no longer exist. Durable
host pointers and unrooted re-entrancy therefore have no representable policy
state.

The focused structure regression pins the one-variant domain, both exhaustive
projections, exact producer, absence of the retired fields and the heap owner's
typed consumption. The root-source structure witness continues to pin the
typed `HostBorrowedValues` linkage; the collector-phase guard changes only its
ending source marker.

## Metadata boundary

This changes passive Rust metadata only. It does not alter a host import,
introduce a durable pointer, establish executable roots, change emitted Wasm or
make the collector executable.

```sh
cargo test -p lila-aot-wasm --test heap_host_boundary_structure
cargo test -p lila-aot-wasm --test heap_root_source_structure
cargo test -p lila-aot-wasm --test heap_collector_phase_structure
cargo test -p lila-aot-wasm --lib heap::tests::heap_host_boundary_is_call_scoped_and_transiently_rooted -- --exact --test-threads=1
cargo check -p lila-aot-wasm --lib
git diff --check
```

The standalone host-policy, root-source and collector-phase guards each pass
`4/4`, and targeted formatting and diff checks pass. The package owner witness
and compilation remain deferred to the shared batch checkpoint. Broad
workspace, golden and Test262 verification remain batch-level work.
