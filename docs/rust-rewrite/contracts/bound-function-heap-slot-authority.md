# Bound-function heap-slot identity authority

## Closed layout identities

The passive bound-function record contains exactly six capability-free
`BoundFunctionHeapSlot` identities in target-payload, target-tag,
bound-this-payload, bound-this-tag, arguments-payload and self-payload order:

- `TargetPayload`;
- `TargetTag`;
- `ThisPayload`;
- `ThisTag`;
- `ArgumentsPayload`;
- `SelfPayload`.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. Target and bound-this tags remain scalar 8-byte words at
offsets 0 and 16. Their payloads remain pointer-classified 8-byte words at
offsets 8 and 24, and the arguments and bound-function self payloads remain
pointer-classified 8-byte words at offsets 32 and 40. The typed registry keeps
the existing payload-before-tag inventory order for both tagged values even
though each tag precedes its payload in the byte layout.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves the exact typed registry order and
verifies that no second Rust source constructs free-form bound-function rows.
The bounded heap owner witness asserts every projected field and retains the
existing collision, record-size and pointer census checks.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change bound-function allocation, call or construct behavior, Realm lookup,
`instanceof`, emitted Wasm, root scanning or collector execution.

```sh
cargo test -p lila-aot-wasm --test bound_function_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::bound_function_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_bound_function_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/bound_function_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, the two-scalar/four-pointer census,
typed registry order and unchanged runtime offset consumers. At the Batch AB
checkpoint, `cargo xc` is green, the structure target passes `4/4`, the exact
layout-owner unit passes `1/1`, and the heap registry filter passes `2/2`.
Runtime, semantic-golden and Test262 checks do not apply to this passive
layout-only migration and were not run.
