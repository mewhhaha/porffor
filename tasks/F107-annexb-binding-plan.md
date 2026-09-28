# F107: Resolve Annex B block function storage consistently

- **Status:** open
- **Owner:** lila-ir binding planning and lowering.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 0, NotImplemented 1, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F107.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Annex B lowering resolves f to scoped.lex storage but its declaration plan expects annexb.block storage. The identity consistency check catches incompatible binding plans for a deprecated redeclaration.

## Source evidence

- [crates/lila-ir/src/lowering.rs:5862](../crates/lila-ir/src/lowering.rs#L5862): Exact disagreement check for planned versus resolved storage.

## Work

Give the declaration and its Annex B copy one binding identity across redeclaration/scope planning rather than independently deriving names.

## Validation

Run the exact sloppy block-scoped-functions-deprecated-redecl fixture plus Annex B duplicate-owner cases.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F107.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F107-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/lexical-environment/block-scoped-functions-deprecated-redecl.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: Annex B declaration `f` resolved block storage `$scoped.lex.242.15.242.16.f` instead of planned `$annexb.block.6136.6162.f`
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
