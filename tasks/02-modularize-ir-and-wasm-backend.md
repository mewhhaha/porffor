# T02 — Modularize the IR and Wasm backend

**Status:** In progress — initial boundaries landed; large ownership bottlenecks remain

**Parallel group:** Bootstrap/foundation  
**Depends on:** None  
**Blocks:** Safe parallel work in T04-T24

## Current repository state

Both crates now expose dedicated IR, lowering, analysis, diagnostics,
operations, ABI, heap, object, function, environment, control-flow and builtin
modules, and `./scripts/check-module-boundaries.sh` passes. The split is only
partial: `porffor-ir/src/lib.rs`, `lowering.rs`, several Wasm builtin files and
object/operation emitters remain very large implementation stores. Treat the
existing module boundaries as usable, but continue coordinating broad edits to
those remaining hotspots.

## Objective

Split the current monolithic compiler implementation into stable ownership boundaries without changing JavaScript behavior or emitted semantics. At the time this plan was written, `porffor-ir/src/lib.rs` and `porffor-aot-wasm/src/lib.rs` are tens of thousands of lines and are the primary merge-conflict bottleneck.

## Required module boundaries

The exact filenames may change, but the resulting architecture must expose equivalent boundaries.

### `porffor-ir`

- `ir/`: public `ProgramIr`, statements, expressions, functions, classes, properties, shapes, value information and IDs.
- `lowering/`: AST-to-spec-IR lowering, split by declarations, expressions, statements, functions/classes and modules.
- `early_errors/`: checks that are not delegated blindly to parser diagnostics.
- `builtins/`: builtin IDs, metadata, intrinsic ownership and feature registration.
- `analysis/`: scope/capture analysis, static shape/value analysis and unsupported-feature reporting.
- `operations/`: typed representations of shared ECMAScript abstract operations consumed by backends.
- `diagnostics/`: structured diagnostic codes and source locations.

### `porffor-aot-wasm`

- `module/`: sections, imports/exports, tables, globals, data and validation.
- `abi/`: tagged values, call/construct convention, completion convention and host imports.
- `heap/`: allocation/layout/string/object/environment storage and memory growth.
- `emit/`: statement/expression/control-flow dispatch.
- `operations/`: code generation for shared abstract operations.
- `objects/`, `functions/`, `environments/`: internal-method emitters.
- `builtins/<family>.rs`: separate builtin families such as array, string, regexp, typed-array, date and intl.
- `intrinsics/`: realm bootstrap and property descriptor installation.

Keep a small `lib.rs` that re-exports the public API and invokes the top-level pipeline.

## Implementation sequence

1. Add characterization tests before moving code: compile representative fixtures, validate Wasm, execute with the project's lower-bound runtime (experimental Wasmtime feature set per `AGENTS.md`), and record observable outputs/completion kinds. Do not gate characterization on engines that lack the lower-bound features (Wasm GC, typed function references, `exnref`).
2. Extract pure data types and constants first, then helpers, then large emit branches.
3. Replace magic global/table/layout numbers with named registries that can assert uniqueness and stable ordering.
4. Use private modules and narrow `pub(crate)` interfaces. Do not make every helper public to avoid import work.
5. Preserve error text only where tests rely on classification; otherwise prefer stable diagnostic codes over brittle strings.
6. Move tests beside the module they cover while retaining end-to-end crate tests.
7. Keep each extraction commit buildable so regressions are bisectable.

## Non-goals

- No new builtin coverage.
- No redesign of the value representation or completion ABI; those belong to T04/T05.
- No bulk formatting of legacy JavaScript code.
- No generated Test262 count changes beyond accidental stale-status cleanup.

## Acceptance criteria

- Both giant `lib.rs` files become orchestration/re-export surfaces rather than implementation stores.
- Feature families have clear files that separate agents can own.
- There are no cyclic module dependencies or duplicate constant registries.
- Public APIs used by `porffor-engine` remain coherent and documented.
- Representative emitted artifacts behave identically before and after extraction. If byte identity is not practical, compare imports, exports, validation, output, completion kind and thrown error class.
- Workspace compile time and binary size do not regress materially solely because of module movement.

## Required tests

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p porffor-ir --quiet
cargo test -p porffor-aot-wasm --quiet
cargo test -p porffor-engine --quiet
cargo test -p porffor-cli --quiet
./target/debug/porf test262 run language/wasm/pass \
  --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 \
  --execution-backend wasm
```

Also run several previously green real Test262 filters from different families to detect moved-helper regressions.
