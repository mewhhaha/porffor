# F091: Install and execute change-by-copy Array methods in created realms

- **Status:** open
- **Owner:** lila-aot-wasm created-realm Array intrinsics and allocation
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F091.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Calling foreign Array.prototype.with/toSpliced/toReversed/toSorted hits a nullish property/method read before expected result-realm assertions. Created realm prototype method installation or function environment is incomplete; the exact first missing method needs isolation.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/host.rs:4104](../crates/lila-aot-wasm/src/builtins/host.rs#L4104): Created realm Array prototype construction owner.
- [test262/vendor/test262/test/staging/sm/Array/change-array-by-copy-cross-compartment-create.js:12](../test262/vendor/test262/test/staging/sm/Array/change-array-by-copy-cross-compartment-create.js#L12): Fixture starts with the foreign with method.

## Work

Compare every created-realm Array prototype entry with the entry-realm registry and carry the method defining realm into returned-array allocation.

## Validation

Run both change-array-by-copy-cross-compartment-create variants and inspect all four result prototypes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F091.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F091-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Array/change-array-by-copy-cross-compartment-create.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@2502312: Cannot read properties of null or undefined)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
