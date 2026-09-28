# F053: Observe inherited and mutated replacer array elements

- **Status:** open
- **Owner:** lila-aot-wasm JSON.stringify replacer normalization; array index Get
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F053.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Replacer normalization omits keys visible through inherited array elements after holes/deletions or getter-driven mutation. The implementation freezes length once and then uses a specialized array-index read; that path must honor getters/prototypes throughout the captured bounds.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/json.rs:1710](../crates/lila-aot-wasm/src/builtins/json.rs#L1710): Specialized normalization reads array indices via emit_array_index_get.
- [test262/vendor/test262/test/staging/sm/JSON/stringify-replacer-array-hijinks.js:14](../test262/vendor/test262/test/staging/sm/JSON/stringify-replacer-array-hijinks.js#L14): The missing entry is intentionally inherited after deletion.

## Work

Audit array-index Get in replacer normalization against ordinary Get, retaining initial length while observing current own/inherited values and ToString side effects.

## Validation

Run both replacer-array-hijinks and trailing-holes modes and compare complete serialized strings.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F053.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F053-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/JSON/stringify-replacer-array-hijinks.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1640280: Expected SameValue(«"{\"0\":{\"1\":{\"3\":{\"3\":3}},\"3\":3},\"3\":3}"», «"{\"0\":{\"1\":{}}}"») to be true)
```

- `sloppy-script:staging/sm/JSON/stringify-replacer-array-trailing-holes.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1641536: Expected SameValue(«"{\"0\":\"hi\"}"», «"{\"0\":\"hi\",\"1\":\"n-nao\",\"2\":\"run away!\",\"3\":\"bye\"}"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
