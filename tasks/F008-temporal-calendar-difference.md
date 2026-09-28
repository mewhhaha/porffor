# F008: Implement non-ISO CalendarDateUntil and relative rounding

- **Status:** open
- **Owner:** lila-aot-wasm temporal_difference.rs, temporal_zoned_difference.rs, Temporal PlainDate/YearMonth difference paths
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 324 executions across 162 physical files (Bug 324, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F008.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

PlainDate and shared DateTime/ZonedDateTime differences still use emit_temporal_difference_iso_date, which has no calendar input. YearMonth differences directly compute (otherYear-year)*12 + otherMonth-month and divide into twelve-month years. Relative rounding helpers also add ISO dates. These paths cannot count calendar months, leap months, or non-ISO years, even where PlainDate construction already uses the calendar kernel.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs:481](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs#L481): The shared difference implementation is explicitly ISO-only and does not accept a calendar.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs:2248](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs#L2248): PlainDate until/since still calls that helper.
- [crates/lila-aot-wasm/src/builtins/temporal_difference.rs:237](../crates/lila-aot-wasm/src/builtins/temporal_difference.rs#L237): DateTime relative difference also calls the same ISO helper.
- [crates/lila-aot-wasm/src/builtins/temporal_zoned_difference.rs:503](../crates/lila-aot-wasm/src/builtins/temporal_zoned_difference.rs#L503): Zoned differences likewise use ISO calendar components.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs:1729](../crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs#L1729): YearMonth computes twelve-month-year arithmetic directly.

## Work

Introduce the calendar-aware DateUntil operation alongside calendar DateAdd, use it for all carriers and relative rounding, and retain ISO epoch-day/time-zone handling only where appropriate. Model largest/smallest unit and rounding separately from the selected calendar month structure.

## Validation

Run all 324 assigned executions in both modes plus the full non-ISO until/since trees after construction prerequisites. Check signed direction, largestUnit years/months/weeks/days, end-of-month asymmetry, leap months, era transitions and relative rounding.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F008.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F008-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDate/prototype/since/basic-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2349456: 1 month in same year (30-day month to 29-day month) (largestUnit = years): months result: Expected SameValue(«0», «-1») to be true)
```

- `sloppy-script:intl402/Temporal/PlainDate/prototype/since/basic-hebrew.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2389160: 1 month in different year: months result: Expected SameValue(«0», «-1») to be true)
```

- `sloppy-script:intl402/Temporal/PlainDate/prototype/since/basic-indian.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2348824: 1 month in same year: months result: Expected SameValue(«0», «-1») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
