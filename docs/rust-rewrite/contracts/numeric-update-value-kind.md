# Numeric-update value-kind ownership

`NumericUpdateValueKind::{Number, BigInt, Dynamic}` is the complete IR domain
for the value produced by `ToNumeric` before an increment or decrement. Its
total `value_kind()` projection is the only conversion back to the wider
runtime `ValueKind` vocabulary.

Every identifier, global-property, ordinary-property and Super-property
numeric-update carrier stores this closed domain. Lowering must therefore
choose one of the two statically known numeric kinds or the runtime-dispatched
kind; another ECMAScript value kind cannot reach update emission while still
compiling.

The Wasm backend's sole delta emitter matches the three variants exhaustively.
Number emits the floating-point delta, BigInt emits the integer delta, and
Dynamic selects between those paths from the runtime tag after `ToNumeric`.
The former one-caller static delta emitter and all defensive impossible-kind
`unreachable!` arms are gone.

This is an IR and backend ownership change. It preserves coercion, GetValue and
PutValue order, prefix/postfix publication, completion propagation, emitted
numeric operations and the value ABI.

```sh
cargo test -p lila-aot-wasm --test numeric_update_value_kind_structure
cargo test -p lila-aot-wasm --test ordinary_property_numeric_update_structure
cargo test -p lila-aot-wasm --test super_property_reference_mutation_structure
cargo test -p lila-aot-wasm --test global_object_environment_numeric_update_structure
cargo test -p lila-aot-wasm --test with_environment_numeric_update_structure
```

The closed-domain target passes `4/4`; the ordinary-property,
Super-property, global-object-environment and with-environment neighboring
targets pass `7/7`, `6/6`, `4/4` and `4/4`. The ordinary-property,
script-global nested-update and global-object-environment CLI controls pass
`3/3`, and the filtered IR numeric-update tests pass `4/4`. The
with-environment CLI control does not reach execution: Wasmtime
rejects its existing 6,630,529-byte generated function as too large. The
shared `cargo xc`, workspace formatting, diff, module-boundary and task-plan
checks are green.
