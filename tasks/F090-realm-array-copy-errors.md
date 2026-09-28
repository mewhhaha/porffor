# F090: Construct Array copy-method errors in the builtin defining realm

- **Status:** open
- **Owner:** lila-aot-wasm Array copy builtins and error construction
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F090.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A foreign toSorted called with an invalid comparator throws a TypeError from the wrong realm. The builtin validation uses the current/entry error prototype rather than the active builtin defining realm; adjacent copy-method errors need the same audit.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/array.rs:11777](../crates/lila-aot-wasm/src/builtins/array.rs#L11777): Comparator validation entry point.
- [test262/vendor/test262/test/staging/sm/Array/change-array-by-copy-errors-from-correct-realm.js:36](../test262/vendor/test262/test/staging/sm/Array/change-array-by-copy-errors-from-correct-realm.js#L36): First recorded constructor-identity mismatch.

## Work

Use the function-realm error constructor for all Array change-by-copy validation paths and preserve user-thrown errors unchanged.

## Validation

Run both change-array-by-copy-errors-from-correct-realm variants across TypeError and RangeError cases.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F090.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F090-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Array/change-array-by-copy-errors-from-correct-realm.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2525000: toSorted - bad comparator Expected a TypeError but got a different error constructor with the same name)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
