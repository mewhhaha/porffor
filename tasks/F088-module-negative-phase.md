# F088: Preserve module dependency parse errors as resolution failures

- **Status:** open
- **Owner:** lila-test262 negative matching; lila-engine module diagnostics
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 2 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F088.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A SyntaxError while parsing an imported dependency is reported with parser phase. compile_negative_error_matches explicitly rejects parser-phase errors for resolution negatives, so these tests are marked mismatches even though the error constructor is SyntaxError. The diagnostic must retain where the parse failure occurred in the module graph.

## Source evidence

- [crates/lila-test262/src/lib.rs:10883](../crates/lila-test262/src/lib.rs#L10883): The parse diagnostic phase cross-product rejects Resolution expectations.

## Work

Represent dependency parse failure as a module resolution-phase diagnostic without changing top-level parse-negative matching or accepting arbitrary compiler errors.

## Validation

Run instn-resolve-err-syntax-1 and instn-resolve-order-depth, plus top-level parse-negative and wrong-error-kind regressions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F088.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F088-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/instn-resolve-err-syntax-1.js` — Bug

```text
[origin:boa-parser] negative test error mismatch: expected SyntaxError, got E_ILLEGAL_BREAK SyntaxError parse error: illegal break statement at line 1, col 1: parse error: illegal break statement at line 1, col 1
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
