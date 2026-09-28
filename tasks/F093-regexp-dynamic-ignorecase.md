# F093: Compile runtime-built Unicode ignoreCase matcher variants

- **Status:** open
- **Owner:** lila-ir finite RegExp sources; lila-aot-wasm matcher dispatch
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F093.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

unicode-ignoreCase creates pattern strings from code-point parameters and uses iu; execution reaches unsupported pattern rather than testing the fold equivalence. The runtime (source,flags) table lacks some generated patterns or the fallback lacks their Unicode semantics.

## Source evidence

- [test262/vendor/test262/test/staging/sm/RegExp/unicode-ignoreCase.js:18](../test262/vendor/test262/test/staging/sm/RegExp/unicode-ignoreCase.js#L18): Pattern is built from runtime code-point arguments.
- [crates/lila-aot-wasm/src/builtins/string.rs:1412](../crates/lila-aot-wasm/src/builtins/string.rs#L1412): Runtime program lookup selects precompiled matcher variants.

## Work

Determine the first missing runtime pattern and give the supported RegExp runtime parser/matcher the required Unicode folding semantics, or compile bounded source families through general source analysis without fixture-specific expansion.

## Validation

Run both full Unicode ignoreCase fixtures and confirm each generated literal/class variant uses the same fold data.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F093.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F093-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/unicode-ignoreCase.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1639520: RegExp.prototype.exec unsupported pattern)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
