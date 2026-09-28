# F057: Escape RegExp source for slash and line terminators

- **Status:** open
- **Owner:** lila-aot-wasm RegExp source getter
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F057.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

RegExp.prototype.source returns the stored original source payload directly. A slash therefore returns / or a/b instead of the escaped source required for reconstructing a literal. The same error appears through a cross-realm getter.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/string.rs:2053](../crates/lila-aot-wasm/src/builtins/string.rs#L2053): The getter loads HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET without escaping.

## Work

Apply EscapeRegExpPattern to the original source and flags with correct empty pattern, slash, line terminator and UTF-16 handling.

## Validation

Run source.js and cross-compartment-getter in both modes and the standard source value regressions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F057.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F057-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/cross-compartment-getter.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2530376: Expected SameValue(«"a/b"», «"a\\/b"») to be true)
```

- `sloppy-script:staging/sm/RegExp/source.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1640120: Expected SameValue(«"/"», «"\\/"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
