# String heap-slot identity authority

## Closed layout identities

The passive String record layout contains exactly four capability-free
`StringHeapSlot` identities in code-units-pointer, byte-length,
code-unit-length and intern-id order:

- `CodeUnitsPointer`;
- `ByteLength`;
- `CodeUnitLength`;
- `InternId`.

One private exhaustive `metadata()` projection is the sole authority for all
four identities' record names, slot names, offsets, widths and pointer
classifications. The code-units address remains the sole pointer-classified
8-byte word at offset zero. Byte length, code-unit length and intern identity
remain scalar 8-byte words at offsets 8, 16 and 24. The existing
`LinearSideStorage::StringCodeUnits` identity remains the separate authority
for the retained UTF-16 side-storage element representation.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form String record rows. The
bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change String allocation, emitted Wasm, code-unit storage, interning or
collector execution.

```sh
cargo test -p lila-aot-wasm --test string_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::string_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_string_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/string_heap_slot_structure.rs
git diff --check
```

The recursive structure target passes `4/4`, the exact owner witness passes
`1/1`, and the collision/pointer registry witnesses pass `2/2`. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
Golden and conformance execution do not apply to this passive metadata-only
closure.
