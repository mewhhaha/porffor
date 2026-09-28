# F100: Preserve valid UTF-16/WTF-8 through replaceAll result construction

- **Status:** open
- **Owner:** lila-aot-wasm String.replaceAll and host string boundary
- **Cause assessment:** unresolved
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F100.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Both replaceAll executions produce a payload the host rejects as invalid UTF-8/WTF-8 at byte 23. The diagnostic proves malformed string bytes or length/offset metadata, but it may occur while formatting a prior assertion failure rather than in the returned replacement itself.

## Source evidence

- [crates/lila-aot-wasm/src/heap_string_layout.rs:6](../crates/lila-aot-wasm/src/heap_string_layout.rs#L6): String payload/encoding boundary owner.
- [test262/vendor/test262/test/staging/sm/String/replaceAll.js:53](../test262/vendor/test262/test/staging/sm/String/replaceAll.js#L53): Fixture exercises literal and function substitutions across many patterns.

## Work

Capture the raw string payload at the first failing boundary, reduce the replacement/search case, and verify code-unit slicing plus byte lengths and result encoding.

## Validation

Run both complete replaceAll modes including empty searches, surrogate boundaries and functional replacers; ensure failures themselves remain printable.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F100.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F100-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/String/replaceAll.js` — Bug

```text
[origin:unknown] wasm string result is not valid UTF-8/WTF-8 at byte 23
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
