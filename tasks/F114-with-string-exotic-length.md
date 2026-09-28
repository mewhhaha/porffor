# F114: Preserve boxed String length attributes through with assignment

- **Status:** open
- **Owner:** lila-aot-wasm with references and String exotic Set
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 1 executions across 1 physical files (Bug 1, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F114.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Assigning length=0 inside with(new String(...)) overwrites the boxed string nonwritable length. The with Set path bypasses String exotic/descriptor restrictions or uses a different property-write route.

## Source evidence

- [crates/lila-ir/src/lowering.rs:13997](../crates/lila-ir/src/lowering.rs#L13997): With-scoped identifier write lowering owner.
- [crates/lila-aot-wasm/src/builtins/string.rs:669](../crates/lila-aot-wasm/src/builtins/string.rs#L669): String exotic length derives from the wrapped string.

## Work

Route the evaluated with reference through the same descriptor-aware Set operation used for direct string-object writes and preserve sloppy failure semantics.

## Validation

Run the original S15.5.5.1_A4_T1 sloppy fixture and verify length remains eight.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F114.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F114-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/String/S15.5.5.1_A4_T1.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1634016: #4: var __str__instance = new String("globglob"); with(__str__instance) length = 0; __str__instance.length === 8(after redefine length property with using "with"). Actual: __str__instance.length ===0)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
