# Versioned integer arithmetic and bitwise generation

The public `ArithmeticGenerationPlan` requires a closed `ArithmeticGrammar`.
The existing CLI parser projects `--grammar` into that same domain and defaults
new requests to `IntegerBitwiseV2`. Seeds, check/depth bounds, replay budgets,
schema-v1 source closure and the two explicit oracle gates remain consumed by
the existing campaign. There is no separate runner or reducer.

`IntegerArithmeticV1` remains available only by explicit plan/CLI selection.
It keeps the historical low-bit Add/Sub operation selection, SplitMix64-v1 draw
order, `-32..=32` leaves, parentheses, numeric inequality checks, corpus id and
filename. The existing committed seed-1 fixture test explicitly selects V1;
its source bytes and expected fingerprint are unchanged. This is reproducible
historical corpus identity, not a JavaScript implementation compatibility path.

`IntegerBitwiseV2` draws from Add/Sub, `&`, `|`, `^`, `<<`, `>>`, `>>>` and `~`.
The AST distinguishes unary and binary nodes, so a generated shift always has
two operands and complement has one. Rendering, numeric evaluation, complexity
and reduction exhaustively consume both arities. Leaves retain the small
integer range; nested shifts expose the complete signed/unsigned 32-bit result
ranges. All intermediate results must still construct `ExactInteger`, whose
shared domain is finite integral binary64 safe integers and both zero signs.
V1/V2 produce only positive zero; V3 owns signed-zero arithmetic results.

The evaluator accepts `ExactInteger` operands, then converts modulo 2^32 using
`rem_euclid`. ToInt32 reinterprets those low bits as a signed 32-bit value.
Both signed shifts and unsigned right shift mask the converted right operand
to five bits. Left shift wraps the 32-bit word; signed right shift preserves
the sign; unsigned right shift returns an integer in `0..=2^32-1`.
These rules follow
[ToInt32/ToUint32](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toint32)
and the [Number bitwise and shift operations](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-leftShift).

V1/V2 do not generate negative zero. Their admitted Add/Sub, bitwise operations
and nonzero negative literal spellings cannot produce it from the admitted
positive-zero input domain. V2 emits `Object.is` checks so a backend's incorrect
negative-zero result still fails. The shared evaluator now represents both
zero signs for the separate bounded
[`IntegerProductV3` grammar](differential-integer-product-grammar.md), which owns
multiply and arithmetic unary minus. Those operations remain outside V1/V2;
fractions, infinities, NaN, BigInt and division remain outside all three.

A generated program carries its grammar and validates every check against it
at construction. A V1 program therefore cannot hold a bitwise node. Corpus
materialization rejects a plan with another grammar; ids and filenames derive
from the held grammar. Range-removal, child replacement, unary-operand and
literal shrinking retain that grammar and rebuild each expected result. Only
strictly decreasing nonempty programs become reducer candidates. The existing
typed mismatch direction, backend phase and replay-limit owner remain intact.

Source regressions cover modulo wrap at both signs and the safe-integer bounds,
signed/unsigned shifts and masked counts, source grammar identity, unary/shift
reduction and positive-zero checks. A consumed hand-authored Script probe uses
the ordinary Wasm-first replay with the feature-gated spec-exec oracle; it is
not a generated or executed corpus capture.

This 2026-10-03 change is source-only and uncompiled. No generator, oracle,
CLI runtime or test command ran. Fresh compilation, these regressions, the
retained exact V1 committed corpus, a V2 deterministic capture and the broader
campaign checkpoint remain pending. T25 and full conformance remain open.
