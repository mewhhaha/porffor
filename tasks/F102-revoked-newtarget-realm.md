# F102: Throw when GetFunctionRealm reaches a revoked Proxy newTarget

- **Status:** open
- **Owner:** lila-aot-wasm constructors and functions/function_realm.rs
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F102.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

At least one constructor accepts a newTarget whose prototype lookup revokes its Proxy, missing the required GetFunctionRealm TypeError when the returned prototype is primitive. The common realm helper models Revoked explicitly, so a constructor may bypass it or choose a wrong route.

## Source evidence

- [crates/lila-aot-wasm/src/functions/function_realm.rs:50](../crates/lila-aot-wasm/src/functions/function_realm.rs#L50): Constructor paths must consume Revoked via an explicit throwing route.
- [test262/vendor/test262/test/staging/sm/Proxy/revoked-get-function-realm-typeerror.js:9](../test262/vendor/test262/test/staging/sm/Proxy/revoked-get-function-realm-typeerror.js#L9): Fixture iterates a broad set of intrinsic and user constructors.

## Work

Find the first constructor in the fixture that succeeds, then require every fallback-prototype path to consume the common realm result using the throwing revoked route.

## Validation

Run both full constructor-list fixtures, retaining lookup/revocation ordering.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F102.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F102-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Proxy/revoked-get-function-realm-typeerror.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2061176: Expected a TypeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
