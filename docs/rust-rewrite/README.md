# Scriptw Rust Rewrite

[Project rename](scriptw-identity-migration.md) ·
[Wasmtime/V8 benchmark methodology](v8-baseline.md) ·
[Runtime requirements and eval limits](runtime-requirements.md) ·
[Archived development history](../../DEVELOPMENT_HISTORY.md).

[Current implementation, verified builds and remaining work](current-inventory.md)
is the project status entry point. Historical notes below retain their source/date.

[React SSR through Wasm-AOT](react-ssr.md) records a successful three-case
synchronous React 19.3.1 render and the failed public browser entry, with exact
Node comparisons and emitted artifact hashes.

Big rule first: Scriptw compiles JavaScript directly to Wasm. Scriptw does not sneak
an interpreter into Wasm and call that victory.

## Project Ground
- Root `AGENTS.md` freezes the rewrite goal and product bans.
- The Rust workspace under `crates/` owns the library, CLI, runtime semantics, and conformance tooling.
- The retired JavaScript implementation exists only in Git history; it is not a development surface or product oracle.

## Workspace Map
- `scriptw-front`: parse and source-unit plumbing.
- `scriptw-ir`: spec-shaped lowering stages and IR metadata.
- `scriptw-runtime`: realm plus typed host clock, randomness, and output capabilities.
- `scriptw-aot-wasm`: sole product backend, compiling JavaScript directly to Wasm.
- C/native output and a REPL are outside product scope; their placeholder routes
  and scaffold crates are retired.
- `scriptw-engine`: public Rust library API.
- `scriptw-cli`: clean-break `scriptw` CLI.
- `scriptw-test262`: conformance taxonomy and harness rewrite support.

## Array Find Follow-up

The four generic Array find methods now share observable length acquisition and
live indexed reads, while strict TypedArray entries retain private validation.
The [find-family notes](aot-array-find.md) document the compiler contract, 24
regression programs, verification commands and evidence limitations. Published
full-suite conformance counts are unchanged.

## Publication Restart Recovery

The low-RAM Wasm-AOT publication driver persists its observed matrix progress
across restarts, rejecting rollback, changed totals and missing checkpoints for
families with existing results. Its schema-2 migration and 45 control-flow
regressions are documented in [the restart-recovery contract](publication-progress-recovery.md).
This does not add compiler semantics or change the generated conformance status.

## DisplayNames and RelativeTimeFormat source batch

The isolated complete service batch adds real compiled constructors and output
methods, checked native profiles and host ABI 10 dispatch. JavaScript option
reads, coercions, errors, brands and Realm output allocation stay in Wasm. The
[consumer contract](contracts/intl-display-relative-wasm.md) and
[provider integration](intl-display-relative-provider.md) describe the authored
controls and pending coherent runtime gates. Source generation and review do
not establish MAIN admission or pinned conformance.

## Hard Invariants
- Production compile path is `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
- Hidden debug interpreter is allowed only as non-product engineering tool.
- `build wasm` must emit compiled user program semantics and lowered builtins only.
