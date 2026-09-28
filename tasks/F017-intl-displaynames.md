# F017: Implement the missing Intl.DisplayNames family

- **Status:** open
- **Owner:** lila-ir builtin catalog; lila-aot-wasm Intl intrinsics; lila-intl service/provider
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 110 executions across 55 physical files (Bug 110, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F017.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Intl.DisplayNames is absent from the IR/builtin namespace constructor inventory. The shared namespace installer can only install that inventory, so its value is undefined and construction or property inspection fails before the intended API assertions. A lila-intl service capability name alone does not expose a JavaScript builtin. The two supportedValuesOf consumers also fail when they construct DisplayNames.

## Source evidence

- [crates/lila-ir/src/names.rs:155](../crates/lila-ir/src/names.rs#L155): Complete installed constructor inventory omits DisplayNames.
- [crates/lila-aot-wasm/src/planning/intl_namespace.rs:50](../crates/lila-aot-wasm/src/planning/intl_namespace.rs#L50): Both the rooted member plan and installer consume the same incomplete inventory.
- [crates/lila-intl/src/lib.rs:252](../crates/lila-intl/src/lib.rs#L252): Provider service naming exists, but does not install or implement the JavaScript constructor.

## Work

Implement DisplayNames construction, locale resolution, required type/style/fallback/languageDisplay options, internal slots, of, resolvedOptions, and standard property descriptors; install the family in both entry and created realms.

## Validation

Run every listed execution and the full intl402/DisplayNames section in both modes, including subclass/newTarget, property descriptors, option conversion order, receiver validation, and cross-realm cases. Preserve the recorded Bug outcomes until a rerun verifies the implementation.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F017.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F017-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/DisplayNames/ctor-custom-get-prototype-poison-throws.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636104: Expected a Test262Error but got a TypeError)
```

- `sloppy-script:intl402/DisplayNames/ctor-custom-prototype.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1634512: Reflect.construct target is not a constructor)
```

- `sloppy-script:intl402/DisplayNames/ctor-default-prototype.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1631712: target is not a constructor)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
