# Private-environment heap-slot identity authority

## Closed layout identities

The passive private-environment layout contains exactly two capability-free
`PrivateEnvironmentHeapSlot` identities, in parent-then-class-scope order:

- `Parent`;
- `ClassScope`.

One private exhaustive `metadata()` projection is the sole authority for both
identities' record names, slot names, offsets, widths and pointer
classifications. `Parent` remains the traced 8-byte edge at
`HEAP_PRIVATE_ENV_PARENT_OFFSET`. `ClassScope` remains the scalar 8-byte class
identity at `HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET`. An arbitrary row cannot swap
those meanings, omit the parent edge from tracing or scan the class-scope
number as a pointer.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form private-environment layout
rows. The bounded heap owner witness exercises both projected rows through the
existing collision and record-size checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm, private-name lookup or
class semantics. It does not migrate semantic values to Wasm GC, execute
tracing, reclaim an object, collect a cycle or implement `gc()`.

```sh
cargo test -p lila-aot-wasm --test private_environment_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::private_environment_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
rustfmt --check --config skip_children=true crates/lila-aot-wasm/src/heap_private_environment_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/src/lib.rs crates/lila-aot-wasm/tests/private_environment_heap_slot_structure.rs
git diff --check
```

The structure target passes `4/4`, the exact identity owner witness passes
`1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`. Only
the workspace's existing warnings are emitted. Targeted formatting with child
module traversal disabled and diff checks pass, and the shared `cargo xc`
checkpoint is green. Golden and conformance execution do not apply to this
passive metadata-only closure.
