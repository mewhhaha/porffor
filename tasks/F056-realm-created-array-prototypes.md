# F056: Allocate builtin-created arrays in the active function realm

- **Status:** open
- **Owner:** lila-aot-wasm Array.from and AggregateError allocation
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F056.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Array.from and AggregateError.errors return arrays that are not instances of the foreign realm Array. Both rely on array allocation defaulting to the active builtin realm; a global/default prototype is leaking into that allocation.

## Source evidence

- [crates/lila-aot-wasm/src/functions/current_function_realm_array_prototype.rs:14](../crates/lila-aot-wasm/src/functions/current_function_realm_array_prototype.rs#L14): Typed active-realm array prototype helper should own these allocations.
- [test262/vendor/test262/test/staging/sm/Error/AggregateError.js:85](../test262/vendor/test262/test/staging/sm/Error/AggregateError.js#L85): Fixture directly compares the errors array prototype.

## Work

Use the existing typed current-function-realm Array prototype selection at both allocation sites, preserving constructor-provided Array.from results.

## Validation

Run from_realms and AggregateError in both modes, checking prototype identity and mapper this realm.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F056.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F056-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Array/from_realms.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2505104: Expected SameValue(«false», «true») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
