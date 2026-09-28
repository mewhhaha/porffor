# F049: Initialize class-name environments at the correct phase

- **Status:** open
- **Owner:** lila-ir class definition lowering; lila-aot-wasm class environments
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F049.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Static class-name TDZ access is accepted and repeated class-heritage construction later throws an unexpected lexical TDZ error. Class inner-name lifetime is being confused with the outer variable/current iteration binding. Exact failing subexpression in the long-chain fixture needs reduction.

## Source evidence

- [crates/lila-ir/src/lowering/class_definition.rs:1](../crates/lila-ir/src/lowering/class_definition.rs#L1): Class binding/heritage lowering owner.
- [test262/vendor/test262/test/staging/sm/class/superPropChains.js:56](../test262/vendor/test262/test/staging/sm/class/superPropChains.js#L56): The RHS heritage must resolve the old outer chain binding.

## Work

Separate class inner-name initialization from outer declaration assignment and capture the evaluated heritage before updating the destination binding.

## Validation

Run fields-static-class-name-binding and superPropChains in both modes, preserving the full 100-link chain.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F049.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F049-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/class/fields-static-class-name-binding.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1645240: Expected a ReferenceError to be thrown but no exception was thrown at all)
```

- `sloppy-script:staging/sm/class/superPropChains.js` — Bug

```text
[origin:unknown] uncaught throw: ReferenceError: wasm-aot completion: object(handle@1651328: lexical binding accessed before initialization)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
