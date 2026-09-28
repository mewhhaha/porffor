# F083: Lower yield in generator method defaults and nested expressions

- **Status:** open
- **Owner:** lila-ir method and resumable lowering
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 0, NotImplemented 2, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F083.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The generator method fixture leaves Yield AST in the ordinary expression path, which has no lowering for it. A method-specific suspension traversal is incomplete.

## Source evidence

- [crates/lila-ir/src/lowering.rs:7635](../crates/lila-ir/src/lowering.rs#L7635): The raw Yield reaches the unsupported expression fallback.
- [test262/vendor/test262/test/staging/sm/class/methDefnGen.js:54](../test262/vendor/test262/test/staging/sm/class/methDefnGen.js#L54): Fixture exercising generator method yield.

## Work

Locate the generator method context lost in methDefnGen and route its yielded expression through the common resumable plan.

## Validation

Run both methDefnGen executions and a reduced generator-method next/return regression.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F083.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F083-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/class/methDefnGen.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Yield(Yield { target: Some(Literal(Literal { kind: Int(1), span: Span((268, 22), (268, 23)) })), delegate: false, span: Span((268, 16), (268, 23)) })
```

- `strict-script:staging/sm/class/methDefnGen.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Yield(Yield { target: Some(Literal(Literal { kind: Int(1), span: Span((269, 22), (269, 23)) })), delegate: false, span: Span((269, 16), (269, 23)) })
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
