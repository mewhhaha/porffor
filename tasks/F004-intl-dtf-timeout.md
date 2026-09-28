# F004: Profile the DateTimeFormat–Temporal calendar comparison timeout

- **Status:** open
- **Owner:** lila-engine Wasmtime execution; lila-aot-wasm DateTimeFormat/Temporal paths; lila-intl calendar provider
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 0, NotImplemented 0, Crash 2)

[Backlog](README.md) · [Exact execution list](cases/F004.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The two executions hit the 60,000 ms Wasmtime epoch deadline. The fixture performs 12 calendars × 101 years × 12 months, repeatedly constructing Temporal dates and querying formatToParts. The recorded timeout does not identify a hot loop or prove nontermination; compiler/runtime/provider profiling is still required.

## Source evidence

- [test262/vendor/test262/test/intl402/DateTimeFormat/prototype/formatToParts/compare-to-temporal.js:48](../test262/vendor/test262/test/intl402/DateTimeFormat/prototype/formatToParts/compare-to-temporal.js#L48): The finite nested calendar/year/month workload performs 14,544 comparisons.
- [test262/vendor/test262/test/intl402/DateTimeFormat/prototype/formatToParts/compare-to-temporal.js:13](../test262/vendor/test262/test/intl402/DateTimeFormat/prototype/formatToParts/compare-to-temporal.js#L13): Each iteration crosses Temporal conversion and DateTimeFormat parts paths.
- [crates/lila-engine/src/lib.rs:3874](../crates/lila-engine/src/lib.rs#L3874): Engine reports the deadline trap, which is only a symptom.

## Work

Reproduce with the saved binary/configuration and isolated calendar/year ranges, capture time and allocation/host-call profiles, then fix the measured excessive work or nontermination. Keep any justified timeout policy change explicit and retain the failure until the full fixture completes.

## Validation

Measure the original fixture in both modes under the 10-core/half-RAM resource limits, record runtime before/after, and require completion with the normal suite deadline. Recheck all assertions after performance is restored.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F004.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F004-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/DateTimeFormat/prototype/formatToParts/compare-to-temporal.js` — Crash

```text
[origin:unknown] timeout exceeded after 60103ms (wasm epoch interrupt, bound 60000ms)
```

- `strict-script:intl402/DateTimeFormat/prototype/formatToParts/compare-to-temporal.js` — Crash

```text
[origin:unknown] timeout exceeded after 60107ms (wasm epoch interrupt, bound 60000ms)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
