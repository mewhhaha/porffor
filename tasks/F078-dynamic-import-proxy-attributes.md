# F078: Resolve JSON imports with attributes obtained through Proxy enumeration

- **Status:** open
- **Owner:** lila-ir dynamic module graph; lila-engine module_loader
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F078.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The fixture uses a literal JSON specifier with a Proxy attributes object. Runtime enumeration reads type=json, but graph lookup reports Cannot find module. Static discovery likely did not load the attribute-qualified JSON module identity represented by the live attributes.

## Source evidence

- [crates/lila-engine/src/module_loader.rs:56](../crates/lila-engine/src/module_loader.rs#L56): JSON module identity has a distinct key prefix.
- [test262/vendor/test262/test/language/expressions/dynamic-import/import-attributes/2nd-param-with-enumeration-enumerable.js:39](../test262/vendor/test262/test/language/expressions/dynamic-import/import-attributes/2nd-param-with-enumeration-enumerable.js#L39): Runtime Proxy enumeration supplies the import type.

## Work

Preserve module type in graph discovery/lookup when attributes are observed dynamically; enumerate exactly once in required order and distinguish missing modules from unsupported type selection.

## Validation

Run both enumerable Proxy import-attribute modes with the original JSON fixture and verify one type getter plus namespace.default=262.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F078.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F078-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/dynamic-import/import-attributes/2nd-param-with-enumeration-enumerable.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:TypeError: Cannot find module ./2nd-param_FIXTURE.json
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
