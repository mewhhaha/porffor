# F045: Infer function names from computed and numeric property keys

- **Status:** open
- **Owner:** lila-ir NamedEvaluation and class field lowering
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 6 executions across 3 physical files (Bug 6, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F045.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Anonymous arrow/function field values created for computed or numeric keys keep the empty name rather than the evaluated property key. Ordinary named function cases pass before the missing arrow/computed name assertion.

## Source evidence

- [crates/lila-ir/src/lowering.rs:8005](../crates/lila-ir/src/lowering.rs#L8005): Existing inferred-name lowering machinery.
- [test262/vendor/test262/test/staging/sm/Function/function-name-computed-01.js:14](../test262/vendor/test262/test/staging/sm/Function/function-name-computed-01.js#L14): Computed property arrow has no explicit name but needs the property name.

## Work

Apply NamedEvaluation after evaluating the property key for object literals and fields, preserving explicit names and symbol-name formatting.

## Validation

Run function-name-computed-01/02 and numeric-fields in both script modes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F045.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F045-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Function/function-name-computed-01.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1665528: Expected SameValue(«""», «"arrowFunc"») to be true)
```

- `sloppy-script:staging/sm/Function/function-name-computed-02.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1665304: Expected SameValue(«""», «"5"») to be true)
```

- `sloppy-script:staging/sm/fields/numeric-fields.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1639016: Expected SameValue(«""», «"128"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
