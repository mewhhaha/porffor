# Atomics async-waiter heap-slot identity authority

## GC checkpoint progress — 2026-10-07

The GC timeout checkpoint preserves the caller's complete completion and returns
its progress in a typed i32 local. Main loads and releases that local before
its event-loop branch; polling releases it internally because the Promise queue
already owns continuation. Neither caller consumes an implicit stack result.
The prior unit-returning emitter left main's `br_if` without a condition and a
Promise polling caller with a spurious `drop`, producing invalid Wasm when the
monotonic-clock path was selected.

Drain returns as soon as it settles a waiter, before waiting for other active
waiters. The resulting Promise reaction may notify another waiter. Waiting for
every waiter first would turn that valid notification into a later timeout or
deadlock. The finite native control
`wasm_atomics_wait_async_runs_settled_reactions_before_waiting_again` registers
two original waits and requires the first timeout's reaction to notify the
second before its later deadline. `tasks-symbol-progress1` passes the workspace
type check and the ordinary Await control that previously produced invalid Wasm.
The new waiter control fails before registration: the old constructor plan
aliases Int32Array to a body allocating Float64 storage. Its native diagnostic
retains the actual shared buffer and confirms the wrong element kind. The
joined constructor repair passes the workspace type check and the unchanged
waiter control in `tasks-constructor-progress1`. The original first timeout's
reaction observes one notification and the later Promise resolves with `ok`.
The same checkpoint passes the all-kind constructor storage control. Historical
passive-layout results below retain their earlier scope.

Current dry source — 2026-10-05: the raw-offset source-mirror target below
is retired from the atomic semantic GC draft. It checked passive layout
spelling, offsets and source occurrence counts. The current GC authoring uses
typed registry fields and complete semantic value owners; these earlier
mirrors cannot verify that representation or its behavior. Historical source
descriptions, commands and results below retain their original scope.

The full GC source cutover, remaining Temporal and AsyncDisposableStack
authoring, compiler checks and runtime verification remain pending. See the
[current value/heap architecture](../value-heap-gc.md). Weak reachability retains
its explicit [unavailable facility boundary](weak-unavailable-runtime-boundary.md).
No verification or conformance result is inferred from this retirement.

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
