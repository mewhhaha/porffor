# F030: Preserve UTF-16 code units and surrogate pairs in RegExp matching

- **Status:** open
- **Owner:** lila-ir regexp literals; lila-aot-wasm UTF-16 matcher
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 14 executions across 7 physical files (Bug 14, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F030.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Raw and escaped surrogate/braced-code-point fixtures either miss valid matches or reach unsupported pattern, including the standard u-null-character escape. These source forms expose incomplete Unicode/UTF-16 compilation or fallback matching. Each exact pattern must be reduced; no single low-level defect has been proven.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/regexp.rs:3286](../crates/lila-aot-wasm/src/builtins/regexp.rs#L3286): Matcher boundary for UTF-16 code-unit decoding.
- [crates/lila-ir/src/regexp.rs:38](../crates/lila-ir/src/regexp.rs#L38): Program representation distinguishes code points from code units.

## Work

Trace escaped/raw code units through parsing, string encoding, class ranges and matcher input stepping, preserving lone surrogates and Unicode-mode code-point pairing.

## Validation

Run every attached raw/braced/lead-trail fixture in both modes and compare matched strings/code-unit lengths.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F030.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F030-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/literals/regexp/u-null-character-escape.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1632288: RegExp.prototype.exec unsupported pattern)
```

- `sloppy-script:staging/sm/RegExp/unicode-class-raw.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1992608: <non-scalar UTF-16>)
```

- `sloppy-script:staging/sm/RegExp/unicode-raw.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1979512: Actual argument [null] shouldn't be primitive. )
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
