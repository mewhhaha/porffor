# Coercive arithmetic operation ownership

`compile_coercive_binary_number_to_locals` accepts the closed
`ArithmeticBinaryOp::{Add, Sub, Mul, Div, Mod, Exp}` domain. Addition delegates
to the shared addition emitter before the numeric operand locals are reserved,
so both primitive conversions and string concatenation retain their specified
order. The other five operators share evaluation, ToNumeric, mixed-type
validation and runtime Number/BigInt dispatch.

The Number branch exhaustively identifies every operation. Subtraction,
multiplication and division use their binary64 instructions. Remainder uses
`emit_number_remainder_payload`, shared with typed binary expressions and local
and property compound assignments. It reduces integer binary significands,
then reconstructs the result with the numerator's sign. This preserves exact
remainders when a floating-point quotient would overflow or round away needed
bits, including subnormal results and negative zero. NaN, zero divisors and
infinite operands take explicit branches. Exponentiation retains the shared
Number-power emitter. Addition has already returned through its own conversion
path. A new IR operation requires an explicit backend decision.

The remainder emitter owns five temporary locals. Its callers retain both
operands across child evaluation, so nested right-hand expressions cannot
overwrite the numerator. The planner accounts for the retained operands and
the shared emitter's temporary-local budget.

The BigInt side uses the existing exhaustive
`BigIntHelperOp::from_arithmetic` projection. Payload-only expression emission
calls this same tagged emitter instead of maintaining separate arithmetic and
error branches. The planner includes the four retained operand locals across
child evaluation and conversion/error phases, and preserves the runtime result
tag even when neither raw operand's kind advertises BigInt.

```sh
cargo test --release --locked -j2 -p lila-aot-wasm --test coercive_number_arithmetic_operation_structure --test bigint_helper_op_structure
cargo test --release --locked -j2 -p lila-aot-wasm --lib planning::tests::
cargo test --release --locked -j2 -p lila-ir --test number_remainder
cargo test --release --locked -j2 -p lila-engine --test aot_number_remainder
```

Current verification is recorded in the
[September repair notes](../observed-later-failure-repairs.md). The earlier
operation-table ownership change was byte-equivalent; the subsequent tagged
conversion repair intentionally changes execution behavior for mixed numeric
operands and object coercion.
