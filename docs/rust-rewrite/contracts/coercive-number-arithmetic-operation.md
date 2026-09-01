# Coercive Number arithmetic operation ownership

The Number branch of `compile_coercive_binary_number_to_locals` exhaustively
matches the complete `ArithmeticBinaryOp::{Add, Sub, Mul, Div, Mod, Exp}`
domain after both operands have been evaluated, coerced with `ToNumeric` and
checked for mixed Number/BigInt use.

Each operation owns its complete Wasm sequence in that total match. Add,
subtract, multiply and divide use their direct binary64 instruction. Remainder
retains the quotient/truncate/multiply/subtract sequence, and exponentiation
retains the shared Number-power emitter. There is no preclassification with a
second partial match and no impossible-operation `unreachable!` arm. Adding an
IR arithmetic operation therefore requires an explicit Number algorithm before
the backend builds.

The BigInt side remains the existing exhaustive
`BigIntHelperOp::from_arithmetic` projection. This source-equivalent ownership
change preserves operand evaluation, coercion and mixed-kind error order and
emits the same Number instructions for every existing operation.

```sh
cargo test -p lila-aot-wasm --test coercive_number_arithmetic_operation_structure
cargo test -p lila-aot-wasm --test bigint_helper_op_structure
```

The closed-domain target passes `3/3`, and the neighboring Number
conversion-order and BigInt helper targets pass `8/8`. The exact
ordinary-property eager compound reference CLI witness passes `1/1`, covering
all six Number operations through the Wasm-AOT backend. The shared `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green.
