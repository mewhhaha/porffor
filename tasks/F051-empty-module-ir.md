# F051: Emit modules whose only statements are import/export declarations

- **Status:** open
- **Owner:** lila-ir module lowering; lila-aot-wasm emit.rs
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 4 physical files (Bug 0, NotImplemented 4, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F051.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Four module import/export fixtures reach Wasm emission without a lowered script IR. Their declaration-only/namespace re-export shapes likely produce no executable body and are treated as absent source rather than a valid empty module. Exact front-end omission needs confirmation.

## Source evidence

- [crates/lila-aot-wasm/src/emit.rs:993](../crates/lila-aot-wasm/src/emit.rs#L993): Emission fails when lowered_script is absent.

## Work

Trace the module admission/link output and preserve an explicit empty script/module execution body where instantiation has no statements.

## Validation

Run all four attached module fixtures and verify exports/import attributes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F051.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F051-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/export-expname-from-star-string.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: no lowered script ir. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
