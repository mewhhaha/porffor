# F075: Honor Date default-hint primitive conversion after method overrides

- **Status:** open
- **Owner:** lila-aot-wasm ToPrimitive and Date Symbol.toPrimitive
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F075.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The Date defaultvalue fixture sees false for an expected equality after exercising own valueOf/toString overrides. The saved assertion does not locate the first failing case, so the defect may be default-hint selection, overridden method dispatch, or cross-realm Date handling.

## Source evidence

- [crates/lila-aot-wasm/src/operations.rs:2957](../crates/lila-aot-wasm/src/operations.rs#L2957): Common ToPrimitive dispatch boundary.
- [test262/vendor/test262/test/staging/sm/Date/defaultvalue.js:23](../test262/vendor/test262/test/staging/sm/Date/defaultvalue.js#L23): Fixture overrides Date coercion methods.

## Work

Reduce the first equality failure and trace Symbol.toPrimitive/default-hint dispatch through method lookup; preserve user overrides and correct Date string preference.

## Validation

Run both full defaultvalue modes and reduced equality/addition cases with overridden methods.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F075.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F075-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Date/defaultvalue.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2588152: Expected SameValue(«false», «true») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
