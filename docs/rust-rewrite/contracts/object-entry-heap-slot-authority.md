# Object-entry heap-slot identity authority

## Closed layout identities

The passive ordinary object-entry record contains exactly eight
capability-free `ObjectEntryHeapSlot` identities in key, descriptor-kind,
data-tag, data-payload, getter-tag, getter-payload, setter-tag and
setter-payload order.

One private exhaustive `metadata()` projection is the sole authority for all
eight identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Key and descriptor kind
occupy offsets 0 and 8, data tag and payload occupy offsets 16 and 24, getter
tag and payload occupy offsets 32 and 40, and setter tag and payload occupy
offsets 48 and 56. Key and all three payloads remain pointer-classified;
descriptor kind and all three tags remain scalar.

This four-scalar/four-pointer census is a retention invariant. An ordinary
property entry must keep its key and any stored data, getter and setter
payloads visible to tracing, while no descriptor-kind or value-tag word may be
scanned as an address. An arbitrary row can no longer reverse either side of
that relation or reorder one tag independently of its payload.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form object-entry rows. The bounded
heap owner witness asserts every projected field. `DescriptorWord`,
`StoredDescriptorKind` and the stored descriptor local types remain the
independent semantic authorities for legal descriptor contents.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change property allocation, lookup, descriptor creation or update, accessor
invocation, Array or Object builtins, emitted Wasm, root scanning or collector
execution. All object-entry runtime offset and size consumers remain
unchanged.

```sh
cargo test -p lila-aot-wasm --test object_entry_heap_slot_structure
cargo test -p lila-aot-wasm --test stored_descriptor_role_relation_structure
cargo test -p lila-aot-wasm --test stored_property_attributes_structure
cargo test -p lila-aot-wasm --lib heap::tests::object_entry_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_object_entry_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/object_entry_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact eight rows, offsets 0, 8, 16, 24, 32, 40, 48
and 56, the four-scalar/four-pointer census, typed registry order and unchanged
runtime offset consumers. At the Batch AM checkpoint, `cargo xc` is green, the
new structure target passes `4/4`, both descriptor neighbors pass `4/4`, the
bounded heap owner passes `1/1`, and the registry checks pass `2/2`. No runtime
CLI, Test262 leaf or semantic golden was required or run for this passive
metadata change.
