# Development and verification workflow

[Current inventory](current-inventory.md) owns measured status and open gates.
[Maintenance hills](../maintenance-hills.md) owns the cleanup backlog.
[Earlier workflow and measurements](batch-workflow-history-20261021.md) preserve
all dated procedures and their original source identities. Historical commands,
worker counts and timing estimates are not current defaults.

The [checkout guide](development-checkout.md) lists sparse inputs and cache flags.
Linux CI discovers [all tooling modules](tooling-ci-inventory.md). The AOT workflow
owns debug unit shards and every integration target; main CI owns the complete
release AOT selection. Focused Array/Arguments jobs retain their semantic checks
without repeating the full backend suite. Named shared commands live in
`scripts/ci-targets.json`; `run-ci-target.py --print-command` exposes exact argv.

## Prepare and implement a coherent batch

1. Read [AGENTS.md](../../AGENTS.md), the owning
   [acceptance criteria](conformance-acceptance.md), and existing regression tests.
2. Record the source revision, suite pins, selected target inventory and reusable
   build identities. Use a fresh evidence directory; never restart an unrelated
   or incompatible receipt directory.
3. Identify all edits before expensive compilation. Give shared files one owner.
   Luna agents may explore and review; the primary agent writes implementation.
4. Complete code, tests and documentation, then run cheap checks. Compile early
   only to resolve a specific uncertainty that blocks the remaining work.
5. Compile the batch once, run focused and adjacent checks, then the required
   broader checkpoint. Investigate new failures before moving on.

Current shared coordination points are the workspace manifests, engine facade,
IR algebra/lowering, Wasm emitter and runtime imports, Test262 taxonomy, source
identity generators, and public status publisher. Prefer existing domain modules
before adding another abstraction or crate.

## Environment and resource policy

Use the repository's Rust toolchain and Cargo configuration. `scripts/dev.sh`
and ordinary Cargo share flags; do not inject conflicting RUSTFLAGS. On this
managed cloud machine activate `/workspace/.scriptw-tools/activate.sh` first.
The inherited cgroup memory cap is the budget, with serial defaults:

```sh
python3 -B scripts/limited_verification.py --cloud -- <command>
```

`--cloud-cpus auto` permits quota-bounded native compilation. Cargo and libtest
remain serial; explicit case workers, deadlines and cache limits stay unchanged.
Keep `scriptw-cli:cli` execution under the serial wrapper: its SDK tests explicitly
configure a one-job compilation pool, so a process already initialized by the
four-job opt-in fails those setup checks. Bounded Cargo build CPUs remain usable.
Local machines use the same wrapper without `--cloud` and retain its local
memory/swap policy. Do not copy cloud settings into a local default.

For an idle managed-cloud product build, the existing compiler wrapper can give
one serial Cargo unit the bounded LLVM jobserver as well:

```sh
RUSTC_WRAPPER="$PWD/scripts/test-shared-runtime-images/test-rustc" \
SCRIPTW_TEST_CODEGEN_JOBS=auto \
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  cargo build --locked --offline --release -p scriptw-cli --jobs 1
```

The wrapper preserves compiler queries, flags and product semantics; it does
not enable parallel Cargo or libtest. The native verification driver configures
this itself with `--idle-builds --no-test-incremental`. Merely expanding CPU
affinity leaves LLVM constrained by Cargo's single-slot jobserver: the first
string/storage build took 8m25s at 100% CPU, so do not mistake its scheduling
difference for a compiler-source performance regression.

Set `TMPDIR=/tmp` for tooling tests on this machine: filesystem-admission controls
need separate temporary and workspace filesystems. Report skips explicitly.

## Verification ladder

| Check | Use | Evidence limit |
| --- | --- | --- |
| Diff, formatting, source/identity guards | Every affected batch | No semantic execution |
| Python/shell tooling controls | Runner, generator and workflow changes | Tooling, not product conformance |
| Cargo check | Types and all affected target registrations | No execution |
| Focused crate/area tests | Changed contracts plus adjacent behavior | Selected native cases only |
| Complete affected scopes/workspace checkpoint | Integration and inventory retention | Source-bound native evidence |
| Exact original Test262 cases | Semantic repairs and regression controls | Selected original executions |
| Complete pinned Test262 family | Full measurement and publication | Only its exact compiler and pins |

On this cloud machine, the 2026-10-21 maintenance batch measured 653 tooling
checks at 172.214 seconds and 107 publication/entry-point checks at 63.414
seconds, under the serial cloud wrapper with `TMPDIR=/tmp`. The two additional
source-guard controls are included in the 653-test checkpoint. These are overlapping check groups,
not an aggregate pass count or a compiler benchmark. Evidence is in
`target/maintenance-first-wave-20261021`; the maintenance backlog records the
verified source. The [batch verification record](maintenance-first-wave-20261021.md)
records native/default-compiler checks and distinguishes the original failed
attempts from successful corrections. Historical durations are in the archive. Measure current cold/warm behavior
before claiming speedups; moving functions between files proves none.

Run source-derived identity checks before expensive builds:

```sh
python3 -B scripts/intl_source_identities.py
cargo fmt --all -- --check
python3 -B scripts/limited_verification.py --cloud -- \
  cargo check --locked --workspace --all-targets
```

If source ownership changes, regenerate with `intl_source_identities.py --write`
then check again. Preserve CLDR/IANA semantic data unless the feature batch
explicitly changes it. The recipe list in that script is authoritative.

Engine integration tests use area targets such as `aot_intl`, `aot_builtins`,
`aot_language`, and `aot_realm_modules`. IR integration cases use the
`scriptw_ir_integration` target. Inventory the actual selected names; a zero-test
filtered invocation is not a passing regression check.

For long checks use `scripts/run-watched.sh` with the scope's existing stall
allowance. Native stdout silence is not enough to infer a hang; inspect logs and
resource receipts. Keep failed logs and the original executable identity.

## Complete native verification and artifact reuse

Use `scripts/run_workspace_tests.py --plan-only` to inspect exact Cargo-selected
scopes before execution. `scripts/run_workspace_cohorts.py` bounds linking while
retaining the full target denominator. Its auxiliary library anchor must first
prove a zero-test inventory and never contributes passing tests to suite totals.

The runner isolates Intl's development feature graph automatically. Do not merge
its `compiled_data` development feature into ordinary product dependencies.
`--selected-feature-roots --build-selected-together` narrows focused builds only
after comparing their recursive Cargo graph to the ordinary workspace graph;
unsupported combinations fail before compilation.

Retain the shared immutable runtime image and bounded native caches. The shared
libtest linker changes storage, not executable test ownership or outcomes.
`SCRIPTW_BUILD_MODULE_CACHE_DIR` may isolate the runtime producer's native module
cache from program cache eviction; it does not bypass runtime emission/admission.
Do not reuse an artifact based on its filename or relabel an older receipt.

Continuations must preserve source/artifact identity, features, target inventory,
CPU policy, deadlines and completed outcomes. A passing diagnostic retry does
not replace a failed original run. Saved receipts do not imply live processes.

## Optional sequential CLI chunks

`--partition-cli` still defaults to one chunk per admitted CPU. For a long CLI
scope, `--cli-partition-count 4` separates four chunks from a two-process ceiling:

```sh
python3 -B scripts/limited_verification.py --cloud --cloud-worker-cpus auto -- \
  python3 -B scripts/run_workspace_tests.py --profile release \
    --include-ignored --test-processes 2 --partition-cli --cli-partition-count 4 \
    --only-targets scriptw-cli:cli --output target/cli-four-chunks-fresh
```

Cargo remains serial and each child runs one libtest thread on one assigned CPU.
The driver validates every filtered inventory before any body runs, launches at
most one chunk per CPU and starts the next chunk as soon as that CPU is free.
A failed chunk does not drop later chunks. Receipts bind exact ID/outcome unions,
chunk count, executable hash and CPU pool; resume cannot change chunk count.
The logical target remains idle-exclusive. Original watcher stalls, subprocess
worker/campaign limits and retained-cache ceilings are unchanged.

[Verification evidence](verification-cli-queue-20261022.json) records **74 tooling
checks** and four real filtered inventories whose disjoint union is the exact
**858-test** frozen CLI at `2cb9fc53d`. The real-inventory check ran no test bodies.
The queue integration uses tiny fake libtests with an isolated cloud-launch stub;
separate resource-policy controls check actual admission behavior. Extra chunks
add process and inventory startups; memory/elapsed benefit is **unmeasured**.
Keep the default until an idle, source-bound comparison supports changing it.

## Full Test262 measurement and publication

Use the ordinary product CLI, original pinned suite and a new family identity.
The resumable publication supervisor binds inputs and publishes only a complete,
verified matrix:

```sh
python3 -B scripts/limited_verification.py --cloud -- \
  ./scripts/publish-real-status-low-ram.sh wasm-aot <fresh-snapshot-name>
```

Set `SCRIPTW_BIN`, `SNAPSHOT_DIR` and other documented settings explicitly when
using a different build/output directory. Preserve chosen worker counts and
original deadlines on resume. Inspect `test262 progress-status` and supervisor
receipts for actual progress; a restored cloud instance needs process discovery.

For a baseline sweep without publication, select a verified compiler and a
fresh snapshot directory/name, then run the native matrix directly:

```sh
SCRIPTW_BIN=/absolute/path/to/verified/scriptw
./scripts/run-watched.sh --label baseline --stall 900 -- \
  python3 -B scripts/limited_verification.py --cloud -- \
  "$SCRIPTW_BIN" --jobs 1 test262 report-all --execution-backend wasm-aot \
    --suite-root test262/vendor/test262 \
    --snapshot-dir target/test262-fresh-baseline --snapshot-name fresh-baseline \
    --threads 1 --timeout-ms 60000
```

For an explicit shard, use `test262 shard --shard INDEX/TOTAL [filter]`;
indices are one-based and must lie within the total. A positional `1/2` is now
a literal filter. Options are admitted only by commands that use them: for
example, `--max-matrix-nodes` belongs to `report-all`, `--readme-path` to
`publish-status`, and `--matrix-node` to `run` or `report`. Read-only status
commands reject execution flags such as `--threads` or `--resume`.

For continuation, add `--resume` to the same command only after verifying its
compiler, pins, resources and snapshot identity. Prefer the publication
supervisor above when the goal includes a canonical README refresh.

Only the native publisher may refresh canonical README counts. Targeted checks,
fake-suite results and amended fixtures do not establish a complete original
suite result. The four documented upstream disagreements stay ordinary failures.
Do not repeatedly rerun them as an attempt to turn unchanged expectations green.

## Completion and historical links

Record changed contracts, commands, actual counts, remaining failures and source
identities. Push verified coherent batches to main. Keep CONTINUE/tasks until
all acceptance gates pass; maintenance completion alone does not close them.

The anchors below preserve existing historical links:

<a id="bounded-full-workspace-linking--2026-10-18"></a>
[Bounded linking checkpoint](batch-workflow-history-20261021.md#bounded-full-workspace-linking--2026-10-18).

<a id="fast-source-structure-preflight--2026-10-15"></a>
[Source preflight checkpoint](batch-workflow-history-20261021.md#fast-source-structure-preflight--2026-10-15).

<a id="explicit-cloud-conformance-workers--2026-10-15"></a>
[Cloud conformance workers checkpoint](batch-workflow-history-20261021.md#explicit-cloud-conformance-workers--2026-10-15).

<a id="opt-in-iteration-tooling--2026-10-11"></a>
[Iteration tooling checkpoint](batch-workflow-history-20261021.md#opt-in-iteration-tooling--2026-10-11).


### Cached metadata diagnostics

A metadata-only rustc check against exact cached release externs can catch type
and borrow errors before LLVM/codegen. Treat it as a diagnostic: it does not
produce a product image or establish source/package/native acceptance. The
product build and appropriate native/conformance scopes remain mandatory.
Keep scratch source trees physically separate. Check resolved destination parents
before copying overrides; a symlink to the checkout can invalidate the source
freeze even when no existing tracked file is overwritten.

Preserve completed outputs by digest before staging them into tmpfs, accounting
for the inherited cgroup budget as well as tmpfs capacity. Compare actual
filesystem free space: removing lower-layer originals from an overlay filesystem
can preserve all archive bytes yet reclaim no writable blocks. Before any rebuild,
detach owned output symlinks for the crates/images being regenerated so a producer
cannot overwrite a preserved historical target through its link. Never unlink a
running executable. These operations do not authorize removing source, fixtures,
current dependency inputs or unpreserved verification evidence.
