# F070: Use throwing Set for Array.prototype.fill writes

- **Status:** open
- **Owner:** lila-aot-wasm Array.prototype.fill
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F070.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

fill accepts a nonwritable/accessor/frozen destination where the fixture requires TypeError. Its loop calls the generic emit_object_write rather than an explicitly throwing Set result contract, allowing failed writes to be ignored on at least one receiver form.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/array.rs:12480](../crates/lila-aot-wasm/src/builtins/array.rs#L12480): Fill loop delegates to generic object-write emission.
- [test262/vendor/test262/test/staging/sm/Array/fill.js:71](../test262/vendor/test262/test/staging/sm/Array/fill.js#L71): The failing family checks required failed-Set TypeErrors.

## Work

Emit Set(O, key, value, true) for every fill destination, including boxed string and frozen array/object paths.

## Validation

Run both fill modes with getters, read-only properties, frozen objects/arrays, strings and Proxy setters.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F070.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F070-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Array/fill.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2340096: Expected a TypeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
