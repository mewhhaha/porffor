# F034: Preserve iterator records across awaited for-of heads

- **Status:** open
- **Owner:** lila-ir lowering/for_of.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 12 executions across 12 physical files (Bug 0, NotImplemented 12, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F034.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The for-of async path explicitly requires an eager iterable and a body without break or continue. Awaited iterable/head expressions fail that admission check before a resumable iterator record is built.

## Source evidence

- [crates/lila-ir/src/lowering/for_of.rs:172](../crates/lila-ir/src/lowering/for_of.rs#L172): Exact restrictive admission condition.

## Work

Model iterator acquisition and head/body suspension as separate resumable states, with IteratorClose on abrupt exits.

## Validation

Run the attached top-level await for-of cases and focused awaited iterable/break/continue regressions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F034.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F034-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/top-level-await/syntax/for-of-await-expr-array-literal.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: async for-of with await requires an eager iterable and a body without break or continue
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
