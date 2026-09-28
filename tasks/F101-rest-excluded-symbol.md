# F101: Exclude Symbol keys before proxy descriptor traps in object rest

- **Status:** open
- **Owner:** lila-ir destructuring references; lila-aot-wasm CopyDataProperties
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F101.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Object rest invokes getOwnPropertyDescriptor on an excluded Symbol key. The current copy helper checks exclusions before descriptors, so the likely discrepancy is symbol payload/tag identity supplied by the destructuring exclusion list, rather than descriptor sequencing inside that helper.

## Source evidence

- [crates/lila-aot-wasm/src/control_flow.rs:11173](../crates/lila-aot-wasm/src/control_flow.rs#L11173): Exclusion equality precedes descriptor lookup, directing investigation to key representation.

## Work

Trace key canonicalization and preserve Symbol identity consistently in the evaluated key and excluded list; do not stringify symbols.

## Validation

Run both original proxy trap-log executions and verify excluded keys cause no descriptor trap.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F101.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F101-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/object/dstr/object-rest-proxy-gopd-not-called-on-excluded-keys.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1668560: Actual [Symbol(excluded_symbol), Symbol(included_symbol), includedString, 1] and expected [Symbol(included_symbol), includedString, 1] should have the same contents. )
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
