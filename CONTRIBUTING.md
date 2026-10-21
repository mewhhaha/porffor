# Contributing to Scriptw

Scriptw is a Rust JavaScript-to-Wasm AOT compiler, library, CLI, and conformance
harness. It is a research project with an uncompromising goal: compile
JavaScript directly through spec-shaped compiler stages to Wasm and reach full
pinned Test262 conformance without hiding unsupported or failing cases.

The repository is intentionally Rust-first. The retired JavaScript Porffor
implementation is available through Git history, but it is not a development
surface or an oracle. Do not reintroduce a JavaScript compiler/runtime, npm or
JSR packaging, or a source evaluator inside emitted Wasm.

Before starting, read [AGENTS.md](AGENTS.md), the
[conformance ownership](docs/rust-rewrite/conformance-ownership.md), and the
[batch workflow](docs/rust-rewrite/batch-workflow.md).

## Setup

Install a current stable Rust toolchain with Cargo, clone the canonical
repository, and build the CLI:

```sh
git clone https://github.com/mewhhaha/scriptw.git scriptw
cd scriptw
./scripts/dev.sh build
```

The development wrapper shares Cargo's normal `target/` directory, uses `lld`
when available, and bounds parallelism for this large workspace. Set
`SCRIPTW_JOBS` to request a lower job count.

For sparse checkouts and shared Cargo caches, retain the
[development inputs](docs/rust-rewrite/development-checkout.md), including
`.cargo/config.toml`.

Run the compiler from the built binary:

```sh
./target/debug/scriptw --help
./target/debug/scriptw inspect crates/scriptw-cli/tests/fixtures/hello.js
./target/debug/scriptw run crates/scriptw-cli/tests/fixtures/hello.js
./target/debug/scriptw build wasm crates/scriptw-cli/tests/fixtures/hello.js
```

You can also invoke it through Cargo:

```sh
cargo run -p scriptw-cli -- inspect crates/scriptw-cli/tests/fixtures/hello.js
```

## Workspace architecture

The product pipeline is:

```text
JavaScript source -> parse and early errors -> spec IR -> lowering IR -> Wasm
```

The primary crates are:

- `scriptw-front`: parsing and source units.
- `scriptw-ir`: spec-shaped IR, diagnostics, and lowering metadata.
- `scriptw-runtime`: realms and host hooks.
- `scriptw-aot-wasm`: direct Wasm code generation.
- `scriptw-engine`: public Rust library API.
- `scriptw-cli`: the `scriptw` command.
- `scriptw-test262`: Test262 discovery, execution, snapshots, and reporting.
- `scriptw-spec-exec`: feature-gated differential/debug oracle.

Wasm-AOT is the sole product backend. C/native output and a REPL are outside scope.
`spec-exec` must never become the product default, a silent fallback, or part of
an emitted artifact.

The [maintenance backlog](docs/maintenance-hills.md) lists prioritized cleanup
hills and the evidence required to complete them.

## Choosing and owning work

Every non-trivial change should have a permanent
[conformance owner](docs/rust-rewrite/conformance-ownership.md). Start with a
reproducible failure or missing invariant, identify the smallest coherent
feature batch, and coordinate edits to shared IR, lowering, object-operation,
and Wasm backend files.

Prefer compiler-enforced invariants over repeated runtime checks:

- use enums and newtypes for closed domains;
- make lifecycle and ordering constraints visible in types;
- use exhaustive matches when adding a new case must force every consumer to
  account for it;
- keep mutation local where it materially improves performance;
- avoid speculative abstractions that do not turn a plausible mistake into a
  compile error.

Conformance changes must implement general ECMAScript semantics. Never branch
on a Test262 path, source string, or assertion text to manufacture a result.

## Development workflow

Batch the code, tests, types, and documentation for a coherent change before
running expensive suites. During implementation, use read-only inspection and
cheap checks; compile early only when it resolves an uncertainty or validates a
risky foundation.

Useful commands include:

```sh
./scripts/dev.sh check
cargo fmt --all -- --check
cargo test -p scriptw-engine --quiet
cargo test -p scriptw-cli --quiet
./scripts/conformance_owners.py
./scripts/check-no-interpreter-in-product-graph.sh
```

Run focused regressions before broad suites, then finish with one broad
verification checkpoint so Cargo can reuse build artifacts. Long-running
commands should use the repository stall guard:

```sh
./scripts/run-watched.sh --label cli --stall 900 -- \
  cargo test -p scriptw-cli --test cli -- --test-threads=2
```

See [docs/rust-rewrite/batch-workflow.md](docs/rust-rewrite/batch-workflow.md)
for the measured verification ladder and current coordination hotspots.

## Test262 evidence

Repository fake suites are smoke tests, not ECMAScript conformance evidence.
Only the complete pinned real Test262 suite through Wasm-AOT can establish
product conformance. `Unsupported`, timeout, crash, and bug outcomes are all
non-passing and must remain visible.

For focused work, start with an exact real case or subtree and finish by running
the same command after the fix. Also run adjacent filters that share the
abstract operation or builtin you changed. Do not hand-edit published status
counts; use the status publisher only after a complete verified matrix.

Representative smoke commands are:

```sh
./target/debug/scriptw test262 run language/wasm/pass \
  --suite-root crates/scriptw-test262/tests/fixtures/fake_test262/vendor/test262 \
  --execution-backend wasm

./target/debug/scriptw test262 run \
  --suite-root crates/scriptw-test262/tests/fixtures/fake_test262/vendor/test262
```

Use `./scripts/publish-real-status-low-ram.sh wasm-aot <snapshot-name>` only for
a complete resumable real-suite publication.

## Tests worth keeping

Temporary exploratory harnesses should live outside the permanent suite and be
removed after the behavior is understood. Keep regression tests when they
describe a contract that compiler or library consumers rely on. Prefer focused
fixtures near the owning crate over broad duplicate coverage.

## Pull request evidence

Report:

- the owning task ID;
- the exact baseline command and result;
- the semantic invariant added or corrected;
- the exact post-change commands and results;
- files and modules touched;
- materializations added or removed;
- remaining failures and follow-up task IDs;
- anything not verified.

If a change affects user-visible capabilities, CLI behavior, architecture,
conformance, or workflow, update `README.md` in the same patch. Generated status
numbers may change only through the status publication workflow.

## Resources

- [ECMAScript language specification](https://tc39.es/ecma262/)
- [Test262](https://github.com/tc39/test262)
- [WebAssembly specifications](https://webassembly.github.io/spec/)
- [Wasmtime documentation](https://docs.wasmtime.dev/)
