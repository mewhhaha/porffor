# T12 — Modules, linking, loading and namespace objects

**Status:** Blocked on T06-T10  
**Parallel group:** Feature lane  
**Depends on:** T06, T07, T08, T09, T10  
**Blocks:** Module portion of T14, T23 and T26

## Objective

Compile complete ECMAScript module graphs ahead of time, with live bindings, cyclic linking and host-controlled resolution, without evaluating module source through an embedded interpreter.

## Compile-time model

Add module IR for:

- requested modules and import attributes present in the pin;
- local/import/indirect/star export entries;
- top-level declarations and module environment bindings;
- `import.meta` and dynamic import expressions;
- async/top-level-await status and dependency edges;
- source phase/module-source features if present in the pinned suite.

The CLI/library should accept an entry module plus a loader/resolver and produce a deterministic graph. Cache modules by normalized host key and reject inconsistent duplicate loads.

## Linking and evaluation

Implement spec-shaped phases:

1. parse all reachable modules;
2. resolve exports, including ambiguity and star cycles;
3. create module environments and namespace objects;
4. instantiate declarations/functions;
5. evaluate in dependency order with cycle handling;
6. coordinate async evaluation/top-level await through T14's job model.

Live imported bindings must reference exporter cells and remain read-only to the importer. Cyclic graphs must not be flattened into initialization-order guesses.

## Module namespace exotic object

Implement exact namespace behavior:

- sorted exported-string keys plus symbols in correct order;
- live getters/read-only semantics;
- null prototype, non-extensibility and `@@toStringTag`;
- custom internal methods and descriptor behavior;
- identity caching per module.

## Host loader contract

Define a Rust trait for resolve/load with referrer, attributes and module type. The Test262 loader may use repository files; product embedders may supply other sources. Prevent path traversal in the default filesystem loader. Do not bake Test262 paths into module semantics.

## Artifact strategy

Document whether a graph is emitted as one Wasm module or multiple linked modules. The first complete implementation may emit one module, but module records and live bindings must remain explicit so the design can evolve. `build wasm` must include compiled semantics, not source strings fed to a runtime parser.

## Acceptance criteria

- Static import/export, re-export, namespace import and side-effect-only import cases pass.
- Cycles, live bindings, star ambiguity, TDZ and evaluation order pass.
- Module namespace descriptor/internal-method tests pass.
- Top-level `this`, strictness, `import.meta` and host resolution behavior are correct.
- Parse/link/evaluate failures are classified at the right phase.
- Dynamic import is integrated with promises/jobs rather than synchronous source evaluation.
- The pinned `language/module-code` and related module builtins reach zero failures.

## Required tests

```sh
cargo test -p porffor-ir module_ --quiet
cargo test -p porffor-engine module_ --quiet
cargo test -p porffor-cli module_ --quiet
./target/debug/porf test262 run language/module-code --execution-backend wasm --timeout-ms 120000 --threads 4
```

Add filesystem-loader tests for cycles, missing modules, traversal rejection, duplicate normalized specifiers and import attributes.