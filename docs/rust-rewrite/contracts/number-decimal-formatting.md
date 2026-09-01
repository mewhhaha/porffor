# Number decimal formatting

Status: implemented; focused verification is recorded below.

## Closed formatting boundary

Dynamic `Number.prototype.toFixed`, `toExponential` and `toPrecision`
formatting crosses one private `NumberDecimalFormat` boundary. Its three cases
carry only valid emitter state: fixed fraction digits, an exponential policy or
precision significant digits. The exponential case contains a second private
closed domain selecting shortest spelling or explicit fraction digits, so a
shortest mode cannot be paired with either of the other methods. The shared
decimal core consumes both domains with exhaustive matches; there is no
Boolean, string mode, wildcard or equality projection through which a caller
can silently select another method's rounding or placement rules.

Exactly three method wrappers call the core. They continue to own observable
receiver/argument coercion, range errors and omitted-argument behavior.
`toExponential` explicitly selects either its digits-bearing mode or its
shortest mode, while `toFixed` and `toPrecision` supply their validated digit
locals. The core owns the common finite binary64 decomposition, decimal
rounding and final fixed/scientific placement. Non-finite values and the
`toFixed` `10^21` threshold retain their ECMAScript spelling behavior.

The previous empty-string sentinel, precision value table and special-case
integer spelling are absent. In particular, the formatter is not a collection
of fixture-specific answers. The existing Ryū owner remains the authority for
shortest decimal spelling; this contract covers the distinct digit-constrained
rounding and placement algorithms.

## Durable evidence

`number_decimal_formatting_structure.rs` pins the exact private domain, the
exhaustive shared consumer, the three-caller census and removal of the obsolete
empty/table/magic fallbacks.

`wasm_number_decimal_formatting.js` drives all three methods through dynamic
functions so literal folding cannot satisfy the assertions. Its finite matrix
independently covers halfway rounding, carry propagation, fixed and scientific
placement, positive and negative values, signed and ordinary zero, omitted
arguments, the `toFixed` large-value threshold, the `toPrecision` notation
thresholds, minimum subnormal, maximum finite, non-finite spellings and a
representative binary64 value below its apparent decimal midpoint. Long-digit
cases prove that supplied precision rounds the exact binary64 value rather than
its shortest spelling, and a lower-threshold carry proves that notation is
selected after rounding. The older
`wasm_number_builtin_family.js` remains registered as the aggregate Number
builtin regression.

```sh
cargo test -p lila-aot-wasm --test number_decimal_formatting_structure --quiet
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_formats_dynamic_numbers_with_decimal_rounding -- --exact --test-threads=1
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_number_builtin_family_fixture -- --exact --test-threads=1
```

This focused closure does not claim ECMA-402 locale formatting or the complete
pinned Number Test262 tree.

The focused structure targets for this contract, the Number policy domain and
shortest-integral separation pass `12/12`. The dynamic decimal matrix and the
two older Number CLI regressions pass `3/3`. The exact pinned `toFixed`
`exactness.js`, `toExponential` `return-values.js` and `toPrecision`
`return-values.js` leaves pass all `6/6` sloppy/strict Wasm-AOT executions with
every non-success bucket at zero. The coordinated semantic golden passes `2/2`
in 672.44 seconds with 680 dumps: it adds only the dynamic decimal fixture,
removes none and leaves all 679 retained dumps structurally equal after
accounting normalization. Function and local topology is unchanged; the
retained emitted-code increase is the expected shared formatter body.
