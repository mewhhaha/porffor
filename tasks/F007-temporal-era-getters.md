# F007: Project non-ISO Temporal era and eraYear from the calendar kernel

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_date.rs accessors and carrier-specific getters
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 492 executions across 246 physical files (Bug 492, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F007.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The shared era accessor initializes undefined and iterates TemporalCalendarId::ALL, which still contains only iso8601, gregory, and buddhist. Other accepted calendars never write an era or eraYear. PlainDateTime, PlainYearMonth, and ZonedDateTime still call that accessor, so helpers fail before their arithmetic assertions. PlainDate has a newer kernel path, which makes cross-carrier conversions reveal the inconsistency.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:98](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L98): The old accessor domain only has three calendars.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:1082](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L1082): The result defaults to undefined and only those calendars can populate it.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time.rs:576](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time.rs#L576): DateTime still uses that old shared accessor.
- [crates/lila-aot-wasm/src/builtins/temporal_calendar.rs:260](../crates/lila-aot-wasm/src/builtins/temporal_calendar.rs#L260): A full-calendar fields query is already available in the working tree.

## Work

Use one complete calendar authority for all carrier projections; query era/eraYear from the stored ISO date through the pinned calendar kernel. Remove the incomplete parallel calendar enum from these paths and preserve undefined only for genuinely era-free calendars.

## Validation

Run the 492 assigned executions in both modes after the projection change. Cover every accepted calendar, era boundaries, canonical era strings, cross-carrier conversions, and extreme dates; record any newly exposed downstream failure instead of assuming one fix clears the whole fixture.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F007.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F007-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDate/prototype/withPlainTime/basic-roc.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1962928: eraName must be string or undefined in canonicalizeCalendarEra Expected SameValue(«"undefined"», «"string"») to be true)
```

- `sloppy-script:intl402/Temporal/PlainYearMonth/prototype/with/cross-era-boundary.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1808336: resulting month should have crossed an era boundary Expected SameValue(«undefined», «undefined») to be false)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
