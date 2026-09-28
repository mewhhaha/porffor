# F062: Preserve Symbol results returned by ToPrimitive during ToPropertyKey

- **Status:** open
- **Owner:** lila-ir property key lowering; lila-aot-wasm operations.rs
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F062.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Boxed symbols and objects whose toString returns a Symbol are sent through string conversion, throwing Cannot convert a Symbol value to a string. The common ToPropertyKey emitter has a Symbol-result branch; some property operation bypasses it or loses its tag.

## Source evidence

- [crates/lila-aot-wasm/src/operations.rs:5118](../crates/lila-aot-wasm/src/operations.rs#L5118): Common helper preserves Symbol primitive results; bypassing callers need audit.
- [test262/vendor/test262/test/staging/sm/expressions/ToPropertyKey-symbols.js:15](../test262/vendor/test262/test/staging/sm/expressions/ToPropertyKey-symbols.js#L15): Valid property-key coercion returns a Symbol rather than a string.

## Work

Trace the first failing key context in each fixture, route every property-key coercion through ToPrimitive(string) followed by Symbol preservation, and retain marker/tag identity.

## Validation

Run property-basics and ToPropertyKey-symbols in both modes over computed getters, writes, super, delete, in and Object methods.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F062.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F062-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Symbol/property-basics.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@2255248: Cannot convert a Symbol value to a string)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
