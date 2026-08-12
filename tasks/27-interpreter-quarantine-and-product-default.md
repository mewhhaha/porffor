# T27 — Interpreter quarantine and Wasm-AOT product default

**Status:** In progress — all product-boundary implementation landed; final batch verification remains

**Parallel group:** Validation/closure  
**Blocks:** T26 release gate and any truthful "no interpreter" claim

**Depends on:** T02, T03

## Current repository state

Wasm-AOT is now the engine and CLI default, `lila-spec-exec`/`boa_engine` is
behind the off-by-default `spec-exec-oracle` feature, and
`./scripts/check-no-interpreter-in-product-graph.sh` passes. There is no silent
fallback in the default product path. Publication now accepts only the closed
`PublicationBackend::WasmAot` domain; both the CLI and low-RAM script reject an
oracle publication request before writing status. The CI-wired product-artifact
test proves representative Wasm contains compiled user semantics, no embedded
source marker and no evaluator import, and it is green in the current batch.
Only the final combined repository/CLI verification remains before this task's
status can be closed.

## Objective

Keep the Wasm-AOT compiler as the product execution path in code, not only in
documentation. Before this task's core implementation landed,
`lila-spec-exec` wrapped the Boa JavaScript interpreter as the engine-wide
default, a first-class CLI backend and a co-equal published conformance target.
Under `AGENTS.md` an interpreter may exist only as a hidden
debug/differential oracle: never the CLI runtime path, never the default, never
a silent fallback, never linked into product builds or emitted artifacts, and
never the source of published conformance numbers.

## Work items

### 1. Flip the default backend

- Change the `lila-engine` `ExecutionBackend` default and every CLI entry point (`lila run`, `lila build`, `lila test262 ...`) to Wasm-AOT.
- Selecting spec-exec requires an explicit developer-only flag (for example `--execution-backend spec-exec` marked internal/debug in help text); document it as the differential oracle, not a runtime.
- A Wasm-AOT compilation or execution failure must surface as a failure. Audit for and remove any path that silently retries or falls back to spec-exec.

### 2. Quarantine the interpreter dependency

- Gate `lila-spec-exec` (and its `boa_engine` dependency) behind a cargo feature that is off for product/release builds of `lila-engine` and `lila-cli`, or split oracle functionality into a separate developer binary, so product binaries link no JavaScript interpreter/VM engine.
- Add an automated dependency check (a CI `cargo tree` assertion or a test) proving the product build graph contains no interpreter engine crate.
- The vendored Boa *parser/AST* crates used by `lila-front` are compile-time tools and remain allowed; the quarantine boundary is the interpreter/engine, not the parser.

### 3. Artifact audit

- Add a test that `lila build wasm` output for representative programs contains compiled user semantics: no interpreter loop and no embedded user source text that a runtime evaluator consumes (module records and diagnostics may retain source metadata; it must not be an execution input).
- Wire this audit into the T26 integrity audit so closure re-verifies it.

### 4. Status and reporting policy

- Published README/status artifacts report Wasm-AOT as the only Lila conformance number.
- `spec-exec` matrices/snapshots remain producible for oracle triage but are labeled oracle-diagnostic, stored separately and excluded from product status blocks.
- Update `lila test262 publish-status` / `scripts/publish-real-status-low-ram.sh` so a spec-exec snapshot cannot be published as the product conformance block.

## Out of scope

- Deleting spec-exec entirely; T25's differential framework is its one legitimate consumer.
- Making Wasm-AOT pass tests it cannot yet pass. Flipping the default will expose honest failures; that visibility is the point, not a regression to hide.

## Acceptance criteria

- The default engine and CLI execution backend is Wasm-AOT everywhere; spec-exec requires an explicit internal flag.
- No code path silently falls back from Wasm-AOT to spec-exec.
- Product/release builds of the library and CLI link no interpreter engine crate, verified by an automated check that fails CI on reintroduction.
- The `build wasm` artifact audit test exists and passes for representative programs.
- Product status artifacts and the README conformance block contain only Wasm-AOT numbers; any oracle numbers present are clearly labeled non-product diagnostics.
- T25 differential runs can still invoke spec-exec through the developer-only configuration.

## Required tests

```sh
cargo test -p lila-engine --quiet
cargo test -p lila-cli --quiet
# Dependency quarantine: expect no interpreter engine crate in the product graph
# (exact feature flags per the implemented gating; the check must be wired into CI).
cargo tree -p lila-cli | grep -c boa_engine   # expect 0 in the product configuration
# Prove the default backend is Wasm-AOT by running the fake suite with no backend flag:
./target/debug/lila test262 run language/wasm/pass \
  --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262
```

Also run a representative previously green real Test262 filter through the default (flag-free) CLI invocation and confirm the reported backend in its output/snapshot metadata is `wasm-aot`.
