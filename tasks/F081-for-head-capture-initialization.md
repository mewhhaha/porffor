# F081: Capture initialized for-let head bindings before per-iteration cloning

- **Status:** open
- **Owner:** lila-ir for-loop binding lifecycle; lila-aot-wasm lexical environments
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F081.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A closure created in a later for-let initializer throws TDZ when reading an earlier initialized binding. The head binding initialization and first per-iteration environment cloning appear out of order or capture the wrong cell.

## Source evidence

- [crates/lila-ir/src/lowering.rs:3963](../crates/lila-ir/src/lowering.rs#L3963): For-head expressions have their own TDZ handling.
- [test262/vendor/test262/test/staging/sm/lexical-environment/bug-1216623.js:12](../test262/vendor/test262/test/staging/sm/lexical-environment/bug-1216623.js#L12): Closure in the second initializer observes the first initialized cell.

## Work

Initialize head declarations left-to-right, bind initializer closures to the head environment, and clone only the per-iteration bindings at the specified boundary.

## Validation

Run both bug-1216623 modes; closure values must remain the original zero/eleven as the loop bindings increment.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F081.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F081-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/lexical-environment/bug-1216623.js` — Bug

```text
[origin:unknown] uncaught throw: ReferenceError: wasm-aot completion: object(handle@1635208: lexical binding accessed before initialization)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
