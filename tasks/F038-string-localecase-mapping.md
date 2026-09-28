# F038: Implement Turkish, Azeri, and Lithuanian locale casing

- **Status:** open
- **Owner:** lila-aot-wasm string case emitters; Unicode casing data
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 12 executions across 6 physical files (Bug 12, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F038.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The locale-specific casing methods call exactly the same emit_lowercase_string_payload_from_local or uppercase emitter as their nonlocale counterparts, with no locale parameter. Turkish/Azeri dotted I and Lithuanian context-sensitive dot rules therefore use default Unicode casing.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:30581](../crates/lila-aot-wasm/src/builtins/standard.rs#L30581): Locale and nonlocale methods share one implementation.
- [crates/lila-aot-wasm/src/builtins/standard.rs:30616](../crates/lila-aot-wasm/src/builtins/standard.rs#L30616): No locale reaches the mapping emitter.
- [test262/vendor/test262/test/intl402/String/prototype/toLocaleLowerCase/special_casing_Azeri.js:5](../test262/vendor/test262/test/intl402/String/prototype/toLocaleLowerCase/special_casing_Azeri.js#L5): Azeri fixture checks the locale-specific dotted-I mapping.

## Work

Pass a validated selected casing locale into the Unicode mapping operation and implement SpecialCasing context rules for tr, az, and lt, preserving full-string context and expansions.

## Validation

Run the twelve assigned executions and all locale casing fixtures; verify dotted/dotless I, combining marks, soft-dotted context, default locale behavior, and ordinary toLowerCase/toUpperCase regressions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F038.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F038-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/String/prototype/toLocaleLowerCase/special_casing_Azeri.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1634488: LATIN CAPITAL LETTER I WITH DOT ABOVE Expected SameValue(«"i̇"», «"i"») to be true)
```

- `sloppy-script:intl402/String/prototype/toLocaleLowerCase/special_casing_Lithuanian.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636744: LATIN CAPITAL LETTER I followed by COMBINING GRAVE ACCENT Expected SameValue(«"ì"», «"i̇̀"») to be true)
```

- `sloppy-script:intl402/String/prototype/toLocaleUpperCase/special_casing_Azeri.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1634072: LATIN SMALL LETTER I Expected SameValue(«"I"», «"İ"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
