# F013: Implement the missing Intl.ListFormat family

- **Status:** open
- **Owner:** lila-ir builtin catalog; lila-aot-wasm Intl intrinsics; lila-intl service/provider
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 160 executions across 80 physical files (Bug 160, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F013.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Intl.ListFormat is absent from the IR/builtin namespace constructor inventory. The shared namespace installer can only install that inventory, so its value is undefined and construction or property inspection fails before the intended API assertions. A lila-intl service capability name alone does not expose a JavaScript builtin.

## Source evidence

- [crates/lila-ir/src/names.rs:155](../crates/lila-ir/src/names.rs#L155): Complete installed constructor inventory omits ListFormat.
- [crates/lila-aot-wasm/src/planning/intl_namespace.rs:50](../crates/lila-aot-wasm/src/planning/intl_namespace.rs#L50): Both the rooted member plan and installer consume the same incomplete inventory.
- [crates/lila-intl/src/lib.rs:251](../crates/lila-intl/src/lib.rs#L251): Provider service naming exists, but does not install or implement the JavaScript constructor.

## Work

Implement ListFormat construction, locale/type/style resolution, iterable string consumption with IteratorClose and abrupt completions, format/formatToParts/resolvedOptions/supportedLocalesOf, and realm-aware intrinsic installation.

## Validation

Run every listed execution and the full intl402/ListFormat section in both modes, including subclass/newTarget, property descriptors, option conversion order, receiver validation, and cross-realm cases. Preserve the recorded Bug outcomes until a rerun verifies the implementation.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F013.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F013-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/ListFormat/constructor/constructor/locales-invalid.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@4027512: Expected a CustomError but got a TypeError)
```

- `sloppy-script:intl402/ListFormat/constructor/constructor/locales-valid.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1639424: target is not a constructor)
```

- `sloppy-script:intl402/ListFormat/constructor/constructor/newtarget-undefined.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1632960: Expected SameValue(«"undefined"», «"function"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
