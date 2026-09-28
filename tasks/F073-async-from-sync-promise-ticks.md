# F073: Preserve AsyncFromSyncIterator promise resolution and job order

- **Status:** open
- **Owner:** lila-aot-wasm async iterator and for-await scheduling
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F073.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

for-await over a sync iterator yielding an already resolved Promise omits constructor lookups and promise ticks. An identity/fast path bypasses required PromiseResolve/await continuation jobs.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/async_iterator.rs:1](../crates/lila-aot-wasm/src/builtins/async_iterator.rs#L1): Async iterator builtin ownership.
- [test262/vendor/test262/test/language/statements/for-await-of/ticks-with-sync-iter-resolved-promise-and-constructor-lookup.js:7](../test262/vendor/test262/test/language/statements/for-await-of/ticks-with-sync-iter-resolved-promise-and-constructor-lookup.js#L7): Fixture makes constructor accesses and microtask ticks observable.

## Work

Trace AsyncFromSyncIteratorContinuation and for-await awaiting separately, preserving every required PromiseResolve constructor access and job boundary.

## Validation

Run both attached tick-trace modes and compare the complete expected log.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F073.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F073-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/for-await-of/ticks-with-sync-iter-resolved-promise-and-constructor-lookup.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:Test262Error: Test262Error: Actual [pre, constructor, tick 1, loop, tick 2, post, tick 3, tick 4] and expected [pre, constructor, constructor, tick 1, tick 2, loop, constructor, tick 3, tick 4, post] should have the same contents. Ticks and constructor lookups
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
