# Arguments `callee` descriptor boundary

Status: implemented and focused-verified through Batch AO.

## Closed boundary

Arguments exotic `[[DefineOwnProperty]]` for `callee` accepts one private
`ArgumentsCalleeDescriptorLocals`. Its six named fields each carry one
`RuntimeDescriptorField<T>`, so every payload is attached to its run-time
presence fact and tagged value roles cannot be transposed with Boolean flag
roles. Static or absent presences are not part of this boundary's domain.

The sole producer is the `Object.defineProperty` compiler after
`ToPropertyDescriptor` has emitted the 6.2.6.5 step-9 rejection. The aggregate's
only descriptor projection constructs a `WasmPartialDescriptor` and crosses
the audited `from_runtime_checked` escape hatch. The consumer classifies that
validated descriptor once and exhaustively projects both `DescriptorSide`
values through `emit_array_descriptor_side_present_to_local`. It does not
re-derive data/accessor kind from the six raw presence locals.

The former fifteen positional descriptor parameters are gone. Adding another
callee producer, omitting one field, swapping a tagged value for a flag, or
passing a general validated descriptor whose presences are not all run-time is
now a source-level change to this closed domain rather than a silently accepted
call.

## Durable witness

`arguments_callee_descriptor_structure.rs` pins the exact six-field aggregate,
its one validated projection, the single producer and consumer, removal of the
positional signature, and canonical classification as the only descriptor-kind
decision.

```sh
cargo test -p lila-aot-wasm --test arguments_callee_descriptor_structure --quiet
```

This source-equivalent invariant migration does not claim broader Arguments,
Object descriptor, or Test262 closure. The existing callee compatibility and
storage algorithm is unchanged; semantic verification remains owned by the
shared T10 checkpoint.

Batch AO moves this carrier, its sole producer and consumer, the sibling
Arguments-index descriptor helper and the complete `Object.defineProperty`
compiler into the private `builtins/object/define_property.rs` owner. The
2,500-line child and reduced 5,751-line parent have SHA-256
`01ea9de92ace5f710bc6fcea6b7b4d64326e8d726e165920a06fdb1d8368b4c6`
and `d8e910a3b8e2edcd7ab4e9fd6ee19507d86de6fbd6721d1da29655ffef817a53`.
No emitted behavior is intended to change. At the Batch AO checkpoint,
`cargo xc` is green, the strengthened Object owner target and five
descriptor/proxy neighbors pass `25/25`, and the exact object-descriptor CLI
passes `1/1`.
