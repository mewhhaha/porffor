# Constant Number conditions in AOT branch emission

After parsing, early errors, IR lowering and backend planning, the ordinary
`StatementIr::If` emitter can select a branch from a closed, effect-free
expression tree. Both source branches still reach the existing compiler
analysis and declaration machinery. The selected statement starts with the
same undefined completion seed as the ordinary If emitter, preserving
UpdateEmpty even for an empty branch or an absent else branch.

The proof accepts Number literal IR, unary plus/minus/complement, Number
addition/subtraction/multiplication/division/remainder, bitwise operations,
numeric comparisons, Boolean literals and logical negation. Both operands
must independently satisfy the proof. Number shifts use exact binary64
decomposition for ToUint32, truncate fractional values, wrap modulo 2^32,
mask the shift count to five bits and distinguish signed/unsigned results.
Number equality and ordering preserve NaN and signed-zero semantics.
Exponentiation is outside the proof.

This proof reads IR only. Existing AST constant-folding behavior, including
the lowerer's separate exponentiation/property shortcuts, is unchanged; it
cannot recover effects that an earlier lowering step has already erased.

Identifiers, mutable properties, calls, comma expressions, explicit
conversions, BigInts and other IR forms retain the runtime condition,
regardless of their inferred value kind. Exhausting the bounded proof depth
also retains runtime emission. No source spelling, function name, harness
identity or test outcome participates in the decision. There is no new
runtime evaluator, dependency or compiler-limit change.

Checkpoint6's 24 remaining shift stress executions still failed with oversized
Wasm functions. The first left-shift case emitted 5,138,138 bytes for main.
Paired one/nine-site probes measured only 159–178 bytes per Number shift but
7,915 bytes per guarded ordinary constructor site. Omitting an unreachable
constructor branch addresses that measured cause. General constructor
outlining would instead change shared call, allocation, newTarget, Realm and
abrupt-completion machinery; it is a separate optimization.

Verification targets are the `constant_number_condition` unit tests in
lila-aot-wasm, `constant_number_condition_emission`, and
`aot_constant_number_condition` in lila-engine. Native verification and the
24-case stress replay remain necessary before claiming the baseline fixed.
