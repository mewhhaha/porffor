# Math extremum argument reduction

Status: normative for the Wasm-AOT `Math.min` and `Math.max` argument walk.

## Specification boundary

[`Math.max`](https://tc39.es/ecma262/#sec-math.max) and
[`Math.min`](https://tc39.es/ecma262/#sec-math.min) accept a variadic argument
list. Each operation first applies `ToNumber` to every argument in source
order. Only after all conversions complete does it choose the extremum. This
has three observable consequences:

- no finite argument-count prefix may stand in for the complete list;
- encountering `NaN` does not suppress conversion of later arguments; and
- an abrupt `ToNumber` completion stops conversion before the following
  argument and is propagated unchanged.

The reduction identities are `-Infinity` for `Math.max` and `+Infinity` for
`Math.min`. Positive zero wins over negative zero for `Math.max`; negative zero
wins over positive zero for `Math.min`.

## Pre-change defect

The call ABI already carries a runtime `argc` and an internal `argv` Array of
arbitrary length. `emit_math_extremum_builtin` did not consume that domain. It
special-cased argument zero, then conditionally read indices one and two. A
fourth or later argument was neither coerced nor reduced. This changed the
numeric result, hid later conversion side effects and hid later abrupt
completions.

The three-argument pinned Sputnik matrix could not expose the cap. The pinned
`Math.min_each-element-coerced.js` and `Math.max_each-element-coerced.js`
instead establish the independent rule that `NaN` does not end the coercion
pass.

## Closed reduction protocol

The private `MathExtremum` enum is the complete backend domain:

- `Minimum` owns the `+Infinity` identity and `f64.min` reduction; and
- `Maximum` owns the `-Infinity` identity and `f64.max` reduction.

There is no Boolean selector and no identity or instruction supplied by a
call site. Both projections match the enum exhaustively. Adding another
extremum without choosing its identity or reduction is therefore a Rust
compile error.

The emitter initializes the accumulator from that policy and walks the
runtime `argv` with an index local until `index == argc`. Each iteration reads
exactly that internal argument entry, applies the shared `ToNumber` boundary,
stores the produced payload, explicitly routes a pending Throw completion out
of the builtin, then combines the normal result and advances by one. Reduction
and increment are therefore unreachable after abrupt conversion even though
the loop introduces two additional Wasm control frames. The loop has no `NaN`
exit. Although the specification describes conversion and reduction as two
passes, fusing them is observationally equivalent here: the reduction performs
no JavaScript operation, cannot complete abruptly and does not control whether
the next argument is converted.

WebAssembly `f64.min` and `f64.max` propagate NaN and define the required
opposite-zero preference, so the backend uses those instructions rather than
reconstructing a second comparison policy.

## Durable witness

`wasm_math_min_max_arity.js` covers both operations with more than eight
arguments, extrema appearing after the old three-argument prefix, and signed
zero appearing after that prefix. It also uses observable `valueOf` hooks to
prove that:

- every later argument is converted even when an earlier conversion produced
  `NaN`; and
- a throw from a later conversion is preserved while the following argument
  remains unconverted.

The witness uses the normal CLI Wasm product path. It is not a Test262 rewrite
or a static materialization.

## Owned files and verification

- `crates/lila-aot-wasm/src/builtins/math.rs`
- `crates/lila-cli/tests/fixtures/wasm_math_min_max_arity.js`
- `crates/lila-cli/tests/cli/language_numerics.rs`
- `scripts/check-module-boundaries.sh`
- this contract
- `tasks/20-number-bigint-math-json.md`

Static freeze gates are scoped `rustfmt --check`, `node --check`, source
inventory and `git diff --check`. `scripts/check-module-boundaries.sh` pins the
typed policy, full-vector loop and exact convert/store/throw-route/reduce/
advance/backedge order. It also pins exact counts for both index reads, both
index writes, the increment constant, addition and loop branch, so deleting or
duplicating any part of the walk fails the static gate. Its complete run can
still be blocked by an independently active shared-file budget. Cargo, the
focused CLI regression, the pinned `Math/min` and `Math/max` trees, the complete
Math tree and the broad batch ladder remain owned by the central verifier.

## Nonclaims

This seam does not close `Math.hypot`, `Math.sumPrecise`, implementation-
approximated transcendental functions, Number formatting, BigInt, JSON, the
complete Math tree or T20. It does not change the standard `length` property of
either extremum function, which remains 2.
