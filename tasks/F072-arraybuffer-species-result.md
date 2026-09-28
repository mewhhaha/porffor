# F072: Accept valid subclass ArrayBuffers returned by proxied species constructors

- **Status:** open
- **Owner:** lila-aot-wasm ArrayBuffer.slice species construction
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F072.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A Proxy species constructor returns a new MyArrayBuffer subclass instance, but slice rejects it as an invalid ArrayBuffer. Construction/result tag or internal-slot checks are losing the valid subclass buffer identity through proxy/newTarget handling.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:20561](../crates/lila-aot-wasm/src/builtins/standard.rs#L20561): Exact validation error occurs at several species-result checks.
- [test262/vendor/test262/test/staging/sm/ArrayBuffer/slice-species.js:41](../test262/vendor/test262/test/staging/sm/ArrayBuffer/slice-species.js#L41): Proxy species returns a branded subclass buffer.

## Work

Reduce the first proxied species return, inspect resulting internal slots/byteLength, and validate by ArrayBuffer brand rather than constructor/prototype identity.

## Validation

Run both slice-species modes and preserve the expected species/prototype trap order.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F072.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F072-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/ArrayBuffer/slice-species.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@2015024: ArrayBuffer species constructor returned invalid ArrayBuffer)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
