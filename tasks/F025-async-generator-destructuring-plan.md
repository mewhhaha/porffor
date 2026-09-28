# F025: Order async-generator destructuring and iterator suspension points

- **Status:** open
- **Owner:** lila-ir resumable planning and lowering.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 26 executions across 13 physical files (Bug 0, NotImplemented 26, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F025.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The compiler consumes a ForAwaitNext suspension at plan index zero but the plan records a Yield from destructuring instead. Planning order and lowering order disagree for yielded defaults/nested patterns in for-await heads.

## Source evidence

- [crates/lila-ir/src/lowering.rs:5301](../crates/lila-ir/src/lowering.rs#L5301): Exact kind/order consistency failure.

## Work

Use one ordered plan for iterator-next suspension and destructuring evaluation, so lowering consumes the same typed suspension sequence produced by planning.

## Validation

Run all 26 attached async-generator destructuring variants, including iterator return/close paths.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F025.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F025-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/for-await-of/async-gen-decl-dstr-array-elem-init-yield-expr.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: async-generator resumable plan for `f10` expected ForAwaitNext at index 0, found Yield
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
