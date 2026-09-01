# DisposableStack-entry heap-slot identity authority

## Closed layout identities

The passive DisposableStack entry contains exactly five capability-free
`DisposableStackEntryHeapSlot` identities in kind, value-tag, value-payload,
method-tag and method-payload order:

- `Kind`;
- `ValueTag`;
- `ValuePayload`;
- `MethodTag`;
- `MethodPayload`.

One private exhaustive `metadata()` projection is the sole authority for all
five identities' record names, slot names, offsets, widths and pointer
classifications. The kind, value tag and method tag remain scalar 8-byte words
at offsets 0, 8 and 24. The value and method payloads remain pointer-classified
8-byte words at offsets 16 and 32. An arbitrary row cannot trace a kind or tag,
omit either retained payload edge or separate either tag from its payload.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form DisposableStack entry rows.
The bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change allocation, stack lifecycle, capability transfer, resource or method
retention, disposal order, emitted Wasm or collector execution.

```sh
cargo test -p lila-aot-wasm --test disposable_stack_entry_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::disposable_stack_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_disposable_stack_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/disposable_stack_entry_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact five rows, the three-scalar/two-pointer census,
typed registry order and unchanged runtime offset consumers. At the 2026-08-28
Batch X checkpoint, the recursive structure target passes `4/4`, the exact heap
owner witness passes `1/1`, the collision/pointer registry filter passes `2/2`,
and `cargo xc`, formatting, diff, module-boundary and task-plan checks are
green. No semantic golden or Test262 rerun applies to this passive layout-only
migration.
