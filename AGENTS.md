# AGENTS.md

GRUG GOAL: rewrite Porffor fully in Rust as library and CLI. Keep AOT-first Wasm heart. Porffor must compile JavaScript itself to Wasm; do not ship a JS interpreter or VM compiled to Wasm as the execution strategy. Reach literal 100% ECMAScript and Test262 pass with no spec cheats, no silent skips, no regressions. Spec truth bigger than clever trick.

## Grug Rules
- Grug build real JS -> Wasm compiler path. User program semantics go through parse, early errors, spec IR, lowering IR, and real Wasm codegen.
- Grug may keep tiny interpreter only as hidden debug and differential tool. Grug must not ship it as product path, CLI runtime path, or Wasm artifact path.
- Grug prefer correctness before speed. If clever thing and spec thing fight, grug pick spec thing.
- Grug keep Rust library and CLI as main future surface. Old JS code is reference and oracle until Rust path wins.
- Grug treat permanent skip lists and silent expected failures as bad cave smoke. Every conformance failure needs owner and reason.
- Grug keep `README.md` current on conformance. If fake suite count, wasm-safe subset count, pinned real Test262 status, or major green/red milestone changes, same patch updates README status block.
- Grug never call fake-suite green “100% ECMAScript” or “100% Test262”. Fake subset truth and full pinned Test262 truth stay separate.
- README status block must include refresh commands and exact counts/date when changed.
- Use `./target/debug/porf test262 publish-status --execution-backend <spec-exec|wasm-aot>` or equivalent `cargo run -p porffor-cli -- test262 publish-status ...` to refresh pinned real-suite artifact and README block. Do not hand-edit status numbers.
- Low-RAM real-suite refresh path: use `./scripts/publish-real-status-low-ram.sh <spec-exec|wasm-aot> <snapshot-name>` so top-level matrix checkpoints one node per process, then publishes README only after verified completion.

## Rewrite Ground
- Rust workspace lives under `crates/`.
- `crates/porffor-engine` is public library face.
- `crates/porffor-cli` is clean-break `porf` face.
- `crates/porffor-test262` owns conformance taxonomy and harness rewrite.

## Big No
- No “compile JS interpreter to Wasm and then feed source into it.”
- No product shortcut that makes `build wasm` emit evaluator blob instead of compiled user program.
