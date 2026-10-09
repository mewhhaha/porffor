# Numeric-update value-kind ownership

`NumericUpdateValueKind::{Number, BigInt, Dynamic}` is the complete IR domain
for the value produced by `ToNumeric` before an increment or decrement. Its
total `value_kind()` projection is the only conversion back to the wider
runtime `ValueKind` vocabulary.

Identifier, ordinary-property and Super-property numeric-update carriers
store this closed domain. Global and captured Environment References use the
same Dynamic numeric path after retaining the resolved Reference and the
complete result of `ToNumeric`; they do not introduce a second kind field.

The Wasm backend's sole delta emitter matches the three variants exhaustively.
Number emits the floating-point delta. BigInt calls the canonical arbitrary
precision Add/Sub helper with `1n`. Dynamic selects those same kernels from
the already numeric Value tag. BigInt results are rooted GC values, with the
canonical helper owning allocation and arbitrary precision arithmetic. No
integer payload arithmetic or separate heap/inline BigInt dispatch is involved.

Every Reference consumer preserves the complete old `ValueLocals` until
PutValue finishes. Prefix returns the new value and postfix returns the old
value. Numeric conversion, GetValue, PutValue, and abrupt completion keep their
existing order. The three static admission choices affect only the delta;
they do not skip evaluation or conversion at the Reference boundary.

Immutable updates still evaluate `ToNumeric(GetValue(reference))` before the
immutable-binding error. Conversion results travel in `CompletionLocals`, and
the consumer propagates a throw before copying a normal numeric value. This
retains the original Number, arbitrary precision BigInt, and thrown-object
identity across nested updates and caller catch/finally paths.

The current source repair restores consumption of the closed kind after the GC
migration had left that parameter unused. The Dynamic arithmetic kernels are
unchanged. Current artifact, native, and source-structure verification is
pending; the earlier results below are historical evidence only.

```sh
cargo test -p lila-aot-wasm --test structure_language -- numeric_update_value_kind_structure::
cargo test -p lila-aot-wasm --test primitive_to_number_throw_routing_structure
cargo test -p lila-aot-wasm --test structure_language -- ordinary_property_numeric_update_structure::
cargo test -p lila-aot-wasm --test structure_language -- super_property_reference_mutation_structure::
cargo test -p lila-aot-wasm --test structure_language -- global_object_environment_numeric_update_structure::
cargo test -p lila-aot-wasm --test structure_language -- with_environment_numeric_update_structure::
cargo test -p lila-engine --test aot_builtins -- aot_bigint_numeric_updates:: --test-threads=1
```

At the preceding closed-domain checkpoint, the target passed `4/4`; the ordinary-property,
Super-property, global-object-environment and with-environment neighboring
targets pass `7/7`, `6/6`, `4/4` and `4/4`. The ordinary-property,
script-global nested-update and global-object-environment CLI controls pass
`3/3`, and the filtered IR numeric-update tests pass `4/4`. The
with-environment CLI control does not reach execution: Wasmtime
rejects its existing 6,630,529-byte generated function as too large. The
shared `cargo xc`, workspace formatting, diff, module-boundary and task-plan
checks were green. Those results predate the canonical BigInt delta repair.
