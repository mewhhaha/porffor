# F086: Keep frozen function caller/arguments values stable

- **Status:** open
- **Owner:** lila-aot-wasm functions/legacy_activation.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 2 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F086.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Activation entry and arguments publication overwrite caller/arguments property payloads directly. The write helper bypasses descriptor flags, so a nonwritable, nonconfigurable data value changes from null to the current caller/arguments object, violating own-property invariants.

## Source evidence

- [crates/lila-aot-wasm/src/functions/legacy_activation.rs:184](../crates/lila-aot-wasm/src/functions/legacy_activation.rs#L184): Direct payload/tag stores ignore the property descriptor flags.

## Work

Stop activation bookkeeping from changing frozen data-property values; keep any internal observation state separate from immutable own descriptors or use a spec-permitted representation.

## Validation

Run both attached sloppy descriptor-invariant cases and check recursive activation does not mutate frozen values.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F086.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F086-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/Object/internals/DefineOwnProperty/consistent-value-function-arguments.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1639072: Expected SameValue(«null», «[object Arguments]») to be true)
```

- `sloppy-script:built-ins/Object/internals/DefineOwnProperty/consistent-value-function-caller.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1640448: Expected SameValue(«null», «function g() {
  return f();
}») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
