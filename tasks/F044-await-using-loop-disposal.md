# F044: Dispose await-using resources at each loop scope exit

- **Status:** open
- **Owner:** lila-ir async resource scopes; lila-aot-wasm control_flow.rs
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 6 executions across 3 physical files (Bug 6, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F044.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

All three loop forms execute the trailing push but omit every per-iteration async-disposer side effect. The loop body disposal capability/finalizer is not run on the normal iteration edge, or is reset before it can run.

## Source evidence

- [crates/lila-aot-wasm/src/control_flow.rs:5459](../crates/lila-aot-wasm/src/control_flow.rs#L5459): Async resource capability lifecycle owner.
- [test262/vendor/test262/test/staging/explicit-resource-management/await-using-in-for-statement.js:16](../test262/vendor/test262/test/staging/explicit-resource-management/await-using-in-for-statement.js#L16): Expected values require disposal at each iteration.

## Work

Preserve a separate disposal capability per iteration and route the normal, continue, break and abrupt edges through awaited disposal before advancing the loop.

## Validation

Run for/for-in/for-of await-using fixtures in both modes; verify disposer order and completion preservation.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F044.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F044-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/explicit-resource-management/await-using-in-for-in-statement.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:Test262Error: Test262Error: Actual [2] and expected [0, 1, 2] should have the same contents.\u0020
```

- `sloppy-script:staging/explicit-resource-management/await-using-in-for-statement.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:Test262Error: Test262Error: Actual [3] and expected [0, 1, 2, 3] should have the same contents.\u0020
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
