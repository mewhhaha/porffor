# F016: Implement ShadowRealm intrinsic and callable boundary

- **Status:** open
- **Owner:** lila-ir builtin registry; lila-aot-wasm realm intrinsics
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 124 executions across 64 physical files (Bug 124, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F016.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

ShadowRealm has no entry in the current builtin registry/intrinsic bootstrap. Tests fail at the missing global or its descriptor before reaching realm behavior. Realm construction, descriptors, wrapped calls, and importValue remain required; genuinely dynamic evaluate source must be reported as the explicit AOT boundary only after the intrinsic exists.

## Source evidence

- [crates/lila-ir/src/builtins.rs:927](../crates/lila-ir/src/builtins.rs#L927): Exhaustive builtin registry has no ShadowRealm variant.
- [test262/vendor/test262/test/built-ins/ShadowRealm/descriptor.js:11](../test262/vendor/test262/test/built-ins/ShadowRealm/descriptor.js#L11): Fixture checks the missing own global property.

## Work

Add the required ShadowRealm object/intrinsics and realm/call wrappers. Compile finite evaluate/import targets where supported, retaining typed unsupported dynamic-source outcomes for arbitrary evaluated text.

## Validation

Run every attached ShadowRealm execution; validate construction/descriptor tests independently of evaluate/import behavior.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F016.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F016-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:built-ins/ShadowRealm/prototype/importValue/import-value.js` — Bug

```text
[origin:unknown] uncaught throw: ReferenceError: wasm-aot completion: object(handle@1647344: unbound identifier)
```

- `sloppy-script:built-ins/ShadowRealm/constructor.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1635056: This test must fail if ShadowRealm is not a function Expected SameValue(«"undefined"», «"function"») to be true)
```

- `sloppy-script:built-ins/ShadowRealm/descriptor.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2005344: ShadowRealm should be an own property)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
