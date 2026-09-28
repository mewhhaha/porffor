# F018: Convert and merge non-ISO calendar fields before storing ISO dates

- **Status:** open
- **Owner:** lila-aot-wasm Temporal PlainDateTime/PlainYearMonth/ZonedDateTime from/with paths
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 102 executions across 51 physical files (Bug 102, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F018.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Remaining property-bag and with paths run non-ISO calendar year/month/day fields through the old ISO resolver, or merge incoming calendar fields with the receiver ISO fields. That yields wrong dates or Gregorian range errors before the intended operation. The fixtures labelled canonicalize-calendar also contain Islamic property bags: their assertion label alone is not evidence of an alias bug. PlainYearMonth locale-formatting receives the wrong stored reference date for the same reason.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs:1130](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs#L1130): DateTime follows era resolution with emit_temporal_plain_date_resolve_fields rather than kernel calendar conversion.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs:201](../crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs#L201): YearMonth synthesizes ISO day 1 and regulates ISO month/year rather than calendar fields.
- [crates/lila-aot-wasm/src/builtins/temporal.rs:1422](../crates/lila-aot-wasm/src/builtins/temporal.rs#L1422): Zoned property bags still enter the old ISO resolution path before instant conversion.
- [test262/vendor/test262/test/intl402/Temporal/PlainDateTime/prototype/equals/canonicalize-calendar.js:14](../test262/vendor/test262/test/intl402/Temporal/PlainDateTime/prototype/equals/canonicalize-calendar.js#L14): The same-date assertion includes an Islamic property bag as well as an annotated ISO string.
- [crates/lila-aot-wasm/src/builtins/temporal_calendar.rs:279](../crates/lila-aot-wasm/src/builtins/temporal_calendar.rs#L279): Existing full-calendar from-fields support supplies the correct conversion boundary.

## Work

Project receiver ISO records into calendar fields before CalendarMergeFields, resolve complete fields through the calendar kernel, then store the resulting ISO date and perform time-zone conversion. Keep constructors/annotated strings interpreted as ISO dates, and select YearMonth reference dates in the chosen calendar.

## Validation

Run all 102 assigned executions in both modes after era/month-code prerequisites. Verify property-bag versus annotated-string equivalence, with field preservation, YearMonth reference days, overflow boundaries and locale formatting of the resulting dates.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F018.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F018-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/add/basic-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: RangeError: wasm-aot completion: object(handle@1969160: Temporal.PlainDate is not a valid ISO date)
```

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/equals/canonicalize-calendar.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1808592: calendar ID is canonicalized)
```

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/with/non-iso-calendar-fields.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1807512: month is changed because year has different number of months Expected SameValue(«8», «11») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
