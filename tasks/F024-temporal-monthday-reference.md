# F024: Select calendar-correct MonthDay reference dates

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_month_day.rs and calendar kernel
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 34 executions across 17 physical files (Bug 34, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F024.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

MonthDay regulation treats month/day as Gregorian fields and overwrites the stored reference year with 1972. Its reference-date constant is justified only by the old three-calendar Gregorian arithmetic enum, while canonicalization now accepts lunisolar and other calendars. Non-ISO month lengths, leap-day choice, latest eligible reference date, equals(propertyBag), and locale formatting therefore observe the wrong ISO record.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:59](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L59): The constant proves only the old Gregorian-compatible calendars and evaluates to 1972.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:346](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L346): Resolver uses Gregorian regulation and resets reference year to the constant.
- [test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/from/chinese-month-codes.js:6](../test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/from/chinese-month-codes.js#L6): Fixture exercises Chinese month-code reference dates, whose ISO years need not be 1972.
- [test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/prototype/toLocaleString/dateStyle.js:22](../test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/prototype/toLocaleString/dateStyle.js#L22): Ramadan formatting relies on the MonthDay storing the correctly converted non-ISO reference date.

## Work

Implement MonthDayFromFields through the calendar kernel with calendar month identity, valid-day constrain/reject behavior, and the required reference-date search. Store the resulting ISO year/month/day; do not force 1972 for all calendars.

## Validation

Run all 34 assigned executions after leap-month parsing/required-field prerequisites, checking latest permitted reference dates, rare leap-month search, calendar month lengths, round-trip equals and Islamic locale formatting.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F024.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F024-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/chinese-calendar-dates.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1976240: md: monthCode result: Expected SameValue(«"M05"», «"M04L"») to be true)
```

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/chinese-month-codes.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1980744: M01-30: referenceISOYear result: Expected SameValue(«1972», «1970») to be true)
```

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/constrain-to-leap-day.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1829904: chinese: M01-31 should constrain to 30, not 29 Expected SameValue(«31», «30») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
