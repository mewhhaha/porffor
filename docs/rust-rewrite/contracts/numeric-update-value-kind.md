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
Number emits the floating-point delta. BigInt calls the canonical arbitrary
precision Add/Sub helper with `1n`; Dynamic recognizes both inline and heap
BigInt tags after `ToNumeric`. The helper returns a new payload and tag because
an update can cross either representation boundary. The former payload-only
integer delta emitter is gone.

Every Reference consumer preserves the old numeric payload/tag pair until
PutValue finishes. Prefix returns the new pair and postfix returns the old
pair. Identifier and global-property consumers share one write between those
result choices; prepared-eval environment updates also preserve the old tag.
Static BigInt inference cannot replace either runtime representation tag.
Numeric conversion, GetValue and PutValue keep their existing order and
completion routes.

```sh
cargo test -p lila-aot-wasm --test numeric_update_value_kind_structure
cargo test -p lila-aot-wasm --test ordinary_property_numeric_update_structure
cargo test -p lila-aot-wasm --test super_property_reference_mutation_structure
cargo test -p lila-aot-wasm --test global_object_environment_numeric_update_structure
cargo test -p lila-aot-wasm --test with_environment_numeric_update_structure
cargo test -p lila-engine --test aot_bigint_numeric_updates -- --test-threads=1
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
