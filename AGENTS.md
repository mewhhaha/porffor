# AGENTS.md

GRUG GOAL: rewrite Porffor fully in Rust as library and CLI. Keep AOT-first Wasm heart. Porffor must compile JavaScript itself to Wasm; do not ship a JS interpreter or VM compiled to Wasm as the execution strategy. Reach literal 100% ECMAScript and Test262 pass with no spec cheats, no silent skips, no regressions. Spec truth bigger than clever trick.

## Grug Rules
- Grug build real JS -> Wasm compiler path. User program semantics go through parse, early errors, spec IR, lowering IR, and real Wasm codegen.
- Grug may keep tiny interpreter only as hidden debug and differential tool. Grug must not ship it as product path, CLI runtime path, or Wasm artifact path.
- Grug prefer correctness before speed. If clever thing and spec thing fight, grug pick spec thing.
- Grug keep Rust library and CLI as main future surface. Old JS code is reference and oracle until Rust path wins.
- Grug treat permanent skip lists and silent expected failures as bad cave smoke. Every conformance failure needs owner and reason.

## Rewrite Ground
- Rust workspace lives under `crates/`.
- `crates/porffor-engine` is public library face.
- `crates/porffor-cli` is clean-break `porf` face.
- `crates/porffor-test262` owns conformance taxonomy and harness rewrite.

## Big No
- No “compile JS interpreter to Wasm and then feed source into it.”
- No product shortcut that makes `build wasm` emit evaluator blob instead of compiled user program.
