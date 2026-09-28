# F019: Represent generator yields in loops and statement control flow

- **Status:** open
- **Owner:** lila-ir lowering_helpers.rs and resumable planning
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 67 executions across 34 physical files (Bug 0, NotImplemented 67, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F019.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The linear generator suspension planner explicitly has no representation for yields in the attached loop/destructuring/control-flow forms; both unsupported statement kinds and multiple nested suspension positions hit its refusal variants.

## Source evidence

- [crates/lila-ir/src/lowering_helpers.rs:777](../crates/lila-ir/src/lowering_helpers.rs#L777): Explicit missing multiple-suspension loop support.
- [crates/lila-ir/src/lowering_helpers.rs:796](../crates/lila-ir/src/lowering_helpers.rs#L796): Explicit switch/label/do-while/for-in refusal.

## Work

Replace shape-limited linear admission with typed resumable control-flow states for these statement forms, preserving lexical environments and IteratorClose.

## Validation

Run all attached generator/for-of cases and focused next/throw/return tests for each new control-flow edge.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F019.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F019-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/for-of/dstr/array-elem-init-yield-expr.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: generator body: a yield inside a statement kind with no resumable lowering (`switch`, a label, `do`-`while`, `for`-`in`) has no linear suspension plan
```

- `sloppy-script:staging/sm/generators/delegating-yield-8.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: generator body: a loop body requiring multiple suspension positions or an unsupported nested yield has no linear suspension plan
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
