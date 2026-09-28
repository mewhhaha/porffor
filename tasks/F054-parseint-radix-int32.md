# F054: Use ECMAScript ToInt32 for large parseInt radix arguments

- **Status:** open
- **Owner:** lila-aot-wasm builtins/host.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F054.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

parseInt converts finite radix with saturating f64-to-i64 then wraps to i32. Values beyond the i64 range saturate before modulo reduction, unlike ECMAScript ToInt32; the fixture expects radix 0 from 1e308 and therefore parses 0x10 as 16.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/host.rs:338](../crates/lila-aot-wasm/src/builtins/host.rs#L338): Saturating truncation in radix conversion loses the required modulo behavior.
- [test262/vendor/test262/test/staging/sm/Number/parseInt-01.js:92](../test262/vendor/test262/test/staging/sm/Number/parseInt-01.js#L92): The large-radix expectation is explicit.

## Work

Use the shared correct ToInt32 conversion for radix rather than I64TruncSatF64S followed by wrapping.

## Validation

Run Number.parseInt and global parseInt staging cases including 1e308, large positive/negative multiples of 2^32, NaN, and infinities.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F054.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F054-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/Number/parseInt-01.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1871656: Expected SameValue(«NaN», «16») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
