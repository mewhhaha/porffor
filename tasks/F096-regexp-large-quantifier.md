# F096: Represent RegExp quantifier bounds beyond machine-sized eager expansion

- **Status:** open
- **Owner:** lila-ir regexp quantifier parsing/program generation
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F096.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The quantifier-integer-limit fixture reaches unsupported pattern. Large valid repetition bounds exceed an internal parser/program limit or expansion strategy; the saved diagnostic does not distinguish which.

## Source evidence

- [crates/lila-ir/src/regexp.rs:961](../crates/lila-ir/src/regexp.rs#L961): Quantified terms are represented here before matcher-program generation.

## Work

Inspect the exact rejected bound and use a spec-correct bounded counter representation instead of rejecting valid quantifiers or expanding repetitions into unbounded code.

## Validation

Run both original quantifier-integer-limit variants and check overflow/infinity/zero-length repetition cases.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F096.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F096-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/RegExp/quantifier-integer-limit.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1632096: RegExp.prototype.exec unsupported pattern)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
