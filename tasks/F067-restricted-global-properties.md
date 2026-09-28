# F067: Protect restricted global names during declarations and assignment

- **Status:** open
- **Owner:** lila-ir global declaration instantiation; lila-aot-wasm global property writes
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 3 executions across 2 physical files (Bug 3, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F067.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

let undefined is admitted instead of throwing a runtime SyntaxError and strict assignment to global undefined succeeds. Restricted global descriptor attributes are not consistently enforced at declaration-instantiation and PutValue boundaries.

## Source evidence

- [crates/lila-ir/src/lowering.rs:2227](../crates/lila-ir/src/lowering.rs#L2227): Global declaration planning entry point.
- [test262/vendor/test262/test/language/global-code/decl-lex-restricted-global.js:18](../test262/vendor/test262/test/language/global-code/decl-lex-restricted-global.js#L18): Exact restricted name declaration expected to fail at runtime.

## Work

Use the realm global object descriptor as the source for HasRestrictedGlobalProperty and failed strict writes; preserve global lexical/property separation.

## Validation

Run the two lexical collision modes and strict assignment fixture; validate original nonwritable/nonconfigurable global descriptors.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F067.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F067-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/global-code/decl-lex-restricted-global.js` — Bug

```text
[origin:boa-runtime] negative test expected runtime error but execution succeeded
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
