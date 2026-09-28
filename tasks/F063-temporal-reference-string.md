# F063: Keep non-ISO reference date fields when calendarName is never

- **Status:** fixed
- **Owner:** lila-aot-wasm temporal_plain_month_day.rs and temporal_plain_year_month_methods.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F063.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The frozen MonthDay and YearMonth serializers gated their reference ISO date fields on the calendar-annotation predicate. calendarName:never therefore removed the reference year or day even for non-ISO calendars, although those fields must remain independently of annotation visibility.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs:2251](../crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs#L2251): The shared inclusion predicate is independent of the annotation predicate.
- [crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs:1324](../crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs#L1324): MonthDay uses the separate predicate when including its reference year.
- [test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/prototype/toString/calendarname-never.js:6](../test262/vendor/test262/test/intl402/Temporal/PlainMonthDay/prototype/toString/calendarname-never.js#L6): Expected non-ISO Gregorian result retains the year with calendarName never.

## Work

Separate reference-date inclusion from annotation visibility. Include the reference field for every non-ISO calendar and for always/critical; keep never suppressing only the annotation.

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

## Resolution

Fixed in `c0914161f349eab7489b7236fff691340c320f29` and accepted after independent review. [Verification record](evidence/resolutions/sol-batch-20260928.json) retains compiler identity, native snapshots, exact execution identities, focused checks, and review evidence.

- Fresh release compiler: `41300185c891072679474d8066d7c51634699253a3cc1f574de9cc90c2ce0ab9`.
- Exact replay: **4/4 Success**, with native evidence under `target/test262-scratch/sol-batch-20260928/replay/`.
- Adjacent Test262: **126/126 Success**.
- Focused Wasm-AOT engine regressions: **3/3 passed** in `aot_temporal_reference_string`.
- The regression control passes on the new compiler and fails on the frozen pre-batch compiler.

Accepted without implementation corrections. Three Wasm regressions cover ISO/Gregorian values, all calendar display modes, expanded years, toJSON, option coercion, and error ordering.

GPT-6-Sol at max reasoning implemented the fix. The integrated checkpoint passed 43 engine tests, 29 structural checks, and two unit tests. Builds and runs were capped at 10 CPUs and half physical RAM with swap disabled. The full pinned matrix was not rerun; frozen evidence and published README counts remain unchanged.
