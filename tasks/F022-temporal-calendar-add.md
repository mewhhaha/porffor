# F022: Use calendar-aware addition and subtraction for every Temporal carrier

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_date_time_methods.rs, temporal_plain_year_month_methods.rs, temporal_zoned_difference.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 44 executions across 22 physical files (Bug 44, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F022.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

DateTime, YearMonth, and ZonedDateTime arithmetic still calls emit_temporal_add_iso_date. That helper adds years/months on the Gregorian twelve-month axis and constrains by ISO month length, without receiving the calendar. It cannot preserve non-ISO month identity across leap years/months or implement the requested non-ISO overflow behavior. Calendar-aware PlainDate support in the working tree has not been propagated to these carriers.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs:379](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs#L379): The helper signature contains no calendar argument and balances ISO year/month.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs:1858](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs#L1858): DateTime add/subtract still invokes the ISO helper.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs:1485](../crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs#L1485): YearMonth arithmetic also uses the helper.
- [crates/lila-aot-wasm/src/builtins/temporal_zoned_difference.rs:251](../crates/lila-aot-wasm/src/builtins/temporal_zoned_difference.rs#L251): Zoned date-duration arithmetic uses ISO date addition before converting back to an instant.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs:1735](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs#L1735): PlainDate already provides an integration example in the working tree.

## Work

Route calendar years/months/weeks/days through the full-calendar DateAdd operation for each carrier; preserve time-of-day balancing and zoned disambiguation around that boundary. Implement reject/constrain and YearMonth reference-date semantics with calendar month identity.

## Validation

Run the 44 assigned executions after field conversion/projection prerequisites, then the entire non-ISO add/subtract subtrees because earlier-blocked fixtures can expose additional arithmetic failures. Cover leap-month carry, leap-day constrain/reject and era transitions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F022.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F022-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/add/leap-month-chinese-numerical-months.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1808792: Adding a year and a month to a numerical (leap) month. Expected a RangeError to be thrown but no exception was thrown at all)
```

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/subtract/leap-month-chinese-numerical-months.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1808824: Subtracting a year and a month to a numerical (leap) month. Expected a RangeError to be thrown but no exception was thrown at all)
```

- `sloppy-script:intl402/Temporal/PlainYearMonth/prototype/add/basic-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2100840: Adding 6 months, with result in next year (leap year): monthCode result: Expected SameValue(«"M06"», «"M05"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
