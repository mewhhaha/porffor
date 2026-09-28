# F099: Correct RegExp split species and lastIndex protocol

- **Status:** open
- **Owner:** lila-aot-wasm RegExp Symbol.split
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F099.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The original trace fixture observes 0 where undefined is expected. It customizes species construction, lastIndex getters/setters and exec results; the saved assertion does not reveal which invocation diverged. Do not merge with replacement-order failures solely because both are traces.

## Source evidence

- [test262/vendor/test262/test/staging/sm/RegExp/split-trace.js:65](../test262/vendor/test262/test/staging/sm/RegExp/split-trace.js#L65): The species-created splitter validates each lastIndex write.
- [crates/lila-aot-wasm/src/builtins/string.rs:1520](../crates/lila-aot-wasm/src/builtins/string.rs#L1520): Split protocol implementation owner.

## Work

Reduce the first failing split trace case and audit splitter construction, ToLength(lastIndex), progress handling, captures and CreateDataProperty result writes.

## Validation

Run both split-trace modes and compare complete logs, including inherited Array setter observations.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F099.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F099-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/split-trace.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2036320: Expected SameValue(«0», «undefined») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
