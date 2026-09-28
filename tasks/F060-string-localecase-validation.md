# F060: Validate every locale passed to locale case conversion

- **Status:** open
- **Owner:** lila-aot-wasm builtins/standard.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F060.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

toLocaleLowerCase and toLocaleUpperCase share the ordinary case-conversion branch and never read their locales argument. Invalid later entries therefore escape CanonicalizeLocaleList validation entirely.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:30579](../crates/lila-aot-wasm/src/builtins/standard.rs#L30579): The locale variants enter the same branch as nonlocale lower/uppercase.
- [test262/vendor/test262/test/intl402/String/prototype/toLocaleLowerCase/validates-all-locale-identifiers.js:6](../test262/vendor/test262/test/intl402/String/prototype/toLocaleLowerCase/validates-all-locale-identifiers.js#L6): This fixture requires validation of all locale identifiers, not just the locale selected for casing.

## Work

After the required receiver/string conversion, run CanonicalizeLocaleList on the supplied locales and preserve each property access and abrupt completion before selecting the casing locale.

## Validation

Run both validation fixtures in both modes plus observable locale-list getter and invalid trailing-locale regressions; coordinate with locale-sensitive mappings without suppressing validation.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F060.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F060-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/String/prototype/toLocaleLowerCase/validates-all-locale-identifiers.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1635304: Expected a RangeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
