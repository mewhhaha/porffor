# F082: Resolve arguments to the correct function activation binding

- **Status:** open
- **Owner:** lila-ir function environment analysis; lila-aot-wasm arguments materialization
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 2 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F082.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A global variable named arguments incorrectly shadows a function own arguments binding; another fixture triggers an unexpected TDZ error across argument length, arrow capture, var redeclaration and eval cases. Arguments ownership/materialization is not consistently represented as an initialized activation binding.

## Source evidence

- [crates/lila-ir/src/lowering/function_environment.rs:17](../crates/lila-ir/src/lowering/function_environment.rs#L17): Arguments has special storage-name mapping during body initialization.
- [test262/vendor/test262/test/language/statements/function/S13_A15_T5.js:13](../test262/vendor/test262/test/language/statements/function/S13_A15_T5.js#L13): Outer variable must not replace the nested function arguments object.

## Work

Separate the function implicit arguments binding from outer bindings and var redeclarations; retain one initialized cell for arrows/eval and materialize lazily without introducing a TDZ.

## Validation

Run S13_A15_T5 and argumentsLengthOpt with zero/multiple arguments, arrow capture, var arguments and eval mutation.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F082.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F082-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/function/S13_A15_T5.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1639464: #1: __func() !== THE_ANSWER)
```

- `sloppy-script:staging/sm/argumentsLengthOpt.js` — Bug

```text
[origin:unknown] uncaught throw: ReferenceError: wasm-aot completion: object(handle@2040424: lexical binding accessed before initialization)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
