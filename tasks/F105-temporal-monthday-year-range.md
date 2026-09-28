# F105: Check MonthDay supplied-year limits for every accepted calendar

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_month_day.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F105.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The MonthDay pre-month-information year-range guard iterates only TemporalCalendarId::ALL (ISO/Gregorian/Buddhist). Newly accepted calendars such as Indian bypass the guard, so year -999999 does not throw before month calculations as required.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:390](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L390): The loop immediately below uses the incomplete three-calendar enum.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:98](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L98): Indian and other accepted non-ISO calendars are outside the guard domain.
- [test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/from/dont-calculate-month-info-for-out-of-range-year.js:6](../test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/from/dont-calculate-month-info-for-out-of-range-year.js#L6): Fixture requires out-of-range rejection before calendar month information is calculated.

## Work

Express the supplied-year range policy across the complete accepted calendar domain and perform the guard before any expensive calendar/month query, while retaining the specified ISO exception.

## Validation

Run both recorded executions and boundary cases for each accepted calendar, verifying rejection ordering with poisoned later fields and no calendar-query work for already invalid years.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F105.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F105-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/dont-calculate-month-info-for-out-of-range-year.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1976064: indian bails out when year is -999999 Expected a RangeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
