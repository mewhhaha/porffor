# F039: Key template objects by parsed site and realm across eval

- **Status:** open
- **Owner:** lila-ir template site allocation; lila-aot-wasm template object globals
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 12 executions across 6 physical files (Bug 12, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F039.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Distinct template sites created by eval/Function and distinct realms return the same template object. The backend stores template objects in one map/global per numeric site, initialized once; source/realm identity appears lost or reused when specializing dynamic source.

## Source evidence

- [crates/lila-aot-wasm/src/data.rs:444](../crates/lila-aot-wasm/src/data.rs#L444): Template objects are stored by a bare site id.
- [crates/lila-aot-wasm/src/emit.rs:5033](../crates/lila-aot-wasm/src/emit.rs#L5033): Objects are allocated into site globals during initialization.

## Work

Give parsed template sites stable source-instance identity and allocate/cache each site separately per realm; repeated execution of one site must still share its own object.

## Validation

Run all attached tagged-template cache cases, with same-site identity and distinct parse/eval/realm identity checks.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F039.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F039-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/tagged-template/cache-differing-expressions-eval.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1984792: Expected SameValue(«head,tail», «head,tail») to be false)
```

- `sloppy-script:language/expressions/tagged-template/cache-differing-expressions-new-function.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1637144: The realm's template cache is by site, not string contents Expected SameValue(«head,tail», «head,tail») to be false)
```

- `sloppy-script:language/expressions/tagged-template/cache-eval-inner-function.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1994112: Expected SameValue(«,,», «,,») to be false)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
