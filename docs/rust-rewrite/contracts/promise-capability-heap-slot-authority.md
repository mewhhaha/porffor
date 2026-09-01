# Promise-capability heap-slot identity authority

## Closed layout identities

The passive Promise capability record contains exactly six capability-free
`PromiseCapabilityHeapSlot` identities in promise-tag, promise-payload,
resolve-tag, resolve-payload, reject-tag and reject-payload order.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. The Promise tag and
payload occupy offsets 0 and 8, the resolve tag and payload occupy offsets 16
and 24, and the reject tag and payload occupy offsets 32 and 40. The three tags
remain scalar while all three payloads remain pointer-classified.

This three-scalar/three-pointer census is a retention invariant. Async-generator
request records retain a Promise capability whose Promise, resolve function and
reject function payloads must all remain visible to tracing. An arbitrary row
can no longer trace a tag, omit one of those payload edges or reorder one
tag/payload pair independently of the closed identity registry.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Promise capability rows. The
bounded heap owner witness asserts every projected field. The existing
async-generator retention witness keeps the three payload classifications
aligned with that live consumer.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change Promise capability allocation or initialization, Promise settlement,
reaction or job scheduling, async-generator behavior, emitted Wasm, root
scanning or collector execution.

```sh
cargo test -p lila-aot-wasm --test promise_capability_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::promise_capability_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::async_generator_records_expose_queue_activation_and_promise_edges_to_gc -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_promise_capability_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/promise_capability_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, offsets 0, 8, 16, 24, 32 and 40,
the three-scalar/three-pointer census, typed registry order and unchanged
runtime offset consumers. At the shared Batch AF checkpoint, `cargo xc` is
green, the Promise-capability structure target passes `4/4`, exact
`heap::tests::promise_capability_heap_slot_identities_own_layout_metadata`
passes `1/1`, exact
`heap::tests::async_generator_records_expose_queue_activation_and_promise_edges_to_gc`
passes `1/1`, and the `heap_layout_registry_` filter passes `2/2`. No CLI,
Test262 or semantic-golden verification applies to this source/type-only heap
ownership change, and none was run.
