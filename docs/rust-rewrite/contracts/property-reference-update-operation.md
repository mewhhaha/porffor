# Property Reference update operation ownership

Status: implemented and structure-verified on 2026-08-27.

## One-shot operation authority

The private, capability-free
`PropertyUpdateOp::{Arithmetic, Bitwise, Logical}` domain is the sole operation
authority for the shared private-property compound-assignment and
super/private logical-assignment Reference path. Arithmetic and bitwise
producers are private-property-only; the logical producer deliberately serves
both super and private References.

The authority is handed to `lower_property_reference_update` by value and
consumed in one exhaustive match. Each arm owns both RHS reachability and the
result composition:

- `Logical` uses conditional RHS lowering, merges the read and written shapes,
  and produces `Composition::ShortCircuit`;
- `Arithmetic` lowers the RHS ordinarily and applies its `ArithmeticOp`;
- `Bitwise` lowers the RHS ordinarily, exhaustively maps its `BitwiseOp`, and
  applies the resulting binary operation.

The enum derives no debug, clone, copy, equality or default capability. A
second consuming observation is therefore a move error, while a new variant
must state its RHS reachability, value operation, shape and composition before
the crate builds. The former preliminary `matches!` reachability probe is gone.

## Preserved Reference lifecycle

The change does not alter ECMAScript evaluation order. Lowering still obtains
the property read, reconstructs the single Reference, captures `[[Strict]]`,
pins its operands and performs `GetValue` before considering the operation.
The selected arm then lowers the RHS according to its existing reachability
law. Shape recording remains after value construction, and the same
`ReferenceRecord` and `ReferencePins` are consumed by the final write and
materialization.

The exact recursive ownership census falls from nine to eight identifiers:
the declaration, owned parameter, three producers and three exhaustive arms.
Every variant therefore has exactly one producer and one consumer route.

## Evidence and nonclaims

The Rust-lexical structure target pins the capability-free declaration, exact
census, all three ordered assignment producers, complete consumer body, sole
match, Reference lifecycle and absence of alternate observations:

```console
cargo check -p lila-ir
cargo test -p lila-ir --test property_update_op_ownership_structure -- --test-threads=1
cargo test -p lila-aot-wasm --test ordinary_property_logical_assignment_structure -- --test-threads=1
```

The new structure target passes `4/4`, the neighboring ordinary-property
logical-assignment target passes its retained suite, and the focused package
check is green. The four exact private-reference Test262 leaves covering
arithmetic, bitwise, taken logical and short-circuit logical behavior pass all
`8/8` sloppy/strict Wasm-AOT executions. Every Parser, EarlyError, Lowering,
Runtime, WasmBackend, HostHarness and Unsupported bucket is zero; all eight
outcomes are `Success`. This source-equivalent ownership closure does not claim
broader private-field, `super`, assignment-expression or Test262 conformance.
