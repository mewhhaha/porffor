# F087: Use one Math property installation order across realms

- **Status:** open
- **Owner:** lila-aot-wasm bootstrap.rs and created-realm host.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F087.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Entry-realm Math installs min/max after trunc, while created-realm Math installs max/min before pow. Reflect.ownKeys therefore differs between otherwise equivalent realm intrinsics exactly as recorded.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/bootstrap.rs:901](../crates/lila-aot-wasm/src/builtins/bootstrap.rs#L901): Entry realm appends min/max after trunc.
- [crates/lila-aot-wasm/src/builtins/host.rs:1756](../crates/lila-aot-wasm/src/builtins/host.rs#L1756): Created realm places max/min before pow.

## Work

Drive both Math installations from one ordered intrinsic catalog rather than duplicate method arrays.

## Validation

Run both ownKeys staging modes and compare complete Math own-key arrays in entry and created realms.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F087.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F087-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Reflect/ownKeys.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2759616: Actual [E, LN10, LN2, LOG10E, LOG2E, PI, SQRT1_2, SQRT2, abs, acos, acosh, asin, asinh, atan, atan2, atanh, cbrt, ceil, clz32, cos, cosh, exp, expm1, f16round, floor, fround, hypot, imul, log, log10, log1p, log2, max, min, pow, random, round, sign, sin, sinh, sqrt, sumPrecise, tan, tanh, trunc, Symbol(Symbol.toStringTag)] and expected [E, LN10, LN2, LOG10E, LOG2E, PI, SQRT1_2, SQRT2, abs, acos, acosh, asin, asinh, atan, atan2, atanh, cbrt, ceil, clz32, cos, cosh, exp, expm1, f16round, floor, fround, hypot, imul, log, log10, log1p, log2, pow, random, round, sign, sin, sinh, sqrt, sumPrecise, tan, tanh, trunc, min, max, Symbol(Symbol.toStringTag)] should have the same contents. )
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
