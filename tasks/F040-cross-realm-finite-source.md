# F040: Specialize literal cross-realm eval and empty Function constructors

- **Status:** open
- **Owner:** lila-ir dynamic source planning; lila-aot-wasm created realm functions
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 8 executions across 4 physical files (Bug 0, NotImplemented 8, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F040.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Four Proxy fixtures are rejected by the generic dynamic-source feature gate although their source is a literal cross-realm eval or an empty other.Function constructor. These are finite source/realm planning gaps; arbitrary runtime evaluation is not needed.

## Source evidence

- [test262/vendor/test262/test/built-ins/Proxy/apply/arguments-realm.js:14](../test262/vendor/test262/test/built-ins/Proxy/apply/arguments-realm.js#L14): The realm eval receives a literal proxy/function source.
- [test262/vendor/test262/test/built-ins/Proxy/construct/trap-is-undefined-proto-from-newtarget-realm.js:50](../test262/vendor/test262/test/built-ins/Proxy/construct/trap-is-undefined-proto-from-newtarget-realm.js#L50): The other-realm Function constructor has an empty body.
- [crates/lila-aot-wasm/src/builtins/host/created_realm_dynamic_function_intrinsics.rs:1](../crates/lila-aot-wasm/src/builtins/host/created_realm_dynamic_function_intrinsics.rs#L1): Created realm dynamic Function intrinsic ownership.

## Work

Carry known realm identity and finite source into the existing AOT source planner, then preserve caller-realm argument-array allocation and newTarget fallback prototypes.

## Validation

Run the eight attached executions and relevant cross-realm Proxy construct/apply tests.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F040.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F040-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/Proxy/apply/arguments-realm.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: feature `dynamic-source` is not implemented. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
