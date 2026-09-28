# F068: Respect reference strictness when a super property write fails

- **Status:** open
- **Owner:** lila-aot-wasm super property mutation; lila-ir strictness
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 3 executions across 3 physical files (Bug 3, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F068.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Non-strict object methods with failed super assignments throw Cannot assign to super property although their references require silent failure. A staging fixture covering strict/non-strict assignment also fails. The strictness passed to the shared set-result handler needs tracing.

## Source evidence

- [crates/lila-aot-wasm/src/expressions/super_property_mutation.rs:104](../crates/lila-aot-wasm/src/expressions/super_property_mutation.rs#L104): The common PutValue helper already accepts a typed strictness; callers must preserve it.

## Work

Preserve the reference strictness from source context through super assignment emission and only throw on failed Set for strict references.

## Validation

Run dot/computed non-strict super fixtures and superPropStrictAssign with writable/nonwritable receiver cases.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F068.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F068-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/super/prop-dot-obj-ref-non-strict.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1641656: Cannot assign to super property)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
