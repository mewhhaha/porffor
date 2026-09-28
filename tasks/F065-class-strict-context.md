# F065: Carry class strictness through constructors, heritage and nested functions

- **Status:** open
- **Owner:** lila-front class parsing; lila-ir function/class analysis
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 3 executions across 3 physical files (Bug 3, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F065.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Only sloppy outer-script variants of class strictness fixtures fail: nested constructor assignments create globals and restricted arguments/failed property writes do not throw. Strictness appears inherited from the outer script or lost in class heritage/computed-name/nested function contexts.

## Source evidence

- [crates/lila-ir/src/lowering/class_definition.rs:1](../crates/lila-ir/src/lowering/class_definition.rs#L1): Class definition lowering owner.
- [test262/vendor/test262/test/staging/sm/class/strictExecution.js:9](../test262/vendor/test262/test/staging/sm/class/strictExecution.js#L9): Fixture covers constructor, computed-property and heritage strictness.

## Work

Derive class strict context once and carry it into constructors, heritage, computed names, nested functions, and arguments object construction.

## Validation

Run all attached class strictness cases in their declared variants and check ReferenceError, TypeError and restricted arguments access.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F065.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F065-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/class/definition/constructor-strict-by-default.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1641280: Expected a ReferenceError to be thrown but no exception was thrown at all)
```

- `sloppy-script:language/statements/class/strict-mode/arguments-callee.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636808: Expected a TypeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
