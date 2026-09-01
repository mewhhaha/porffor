# Atomics async-waiter heap-slot identity authority

## Closed layout identities

The passive Atomics async-waiter record contains exactly six capability-free
`AtomicsAsyncWaiterHeapSlot` identities in state, address, Promise-record,
deadline, next-link and host-identity order:

- `State`;
- `Address`;
- `PromiseRecord`;
- `DeadlineNanos`;
- `Next`;
- `HostId`.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. The retained Promise record and waiter-list link remain
pointer-classified 8-byte words at offsets 16 and 32. State, the linear-memory
wait address, the monotonic deadline and the opaque host identity remain scalar
8-byte words at offsets 0, 8, 24 and 40. An arbitrary row cannot omit either
retained edge or treat a host identity or linear-memory address as a traced heap
pointer.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form Atomics async-waiter rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change waiter allocation or traversal, timeout processing, host-agent calls,
Promise settlement, emitted Wasm, root scanning or collector execution.

```sh
cargo test -p lila-aot-wasm --test atomics_async_waiter_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::atomics_async_waiter_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_atomics_async_waiter_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/atomics_async_waiter_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, the four-scalar/two-pointer census,
typed registry order and unchanged runtime offset consumers. At the Batch AA
checkpoint, `cargo xc` is green, the structure target passes `4/4`, the exact
layout-owner unit passes `1/1`, and the heap registry filter passes `2/2`.
Runtime, semantic-golden and Test262 checks do not apply to this passive
layout-only migration and were not run.
