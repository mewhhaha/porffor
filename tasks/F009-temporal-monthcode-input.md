# F009: Resolve M13 and leap-month codes by calendar instead of ISO month tables

- **Status:** open
- **Owner:** lila-aot-wasm Temporal date/time, year-month, month-day, and zoned property-bag resolvers
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 318 executions across 159 physical files (Bug 318, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F009.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The remaining carrier resolvers recognize monthCode with hardcoded M01..M12 loops. Valid M13 and MxxL codes never match and throw RangeError; ordinary calendar codes are also treated as fixed ordinal ISO months. These are semantic-resolution failures after syntactic monthCode parsing, not malformed fixture input.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs:327](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs#L327): The old resolver still used by DateTime scans M01..M12.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs:201](../crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs#L201): YearMonth has its own M01..M12-only resolver.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:346](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L346): MonthDay repeats the fixed ISO list.
- [crates/lila-aot-wasm/src/builtins/temporal.rs:1459](../crates/lila-aot-wasm/src/builtins/temporal.rs#L1459): ZonedDateTime semantic field resolution recognizes only the fixed M01..M12 set.

## Work

Replace each fixed-list semantic resolver with the shared calendar month-code domain and kernel from-fields operation. Validate ordinal month agreement and constrain/reject behavior in the actual year/calendar; handle leap-month search separately for MonthDay reference dates.

## Validation

Run all 318 assigned executions and each non-ISO monthCode/leap-month subtree in both modes. Include Chinese/Dangi rare leap months, Hebrew M05L, thirteen-month calendars, absent leap months under both overflow modes, and invalid syntactic month codes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F009.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F009-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/add/constrain-day-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: RangeError: wasm-aot completion: object(handle@1968768: Invalid Temporal.PlainDate monthCode)
```

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/chinese-30-day-leap-months.js` — Bug

```text
[origin:unknown] uncaught throw: RangeError: wasm-aot completion: object(handle@1964488: Invalid Temporal.PlainMonthDay monthCode)
```

- `sloppy-script:intl402/Temporal/PlainYearMonth/from/reference-day-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: RangeError: wasm-aot completion: object(handle@1971680: Invalid Temporal.PlainYearMonth monthCode)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
