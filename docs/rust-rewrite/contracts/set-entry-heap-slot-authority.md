# Set-entry heap-slot identity authority

## Closed layout identities

The passive ordinary Set entry layout contains exactly three capability-free
`SetEntryHeapSlot` identities, in present-tag-payload order:

- `Present`;
- `ValueTag`;
- `ValuePayload`.

One private exhaustive `metadata()` projection is the sole authority for all
three identities' record names, slot names, offsets, widths and pointer
classifications. `Present` and `ValueTag` remain scalar 8-byte words at their
existing offsets. `ValuePayload` remains the traced 8-byte edge at
`HEAP_SET_ENTRY_VALUE_PAYLOAD_OFFSET`. An arbitrary row cannot move the strong
edge onto a scalar word or omit the ordinary Set value from tracing.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form ordinary Set entry rows. The
bounded heap owner witness exercises all three projected rows through the
existing collision and record-size checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change a linear-memory offset, allocation, emitted Wasm or Set behavior. It
does not migrate collection values to Wasm GC, execute tracing, reclaim an
object, collect a cycle or implement `gc()`.

```sh
cargo test -p lila-aot-wasm --test set_entry_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::set_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
rustfmt --check --config skip_children=true crates/lila-aot-wasm/src/heap_set_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/src/lib.rs crates/lila-aot-wasm/tests/set_entry_heap_slot_structure.rs
git diff --check
```

The structure target passes `4/4`, the exact identity owner witness passes
`1/1`, and the adjusted collision/pointer registry witnesses pass `2/2`. Only
the workspace's existing warnings are emitted. Targeted formatting with child
module traversal disabled and diff checks pass, and the shared `cargo xc`
checkpoint is green. Golden and conformance execution do not apply to this
passive metadata-only closure.
