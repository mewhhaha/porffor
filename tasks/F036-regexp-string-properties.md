# F036: Compile Unicode properties of strings and RGI emoji

- **Status:** open
- **Owner:** lila-ir regexp matcher grammar; lila-aot-wasm RegExp
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 12 executions across 6 physical files (Bug 12, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F036.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The matcher grammar explicitly classifies Unicode properties of strings as unsupported. All attached generated RGI emoji tests require those sequence-valued properties under v mode and ultimately surface unsupported pattern.

## Source evidence

- [crates/lila-ir/src/regexp.rs:951](../crates/lila-ir/src/regexp.rs#L951): Explicit matcher-program grammar limitation.

## Work

Represent string-valued Unicode sets as ordered multi-code-point alternatives with correct union/intersection/subtraction and case handling, using the existing compiled matcher.

## Validation

Run generated RGI emoji versions 13.1 through 17.0 in both modes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F036.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F036-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/RegExp/unicodeSets/generated/rgi-emoji-13.1.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@3187840: RegExp.prototype.exec unsupported pattern)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
