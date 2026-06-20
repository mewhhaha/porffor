# AGENTS.md

## Project Direction

- Porffor is a greenfield Rust rewrite until 1.0. Optimize for the future Rust library, CLI, compiler architecture, and conformance story.
- Do not preserve legacy internals, APIs, file layouts, or behavior merely for compatibility. Old JavaScript code can be a reference and oracle, but it is not a constraint.
- Breaking changes are acceptable and expected before 1.0 when they move the project toward the correct compiler architecture or better ECMAScript/Test262 conformance.
- Prefer deleting or replacing legacy-shaped code over layering compatibility shims, unless a shim is the smallest temporary step toward the Rust compiler path.

## Core Goal

- Rewrite Porffor fully in Rust as a library and CLI.
- Keep the AOT-first Wasm compiler at the center of the project.
- Porffor must compile JavaScript itself to Wasm. It must not ship a JavaScript interpreter or VM compiled to Wasm as the execution strategy.
- The target is literal 100% ECMAScript and Test262 conformance, with no spec cheats, no silent skips, and no regressions.

## Compiler Contract

- Build the real `JS -> Wasm` compiler path.
- User program semantics must go through parsing, early errors, spec IR, lowering IR, and real Wasm codegen.
- A tiny interpreter is allowed only as a hidden debug or differential-testing tool.
- The interpreter must not be used as the product path, CLI runtime path, or emitted Wasm artifact path.
- Dynamic source evaluation features such as `eval`, `new Function`, and cross-realm `Function` constructors do not need AOT support when support would require bundling a parser, interpreter, or VM into emitted Wasm. Track these as explicit Wasm-AOT unsupported dynamic-code-generation cases, not silent skips.

## Correctness Rules

- Spec correctness comes before speed, cleverness, or legacy compatibility.
- Every conformance failure needs an owner and a reason.
- Permanent skip lists and silent expected failures are not acceptable.
- Never call fake-suite green "100% ECMAScript" or "100% Test262". Fake subset truth and full pinned Test262 truth must stay separate.

## README And Status

- Keep `README.md` current when work changes user-visible capabilities, conformance, CLI behavior, architecture, or development workflow.
- If fake suite counts, wasm-safe subset counts, pinned real Test262 status, or major green/red milestones change, update the README status block in the same patch.
- The README status block must include refresh commands, exact counts, and the refresh date when changed.
- Use `./target/debug/porf test262 publish-status --execution-backend <spec-exec|wasm-aot>` or equivalent `cargo run -p porffor-cli -- test262 publish-status ...` to refresh pinned real-suite artifacts and the README block. Do not hand-edit status numbers.
- For low-RAM real-suite refreshes, use `./scripts/publish-real-status-low-ram.sh <spec-exec|wasm-aot> <snapshot-name>` so the top-level matrix checkpoints one node per process, then publishes the README only after verified completion.

## Workspace Map

- Rust workspace: `crates/`
- Public library face: `crates/porffor-engine`
- Clean-break CLI face: `crates/porffor-cli` and the `porf` command
- Conformance taxonomy and harness rewrite: `crates/porffor-test262`

## Hard Bans

- Do not compile a JavaScript interpreter to Wasm and feed source into it.
- Do not make `build wasm` emit an evaluator blob instead of the compiled user program.
- Do not keep compatibility with legacy Porffor behavior when it conflicts with the Rust AOT compiler direction or ECMAScript correctness.
