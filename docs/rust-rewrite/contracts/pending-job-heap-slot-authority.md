# Pending-job heap-slot identity authority

## Closed layout identities

The passive pending Promise-job record contains exactly seven capability-free
`PendingJobHeapSlot` identities in callback-tag, callback-payload, argument-tag,
argument-payload, Realm, next and kind order.

One private exhaustive `metadata()` projection is the sole authority for all
seven identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Callback tag and payload
occupy offsets 0 and 8, argument tag and payload occupy offsets 16 and 24,
Realm occupies offset 32, next occupies offset 40 and kind occupies offset 48.
The two tags and kind remain scalar, while both payloads, Realm and next remain
pointer-classified.

This three-scalar/four-pointer census is a retention invariant. A queued job
must keep its callback record, argument payload, evaluation Realm and following
FIFO node visible to tracing. An arbitrary row can no longer trace a tag or
kind, omit one of those live edges or reorder one tag/payload pair independently
of the closed identity registry.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form pending-job rows. The bounded
heap owner witness asserts every projected field. The global pointer-registry
witness retains its exact next-edge check through the typed projection.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change job construction, queue ordering, job dispatch, Promise or
async-generator behavior, emitted Wasm, root scanning or collector execution.
The separately typed pending-jobs root source remains unchanged.

```sh
cargo test -p lila-aot-wasm --test pending_job_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::pending_job_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_pending_job_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/pending_job_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact seven rows, offsets 0, 8, 16, 24, 32, 40 and
48, the three-scalar/four-pointer census, typed registry order and unchanged
runtime enqueue/drain offset consumers. At the shared Batch AG checkpoint,
`cargo xc` is green, the recursive structure target passes `4/4`, exact
`heap::tests::pending_job_heap_slot_identities_own_layout_metadata` passes
`1/1`, and the `heap_layout_registry_` filter passes `2/2`. No CLI, Test262 or
semantic-golden verification applies to this source/type-only ownership
change, and none was run; the runtime enqueue/drain code is byte-untouched.
