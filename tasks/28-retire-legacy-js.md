# T28 — Retire the legacy JavaScript implementation

**Status:** Complete as of 2026-08-11

**Parallel group:** Repository ownership and cleanup
**Depends on:** T00
**Blocks:** T29 and any claim that the repository is Rust-only

## Current repository state

The legacy JavaScript Porffor product implementation was retired from the
working tree. Its final recovery point is Git commit
`2107dfe9ad58c730e3d19b0cc1c73ed4390602f8`; Git history is the sole archive.
The supported development and product surfaces are the Rust workspace and its
`lila` binary.

JavaScript remains only where it is data owned by the Rust project: pinned
Test262 content, embedded Test262 harness assets, Rust test fixtures, focused
reproducers, and vendored dependencies. None of those files may become a
product compiler, runtime, package entrypoint, or source evaluator.

## Objective

Remove the obsolete JavaScript compiler/runtime and every repository surface
that built, packaged, published, tested, benchmarked, or presented it as the
product. Make that boundary permanent and leave one honest Rust-first project.

## Removed surfaces

- Product implementation and tools: `compiler/`, `runtime/`, `byg/`, `fuzz/`,
  and `bench/`.
- Package and publication files: `package.json`, `jsr.json`, `publish.js`,
  `.npmignore`, `.github/workflows/publish.yml`, `porf`, and `porf.cmd`.
- Obsolete Test262 tooling: `test262/compare.js`, `test262/fails.cjs`,
  `test262/generateHistoricalData.js`, `test262/index.js`,
  `test262/missingHarness.js`, `test262/read.js`, and
  `test262/history.json`.
- The legacy playground implementation and `logo.png`; `index.html` is now a
  dependency-free Lila project page and `CNAME` continues to serve
  `porffor.dev` during the identity transition.
- Accidentally tracked nested Cargo build artifacts under
  `crates/porffor-cli/target/`.

The local Test262 overlays formerly stored at `test262/harness.js` and
`test262/harness-wasm-aot.js` moved into embedded assets owned by
`lila-test262`; installed binaries no longer infer a repository-root harness
path.

## Repository contract

- Do not reintroduce a JavaScript product compiler or runtime.
- Do not add npm, JSR, Node, Deno, Bun, or another JavaScript publication or
  product-entrypoint workflow.
- Do not build emitted Wasm around a JavaScript interpreter, VM, parser, or
  source evaluator.
- Keep allowed JavaScript subordinate to Rust-owned testing, conformance, or
  vendoring boundaries.
- Recover historical implementation details from the recorded commit instead
  of copying the retired trees back into the main branch.

## Out of scope

- The later coordinated clean-break rename of the Rust crates, binary,
  environment variables, persisted namespaces and diagnostics. T29 owns that
  cutover and its remaining persisted-snapshot verification.
- Deleting the feature-gated Rust `lila-spec-exec` differential oracle or
  vendored Boa crates.
- Publishing replacement packages or native releases.
- Changing ECMAScript semantics or generated Test262 status counts.

## Acceptance criteria

- Every listed legacy surface is absent from the tracked working tree.
- Test262 local overlays are embedded Rust-crate assets with an explicit custom
  file option and no checkout-layout dependency.
- The Node discovery oracle and its CLI/API surfaces are absent.
- CI has no JavaScript toolchain or package-publication path and validates the
  Rust product on pull requests and `main`.
- A repository check rejects reintroduction of the retired product surfaces
  without flagging Test262, fixtures, reproducers, or vendored JavaScript.
- README, contribution guidance, task status, Cargo metadata, and the website
  consistently describe Lila as a Rust-only product.
- The generated README Test262 status block is byte-for-byte unchanged.

## Required tests

```sh
git diff --check
cargo fmt --all -- --check
./scripts/check-no-legacy-js.sh
./scripts/check-task-plan.sh
cargo xc
cargo test -p lila-test262 --lib load_preludes -- --test-threads=2
cargo test -p lila-test262 --lib wasm_agents_run_test262_wait_until_with_exact_assertions -- --test-threads=2
cargo test -p lila-cli parse_test262_args -- --test-threads=2
cargo tree -p lila-cli
```

Also run both fake Test262 suites through the Rust binary and verify that
`lila --help` exposes no Node-oracle command.
