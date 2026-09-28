# F035: Allow nested await within for-await-of bodies

- **Status:** open
- **Owner:** lila-ir lowering/for_of.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 12 executions across 12 physical files (Bug 0, NotImplemented 12, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F035.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The lowering checks Contains(AwaitExpression) in a for-await body and rejects it even when a resumable plan exists. Valid explicit awaits therefore never reach codegen.

## Source evidence

- [crates/lila-ir/src/lowering/for_of.rs:160](../crates/lila-ir/src/lowering/for_of.rs#L160): Exact body-await guard.

## Work

Add explicit body-await continuation states distinct from the implicit iterator-next await and preserve iterator closing.

## Validation

Run the attached for-await body-await top-level variants, checking promise ticks and completion propagation.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F035.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F035-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/top-level-await/syntax/for-await-await-expr-array-literal.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: explicit await in for-await-of body
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
