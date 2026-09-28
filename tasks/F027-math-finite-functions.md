# F027: Implement finite-input transcendental Math operations

- **Status:** open
- **Owner:** lila-aot-wasm builtins/math.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 24 executions across 12 physical files (Bug 24, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F027.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Several Math builtin branches only implement special values (zero, infinities, NaN) and return NaN for ordinary finite inputs. For example Math.exp(1) falls through to NaN. The affected exp/expm1/log1p/log2/hyperbolic/cbrt fixtures expose incomplete numerical kernels.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/math.rs:1844](../crates/lila-aot-wasm/src/builtins/math.rs#L1844): Finite exp inputs other than zero reach the NaN branch.
- [crates/lila-aot-wasm/src/builtins/math.rs:2058](../crates/lila-aot-wasm/src/builtins/math.rs#L2058): Other affected unary branches use special-case-only emission.

## Work

Implement the finite-input mathematical kernels with suitable accuracy and stable near-zero/subnormal handling; retain all specified special cases and signed zero.

## Validation

Run all attached exact/approximation/monotonicity fixtures, with targeted subnormal and boundary checks.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F027.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F027-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Math/acosh-approx.js` — Bug

```text
[origin:unknown] uncaught throw: Error: wasm-aot completion: object(handle@1780736: got NaN, expected a number near 0.0016914556651292944)
```

- `sloppy-script:staging/sm/Math/asinh-approx.js` — Bug

```text
[origin:unknown] uncaught throw: Error: wasm-aot completion: object(handle@1788424: got NaN, expected a number near -6.902103625349695)
```

- `sloppy-script:staging/sm/Math/atanh-approx.js` — Bug

```text
[origin:unknown] uncaught throw: Error: wasm-aot completion: object(handle@2072800: got NaN, expected a number near -6.998237084679027)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
