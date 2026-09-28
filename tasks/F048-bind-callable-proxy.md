# F048: Accept callable Proxy objects in Function.prototype.bind

- **Status:** open
- **Owner:** lila-aot-wasm builtins/function.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F048.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Function.prototype.bind admits only ValueKind::Function. Callable Proxy objects carry an object representation, so valid receivers are rejected as not callable before name/length/bound construction semantics run.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/function.rs:299](../crates/lila-aot-wasm/src/builtins/function.rs#L299): Bind only compares the receiver tag to ValueKind::Function.
- [test262/vendor/test262/test/staging/sm/Function/bound-length-and-name.js:24](../test262/vendor/test262/test/staging/sm/Function/bound-length-and-name.js#L24): Fixture binds a Proxy of a function.

## Work

Use the shared IsCallable predicate and retain the original callable receiver when creating a bound function, including proxy traps and non-constructable targets.

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
