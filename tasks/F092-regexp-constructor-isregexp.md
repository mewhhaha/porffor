# F092: Use IsRegExp and constructor identity before reading internal slots

- **Status:** open
- **Owner:** lila-aot-wasm RegExp construction
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F092.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

RegExp constructor identity/proxy/realm fixture throws an internal-slot TypeError. IsRegExp can be true through Symbol.match on a Proxy or ordinary object without RegExp internal slots; constructor identity and source/flags property reads must select the correct path before slot-only operations.

## Source evidence

- [test262/vendor/test262/test/staging/sm/RegExp/constructor-constructor.js:68](../test262/vendor/test262/test/staging/sm/RegExp/constructor-constructor.js#L68): The fixture explicitly admits an ordinary object as IsRegExp.
- [crates/lila-aot-wasm/src/builtins/string.rs:1782](../crates/lila-aot-wasm/src/builtins/string.rs#L1782): Brand-check call sites must only follow the internal-slot path.

## Work

Reduce the first failing constructor case, separate IsRegExp from internal-slot brand checks, and preserve constructor/flags/getter order through proxies and realms.

## Validation

Run both constructor-constructor modes, including same-realm identity and foreign/proxy source copying.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F092.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F092-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/constructor-constructor.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@2512216: RegExp.prototype.exec receiver is not RegExp)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
