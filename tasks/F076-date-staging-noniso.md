# F076: Implement the pinned staging non-ISO Date parsing expectations

- **Status:** open
- **Owner:** lila-aot-wasm Date string parser
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F076.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The parser attempts strict ISO syntax and then weekday-led display syntax. The staging fixture expects additional legacy forms with spaces, one-digit fields and short offsets, which currently produce NaN. These are pinned staging compatibility expectations and should not be described as proof that every accepted legacy format is required by ECMA-262.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/date/date_string_parse.rs:146](../crates/lila-aot-wasm/src/builtins/date/date_string_parse.rs#L146): Only ISO then weekday display syntax is attempted.
- [test262/vendor/test262/test/staging/sm/Date/non-iso.js:14](../test262/vendor/test262/test/staging/sm/Date/non-iso.js#L14): Fixture explains the legacy cross-implementation extension forms.

## Work

Document the intended legacy Date parsing policy and add the supported general grammar needed by the pinned fixture while keeping strict ISO validation separate; do not special-case test strings.

## Validation

Run both non-iso fixtures and ensure malformed strict ISO forms still produce NaN.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F076.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F076-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Date/non-iso.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1843824: Expected SameValue(«NaN», «857819960000») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
