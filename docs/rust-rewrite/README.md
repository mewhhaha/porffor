# Lila Rust Rewrite

Big rule first: Lila compiles JavaScript directly to Wasm. Lila does not sneak
an interpreter into Wasm and call that victory.

## Phase 0 Ground
- Root `AGENTS.md` freezes rewrite goal and product bans.
- Rust workspace under `crates/` is new home for library, CLI, runtime semantics, and conformance work.
- Existing JavaScript implementation stays in tree as reference oracle until Rust path is proven.

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

## Hard Invariants
- Production compile path is `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
- Hidden debug interpreter is allowed only as non-product engineering tool.
- `build wasm` must emit compiled user program semantics and lowered builtins only.
