# F104: Implement Temporal.Duration locale formatting through DurationFormat

- **Status:** open
- **Owner:** lila-aot-wasm temporal_duration_methods.rs; Intl.DurationFormat
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F104.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The recorded fixture first fails because Intl.DurationFormat is missing. Independently, the Duration toLocaleString builtin is dispatched to the ISO duration-string emitter, which only reads options for toString and ignores locale/options for toLocaleString. Implementing the constructor alone will expose this second confirmed gap.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:22201](../crates/lila-aot-wasm/src/builtins/standard.rs#L22201): The locale method is dispatched to emit_temporal_duration_to_string.
- [crates/lila-aot-wasm/src/builtins/temporal_duration_methods.rs:1970](../crates/lila-aot-wasm/src/builtins/temporal_duration_methods.rs#L1970): reads_options only recognizes TemporalDurationPrototypeToString.
- [test262/vendor/test262/test/intl402/Temporal/Duration/prototype/toLocaleString/returns-same-results-as-DurationFormat.js:42](../test262/vendor/test262/test/intl402/Temporal/Duration/prototype/toLocaleString/returns-same-results-as-DurationFormat.js#L42): The missing constructor is the initial blocker; equality with its localized formatting follows.

## Work

After the Intl.DurationFormat family is installed, give Duration.toLocaleString its own receiver-validation and locale/options path using the DurationFormat kernel and shared duration record semantics.

## Validation

Run both recorded executions after the DurationFormat task, comparing each listed locale/style against Intl.DurationFormat; also verify default options and throwing locale/options conversions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F104.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F104-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/Duration/prototype/toLocaleString/returns-same-results-as-DurationFormat.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1811792: target is not a constructor)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
