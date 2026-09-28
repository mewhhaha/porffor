# F113: Remove with object environment after leaving its statement

- **Status:** open
- **Owner:** lila-ir with environment lowering; lila-aot-wasm environment lifecycle
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F113.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A property written inside with remains resolvable afterward, or the outside lookup fails to throw ReferenceError. The with object environment or its inferred binding effect leaks beyond the statement scope.

## Source evidence

- [crates/lila-ir/src/lowering.rs:7367](../crates/lila-ir/src/lowering.rs#L7367): With name resolution planning owner.
- [test262/vendor/test262/test/language/statements/with/12.10-0-7.js:12](../test262/vendor/test262/test/language/statements/with/12.10-0-7.js#L12): Fixture distinguishes inside mutation from outside lookup.

## Work

Trace scope push/pop and static binding information across with; restore the previous lexical environment on normal and abrupt exits.

## Validation

Run the original 12.10-0-7 fixture and verify the object property changes while the outside identifier stays unresolvable.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F113.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F113-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/with/12.10-0-7.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1632384: Expected true but got false)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
