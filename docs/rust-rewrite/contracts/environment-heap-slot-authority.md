# Environment heap-slot identity authority

## Closed layout identities

The passive environment layout contains exactly three capability-free
`EnvironmentHeapSlot` identities in parent, binding-tag and binding-payload
order:

- `Parent`;
- `BindingTag`;
- `BindingPayload`.

One private exhaustive `metadata()` projection is the sole authority for the
exact record names, slot names, offsets, widths and pointer classifications.
The environment parent remains a traced 8-byte word at `ENV_PARENT_OFFSET`.
Each repeated binding retains a scalar tag at `ENV_SLOT_TAG_OFFSET` followed by
a traced payload at `ENV_SLOT_PAYLOAD_OFFSET`. An arbitrary row cannot mark the
parent or payload scalar, trace the tag, or exchange their identities.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form environment rows. The bounded
heap owner witness asserts every projected field and retains the existing
collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change environment allocation, emitted Wasm, binding access or collector
execution.

```sh
cargo test -p lila-aot-wasm --test environment_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::environment_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_environment_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/environment_heap_slot_structure.rs
git diff --check
```

The recursive structure target passes `4/4`, the exact owner witness passes
`1/1`, and the collision/pointer registry witnesses pass `2/2`. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.
