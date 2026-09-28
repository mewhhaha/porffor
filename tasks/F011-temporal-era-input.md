# F011: Resolve all accepted non-ISO eras in Temporal property bags

- **Status:** open
- **Owner:** lila-aot-wasm temporal_plain_date.rs era resolver and calendar field conversion
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 286 executions across 143 physical files (Bug 286, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F011.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

emit_temporal_calendar_has_eras_i32 recognizes the full lila-intl calendar set, but emit_temporal_resolve_era_to_iso_year matches era spellings only across the old three-calendar TemporalCalendarId::ALL. Valid Coptic, Ethiopic, Hebrew, Indian, Islamic, Japanese, Persian, and ROC eras therefore fall through to Invalid Temporal era for this calendar. This resolver also assumes Gregorian-style year conversion.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:1234](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L1234): The era-presence predicate uses lila_intl::TemporalCalendar::ALL.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs:1422](../crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs#L1422): The following matching loop uses only TemporalCalendarId::ALL and rejects valid non-ISO eras.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs:1130](../crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs#L1130): DateTime conversion still depends on that incomplete resolver.
- [crates/lila-aot-wasm/src/builtins/temporal_calendar.rs:279](../crates/lila-aot-wasm/src/builtins/temporal_calendar.rs#L279): The working tree already has a full-calendar fields request to use after JS observation.

## Work

Resolve the observed era/year pair with the same full-calendar kernel used for from-fields conversion, preserving missing-pair TypeErrors, alias canonicalization, era boundaries, year agreement, overflow, and observable read order. Replace the misleading ISO-year intermediate where non-ISO month/day interpretation is still required.

## Validation

Run all assigned era-input executions and the from/with era-ordering tests; check both forward/backward eras, Japanese transitions, zero and negative years, absent year with eraYear, and mismatched explicit year.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F011.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F011-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/PlainDateTime/from/canonicalize-era-codes-non-gregorian.js` — Bug

```text
[origin:unknown] uncaught throw: RangeError: wasm-aot completion: object(handle@1810208: Invalid Temporal era for this calendar)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
