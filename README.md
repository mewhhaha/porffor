# Lila

Lila is a Rust JavaScript-to-Wasm AOT compiler, library, CLI, and Test262
harness. It is a research project, with full ECMAScript conformance as its
goal. The product compiles JavaScript through parsing, early errors, spec IR,
lowering IR and Wasm code generation. It does not ship an interpreter inside
the emitted Wasm.

Start with the [failure backlog](tasks/README.md),
[architecture and compiler contracts](docs/rust-rewrite/README.md), or
[contributing guide](CONTRIBUTING.md). The project rules are in
[AGENTS.md](AGENTS.md).

## Current status

<!-- lila-status:start -->
Rust rewrite status must be read in layers, not one vanity number:
- Fake wasm-safe Test262 subset: `187/187` green
- Fake full Rust rewrite suite: `191/191` green
- Pinned real Test262 baseline (`wasm-aot`, refreshed `2026-09-28`): `99229/102956` not green (`96.4%`)
- Real Test262 goal: Success=99229/102956 (96.4%); burn down NotImplemented=442, Crash=37, Bug=3248 to zero
- Pinned revisions: `ecma262=ecma262-current-draft` `test262=91b2052adad1f066ae031e2ff3a1e9bd6d732886`
- Current real outcomes: `Success=99229`, `NotImplemented=442`, `Crash=37`, `Bug=3248`
- Biggest current real failing kinds: `Runtime=3454`, `Unsupported=262`, `Parser=9`
- Biggest current real failing origins: `unknown=3709`, `boa-parser=11`, `boa-runtime=7`
- Worst current real matrix targets: `intl402/DurationFormat: 0/220 passed`, `intl402/Temporal/PlainDateTime: 72/250 passed`, `intl402/Temporal/ZonedDateTime: 74/250 passed`
- Published status artifacts: `/home/mewhhaha/src/porffor/.claude/worktrees/t262-upstream/target/test262-scratch/full-native-gc-20260927/snapshots/full-native-gc-20260927-aggregate-12471854949206383352.json` and `/home/mewhhaha/src/porffor/.claude/worktrees/t262-upstream/target/test262-scratch/full-native-gc-20260927/snapshots/full-native-gc-20260927-aggregate-12471854949206383352.txt`

As of `2026-09-28`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until full pinned real Test262 run is green for Rust path.

Status refresh commands:
- `cargo test -p lila-engine --quiet`
- `cargo test -p lila-cli --quiet`
- `./target/debug/lila test262 run language/wasm/pass --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`
- `./target/debug/lila test262 run --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262`
- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`

When counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.
<!-- lila-status:end -->

The completed September 28 run covers all 748 matrix sections. The percentage
above is the pass rate, not progress through the suite. The
[new backlog](tasks/README.md) registers every failed execution with its
observed diagnostic, owning compiler area, root-cause evidence, and exact
replay list. Confirmed causes are distinguished from hypotheses that need a
reproducer. The frozen baseline remains unchanged as individual tasks close;
a new complete published run is required to change the conformance totals.

Fake fixtures check the runner and selected compiler behavior. They do not
establish full ECMAScript conformance. Missing features, runtime errors,
crashes and timeouts remain non-passing outcomes. Runtime source generation
that requires bundling a parser or interpreter is an explicit Wasm-AOT
boundary and remains visible in the failure denominator.

## Build and run

Use the toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml).
Build flags and the default Cargo job cap live in
[.cargo/config.toml](.cargo/config.toml).

```sh
./scripts/dev.sh build
./target/debug/lila --help
./target/debug/lila inspect crates/lila-cli/tests/fixtures/hello.js
./target/debug/lila run crates/lila-cli/tests/fixtures/hello.js
./target/debug/lila build wasm crates/lila-cli/tests/fixtures/hello.js
```

Wasm-AOT is the default product backend. Its runtime assumes experimental
Wasmtime features, including Wasm GC, reference types, typed function
references and exception handling. An engine missing required capabilities
must fail clearly. The current runtime and heap boundary are documented in
[value-heap-gc.md](docs/rust-rewrite/value-heap-gc.md).

The feature-gated `spec-exec` backend is a differential/debug oracle. Its
results cannot be published as product conformance. C/native backend commands
and the REPL are scaffolds.

`lila types` (alias `typegen`) generates Worker-style TypeScript declarations
from a selected entrypoint and JSON, JSONC or TOML configuration:

```sh
./target/debug/lila types src/index.ts worker-configuration.d.ts --config wrangler.jsonc
```

## Workspace

| Crate | Responsibility |
|---|---|
| `lila-front` | Parser boundary and source units |
| `lila-ir` | Spec IR, early errors, lowering and diagnostics |
| `lila-aot-wasm` | Direct Wasm code generation and runtime helpers |
| `lila-runtime` | Realms and typed host capabilities |
| `lila-intl` | Pinned Intl data, provider protocols and host operations |
| `lila-engine` | Public library API and Wasmtime execution |
| `lila-cli` | The `lila` command |
| `lila-test262` | Discovery, execution identity, checkpoints and conformance reports |
| `lila-spec-exec` | Explicitly selected debug/differential oracle |

Retained design documents describe current architecture and enforceable
contracts. Old batch plans and progress diaries live in Git history. The
retired JavaScript implementation is available at recovery commit
`2107dfe9ad58c730e3d19b0cc1c73ed4390602f8`; it is not a development surface.

## Work on a failure

Pick a task in [tasks/README.md](tasks/README.md), claim it, and replay its exact
execution list before changing semantics. Each task records the source of its
root-cause assessment and the validation needed to close it. Tasks include
all modes of a failure separately; a physical test file may run as both a
sloppy Script and a strict Script.

```sh
./scripts/check-task-plan.sh
./scripts/check-module-boundaries.sh
cargo check --workspace --all-targets
```

Batch related implementation before expensive verification. Run focused
regressions, adjacent families, then the appropriate broader suite. The
[batch workflow](docs/rust-rewrite/batch-workflow.md) describes the verification
ladder, golden byte comparisons, concurrency limits and checkpoint recovery.
Compiler-enforced invariants are preferred to repeated runtime checks or
tests that only duplicate implementation details.

For the CLI suite, use the stall guard or resumable area chunks:

```sh
./scripts/run-watched.sh --label cli --stall 900 -- \
  cargo test -p lila-cli --test cli -- --test-threads=2
./scripts/rung1c-chunks.sh
```

The CLI known-failure ledger checks expected outcomes and reasons itself.
A newly passing declared failure, a changed failure reason, an unowned ignore,
or a new failure makes the check fail. Do not add silent skips.

## Test262 runs and publication

A focused run keeps scratch evidence outside the published snapshot directory:

```sh
./target/debug/lila --jobs 1 test262 run built-ins/Array/from \
  --execution-backend wasm-aot --threads 2 \
  --snapshot-dir target/test262-scratch/array-from --snapshot-name array-from
```

For a complete resumable publication, build the release CLI and use the
publication wrapper. It freezes observed source, binary, suite and environment
identity; changing those inputs requires a new snapshot family. Only a
verified complete matrix can publish the README status.

```sh
cargo build --release --locked -p lila-cli
./scripts/publish-real-status-low-ram.sh wasm-aot current-baseline
./target/release/lila test262 progress-status --snapshot-name current-baseline
```

`THREADS`, `JOBS`, and `MAX_MATRIX_NODES` tune the wrapper; case isolation is
on by default. `--threads` controls cases and `--jobs` controls compilation.
Neither alone enforces total resource usage: use an inherited CPU affinity
and an OS memory limit for long runs. The completed baseline used 10 CPUs,
one compiler job per case, and half the machine's RAM with swap disabled.

Use [publication safeguards](docs/rust-rewrite/reproducible-publication-driver.md)
and [snapshot comparison](docs/rust-rewrite/test262-snapshot-comparison.md)
when resuming or comparing runs. Published counts must come from
`lila test262 publish-status`; do not edit the status block by hand.

## Caches and diagnosis

`lila cache status` reports the bounded program-Wasm, native-module and
Cranelift-function caches. `lila cache prune` removes Lila-owned entries;
legacy Wasmtime cache removal requires `--legacy-wasmtime` explicitly.
`LILA_CACHE_DIR` relocates the cache. The per-tier budgets are
`LILA_FUNCTION_CACHE_LIMIT_BYTES`, `LILA_MODULE_CACHE_LIMIT_BYTES` and
`LILA_PROGRAM_CACHE_LIMIT_BYTES`. `LILA_MODULE_MEMORY_CACHE_ENTRIES` bounds the
in-process compiled-module entry count; execution state is never shared.

`LILA_WASM_TRACE=1` reports compiler and runtime stages and cache decisions.
`LILA_WASM_TRACE_DUMP=1` also emits the backend debug dump. Source-dependent
program and module caches help repeated reproducers; a broad suite benefits
more from the shared function cache. See the batch workflow for measured
cache tuning and the golden emission check for pure refactors.
