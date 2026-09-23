# Number bitwise emission

The completed baseline's shift stress cases contain hundreds of literal Number
operations. The frozen main compiler emitted functions of 7.38–9.14 MB for twelve
such files in both strict and sloppy modes, and Wasmtime rejected those functions
as too large. The bitwise emitter included both `ToNumeric` conversions, the
Number/BigInt agreement check, and the BigInt path at every Number site.

The emitter now selects the existing Number projection when both operands have
singleton Number kinds and their expression forms cannot produce a dynamic
result tag. The shared `expr_has_static_number_payload` predicate also governs
the existing direct Number-conversion paths. Inferred Number facts on mutable
bindings, property reads, or calls do not authorize this selection.

The proof also follows unary minus, complement and nested bitwise operands
whose inputs meet that same rule. Their emitters project Number directly;
negative literals no longer restore generic dispatch around a proven Number.
Frozen checkpoint five still failed the shift stress cohort. A component
measurement found 143 bytes per positive shift and 2,452 bytes per negative
shift, so positive-literal growth alone was an incomplete verification boundary.
The expanded size regression covers negative and nested expressions. Full stress
replay remains required after this follow-up.

Both operands are still evaluated once, left to right, before the decision's
emitted arithmetic. The generic path retains ordered `ToNumeric`, mixed-type
errors, arbitrary-precision BigInt operations, and the BigInt `>>>` rejection.
The Number projection remains one shared block using `ToUint32`, a five-bit shift
count, and signed or unsigned result conversion according to the operator.

`numeric_bitwise_emission` measures the incremental user-function bytes for all
six operators. `aot_numeric_bitwise` exercises Number boundaries, observable
operand evaluation and coercion, abrupt values, BigInt operations, and mutable
storage. Frozen baseline replay is required before claiming the historical
failures repaired; the emission change does not alter native compiler limits or
recognize Test262 source text.
