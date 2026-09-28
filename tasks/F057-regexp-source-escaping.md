# F057: Escape RegExp source for slash and line terminators

- **Status:** fixed
- **Owner:** lila-aot-wasm RegExp source getter
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F057.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The frozen RegExp source getter returned OriginalSource directly, and ordinary RegExp toString used that raw source too. Slashes and raw line terminators were not escaped for literal reconstruction; cross-realm getters exposed the same incorrect payload.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/string/regexp_source.rs:32](../crates/lila-aot-wasm/src/builtins/string/regexp_source.rs#L32): Shared serialization emitter escapes the original pattern without changing matcher storage.
- [crates/lila-aot-wasm/src/builtins/string.rs:2013](../crates/lila-aot-wasm/src/builtins/string.rs#L2013): Ordinary RegExp stringification now uses the same escaping operation as the source getter.

## Work

Apply one EscapeRegExpPattern emitter at source serialization sites while preserving the original matcher source. Handle empty patterns, backslash parity, character classes, line terminators, UTF-8/WTF-8, and Unicode-set flags.

## Validation

Run source.js and cross-compartment-getter in both modes and the standard source value regressions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F057.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F057-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/cross-compartment-getter.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2530376: Expected SameValue(«"a/b"», «"a\\/b"») to be true)
```

- `sloppy-script:staging/sm/RegExp/source.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1640120: Expected SameValue(«"/"», «"\\/"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.

## Resolution

Fixed in `c0914161f349eab7489b7236fff691340c320f29` and accepted after independent review. [Verification record](evidence/resolutions/sol-batch-20260928.json) retains compiler identity, native snapshots, exact execution identities, focused checks, and review evidence.

- Fresh release compiler: `41300185c891072679474d8066d7c51634699253a3cc1f574de9cc90c2ce0ab9`.
- Exact replay: **4/4 Success**, with native evidence under `target/test262-scratch/sol-batch-20260928/replay/`.
- Adjacent Test262: **32/46 Success**. The 14 non-successes are unchanged baseline failures: 12 dynamic-code cases owned by F115 and two generic RegExp cases owned by F046. All were executed and retain their native outcomes.
- Focused Wasm-AOT engine regressions: **4/4 passed** in `aot_regexp_source_escaping`.
- The regression control passes on the new compiler and fails on the frozen pre-batch compiler.

Accepted without implementation corrections. Four Wasm regressions cover escaping, cloning/recompilation, Unicode code units, and cross-realm receiver checks; parent review passed 1,220 source/toString and reconstruction comparisons across 305 pattern/flag combinations, using independent Node results.

GPT-6-Sol at max reasoning implemented the fix. The integrated checkpoint passed 43 engine tests, 29 structural checks, and two unit tests. Builds and runs were capped at 10 CPUs and half physical RAM with swap disabled. The full pinned matrix was not rerun; frozen evidence and published README counts remain unchanged.
