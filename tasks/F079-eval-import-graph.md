# F079: Discover module imports inside compiled literal eval

- **Status:** open
- **Owner:** lila-ir module discovery and eval source lowering
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 0, NotImplemented 2, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F079.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

eval receives a literal import expression, but the selected lowered script carries no compiled module graph. The emitted-source guard rejects import() before loading its known fixture target. This is a graph propagation gap for finite source, not the arbitrary eval boundary.

## Source evidence

- [test262/vendor/test262/test/language/expressions/dynamic-import/usage-from-eval.js:26](../test262/vendor/test262/test/language/expressions/dynamic-import/usage-from-eval.js#L26): Literal source and import specifier are compile-time available.
- [crates/lila-ir/src/modules/dynamic.rs:8](../crates/lila-ir/src/modules/dynamic.rs#L8): Dynamic import graph planning owner.

## Work

Include statically compiled eval imports in module graph discovery and preserve the referencing script/module for specifier resolution.

## Validation

Run both usage-from-eval executions with the original relative module fixture and verify namespace values and promise identity.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F079.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F079-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/expressions/dynamic-import/usage-from-eval.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot: module dynamic import without a compiled graph (the host lowered a source that writes `import()` without loading its targets) emission. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
