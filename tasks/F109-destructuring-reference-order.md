# F109: Evaluate destructuring assignment references before source property reads

- **Status:** open
- **Owner:** lila-ir destructuring lowering and with environment references
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F109.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The observed binding::varTarget lookup is missing before reading the source property/default. Destructuring treats the assignment destination as a static name or resolves it too late, losing observable with-environment reference evaluation.

## Source evidence

- [crates/lila-ir/src/lowering.rs:14421](../crates/lila-ir/src/lowering.rs#L14421): Shared destructuring lowering staging point.
- [test262/vendor/test262/test/language/destructuring/binding/keyed-destructuring-property-reference-target-evaluation-order-with-bindings.js:73](../test262/vendor/test262/test/language/destructuring/binding/keyed-destructuring-property-reference-target-evaluation-order-with-bindings.js#L73): Fixture makes destination binding resolution observable.

## Work

Create and retain an evaluated destination Reference before GetValue of the keyed source/default, then perform PutValue on that same reference.

## Validation

Run the original keyed destructuring proxy/with trace and compare the full effect sequence.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F109.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F109-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/destructuring/binding/keyed-destructuring-property-reference-target-evaluation-order-with-bindings.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1666448: Actual [binding::source, binding::sourceKey, sourceKey, get source, binding::defaultValue] and expected [binding::source, binding::sourceKey, sourceKey, binding::varTarget, get source, binding::defaultValue] should have the same contents. )
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
