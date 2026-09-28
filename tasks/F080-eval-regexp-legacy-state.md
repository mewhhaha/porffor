# F080: Publish RegExp legacy match state from compiled eval execution

- **Status:** open
- **Owner:** lila-aot-wasm RegExp legacy statics; lila-ir eval/ASI parsing
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F080.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The fixture evaluates a labelled loop followed by a newline regexp literal and sees empty RegExp.lastMatch instead of y. The log does not distinguish incorrect ASI/eval execution from failure to publish match statics into the caller realm.

## Source evidence

- [test262/vendor/test262/test/staging/sm/statements/regress-642975.js:20](../test262/vendor/test262/test/staging/sm/statements/regress-642975.js#L20): Fixture reads the shared legacy match result after eval.
- [crates/lila-aot-wasm/src/builtins/standard.rs:29950](../crates/lila-aot-wasm/src/builtins/standard.rs#L29950): Legacy RegExp state access owner.

## Work

Reduce the two eval strings, inspect their AST/IR around break/continue line termination, and confirm successful match updates the correct realm legacy record.

## Validation

Run both regress-642975 modes for break and continue strings and check lastMatch immediately after eval.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F080.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F080-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/statements/regress-642975.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1959288: Expected SameValue(«""», «"y"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
