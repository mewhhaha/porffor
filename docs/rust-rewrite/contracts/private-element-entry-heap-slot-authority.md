# Private-element-entry heap-slot identity authority

## Closed layout identities

The passive private-element entry contains exactly six capability-free
`PrivateElementEntryHeapSlot` identities in next, receiver, token, kind,
value-tag and value-payload order.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Next, receiver and token
remain pointer-classified at offsets 0, 8 and 16. Kind and value tag remain
scalars at offsets 24 and 32. Value payload remains pointer-classified at
offset 40.

This two-scalar/four-pointer census is a retention invariant. A private-element
entry must keep the following list node, receiver identity, private-name token
and stored value payload visible to tracing, while kind and value tag must never
be scanned as addresses. An arbitrary row can no longer reverse either side of
that relation or reorder a value tag independently of its payload.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form private-element-entry rows. The
bounded heap owner witness asserts every projected field. The existing
`PrivateElementEntryLocals` and `PrivateElementHeapKind` authorities continue
to own legal row contents and kind wire words independently of byte layout.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change entry construction, linked-list publication or traversal, private-name
lookup, field/method/accessor semantics, emitted Wasm, root scanning or
collector execution. The runtime private-element implementation and its
existing protocol guard and contract remain unchanged.

```sh
cargo test -p lila-aot-wasm --test private_element_entry_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::private_element_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_private_element_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/private_element_entry_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, offsets 0, 8, 16, 24, 32 and 40,
the two-scalar/four-pointer census, typed registry order and unchanged runtime
offset consumers. At the shared Batch AI checkpoint, `cargo xc` exits `0`, the
`private_element_entry_heap_slot_structure` target passes `4/4`, the unchanged
`private_element_entry_protocol_structure` target passes `5/5`, exact
`heap::tests::private_element_entry_heap_slot_identities_own_layout_metadata`
passes `1/1`, and the `heap_layout_registry_` filter passes `2/2`. No CLI,
Test262 or semantic-golden verification is needed for this source/type-only
ownership change, and none was run because the runtime is byte-untouched. Final
formatter, diff, module-boundary, task-plan and 240-entry shortcut-inventory
gates are green.
