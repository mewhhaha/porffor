# Class-function-context heap-slot identity authority

## Closed layout identities

The passive class-function context contains exactly six capability-free
`ClassFunctionContextHeapSlot` identities in lexical-environment,
active-function, home-object-payload, home-object-tag, field-keys and
private-environment order.

One private exhaustive `metadata()` projection is the sole authority for all
six identities' record names, slot names, offsets, widths and pointer
classifications. Every slot remains eight bytes wide. Lexical environment,
active function and home-object payload occupy offsets 0, 8 and 16. The
home-object tag is the sole scalar at offset 24. Field keys and private
environment remain pointer-classified at offsets 32 and 40.

This one-scalar/five-pointer census is a retention invariant. A class-owned
function context must keep its lexical environment, active function, home
object, computed field-key storage and private environment visible to tracing,
while the home-object tag must never be scanned as an address. An arbitrary row
can no longer reverse either side of that relation or reorder the tag away from
its payload independently of the closed identity registry.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form class-context rows. The bounded
heap owner witness asserts every projected field. The global pointer-registry
witness retains its exact sole-home-object-tag-scalar relation through the
typed projection.

## Passive boundary

This invariant reorganizes passive Rust layout metadata only. It does not
change class-context allocation, method or field initialization, `super`
resolution, private-name lookup, emitted Wasm, root scanning or collector
execution. The runtime offset consumers remain unchanged.

```sh
cargo test -p lila-aot-wasm --test class_function_context_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::class_function_context_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_class_function_context_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/class_function_context_heap_slot_structure.rs
git diff --check
```

Dry source review pins the exact six rows, offsets 0, 8, 16, 24, 32 and 40,
the one-scalar/five-pointer census, typed registry order, sole-home-object-tag-
scalar relation and unchanged runtime offset consumers. At the shared Batch AH
checkpoint, `cargo xc` exits `0`, the
`class_function_context_heap_slot_structure` target passes `4/4`, exact
`heap::tests::class_function_context_heap_slot_identities_own_layout_metadata`
passes `1/1`, and the shared `heap_layout_registry_` filter passes `2/2`. No
CLI, Test262 or semantic-golden verification is needed for this source/type-
only ownership change, and none was run. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.
