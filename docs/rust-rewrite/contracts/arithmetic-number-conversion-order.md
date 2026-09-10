# Arithmetic conversion order

`compile_coercive_binary_number_to_locals` owns runtime arithmetic conversion
for the closed `ArithmeticBinaryOp` domain. Both payload-only and tagged
expression emission call it. The inferred normal result kind cannot authorize
a different conversion algorithm: an operation inferred as Number may still
receive a BigInt on the left and must convert the right operand before reporting
a Number/BigInt mismatch.

`Add` delegates to `compile_coercive_add_to_locals`. It evaluates both operands,
then applies ToPrimitive to the left and right with the default hint. A String
primitive selects concatenation; otherwise left and right ToNumeric precede the
numeric type check. This preserves the required distinction between addition
and the other arithmetic operators.

`Sub`, `Mul`, `Div`, `Mod` and `Exp` evaluate both operands first, then apply
ToNumeric to the left and right in order. An abrupt conversion exits immediately;
a type mismatch is checked only after both conversions complete normally.
Both inline and heap BigInts retain their numeric kind and use the canonical
BigInt arithmetic helper. Runtime result tags remain authoritative when object
conversion or arithmetic overflow produces a heap BigInt.

The former Number-only operand-pair helper, its conversion-order selector and
its private operand converter have been deleted with their last callers.
There is one conversion path for each algorithm, independent of whether the
expression result is discarded, stored or passed to a caller.

```sh
cargo test --release --locked -j2 -p lila-aot-wasm --test arithmetic_number_conversion_order_structure --test unary_numeric_ir_structure
cargo test --release --locked -j2 -p lila-engine --lib wasm_backend_outlined_to_numeric_preserves_kind_order_and_abrupt_identity
cargo test --release --locked -j2 -p lila-engine --test aot_declaration_completion
```

Current verification and the exact real execution cohort are recorded in the
[September repair notes](../observed-later-failure-repairs.md). This contract
does not establish full numeric or Test262 conformance.
