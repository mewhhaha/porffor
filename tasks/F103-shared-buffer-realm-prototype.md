# F103: Select SharedArrayBuffer fallback prototype from newTarget realm

- **Status:** open
- **Owner:** lila-aot-wasm SharedArrayBuffer construction and realm intrinsics
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F103.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Reflect.construct(SharedArrayBuffer, [], foreignFunction) with foreignFunction.prototype=null yields the wrong prototype identity. The constructor fallback either uses the current realm or lacks the foreign SharedArrayBuffer prototype mapping.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:19307](../crates/lila-aot-wasm/src/builtins/standard.rs#L19307): Shared ArrayBuffer constructor allocation path.
- [test262/vendor/test262/test/built-ins/SharedArrayBuffer/proto-from-ctor-realm.js:25](../test262/vendor/test262/test/built-ins/SharedArrayBuffer/proto-from-ctor-realm.js#L25): Forces the other-realm intrinsic fallback.

## Work

Trace GetPrototypeFromConstructor through GetFunctionRealm and install the SharedArrayBuffer intrinsic of the resolved newTarget realm.

## Validation

Run both proto-from-ctor-realm variants and preserve empty-constructor finite-source support.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F103.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F103-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/SharedArrayBuffer/proto-from-ctor-realm.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2500688: Expected SameValue(«[object SharedArrayBuffer]», «[object Object]») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
