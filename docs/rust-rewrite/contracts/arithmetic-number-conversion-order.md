# Arithmetic Number conversion-order authority

## Closed input domain

`compile_operand_pair_to_number_locals` accepts the existing closed
`ArithmeticBinaryOp` domain directly. The retired `NumericBinaryOperator`
wrapper admitted a `Bitwise` state that no caller constructed and derived
clone, copy, debug and equality capabilities that the emitter did not need.
Deleting it makes the helper's actual arithmetic-only boundary explicit and
removes an unowned state rather than preserving it behind a runtime branch.

One private exhaustive `arithmetic_applies_to_primitive_before_numeric`
projection owns the only ordering distinction. `Add` applies ToPrimitive to
both evaluated operands before converting either to Number. `Sub`, `Mul`,
`Div`, `Mod` and `Exp` convert the left operand to Number before converting the
right. Adding an arithmetic operator therefore requires a compiler-visible
decision at this projection.

The three expression callers forward the `ArithmeticBinaryOp` they already
hold. No caller can substitute an unlabeled Boolean or construct a bitwise
state for this Number-only helper.

## Preserved execution

The helper still evaluates the left expression and then the right expression
before either conversion. Its `Add` branch still performs left and right
ToPrimitive followed by left and right Number conversion. Its other branch
still performs left and right Number conversion directly. This closure changes
only Rust domain ownership and does not change emitted instructions, the
completion ABI or numeric results.

```sh
cargo test -p lila-aot-wasm --test arithmetic_number_conversion_order_structure --quiet
cargo test -p lila-aot-wasm --test unary_numeric_ir_structure --quiet
cargo xc
./target/debug/lila --jobs 1 test262 run language/expressions/addition/order-of-evaluation.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run language/expressions/multiplication/order-of-evaluation.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

The new structure target passes `4/4`, and the neighboring unary-numeric
structure target remains green at `7/7`. The shared `cargo xc` checkpoint is
green. The pinned addition and multiplication order controls pass all `4/4`
sloppy/strict Wasm-AOT executions with every failure bucket at zero.

## Nonclaims

This invariant does not change BigInt arithmetic, add a bitwise Number route,
alter operator lowering or claim broader numeric/Test262 progress.
