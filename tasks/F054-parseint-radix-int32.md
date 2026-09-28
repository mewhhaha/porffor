# F054: Use ECMAScript ToInt32 for large parseInt radix arguments

- **Status:** fixed
- **Owner:** lila-aot-wasm builtins/host.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F054.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The frozen compiler converted finite radices with a saturating f64-to-i64 conversion followed by an i32 wrap. Values beyond the i64 range lost their required modulo residue; the fixture requires radix 0 from 1e308 and therefore expects 0x10 to parse as 16.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/host.rs:320](../crates/lila-aot-wasm/src/builtins/host.rs#L320): The radix site previously saturated the Number to i64 before wrapping; it now calls the shared 32-bit residue emitter.
- [test262/vendor/test262/test/staging/sm/Number/parseInt-01.js:92](../test262/vendor/test262/test/staging/sm/Number/parseInt-01.js#L92): The large-radix expectation is explicit.

## Work

Use the shared 32-bit Number residue emitter for the radix, then interpret the residue as signed ToInt32.

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

## Resolution

Fixed in `2ad1bf30e1fa203720de048d1dc5f1041e8fc47e` and accepted after independent review. The Wasm-AOT builtin obtains the full 32-bit residue before signed interpretation. [Verification record](evidence/resolutions/F054.json) retains native outcomes and compiler identity.

- Fresh release compiler: `394fbf4adfb31011551aabad7b22990104af6ba0343e0fbdb168caf84f71dadd`.
- Exact replay: **4/4 Success**, with native evidence in `target/test262-scratch/F054-sol-20260928/replay/`.
- Adjacent Test262: **114/114 Success** — global parseInt 110, Number.parseInt 2, and staging default-to-decimal 2. Native snapshots are in `target/test262-scratch/F054-sol-20260928/adjacent-snapshots/`.
- Focused Wasm-AOT CLI regression: **1/1 passed**. The same fixture fails on the original frozen compiler at the large positive radix expectation.
- Independent review: **1,680 checks passed** across 420 numeric inputs, using Python arbitrary-precision integer modulo for expected results. Evidence is in `target/test262-scratch/F054-parent-review/`.

GPT-6-Sol at max reasoning produced the implementation and regression without code corrections from review. Builds and runs were capped at 10 CPUs and half physical RAM, with swap disabled. The full pinned matrix was not rerun; frozen evidence and published README counts remain unchanged.
