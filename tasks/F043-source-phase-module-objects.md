# F043: Implement source-phase module bindings and source objects

- **Status:** open
- **Owner:** lila-ir modules graph resolution, namespace and dynamic imports
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 7 executions across 5 physical files (Bug 7, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F043.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Source-phase handling uses a retained merged-script path and an ordinary null-prototype placeholder rather than a spec-shaped AbstractModuleSource. The recorded failures include missing source re-exports, unresolved module-source requests, missing prototype behavior, and source-text evaluation accepted where SyntaxError is required.

## Source evidence

- [crates/lila-ir/src/modules/namespace.rs:676](../crates/lila-ir/src/modules/namespace.rs#L676): The implementation documents the ordinary-object placeholder.
- [crates/lila-ir/src/modules/graph_resolution.rs:147](../crates/lila-ir/src/modules/graph_resolution.rs#L147): Source-phase binding resolution entry point.

## Work

Create the correct module-source type and phase-specific loading/linking semantics, preserve source bindings through re-exports, and reject unsupported source-text module kinds as specified.

## Validation

Run every attached static and dynamic source-phase case and validate prototype chain, identity, re-export resolution, and rejection type.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F043.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F043-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/ambiguous-export-bindings/namespace-unambiguous-if-import-source-and-export.js` — Bug

```text
[origin:unknown] module ./namespace-import-source-and-export-reexport_FIXTURE.js does not export mod
```

- `module:language/module-code/source-phase-import/reexport-source-binding-named-import.js` — Bug

```text
[origin:unknown] module ./reexport-source-binding_FIXTURE.js does not export x
```

- `module:language/module-code/source-phase-import/reexport-source-binding-namespace-get.js` — Bug

```text
[origin:unknown] unresolved module request: <module source>
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
