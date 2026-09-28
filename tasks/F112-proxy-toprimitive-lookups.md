# F112: Avoid unintended Proxy trap accesses during primitive conversion

- **Status:** open
- **Owner:** lila-aot-wasm ToPrimitive and Proxy Get
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F112.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The toPrimitive fixture expects TypeError but catches a primitive throw from its trap guard. A Proxy handler throws whenever the implementation asks for an unexpected trap, so an extra internal property lookup or wrong conversion path is likely. The saved detail does not identify which guarded operation fired.

## Source evidence

- [test262/vendor/test262/test/staging/sm/object/toPrimitive.js:104](../test262/vendor/test262/test/staging/sm/object/toPrimitive.js#L104): The fixture throws a primitive on any unexpected handler trap lookup.
- [crates/lila-aot-wasm/src/operations.rs:2957](../crates/lila-aot-wasm/src/operations.rs#L2957): Shared conversion dispatch owner.

## Work

Reduce the first guarded conversion and record handler trap names; implement only the specified GetMethod/Call/ordinary conversion accesses and preserve receiver identity.

## Validation

Run the original sloppy toPrimitive fixture including nested proxies, symbol wrappers and deleted conversion methods.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F112.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F112-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/object/toPrimitive.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1877392: Thrown value was not an object!)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
