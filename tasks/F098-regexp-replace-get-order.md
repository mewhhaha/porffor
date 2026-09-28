# F098: Read RegExp replacement match properties in spec order

- **Status:** open
- **Owner:** lila-aot-wasm RegExp Symbol.replace
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F098.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The recorded trap trace reads result[0] and result[index] before result[length], while the fixture expects length first during replacement processing. This is an observable getter-order error after match collection.

## Source evidence

- [test262/vendor/test262/test/staging/sm/RegExp/replace-trace.js:44](../test262/vendor/test262/test/staging/sm/RegExp/replace-trace.js#L44): The fixture records each observable result access.
- [crates/lila-aot-wasm/src/builtins/string.rs:1520](../crates/lila-aot-wasm/src/builtins/string.rs#L1520): RegExp replacement and match extraction implementation.

## Work

Stage match length, matched string, position, captures and groups in the required order before substitution; preserve abrupt completions.

## Validation

Run both replace-trace executions and compare the full getter/call log.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F098.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F098-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/replace-trace.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2013400: Expected SameValue(«"get:flags,set:lastIndex,get:exec,call:exec,get:result[0],get:exec,call:exec,get:result[0],get:result[index],get:result[length],get:result[groups],"», «"get:flags,set:lastIndex,get:exec,call:exec,get:result[0],get:exec,call:exec,get:result[length],get:result[0],get:result[index],get:result[groups],"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
