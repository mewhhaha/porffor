# F026: Lower suspension in for/while loop heads and abrupt control

- **Status:** open
- **Owner:** lila-ir lowering for_loop and while_loop
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 25 executions across 25 physical files (Bug 0, NotImplemented 25, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F026.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Loop lowering explicitly rejects a head containing await or loop control that cannot be represented by its eager-head continuation. Top-level await exposes this restriction; valid source has no resumable head plan.

## Source evidence

- [crates/lila-ir/src/lowering/for_loop.rs:65](../crates/lila-ir/src/lowering/for_loop.rs#L65): Exact eager-head refusal in for-loop lowering.
- [crates/lila-ir/src/lowering/while_loop.rs:15](../crates/lila-ir/src/lowering/while_loop.rs#L15): Same restriction in while-loop lowering.

## Work

Give loop test/update/head evaluation resumable states with explicit break/continue targets, shared with top-level await module lowering.

## Validation

Run all attached syntax variants and verify suspension order, loop continuation, and abrupt completion.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F026.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F026-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/top-level-await/syntax/for-await-expr-array-literal.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: async loop with await requires an eager loop head without break or continue
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
