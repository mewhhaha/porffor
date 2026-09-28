# F002: Throw TypeError instead of trapping on inherited RegExp methods

- **Status:** open
- **Owner:** lila-aot-wasm property call dispatch and RegExp receiver validation
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 12 executions across 6 physical files (Bug 0, NotImplemented 0, Crash 12)

[Backlog](README.md) · [Exact execution list](cases/F002.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Twelve fixtures install RegExp exec/test on Object.prototype and invoke the inherited method on a primitive. The recorded stack contains only lila::main followed by unreachable, so it does not identify the exact guard. The failure occurs before the expected catchable receiver TypeError.

## Source evidence

- [test262/vendor/test262/test/built-ins/RegExp/prototype/exec/S15.10.6.2_A2_T7.js:14](../test262/vendor/test262/test/built-ins/RegExp/prototype/exec/S15.10.6.2_A2_T7.js#L14): The fixture installs the inherited builtin and expects TypeError.
- [crates/lila-aot-wasm/src/builtins/regexp.rs:1](../crates/lila-aot-wasm/src/builtins/regexp.rs#L1): RegExp builtin dispatch owner; exact trap location remains unresolved.

## Work

Reduce one Boolean/number primitive fixture, inspect the emitted trap offset, and correct boxing/method dispatch or receiver error propagation at the responsible guard.

## Validation

Run all twelve original exec/test primitive-receiver cases in both modes; each must catch TypeError without a runtime trap.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F002.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F002-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/RegExp/prototype/exec/S15.10.6.2_A2_T7.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0: 0x41ebc7 - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

- `sloppy-script:built-ins/RegExp/prototype/exec/S15.10.6.2_A2_T8.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0: 0x41ebcd - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

- `sloppy-script:built-ins/RegExp/prototype/exec/S15.10.6.2_A2_T9.js` — Crash

```text
[origin:unknown] wasmtime execution trapped: error while executing at wasm backtrace:
    0: 0x41ebcf - lila!lila::main

Caused by:
    wasm trap: wasm `unreachable` instruction executed

```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
