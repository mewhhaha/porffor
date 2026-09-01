# Map-entry heap-slot identity authority

## Closed layout identities

The passive ordinary Map entry layout contains exactly five capability-free
`MapEntryHeapSlot` identities, in present-key-tag-key-payload-value-tag-value-
payload order:

- `Present`;
- `KeyTag`;
- `KeyPayload`;
- `ValueTag`;
- `ValuePayload`.

One private exhaustive `metadata()` projection is the sole authority for all
five identities' record names, slot names, offsets, widths and pointer
classifications. Every word remains 8 bytes at its existing offset. The key and
value payloads remain the only two strong tracing edges; the presence word and
both tags remain scalar.

This strong-edge classification deliberately contrasts with the closed WeakMap
entry layout, whose five words are all non-pointers because its typed weak-edge
domain owns ephemeron retention. An arbitrary ordinary Map row cannot omit
either key or value from tracing or mark the control words as edges.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order, checks the
WeakMap contrast and verifies that no second Rust source constructs free-form
Map entry rows. The bounded heap owner witness asserts every projected field
and retains the existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm, Map behavior or
collector execution.

```sh
cargo test -p lila-aot-wasm --test map_entry_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::map_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_map_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/map_entry_heap_slot_structure.rs
git diff --check
```

The structure regression passes `4/4`, the exact owner witness passes `1/1`
and the adjusted collision and pointer registry witnesses pass `2/2`, with only
existing workspace warnings. Targeted formatting and `git diff --check` pass.
The shared `cargo xc`, module-boundary and task-plan checkpoints are also green.
Golden and conformance execution do not apply to this passive metadata-only
closure.
