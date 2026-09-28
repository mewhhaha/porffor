# F006: Investigate large Unicode private-field class timeout

- **Status:** open
- **Owner:** lila-front; lila-ir private-name planning; lila-test262 timeout accounting
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 0, NotImplemented 0, Crash 1)

[Backlog](README.md) · [Exact execution list](cases/F006.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The strict-mode Unicode 10 private-class fixture exceeded 360,022 ms; the saved detail has no epoch-interrupt suffix or stack. It may be compile/planning complexity or execution, and no narrower cause is established.

## Source evidence

- [test262/vendor/test262/test/language/identifiers/start-unicode-10.0.0-class.js:15](../test262/vendor/test262/test/language/identifiers/start-unicode-10.0.0-class.js#L15): Fixture contains the large generated private-name declaration set.
- [crates/lila-test262/src/lib.rs:10857](../crates/lila-test262/src/lib.rs#L10857): Timeout accounting distinguishes Wasm runtime from total duration.

## Work

Profile the exact strict case by parsing, lowering, Wasmtime compile, and execution phases under the existing core/RAM cap; fix the identified complexity instead of hiding the timeout.

## Validation

Rerun the same execution and compare strict/sloppy stage timings without changing conformance accounting.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F006.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F006-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `strict-script:language/identifiers/start-unicode-10.0.0-class.js` — Crash

```text
[origin:unknown] timeout exceeded after 360022ms
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
