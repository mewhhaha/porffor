# F031: Enforce calendar-specific MonthDay required fields before range checks

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_month_day.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 14 executions across 7 physical files (Bug 14, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F031.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

MonthDay field resolution only requires day and either month or monthCode; when year is absent it immediately supplies ISO 1972. It never enforces the non-ISO requirement for a year when an ordinal month is supplied. Consequently underspecified bags are accepted and month/monthCode range conflicts win over the required missing-year TypeError.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:346](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L346): The required-fields checks only cover day and month-or-monthCode.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:436](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L436): Missing year is defaulted without the non-ISO ordinal-month requirement.
- [test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/from/calendarresolvefields-error-ordering-chinese.js:13](../test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/from/calendarresolvefields-error-ordering-chinese.js#L13): Fixture requires missing-year TypeError before month/monthCode RangeError.

## Work

Validate the calendar-specific required field set before semantic/range resolution for from and with; distinguish ordinal month from monthCode and preserve error ordering before choosing a reference date.

## Validation

Run all fourteen assigned executions plus MonthDay from/with required-field and ordering fixtures, covering ISO, Gregorian, Chinese, Hebrew, and Islamic calendars with absent year, conflicting monthCode and invalid day.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F031.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F031-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/calendarresolvefields-error-ordering-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1809320: Missing year throws TypeError before month/monthCode conflict throws RangeError Expected a TypeError but got a RangeError)
```

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/fields-missing-properties.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1807744: month, day with non-iso8601 calendar Expected a TypeError to be thrown but no exception was thrown at all)
```

- `sloppy-script:intl402/Temporal/PlainMonthDay/from/fields-object.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1989456: month and non-ISO string calendar Expected a TypeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
