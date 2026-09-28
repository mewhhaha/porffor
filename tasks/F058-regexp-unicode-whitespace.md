# F058: Match the full ECMAScript whitespace set in all RegExp paths

- **Status:** open
- **Owner:** lila-ir regexp class lowering; lila-aot-wasm RegExp matcher
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F058.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Whitespace escapes fail for non-ASCII members such as NBSP and BOM. The shared emitted whitespace helper contains those members, suggesting an alternate simple matcher or class-expansion path uses a smaller/byte-based set.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/regexp.rs:3652](../crates/lila-aot-wasm/src/builtins/regexp.rs#L3652): Shared program helper already lists NBSP/BOM, so alternate routes need inspection.

## Work

Identify whether class escape compilation or fallback execution drops non-ASCII whitespace, then share the complete code-point predicate in both modes.

## Validation

Run character-class-escape-s and unicode-character-class-escape in both modes, including complement cases.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F058.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F058-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/character-class-escape-s.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636064: Expected SameValue(«false», «true») to be true)
```

- `sloppy-script:staging/sm/RegExp/unicode-character-class-escape.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1665808: Actual [\t\r
\u000b\u000c] and expected [\t\r
\u000b\u000c ﻿] should have the same contents. )
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
