# Async-generator-object heap-slot identity authority

## Closed layout identity

The passive async-generator object layout contains exactly one capability-free
`AsyncGeneratorObjectHeapSlot::Activation` identity. One private exhaustive
`metadata()` projection is the sole authority for its
`async-generator-object` record name, `activation` slot name,
`HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET`, 8-byte width and traced-pointer
classification. The typed registry contains exactly that identity.

An arbitrary row can no longer rename, resize, reorder or mark the activation
edge scalar. The runtime allocation and access paths continue to use the same
activation offset; the identity owns passive tracing metadata only.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs a free-form async-generator object row.
The bounded heap owner witness asserts every projected field and retains the
existing collision, header-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change allocation, emitted Wasm, async-generator behavior or collector
execution.

```sh
cargo test -p lila-aot-wasm --test async_generator_object_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::async_generator_object_heap_slot_identity_owns_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_async_generator_object_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/async_generator_object_heap_slot_structure.rs
git diff --check
```

The recursive structure target passes `4/4`, the exact identity owner witness
passes `1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`
with only existing workspace warnings. The shared `cargo xc`, formatting,
diff, module-boundary and task-plan checks are green. Golden and conformance
execution do not apply to this passive metadata-only closure.
