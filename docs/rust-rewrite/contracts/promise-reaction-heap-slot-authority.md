# Promise-reaction heap-slot identity authority

## Closed layout identities

The passive Promise-reaction record contains exactly seven capability-free
`PromiseReactionHeapSlot` identities in capability, handler-tag,
handler-payload, Realm, next, type and callback-kind order.

One private exhaustive `metadata()` projection is the sole authority for all
seven identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Capability occupies
offset 0, handler tag and payload occupy offsets 8 and 16, Realm occupies
offset 24, next occupies offset 32, type occupies offset 40 and callback kind
occupies offset 48. Handler tag, type and callback kind remain scalar, while
capability, handler payload, Realm and next remain pointer-classified.

This three-scalar/four-pointer census is a retention invariant. A Promise
reaction must keep its capability, callable handler payload, captured Realm and
following reaction node visible to tracing, while no wire tag or reaction-kind
word may be scanned as an address. An arbitrary row can no longer reverse
either side of that relation or reorder one field independently of the closed
identity registry.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Promise-reaction rows. The
bounded heap owner witness asserts every projected field. The existing
`PromiseReactionType` and `PromiseReactionCallbackKind` domains remain the
independent semantic authorities for their stored wire words.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change Promise-reaction allocation, list linking, fulfillment or rejection
dispatch, handler tag/payload representation, Realm selection, queued-job
behavior, emitted Wasm, root scanning or collector execution. All Promise
runtime offset consumers remain unchanged.

```sh
cargo test -p lila-aot-wasm --test promise_reaction_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::promise_reaction_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::async_generator_records_expose_queue_activation_and_promise_edges_to_gc -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_promise_reaction_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/promise_reaction_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact seven rows, offsets 0, 8, 16, 24, 32, 40 and
48, the three-scalar/four-pointer census, typed registry order and unchanged
Promise runtime offset consumers. At the 2026-08-28 Batch AK checkpoint,
`cargo xc` is green, the recursive structure target passes `4/4`, the exact
heap owner passes `1/1`, both collision/pointer registry tests pass `2/2`, and
the neighboring async-generator retention witness passes `1/1`. This passive
metadata change requires no runtime CLI, Test262 cohort or semantic golden.
