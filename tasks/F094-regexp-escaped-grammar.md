# F094: Compile valid escaped Unicode-mode RegExp patterns

- **Status:** open
- **Owner:** lila-ir regexp escape parser and program admission
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F094.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

unicode-disallow-extended reaches unsupported pattern while testing a mix of valid escaped punctuation/classes and invalid extended patterns. The first exact failing pattern is not recorded, so the root cause cannot be reduced to missing syntax rejection alone.

## Source evidence

- [crates/lila-ir/src/regexp.rs:2359](../crates/lila-ir/src/regexp.rs#L2359): Escape parsing owner.
- [test262/vendor/test262/test/staging/sm/RegExp/unicode-disallow-extended.js:12](../test262/vendor/test262/test/staging/sm/RegExp/unicode-disallow-extended.js#L12): Fixture mixes accepted escaped punctuation and rejected extensions.

## Work

Instrument the pattern admission result and identify the valid escaped form that lacks a matcher; retain SyntaxError for invalid u-mode extensions.

## Validation

Run both original unicode-disallow-extended modes, then focused valid and invalid grammar pairs.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F094.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F094-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/unicode-disallow-extended.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@2136784: RegExp.prototype.exec unsupported pattern)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
