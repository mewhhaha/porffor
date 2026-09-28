# F074: Preserve undefined argument conversion in boxed String concat

- **Status:** open
- **Owner:** lila-ir builtin/property call selection; lila-aot-wasm String.concat
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F074.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

new String(42).concat(function(){}()) returns 42, instead of 42undefined. The comma suggests a wrong callable/array-like route or argument conversion, but the saved report cannot identify the selected builtin. Current String.concat code does call ToString on each argument.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/string.rs:589](../crates/lila-aot-wasm/src/builtins/string.rs#L589): The proper String.concat path converts each argument with ToString.
- [test262/vendor/test262/test/built-ins/String/prototype/concat/S15.5.4.6_A1_T9.js:15](../test262/vendor/test262/test/built-ins/String/prototype/concat/S15.5.4.6_A1_T9.js#L15): Minimal boxed-receiver/undefined-call reproducer.

## Work

Reduce the boxed receiver call, inspect the resolved function and argument tag/argc, and repair dispatch or conversion at the first divergence.

## Validation

Run both original concat modes and compare primitive/boxed receiver calls with explicit and returned undefined.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F074.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F074-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/String/prototype/concat/S15.5.4.6_A1_T9.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636232: #1: new String(42).concat(function(){}()) === "42undefined". Actual: 42,)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
