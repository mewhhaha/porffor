# F003: Investigate deep WeakMap construction and GC traversal timeout

- **Status:** open
- **Owner:** lila-aot-wasm weak collections and GC traversal
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 0, NotImplemented 0, Crash 2)

[Backlog](README.md) · [Exact execution list](cases/F003.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Both runs are interrupted by Wasm epoch timeout at about 60 seconds. The fixture builds a 99,999-edge WeakMap chain, requests GC, then traverses it. Saved logs cannot identify whether insertion, collection, or traversal dominates; do not classify timeout as successful completion or just raise the budget.

## Source evidence

- [test262/vendor/test262/test/staging/sm/regress/regress-1507322-deep-weakmap.js:12](../test262/vendor/test262/test/staging/sm/regress/regress-1507322-deep-weakmap.js#L12): Large linked WeakMap fixture with explicit host GC.
- [crates/lila-aot-wasm/src/heap_weak_edges.rs:1](../crates/lila-aot-wasm/src/heap_weak_edges.rs#L1): Weak edge ownership and traversal implementation.

## Work

Measure each phase on a reduced chain and inspect lookup/ephemeron traversal complexity, preserving correct reachability and bounded resource use.

## Validation

Rerun the exact fixture and scaling cases after diagnosis within the same execution timeout.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F003.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F003-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/regress/regress-1507322-deep-weakmap.js` — Crash

```text
[origin:unknown] timeout exceeded after 60051ms (wasm epoch interrupt, bound 60000ms)
```

- `strict-script:staging/sm/regress/regress-1507322-deep-weakmap.js` — Crash

```text
[origin:unknown] timeout exceeded after 60083ms (wasm epoch interrupt, bound 60000ms)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
