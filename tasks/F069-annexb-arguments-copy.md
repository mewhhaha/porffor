# F069: Copy Annex B function declarations into the actual arguments binding

- **Status:** open
- **Owner:** lila-ir Annex B binding planning; lila-aot-wasm annex_b_function_copy
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 2 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F069.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A block function named arguments fails to replace the surrounding function arguments object in both plain and eval-visible functions. Annex B copy targets an ordinary variable cell rather than the special arguments binding, or omits its owner copy.

## Source evidence

- [crates/lila-aot-wasm/src/control_flow/annex_b_function_copy.rs:1](../crates/lila-aot-wasm/src/control_flow/annex_b_function_copy.rs#L1): Annex B copy uses planned owner/block bindings.
- [test262/vendor/test262/test/staging/sm/lexical-environment/block-scoped-functions-annex-b-arguments.js:15](../test262/vendor/test262/test/staging/sm/lexical-environment/block-scoped-functions-annex-b-arguments.js#L15): The block declaration must overwrite the enclosing arguments binding.

## Work

Give Annex B owner-copy planning the canonical arguments binding identity and preserve the copy point after block function instantiation.

## Validation

Run both attached sloppy fixtures and verify arguments is an object before the block and a function afterward.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F069.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F069-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/lexical-environment/block-scoped-functions-annex-b-arguments.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636848: Expected SameValue(«"object"», «"function"») to be true)
```

- `sloppy-script:staging/sm/regress/regress-602621.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1985704: function sub-statement must override arguments Expected SameValue(«"function"», «"object"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
