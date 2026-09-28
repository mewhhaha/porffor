# F048: Accept callable Proxy objects in Function.prototype.bind

- **Status:** fixed
- **Owner:** lila-aot-wasm builtins/function.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F048.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The frozen compiler admitted only ValueKind::Function in Function.prototype.bind. Callable Proxies use the Object representation, so valid receivers were rejected before binding. Bound allocation and invocation also assumed an ordinary function layout and dispatch path; supporting Proxies requires observable prototype/descriptor reads, retained Proxy identity, and the caller execution Realm through bound invocation.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/function.rs:299](../crates/lila-aot-wasm/src/builtins/function.rs#L299): Bind now validates the paired receiver with IsCallable before allocation.
- [crates/lila-aot-wasm/src/functions/bound_function_allocation.rs:22](../crates/lila-aot-wasm/src/functions/bound_function_allocation.rs#L22): Binding observes Proxy prototype and own-property traps while keeping the target identity.
- [crates/lila-aot-wasm/src/builtins/function.rs:429](../crates/lila-aot-wasm/src/builtins/function.rs#L429): Invocation keeps the bound record separate and forwards the caller Realm through Proxy call/construct dispatch.
- [test262/vendor/test262/test/staging/sm/Function/bound-length-and-name.js:24](../test262/vendor/test262/test/staging/sm/Function/bound-length-and-name.js#L24): Fixture binds a Proxy of a function.

## Work

Use IsCallable and IsConstructor, perform Proxy-aware prototype and own-length reads, retain the original bound target, and forward calls/constructs through Proxy dispatch with the caller Realm. Keep bound-record storage distinct from the execution context.

## Validation

Run both bound-length-and-name and bound-non-constructable modes; check proxy getOwnPropertyDescriptor/get ordering.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F048.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F048-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Function/bound-length-and-name.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1637360: Function.prototype.bind receiver is not callable)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.

## Resolution

Fixed in `c0914161f349eab7489b7236fff691340c320f29` and accepted after independent review. [Verification record](evidence/resolutions/sol-batch-20260928.json) retains compiler identity, native snapshots, exact execution identities, focused checks, and review evidence.

- Fresh release compiler: `41300185c891072679474d8066d7c51634699253a3cc1f574de9cc90c2ce0ab9`.
- Exact replay: **4/4 Success**, with native evidence under `target/test262-scratch/sol-batch-20260928/replay/`.
- Adjacent Test262: **208/208 Success**.
- Focused Wasm-AOT engine regressions: **6/6 passed** in `aot_bind_callable_proxy`.
- The regression control passes on the new compiler and fails on the frozen pre-batch compiler.

Independent review required a caller-Realm correction after the first implementation trapped in Proxy helpers. The corrected six behavioral tests cover traps, construction, and both directions across Realms. Parent checks also cover nested Proxies, chained binding, missing own length, and revocation.

GPT-6-Sol at max reasoning implemented the fix. The integrated checkpoint passed 43 engine tests, 29 structural checks, and two unit tests. Builds and runs were capped at 10 CPUs and half physical RAM with swap disabled. The full pinned matrix was not rerun; frozen evidence and published README counts remain unchanged.
