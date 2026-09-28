# F084: Forward Iterator.from wrapper next results without extra validation

- **Status:** open
- **Owner:** lila-aot-wasm Iterator.from wrapper builtin
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F084.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The Iterator.from wrapper calls next and then unconditionally rejects primitive results. The pinned fixture expects the wrapper next method to return the wrapped result unchanged; object-result validation belongs to consuming iterator operations.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:17988](../crates/lila-aot-wasm/src/builtins/standard.rs#L17988): This extra post-call validation is the exact thrown error.
- [test262/vendor/test262/test/staging/sm/Iterator/from/wrap-next-not-object-throws.js:25](../test262/vendor/test262/test/staging/sm/Iterator/from/wrap-next-not-object-throws.js#L25): The pinned fixture requires unchanged forwarding.

## Work

Remove the extra result-brand check from the wrapper forwarding operation while keeping receiver and callable checks and validation at consuming operations.

## Validation

Run both wrap-next-not-object-throws fixtures across undefined, null, number, boolean, string and Symbol results.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F084.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F084-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Iterator/from/wrap-next-not-object-throws.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1635480: Iterator.from wrapper next result must be object)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
