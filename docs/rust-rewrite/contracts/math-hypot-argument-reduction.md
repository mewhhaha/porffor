# Math.hypot argument reduction

Status: normative for the Wasm-AOT `Math.hypot` argument walk and finite
norm approximation.

## Specification boundary

[`Math.hypot`](https://tc39.es/ecma262/#sec-math.hypot) accepts a runtime
argument list of arbitrary length. It first applies `ToNumber` to every
argument in source order. Only after every conversion completes does it choose
the result:

1. any positive or negative infinity produces positive infinity;
2. otherwise any NaN produces NaN;
3. otherwise an all-zero list produces positive zero; and
4. otherwise the result approximates the square root of the mathematical sum
   of squares.

An abrupt conversion stops before the following argument and is propagated
unchanged. Infinity and NaN are not conversion-loop exit conditions. The
finite approximation must also avoid the intermediate overflow and underflow
created by squaring unscaled binary64 values.

## Pre-change defect

The call ABI already carries runtime `argc` and an internal `argv` Array. The
old backend did not consume that domain. It squared arguments zero and one,
then used a Rust `2usize..7usize` loop to coerce arguments two through six
without adding them to the norm. Argument seven and every later argument were
not read or coerced at all.

This made `Math.hypot(3, 4, 12)` return `5`, hid side effects and abrupt
completions in the eighth and later arguments, and ignored later Infinity and
NaN values. Direct squaring also made a one-element call overflow or underflow
before the square root: values such as `1e308` became Infinity and values such
as `1e-300` became zero.

The previous Wasm evidence covered only seven of sixteen Math shards. Its
`Math.hypot` numeric result case had two arguments. The seven-argument abrupt
case ended at exactly the old coercion cap, while its three-zero case could not
observe that the third zero was discarded. That partial green evidence did
not exercise the general runtime domain.

## Closed reduction protocol

`emit_math_hypot_argument_reduction` is the sole producer of a private,
non-`Copy`, `#[must_use]` `CompletedMathHypotReduction`. Its fields are the
Wasm locals for the scaled finite norm and the two exceptional observations.
The producer initializes that state, then emits one runtime loop from index
zero until `index == argc`.

Each iteration reads exactly `argv[index]`, applies the shared `ToNumber`
boundary, stores its normal payload, and immediately routes a pending Throw
completion out of the builtin. It then classifies and folds the Number without
performing any JavaScript operation:

- infinity records the infinity observation;
- NaN records the NaN observation;
- zero changes no finite state; and
- a finite nonzero magnitude updates a scale and a sum of scaled squares.

For a magnitude `x` greater than the current scale, the reducer multiplies the
old sum by `(scale / x)^2`, adds one, and replaces the scale with `x`. For a
smaller magnitude it adds `(x / scale)^2`. Every squared ratio is at most one,
so representable inputs cannot overflow merely because they were squared.

The internal fold is total, cannot invoke user code and cannot complete
abruptly. Fusing it with conversion is therefore observationally equivalent
to the specification's separate conversion and inspection passes, provided
that it never controls the loop. Both exceptional observations deliberately
fall through to the common index increment and backedge.

Only `emit_finish_math_hypot` accepts the completed witness. It consumes the
state after the loop and emits the required precedence: Infinity, then NaN,
then positive zero when the scale is zero, then `scale * sqrt(sum)`. There is
no public raw-local constructor and no Boolean policy argument with which a
caller can reverse that precedence.

## Durable witness

`wasm_math_hypot_argument_reduction.js` covers zero through three arguments,
a contributing argument after the old cap, Infinity and NaN outside the old
two-value result prefix, exact left-to-right coercion, an eighth-argument
abrupt completion with the following argument untouched, positive-zero output
from mixed signed zeros, and large and tiny finite triples whose naive squares
overflow or underflow.

The fixture uses the normal CLI Wasm product path. It is not a Test262 rewrite
or static materialization. A source-structure test separately pins the
non-copy phase witness, complete `argc`/`argv` loop, conversion/throw/fold/
advance ordering, lack of a fixed argument prefix, scaled reduction and final
exceptional precedence.

## Owned files and verification

- `crates/lila-aot-wasm/src/builtins/math.rs`
- `crates/lila-aot-wasm/tests/math_hypot_argument_reduction_structure.rs`
- `crates/lila-cli/tests/fixtures/wasm_math_hypot_argument_reduction.js`
- `crates/lila-cli/tests/cli/language_numerics.rs`
- this contract
- `tasks/20-number-bigint-math-json.md`

The lane's freeze gates are scoped `rustfmt --check`, `node --check`, source
inventory and `git diff --check`. Cargo, the focused CLI fixture, the pinned
`built-ins/Math/hypot` tree, the complete Math tree and the broad batch ladder
remain owned by the central verifier.

## Nonclaims

This seam does not claim a correctly rounded last bit for implementation-
approximated `Math.hypot`. It does not close `Math.sumPrecise`, other
implementation-approximated transcendental functions, the existing T20
shortcut inventory, the complete Math tree or T20. It does not establish a
current-pin baseline delta before the active baseline reaches Math.
