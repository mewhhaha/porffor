# Grug Rust Rewrite

Big rule first: Porffor compile JavaScript directly to Wasm. Porffor not sneak interpreter into Wasm and call that victory.

## Phase 0 Ground
- Root `AGENTS.md` freezes rewrite goal and product bans.
- Rust workspace under `crates/` is new home for library, CLI, runtime semantics, and conformance work.
- Existing JavaScript implementation stays in tree as reference oracle until Rust path is proven.

## Workspace Map
- `porffor-front`: parse and source-unit plumbing.
- `porffor-ir`: spec-shaped lowering stages and IR metadata.
- `porffor-runtime`: realm and host-hook scaffolding.
- `porffor-aot-wasm`: primary direct JS -> Wasm backend surface.
- `porffor-backend-c`: future alternate C emitter.
- `porffor-backend-native`: future alternate native emitter.
- `porffor-engine`: public Rust library API.
- `porffor-cli`: clean-break `porf` CLI.
- `porffor-test262`: conformance taxonomy and harness rewrite support.

## Hard Invariants
- Production compile path is `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
- Hidden debug interpreter is allowed only as non-product engineering tool.
- `build wasm` must emit compiled user program semantics and lowered builtins only.
