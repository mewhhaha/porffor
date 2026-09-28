# F063: Keep non-ISO reference date fields when calendarName is never

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_month_day.rs and temporal_plain_year_month_methods.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F063.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

MonthDay prints its reference year and YearMonth prints its reference day only when emit_temporal_show_calendar_annotation_i32 says to show the calendar annotation. calendarName:"never" therefore drops those date fields even for non-ISO calendars; the specification requires the full reference ISO date independently of whether the annotation is suppressed.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:1322](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L1322): Reference-year output is incorrectly gated by the annotation condition.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs:2097](../crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs#L2097): Reference-day output uses the same incorrect gate.
- [test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/prototype/toString/calendarname-never.js:6](../test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/prototype/toString/calendarname-never.js#L6): Expected non-ISO Gregorian result retains the year with calendarName never.

## Work

Separate the full-reference-date decision from calendar-annotation visibility for both serialization paths; use the calendar identity and ShowCalendarName rules independently.

## Validation

Run all four assigned executions and all MonthDay/YearMonth stringification options, checking ISO versus Gregorian/non-ISO values under auto, always, critical, and never.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F063.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F063-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainMonthDay/prototype/toString/calendarname-never.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1821536: built-in Gregorian calendar for calendarName = never Expected SameValue(«"05-02"», «"1972-05-02"») to be true)
```

- `sloppy-script:intl402/Temporal/PlainYearMonth/prototype/toString/calendarname-never.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1821584: built-in Gregorian calendar for calendarName = never Expected SameValue(«"2000-05"», «"2000-05-01"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
