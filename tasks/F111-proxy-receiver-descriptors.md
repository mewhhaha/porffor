# F111: Respect Proxy receiver descriptors before defining a property

- **Status:** open
- **Owner:** lila-aot-wasm OrdinarySet and Proxy receiver operations
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F111.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

OrdinarySet with a Proxy receiver invokes defineProperty and throws the fixture string instead of rejecting an accessor/nonwritable receiver descriptor with TypeError. The receiver GetOwnProperty result is not checked before overwriting.

## Source evidence

- [test262/vendor/test262/test/staging/sm/Proxy/proxy-no-receiver-overwrite.js:22](../test262/vendor/test262/test/staging/sm/Proxy/proxy-no-receiver-overwrite.js#L22): Trap must remain uncalled for invalid receiver descriptors.
- [crates/lila-aot-wasm/src/objects.rs:15074](../crates/lila-aot-wasm/src/objects.rs#L15074): Ordinary Set emission owner.

## Work

Follow OrdinarySetWithOwnDescriptor receiver checks before CreateDataProperty/DefineOwnProperty and propagate a failed Set to strict PutValue.

## Validation

Run the original strict proxy-no-receiver-overwrite fixture and verify defineProperty is never called.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F111.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F111-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `strict-script:staging/sm/Proxy/proxy-no-receiver-overwrite.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1632528: Thrown value was not an object!)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
