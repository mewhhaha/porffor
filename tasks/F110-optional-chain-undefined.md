# F110: Preserve undefined values on optional-chain short circuits

- **Status:** open
- **Owner:** lila-ir optional-chain lowering; lila-aot-wasm conditional expressions
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F110.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The optional-chain fixture returns numeric zero where undefined is expected. The aggregate assertion message does not identify the expression, so the likely tag/payload merge or short-circuit default needs a reduced case before claiming a root cause.

## Source evidence

- [crates/lila-ir/src/lowering.rs:11870](../crates/lila-ir/src/lowering.rs#L11870): Optional-chain IR construction entry point.
- [test262/vendor/test262/test/staging/sm/expressions/optional-chain.js:18](../test262/vendor/test262/test/staging/sm/expressions/optional-chain.js#L18): Fixture begins with explicit nullish short-circuit results.

## Work

Locate the first failing expression and inspect the value kind/tag on each optional branch, including nested calls and computed keys.

## Validation

Run the original sloppy optional-chain fixture and compare value, side-effect count and receiver binding at each branch.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F110.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F110-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/expressions/optional-chain.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2134776: Expected SameValue(«0», «undefined») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
