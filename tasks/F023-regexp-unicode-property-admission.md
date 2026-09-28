# F023: Compile recognized Unicode property patterns instead of unsupported execution

- **Status:** open
- **Owner:** lila-ir regexp property parser; lila-aot-wasm RegExp program dispatch
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 36 executions across 18 physical files (Bug 36, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F023.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A separate set of generated property escapes reaches RegExp.exec with no usable compiled matcher and throws unsupported pattern. Property recognition/data admission and static program emission must be checked for the exact properties; the symptom alone does not prove the same cause as membership mismatches.

## Source evidence

- [crates/lila-ir/src/regexp.rs:2619](../crates/lila-ir/src/regexp.rs#L2619): Recognition/data entry point for Unicode property ranges.
- [crates/lila-aot-wasm/src/builtins/string.rs:14198](../crates/lila-aot-wasm/src/builtins/string.rs#L14198): Fallback raises TypeError when no supported matching route is selected.

## Work

Capture the rejected pattern/property and compile error, add missing property aliases/data or matcher lowering, and propagate real syntax errors separately from implementation gaps.

## Validation

Run all attached property executions, then positive and complement cases for each previously rejected property.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F023.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F023-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/RegExp/property-escapes/generated/IDS_Binary_Operator.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1651776: RegExp.prototype.exec unsupported pattern)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
