# F077: Defer Object.getOwnPropertyDescriptor argument errors to runtime

- **Status:** open
- **Owner:** lila-ir lowering/builtin_call_info.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 0, NotImplemented 2, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F077.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Builtin call analysis rejects missing/unsupported argument kinds at compile time with requires object. The fixture expects ECMAScript ToObject/coercion or a catchable TypeError, so the compiler rejects executable source.

## Source evidence

- [crates/lila-ir/src/lowering/builtin_call_info.rs:464](../crates/lila-ir/src/lowering/builtin_call_info.rs#L464): The no-argument branch unconditionally calls unsupported.

## Work

Remove the compile-time semantic refusal and emit the builtin runtime validation/conversion with proper abrupt completion.

## Validation

Run both getOwnPropertyDescriptor staging modes, including missing/nullish and boxed primitive arguments.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F077.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F077-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/object/getOwnPropertyDescriptor.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: Object.getOwnPropertyDescriptor requires object
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
