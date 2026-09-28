# F033: Add resumable for-in enumeration

- **Status:** open
- **Owner:** lila-ir lowering/for_in.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 12 executions across 12 physical files (Bug 0, NotImplemented 12, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F033.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

for-in lowering explicitly emits unsupported when its loop contains await; the property enumeration state is not carried across suspension.

## Source evidence

- [crates/lila-ir/src/lowering/for_in.rs:14](../crates/lila-ir/src/lowering/for_in.rs#L14): Exact unconditional await refusal.

## Work

Represent enumeration state and binding initialization in the resumable IR, including nullish RHS behavior and abrupt completion.

## Validation

Run all attached for-in top-level await expression variants.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F033.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F033-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/top-level-await/syntax/for-in-await-expr-array-literal.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: await inside a for-in loop
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
