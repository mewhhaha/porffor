# Compiler batch and verification workflow

[AGENTS.md](../../AGENTS.md) defines the required workflow: prepare a complete
coherent batch, write independent pieces concurrently, then verify the integrated
change. [Failure tasks](../../tasks/README.md) record current work;
[ownership domains](conformance-ownership.md) define the stable Txx labels used
by compiler diagnostics and generated reports.

## Prepare and coordinate

Start with exact failing execution identities and a source-level invariant.
Distinguish confirmed causes from hypotheses. Preserve the original compiler,
suite pin, selection, snapshot and transcript so a later passing run is a
comparison against reproducible evidence.

Assign disjoint file ownership before starting parallel lanes. Coordinate shared
changes to these surfaces through one integrator:

- `lila-ir/src/ir.rs`, `lowering.rs`, `lib.rs`, and `builtins/catalog.rs`;
- `lila-aot-wasm/src/builtins/mod.rs`, `builtins/standard.rs`, `lib.rs`,
  `objects.rs`, `control_flow.rs`, `planning.rs`, `data.rs`, `emitted_function.rs`,
  `runtime_helpers.rs`, `heap.rs`, and `gc_types.rs`;
- intrinsic/ABI registries, shared engine configuration, and
  `lila-test262/src/lib.rs`;
- generated conformance artifacts and the root README status block.

Prefer the existing family modules and typed ownership boundaries. Create only
the smallest seam needed for independent implementation. A new builtin or ABI
variant can require coordinated registration across several crates; a file
split does not itself make those edits independent.

During implementation use read-only inspection, formatting, and cheap script
checks. Compile early only to resolve a risky foundation or unblock dependent
work. Once the batch is written, compile once, run focused regressions, then
run the required broader checks sequentially to reuse build artifacts. Fix
failures and repeat affected checks; do not restart the whole ladder after
every small edit.

## Verification ladder

These historical measurements are planning aids from the 16-CPU, approximately
93-GiB development machine, not deadlines or current test-count claims. Compiler
and suite size, cache state, CPU limits and memory pressure change costs.

| Check | Representative command | Historical cost / purpose |
| --- | --- | --- |
| Cheap checks | `cargo fmt --all -- --check`, task and repository scripts | No compilation |
| Type check | `cargo check -p <crate>`; `cargo xc` | Seconds; types, borrows, exhaustiveness and all-target consumers |
| Focused regressions | `cargo test -p <crate> --test <target>` | Seconds to minutes; changed behavior and adjacent controls |
| CLI area | `cargo test -p lila-cli --test cli <area>:: -- --test-threads=2` | Minutes; area integration |
| Whole CLI | `scripts/rung1c-chunks.sh` | Historically about 26 minutes at eight threads; resumable per-area verdicts |
| Refactor bytes | `emit_golden`, described below | Historically about ten minutes per side |
| Fake Test262 | Run the repository fixture suite | Runner smoke coverage only |
| Exact real selection | `test262 run <path-or-prefix>` or task replay list | Family regressions; require a nonempty selection |
| Full real matrix | `test262 report-all --resume` | Hours; milestone/publication work, not an inner loop |

Inspect libtest's `--list` output when choosing a filter. Names do not include
Cargo's target prefix: `binary_data::run_...` is a test name; `cli::binary_data::`
usually selects nothing. Filters are substring matches; use `--exact` for one
specific test. Source counts and compiled counts differ under feature gates.

For the CLI target keep at least two test threads. Its in-process execution
routing relies on named libtest worker threads; a single-thread invocation can
fall back to cold child processes and be much slower. Set
`LILA_MODULE_MEMORY_CACHE_ENTRIES=1` when retained Wasmtime modules exceed the
available memory. A worker-count limit alone is not a memory limit.

## Long runs and resource limits

Use the stall guard so progress reaches a durable log and a quiet run receives
attention:

```sh
./scripts/run-watched.sh --label cli --stall 900 -- \
  cargo test -p lila-cli --test cli -- --test-threads=2
```

The guard writes under `target/watched/`, emits heartbeats and returns 124 when
its silence threshold is exceeded. Silence can mean a slow cold compilation;
inspect the process and log before classifying a timeout. Do not hide progress
behind `tail` or `head` on a live pipeline.

Respect the user's CPU and RAM budget across all processes, including Cargo,
case runners and compiler pools. Set `CARGO_BUILD_JOBS` or `LILA_JOBS` for Rust
builds, and `--threads` / `--jobs` for Test262. Those knobs can interact, so use
an OS-level CPU/memory limit when a hard cap is required. The last full run used
a ten-core limit and half of physical RAM; those are session limits, not a
requirement to consume that much on every machine.

For a resumable sweep, choose a stable snapshot name and directory, preserve
the compiler identity, and repeat the same selection and options after an
interruption. The runner rejects incompatible checkpoint identities.

```sh
./scripts/run-watched.sh --label test262 --stall 900 -- \
  ./target/release/lila test262 report-all \
    --execution-backend wasm-aot \
    --snapshot-name <name> --snapshot-dir <directory> \
    --threads <case-workers> --jobs <compiler-jobs> --resume
```

For detached runs use a persistent service/scope or a detached session that
survives the launching terminal. Check `test262 progress-status`, the native
aggregate and the growing log. A finished runner process means evidence was
collected; it does not mean every test passed.

Program/module caches are source-keyed and can churn during a full matrix.
The shared function cache is useful across cases. Their independent budgets
are `LILA_FUNCTION_CACHE_LIMIT_BYTES`, `LILA_MODULE_CACHE_LIMIT_BYTES`, and
`LILA_PROGRAM_CACHE_LIMIT_BYTES`; keep temporary evidence under `target/` and
avoid filling the tracked snapshot directory with lane experiments.

## Refactor byte comparison

For a pure IR/backend refactor, capture the same CLI fixture corpus before and
after using `crates/lila-aot-wasm/tests/emit_golden.rs`. It emits each compiled
fixture's byte length, content hash and backend debug dump when
`LILA_GOLDEN_OUT` is set.

```sh
LILA_GOLDEN_OUT="$PWD/target/golden/before" \
  cargo test -p lila-aot-wasm --test emit_golden
# Apply the refactor, then capture the same fixtures:
LILA_GOLDEN_OUT="$PWD/target/golden/after" \
  cargo test -p lila-aot-wasm --test emit_golden
diff -r target/golden/before target/golden/after
```

Keep both captures in `target/golden/` until comparison is complete. Use an
isolated checkout for a historical baseline when other work is already dirty;
preserve the user's changes and untracked files. Byte equality is a refactor
check, not a semantic feature gate: feature changes can intentionally alter
Wasm output.

## CLI failure ledger

`crates/lila-cli/tests/known-failures.tsv` and its `known_failures` tests enforce
expected non-green local regression outcomes. They are separate from the real
Test262 failure backlog. A row records a domain owner, reason and tracked source
evidence. A declared failure must carry a nonempty `should_panic(expected =
"...")`; it fails if the test starts passing or fails for a different reason.
Unowned `ignore` attributes, missing test declarations, missing evidence and
expired placeholders fail the hygiene checks.

Both ordinary in-process tests and guarded child processes have execution
bounds. A timeout is a diagnostic to investigate, not permission to skip a
case. Remove the row and expectation together when a defect is fixed.

The scanner requires attributes on one physical line, with the exact
`expected = "..."` spelling. Keep error substrings specific enough to detect
changed behavior. Use a tracked `.tsv` for evidence tables: the repository's
`*.txt` ignore rule applies everywhere. Check new data with
`git check-ignore -v <path>`; exit 1 means no ignore rule matched.

## Triage and publication

Exact case/mode identities are the unit of failure coverage. A directory or
shared assertion message is not automatically a shared root cause. Each
[Fxxx task](../../tasks/README.md) retains its own case list, cause confidence,
source ownership and completion checks. Keep dynamic code generation gaps
explicit; never convert unsupported, crash or timeout outcomes into passes.

Partial snapshots can be inspected with `triage-status` and `failure-details`.
Only complete compatible aggregates support full status publication. On a
low-memory machine use the resumable driver:

```sh
./scripts/publish-real-status-low-ram.sh wasm-aot <snapshot-name>
```

The driver checkpoints matrix nodes in separate processes and publishes only
after verified completion. See [publication](reproducible-publication-driver.md)
and [restart recovery](publication-progress-recovery.md). Use the Rust publisher
to update canonical status artifacts and the generated README block; do not
copy counts into it by hand. Fake-suite and debug-oracle results never authorize
product conformance claims.
