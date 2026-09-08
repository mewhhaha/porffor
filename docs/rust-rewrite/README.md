# Lila Rust Rewrite

Big rule first: Lila compiles JavaScript directly to Wasm. Lila does not sneak
an interpreter into Wasm and call that victory.

## Project Ground
- Root `AGENTS.md` freezes the rewrite goal and product bans.
- The Rust workspace under `crates/` owns the library, CLI, runtime semantics, and conformance tooling.
- The retired JavaScript implementation exists only in Git history; it is not a development surface or product oracle.

## Workspace Map
- `lila-front`: parse and source-unit plumbing.
- `lila-ir`: spec-shaped lowering stages and IR metadata.
- `lila-runtime`: realm plus typed host clock, randomness, and output capabilities.
- `lila-aot-wasm`: primary direct JS -> Wasm backend surface.
- `lila-backend-c`: future alternate C emitter.
- `lila-backend-native`: future alternate native emitter.
- `lila-engine`: public Rust library API.
- `lila-cli`: clean-break `lila` CLI.
- `lila-test262`: conformance taxonomy and harness rewrite support.

## Array Find Follow-up

The four generic Array find methods now share observable length acquisition and
live indexed reads, while strict TypedArray entries retain private validation.
The [find-family notes](aot-array-find.md) document the compiler contract, 24
regression programs, verification commands and evidence limitations. Published
full-suite conformance counts are unchanged.

## Hard Invariants
- Production compile path is `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
- Hidden debug interpreter is allowed only as non-product engineering tool.
- `build wasm` must emit compiled user program semantics and lowered builtins only.
