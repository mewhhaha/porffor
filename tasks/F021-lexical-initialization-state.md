# F021: Enforce uninitialized lexical state on reads and writes

- **Status:** open
- **Owner:** lila-ir binding lifecycle; lila-aot-wasm environments
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 46 executions across 23 physical files (Bug 46, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F021.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

let/const/destructuring and using declarations permit reads or writes before initialization. Resource declarations then see undefined and throw TypeError instead of the earlier ReferenceError. These failures point to missing runtime initialization checks or premature binding initialization, including writes through closures and assignment patterns; the current direct-assignment lowering has a check, so each storage path must be traced.

## Source evidence

- [crates/lila-ir/src/lowering.rs:13907](../crates/lila-ir/src/lowering.rs#L13907): Direct identifier writes claim to check initialization; bypassing paths need comparison.
- [crates/lila-aot-wasm/src/control_flow.rs:7255](../crates/lila-aot-wasm/src/control_flow.rs#L7255): Wrong later TypeError shows TDZ access was not rejected first.

## Work

Unify lexical cell initialization state across declarations, captures, destructuring and resource acquisition. Distinguish InitializeBinding from SetMutableBinding and check uninitialized state before immutability/resource validation.

## Validation

Run all attached modes covering block/function/global TDZ reads and writes, closure assignment, for-of destructuring and using/await using error precedence.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F021.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F021-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/assignment/dstr/array-elem-put-let.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1634784: Expected a ReferenceError to be thrown but no exception was thrown at all)
```

- `sloppy-script:language/statements/await-using/block-local-use-before-initialization-in-declaration-statement.js` — Bug

```text
[origin:unknown] Test262:AsyncTestFailure:Test262Error: Test262Error: Expected a ReferenceError to be thrown asynchronously but got a TypeError
```

- `sloppy-script:language/statements/const/global-use-before-initialization-in-declaration-statement.js` — Bug

```text
[origin:boa-runtime] negative test expected runtime error but execution succeeded
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
