# F108: Return observable iterator-result properties for arguments iteration

- **Status:** open
- **Owner:** lila-aot-wasm Array iterator on arguments objects; harness deep equality
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F108.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

arguments[Symbol.iterator]().next() compares as Object {} rather than {value:10, done:false}. It is not yet established whether the iterator result lacks properties, has wrong descriptors, or the host deep-equality helper cannot observe them.

## Source evidence

- [test262/vendor/test262/test/staging/sm/Function/arguments-iterator.js:53](../test262/vendor/test262/test/staging/sm/Function/arguments-iterator.js#L53): First result must expose value/done to the harness.
- [crates/lila-aot-wasm/src/objects/arguments_properties.rs:64](../crates/lila-aot-wasm/src/objects/arguments_properties.rs#L64): Arguments exotic property access owner.

## Work

Reduce the first next result and inspect own keys/descriptors/values before deepEqual; fix iterator allocation or harness observation at the actual divergence.

## Validation

Run the original sloppy arguments-iterator case, including mapped mutations and deleted/redefined iterator properties.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F108.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F108-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Function/arguments-iterator.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2271712: Expected Object {} to be structurally equal to Object {value: 10, done: false}. )
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
