# AGENTS.md

## Project Direction

- Lila is a greenfield Rust rewrite until 1.0. Optimize for the future Rust library, CLI, compiler architecture, and conformance story.
- Do not preserve legacy internals, APIs, file layouts, or behavior merely for compatibility. Old JavaScript code can be a reference and oracle, but it is not a constraint.
- Breaking changes are acceptable and expected before 1.0 when they move the project toward the correct compiler architecture or better ECMAScript/Test262 conformance.
- Prefer deleting or replacing legacy-shaped code over layering compatibility shims, unless a shim is the smallest temporary step toward the Rust compiler path.

## Core Goal

- Rewrite Lila fully in Rust as a library and CLI.
- Keep the AOT-first Wasm compiler at the center of the project.
- Lila must compile JavaScript itself to Wasm. It must not ship a JavaScript interpreter or VM compiled to Wasm as the execution strategy.
- The target is literal 100% ECMAScript and Test262 conformance, with no spec cheats, no silent skips, and no regressions.

## Compiler Contract

- Build the real `JS -> Wasm` compiler path.
- User program semantics must go through parsing, early errors, spec IR, lowering IR, and real Wasm codegen.
- A tiny interpreter is allowed only as a hidden debug or differential-testing tool.
- The interpreter must not be used as the product path, CLI runtime path, or emitted Wasm artifact path.
- Dynamic source evaluation features such as `eval`, `new Function`, and cross-realm `Function` constructors do not need AOT support when support would require bundling a parser, interpreter, or VM into emitted Wasm. Track these as explicit Wasm-AOT unsupported dynamic-code-generation cases, not silent skips.

## Wasm Runtime Target

- Treat experimental Wasmtime as the lower-bound execution target for backend feature planning unless a task explicitly names a narrower runtime.
- Treat that lower bound as a required backend capability set, not an optional optimization tier. Backend code may assume those features exist by default.
- Design from that lower bound upward: absence of a lower-bound feature such as Wasm GC is a runtime rejection condition, not a requirement to implement a compatibility backend or fallback representation.
- This means Lila may rely on Wasmtime-gated Wasm features when they materially improve the compiler architecture or ECMAScript correctness, including Wasm GC, exception handling with `exnref`, typed function references, reference types, and related GC/reference-heavy infrastructure.
- Do not add complex fallback implementations merely to support runtimes without those lower-bound features. If a backend design is clean with Wasm GC, typed function references, or `exnref`, prefer that design over maintaining a non-GC/non-reference fallback path, and treat non-GC runtimes as outside this backend target unless a task explicitly changes the target.
- Do not build or preserve a second object model, closure representation, exception mechanism, memory layout, or compiler backend solely for engines that lack the experimental Wasmtime lower-bound feature set.
- Treat missing Wasm GC support as an unsupported runtime capability gap for this backend, not as a reason to add a parallel manual heap/object model solely for compatibility.
- Do not spend compiler complexity emulating lower-bound Wasm features that the selected runtime lacks. If GC, reference types, typed function references, `exnref`, or related Wasmtime experimental features are required for the clean design, require that runtime capability and fail clearly when it is absent.
- When the experimental Wasmtime lower bound provides a feature, do not spend implementation complexity preserving support for runtimes that lack it. For example, if GC is required by the chosen object model, the boundary should reject non-GC runtimes instead of adding a second non-GC representation.
- Do not design every Wasm feature around a lowest-common-denominator fallback. For example, if an object model, closure representation, exception path, or reference layout depends on GC, `exnref`, typed function references, or reference types, implement the clean experimental-Wasmtime path and report unsupported runtimes at the boundary.
- Keep those features explicit in code, tests, and docs as Wasmtime experimental/runtime-gated assumptions. Do not imply that wasm3, wasmi, browsers, or every Wasm engine supports the same feature surface.
- Prefer runtime capability checks, feature flags, or clear backend errors over compatibility shims or silent fallbacks when emitted Wasm requires these experimental Wasmtime features.

## Correctness Rules

- Spec correctness comes before speed, cleverness, or legacy compatibility.
- Every conformance failure needs an owner and a reason.
- Permanent skip lists and silent expected failures are not acceptable.
- Never call fake-suite green "100% ECMAScript" or "100% Test262". Fake subset truth and full pinned Test262 truth must stay separate.

## Development Workflow

- Batch implementation before expensive verification. At the start of a task, identify the complete coherent feature batch and the independent chunks within it.
- Implement independent chunks concurrently with subagents. When work is unnecessarily coupled, first create the smallest clean seam that lets the chunks proceed independently; do not add speculative abstractions merely to enable parallel work.
- Finish the code, tests, types, and documentation for the whole batch before running expensive compilation or broad test suites. Do not repeatedly rebuild after every small edit.
- During implementation, prefer read-only inspection and cheap non-compiling checks. Run a focused compile or test early only when its result is needed to resolve an uncertainty, validate a risky foundation, or unblock later code.
- After the batch is written, compile once, run the focused regressions, then run the broad suites sequentially so they reuse build artifacts. Fix all discovered failures, rerun affected focused tests, and finish with one broad verification checkpoint.
- Verification remains mandatory before declaring the work complete. Report exactly what ran and what remains unverified.
- `docs/rust-rewrite/batch-workflow.md` is the operational form of this section: the measured verification ladder, how to run a batch across several lanes, the baseline sweep invocation, and the current list of shared files that still force lanes to coordinate.

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
