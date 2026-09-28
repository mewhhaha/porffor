# F010: Replace ISO-only Temporal calendar field projections

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_date.rs numeric helpers and PlainDateTime/PlainYearMonth/PlainMonthDay/ZonedDateTime getters
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 308 executions across 154 physical files (Bug 308, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F010.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

These carrier getters still derive month/day/year lengths from stored ISO fields: monthsInYear is constant 12, daysInYear and inLeapYear use Gregorian leap rules, and monthCode is formatted from an ordinal ISO month. Calendar year projection only handles the Buddhist offset. The accepted non-ISO calendar string therefore does not determine the observed date fields.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:2243](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L2243): The shared numeric helper directly exposes ISO month/day, constant 12 months, and Gregorian lengths.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:2344](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L2344): Year projection only iterates the old three-calendar enum.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time.rs:559](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time.rs#L559): MonthCode is generated from 1..12 rather than calendar month identity.
- [crates/lila-aot-wasm/src/builtins/temporal_calendar.rs:260](../crates/lila-aot-wasm/src/builtins/temporal_calendar.rs#L260): Full calendar field projection is available but not wired into these getters.

## Work

Wire every date-bearing carrier to one full-calendar projection from its stored ISO date, including calendar year, ordinal month, monthCode/leap bit, day, year/month lengths and inLeapYear. Retain the separate ISO-only week-numbering rules.

## Validation

Run all 308 assigned executions, including string round trips and getter fixtures, then verify all accepted calendar projections agree across PlainDate, PlainDateTime, YearMonth, MonthDay and ZonedDateTime for the same ISO day.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F010.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F010-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDateTime/from/roundtrip-from-string.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2002872: undefined - created from string 2000-01-01T12:34[u-ca=chinese]: year result: Expected SameValue(«2000», «1999») to be true)
```

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/daysInMonth/basic-chinese.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1820200: 1971-01-01T12:34:00[u-ca=chinese] Expected SameValue(«31», «29») to be true)
```

- `sloppy-script:intl402/Temporal/PlainDateTime/prototype/daysInMonth/basic-coptic.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1810936: 1687-01-01T12:34:00[u-ca=coptic] Expected SameValue(«31», «30») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
