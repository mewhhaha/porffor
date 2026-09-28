# F059: Lower spread arguments through RegExp lastIndex helper calls

- **Status:** open
- **Owner:** lila-ir call/expression lowering
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 0, NotImplemented 4, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F059.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Spread AST nodes in the two RegExp staging fixtures reach ordinary expression lowering rather than argument-list expansion. Both modes refuse before runtime.

## Source evidence

- [crates/lila-ir/src/lowering.rs:7635](../crates/lila-ir/src/lowering.rs#L7635): The raw Spread expression hits the generic fallback.

## Work

Trace the spread through call specialization and use the iterable argument-vector lowering while preserving evaluation order and abrupt completions.

## Validation

Run all four attached executions with their original lastIndex assertions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F059.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F059-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/RegExp/lastIndex-match-or-replace.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Spread(Spread { target: Identifier(Identifier { ident: Sym { value: 162 }, span: Span((241, 40), (241, 44)) }), span: Span((241, 37), (241, 44)) })
```

- `strict-script:staging/sm/RegExp/lastIndex-match-or-replace.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Spread(Spread { target: Identifier(Identifier { ident: Sym { value: 163 }, span: Span((242, 40), (242, 44)) }), span: Span((242, 37), (242, 44)) })
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
