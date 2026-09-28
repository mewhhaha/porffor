# F061: Evaluate assignment RHS before null-super PutValue failure

- **Status:** open
- **Owner:** lila-aot-wasm expressions.rs and super_property_mutation.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F061.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Super reference construction calls emit_throw_if_null_super_base before the assignment value is evaluated. The fixtures observe count remaining zero when PutValue should throw only after incrementing the RHS count.

## Source evidence

- [crates/lila-aot-wasm/src/expressions/super_property_mutation.rs:47](../crates/lila-aot-wasm/src/expressions/super_property_mutation.rs#L47): Raw-reference construction eagerly rejects the base.
- [test262/vendor/test262/test/language/expressions/assignment/target-super-identifier-reference-null.js:27](../test262/vendor/test262/test/language/expressions/assignment/target-super-identifier-reference-null.js#L27): The observable RHS must execute before TypeError.

## Work

Represent the raw super reference until PutValue, evaluate the assignment RHS, then perform the base conversion/error; preserve earlier computed-key evaluation.

## Validation

Run both computed/identifier null-super assignment cases in strict and sloppy script modes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F061.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F061-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/assignment/target-super-computed-reference-null.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1958120: Expected SameValue(«0», «1») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
