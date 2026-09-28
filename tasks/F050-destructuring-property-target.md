# F050: Support super and parenthesized property destructuring targets

- **Status:** open
- **Owner:** lila-ir lower_property_assignment_target
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 0, NotImplemented 4, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F050.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Property target lowering rejects AST shapes used by super-property and parenthesized destructuring assignments. These are source-to-IR gaps before execution.

## Source evidence

- [crates/lila-ir/src/lowering.rs:15034](../crates/lila-ir/src/lowering.rs#L15034): The fallback returns unsupported for unhandled reference shapes.

## Work

Lower the valid reference forms with deferred PutValue, receiver preservation, and required key/RHS evaluation order.

## Validation

Run both script modes of superPropDestructuring and destructuring-pattern-parenthesized.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F050.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F050-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/class/superPropDestructuring.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: property assignment target
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
