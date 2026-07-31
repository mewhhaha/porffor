# Batch workflow: write concurrently, verify once, triage afterwards

This describes how to run a batch of compiler work across several agents without
serialising on compilation and without agents conflicting in the shared backend
files. It is the operational form of the workflow sketched in `AGENTS.md`
("batch implementation before expensive verification").

Everything here is measured on the 16-logical-CPU / 93 GiB development machine
unless marked as an estimate.

## Why this shape

Three facts drive the design, and each was measured rather than assumed:

1. **rustc is not the bottleneck.** An incremental engine/CLI rebuild is
   `1.04 s`; a comment-only rebuild in the 48,608-line `builtins/standard.rs` is
   `4.42 s`. A *cold single Test262 case* is `13.73 s`, of which `11.73 s` is
   native Cranelift compilation. The cost is in compiling emitted Wasm, times
   53,131 cases — not in compiling Rust.
2. **The real-suite sweep is a ~15 hour job.** Calibrated at `1.0 s` per case
   with `--threads 8 --jobs 8`. It cannot sit in any inner loop.
3. **The shared backend files are the concurrency limit, not the machine.**
   Adding one builtin touches ~14 places across 6 files. Until the T02 split
   lands, two builtin lanes conflict no matter how they are isolated.

So: agents write in parallel and do not compile; one integrator compiles once;
verification climbs a ladder whose expensive rungs are entered deliberately.

## The ladder

Pick the cheapest rung that can answer the question in front of you.

| Rung | Command | Cost | Catches | Does not catch |
|---|---|---|---|---|
| 0 | `cargo check -p <crate>` / `cargo xc` | 1–5 s / 15–40 s | types, borrows, missing match arms | anything semantic |
| 1 | `cargo test -p porffor-ir`, focused `-p porffor-engine <filter>` | 30 s–3 min | lowering/IR/engine semantics | real-harness shapes (`$262`, `propertyHelper`, async `$DONE`) |
| 1b | one CLI area module, e.g. `--test cli array::` | 1–3 min | that area's end-to-end CLI behaviour | every other area |
| 1c | the whole CLI suite (581 tests) | **~26 min** | end-to-end CLI behaviour | conformance beyond the fixture corpus |
| G | golden capture + `diff -r` (see below) | ~10 min each side | **any** change in emitted bytes | nothing, for refactors — this is the refactor gate |
| 2 | fake fixture suite (190 cases) | 10–60 s warm | the runner itself | conformance; it is green by construction |
| 3 | `shard 1/25` on the real suite | est. 15 min–3 h | broad cross-subtree regressions | families smaller than ~25 cases |
| 4 | the lane's own ownership-map prefix | 2–40 min | the lane's own family | anything outside the lane |
| 5 | `report-all --resume`, 498 nodes | ~15 h | everything | nothing; too slow to iterate on |

Rungs 0–2, 1b/1c and G are measured. Rungs 3–5 derive from the `1.0 s`/case
calibration; rung 3 in particular has never been run end to end.

Rung 1c is an integration checkpoint, not an inner-loop command. A lane should
run rung 1b for its own area — the per-test cost varies by more than 1.7× across
modules (`heap` is `1.5 s`/test, the whole-suite mix is `2.6 s`/test), so do not
extrapolate one module's cost to the suite.

### The CLI suite does not terminate on its own

`cli::binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture`
**hangs indefinitely**: the suite reaches 580 of 581 and then spins. Always skip
it, and expect the failures already recorded in
`crates/porffor-cli/tests/known-failures.txt` — a lane compares against that
list, not against zero.

```sh
./scripts/run-watched.sh --label cli --stall 420 -- \
  cargo test -p porffor-cli --test cli -- --test-threads=8 --skip atomics_wait_core
```

Tracked as a defect under T17; the skip is a workaround, not an accepted
exclusion.

### Always run long commands under the stall guard

Two failure modes here are silent, and both have cost hours:

- **Work that hangs.** Wasm-AOT compilation has no wall-clock bound (the
  `--timeout-ms` check is skipped for that backend) and `Atomics.wait` blocks
  outright. A hung run looks exactly like a slow one.
- **Buffered output.** Piping a long run into `tail`/`head` hides all progress
  until exit, so "no output" becomes indistinguishable from "still working".

`scripts/run-watched.sh` closes both: output always lands in `target/watched/`,
the log is polled for growth, a healthy run emits a heartbeat, and a run whose
log goes quiet for `--stall` seconds is killed and reported with exit code 124.

```sh
./scripts/run-watched.sh --label sweep --stall 900 -- \
  ./target/release/porf test262 report-all --resume --threads 8 --jobs 8
```

Judge a long run by whether its **log is still growing**, never by elapsed time
against an estimate.

### Rung G — the refactor gate

`crates/porffor-aot-wasm/tests/emit_golden.rs` runs the real
`parse -> lower -> emit` pipeline over all 527 CLI fixtures and records emitted
byte length, a content hash, and the backend `debug_dump` per fixture. It is
inert unless `PORFFOR_GOLDEN_OUT` is set.

```sh
git stash
PORFFOR_GOLDEN_OUT=$PWD/target/golden/before cargo test -p porffor-aot-wasm --test emit_golden
git stash pop
PORFFOR_GOLDEN_OUT=$PWD/target/golden/after cargo test -p porffor-aot-wasm --test emit_golden
diff -r target/golden/before target/golden/after
```

Keep captures under `target/` (gitignored), never `/tmp`. Each side costs ten
minutes and is useless alone; a `/tmp` cleaner reaping the baseline part-way
through a refactor means paying for it twice.

When the refactor is spread across new files, `git stash` alone will not park it
— untracked files are not stashed. Move them aside explicitly, capture the
baseline, then restore:

```sh
mv crates/porffor-aot-wasm/src/intrinsics target/golden/intrinsics-parked
git stash push -- crates/porffor-aot-wasm/src/builtins/bootstrap.rs crates/porffor-aot-wasm/src/lib.rs
# ... capture baseline ...
git stash pop && mv target/golden/intrinsics-parked crates/porffor-aot-wasm/src/intrinsics
```

Empty diff means byte identity. This exists because the ordinary suites assert
on program *output*, so a refactor that perturbs emission order, function index
assignment, or property installation order can leave all 581 CLI tests green
while changing the emitted module. Two independent runs were verified
byte-identical, so a non-empty diff is signal, not noise.

Use it for **every** pure refactor of `porffor-ir` or `porffor-aot-wasm`. Do not
use it for feature work — a feature is *supposed* to change the bytes.

## Running a batch

### 1. Prepare (integrator)

Capture the pre-batch gate state so regressions are detectable later.

```sh
git rev-parse --short HEAD
PORFFOR_GOLDEN_OUT=/tmp/golden-pre cargo test -p porffor-aot-wasm --test emit_golden
```

### 2. Write (N lanes, no builds)

One lane per `owner_task_id` from `test262/backlog/ownership-map.tsv` — 72
prefix-to-task rows, disjoint by construction. Lanes share one checkout and work
on disjoint files. **Lanes run no cargo commands at all.**

Because lanes get no compile feedback, two rules are load-bearing:

- **Follow the established shape.** A lane adding a builtin should copy the
  structure of an existing one end to end rather than inventing a variation. The
  first lane of any new kind is a pilot run by an agent *with* build access; the
  rest copy the pilot.
- **Integrate lanes one at a time** (next step). Errors then attach to a lane
  instead of arriving as one undifferentiated wall.

### 3. Integrate (integrator, one lane at a time)

```sh
cargo check -p porffor-aot-wasm      # after each lane's files land
cargo xc                              # once all lanes are in
cargo test --workspace --quiet
./target/debug/porf test262 run --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm
```

### 4. Gate (integrator)

```sh
./target/release/porf test262 shard 1/25 \
  --snapshot-name gate-$(git rev-parse --short HEAD)-post \
  --snapshot-dir target/test262-scratch/gate \
  --threads 8 --jobs 8 --resume
```

Three sharp edges, all verified in the source:

- **Shard indices are 1-based.** `shard 0/25` silently means shard 1.
- **Each shard needs its own `--snapshot-name`.** Snapshots key on the
  whole-manifest hash, which is identical across shards, and resume state is not
  filtered by the shard's case set — two shards under one name cross-contaminate.
- **`compare-snapshots` cannot diff shard runs.** It requires a complete
  498-node aggregate on both sides. Until a `compare-run-snapshots` equivalent
  exists, rung 3 gives a pass count with nothing to compare it against; treat it
  as a smoke test, not a regression gate.

Keep every non-authoritative snapshot under `target/test262-scratch/` (gitignored).
`test262/snapshots/` is already 423 MB across 82,717 files of lane debris.

### 5. Triage and fan out

`generate-backlog` and `publish-status` require a complete aggregate. Until one
exists, use the commands that tolerate a partial sweep:

```sh
./target/release/porf test262 triage-status   --snapshot-name baseline-wasm-aot-aa55200
./target/release/porf test262 failure-details 'built-ins/Array/fromAsync' --snapshot-name baseline-wasm-aot-aa55200
```

`failure-details` is already the right shape for handing to an agent: it groups
by `(detail_hash, outcome, kind, origin)` with representative tests, i.e. one
*failure family*, which is what a fix lane should own. Prioritise
`Crash > Bug > NotImplemented` — but note that **timeouts are currently folded
into `Crash`**, so a crash-ranked list promotes slow-but-correct cases above
genuine defects until that is separated.

Each fix lane verifies with its own prefix at rung 4. `porf test262 run <prefix>`
exits non-zero unless `passed == total`, which is exactly the "start from a
failing filter, end with it green" rule.

## The baseline sweep

```sh
setsid env \
  PORFFOR_FUNCTION_CACHE_LIMIT_BYTES=34359738368 \
  PORFFOR_MODULE_CACHE_LIMIT_BYTES=536870912 \
  PORFFOR_PROGRAM_CACHE_LIMIT_BYTES=536870912 \
  ./target/release/porf test262 report-all \
    --snapshot-name baseline-wasm-aot-$(git rev-parse --short HEAD:test262/vendor/test262) \
    --threads 8 --jobs 8 --resume \
  > target/test262-scratch/baseline.log 2>&1 < /dev/null &
disown
```

- **Use `setsid` and `disown`.** A plain `nohup ... &` launched from an agent
  session was killed at session teardown after 18 of 498 nodes.
- The aggregate is rewritten after every node and cases checkpoint every 10, so
  a kill loses at most 9 cases. Re-running the identical command resumes.
- Poll from another shell with
  `porf test262 progress-status --snapshot-name <name>`.
- Watch for silence: **Wasm-AOT compiles have no wall-clock bound** (the
  `--timeout-ms` check is skipped for that backend), so a pathological compile
  stalls the sweep with no diagnostic. If the log stops growing for >15 minutes,
  suspect a stuck case rather than slow progress.
- Do **not** use `scripts/publish-real-status-low-ram.sh` here. It runs one case
  at a time with one compiler job and re-walks all 53,399 suite files per node;
  it exists for genuinely low-RAM machines.

### Cache tuning matters more than it looks

Measured over 300 cases: the program-Wasm tier grows ~`9 MiB` per case and the
Wasmtime module tier ~`17 MiB`, and **both are keyed by source text**. Every
Test262 case is a distinct source, so across a single sweep neither tier ever
serves a hit — they are pure write and prune churn, and holding the full suite
would take on the order of `1.5 TiB`. The Cranelift stencil tier is keyed per
function, so builtin bodies shared by every case are written once and hit
thereafter.

Hence the asymmetric settings above: a large function tier, minimal
program/module tiers. Raising the single `PORFFOR_CACHE_LIMIT_BYTES` knob
instead would be consumed within a few hundred cases.

Re-sweep **per milestone, not per batch** — and mandatorily whenever the
test262 pin moves, since every prior snapshot becomes uncomparable.

## Current state of the enabling work

Landed:

- `.cargo/config.toml` unifies build flags between `scripts/dev.sh` and bare
  `cargo`, ending recurring full-workspace rebuilds.
- Per-tier cache budgets (`PORFFOR_{FUNCTION,MODULE,PROGRAM}_CACHE_LIMIT_BYTES`).
- The golden capture (rung G).
- `crates/porffor-cli/tests/cli.rs` split into `tests/cli/` — 589 tests across 14
  area modules, so feature lanes no longer all append to one 10,709-line file.

- **`intrinsics/<family>.rs`** — the 4,760-line
  `init_builtin_constructor_object` is split into 15 family modules;
  `bootstrap.rs` went 8,080 to 4,117 lines and its dispatch is now one-line
  delegations. Verified byte-identical across all 527 fixtures. Enforced by
  `check-module-boundaries.sh`.

Still open, and each is a prerequisite for genuinely conflict-free lanes (T02):

- **Descriptor table** — collapse the 9 parallel `match self` tables over the
  583-variant `StandardBuiltinId` into one row per builtin. Watch the ordering
  hazards: `all_functions()` order feeds Wasm function indices, `all_globals()`
  is deliberately *not* declaration order and feeds `globalThis` enumeration
  order, and variant order feeds `Ord` for `BTreeSet` iteration. This also
  unblocks replacing the no-op or-patterns still left in `bootstrap.rs` with an
  `is_intrinsic_root()` guard.
- **Flatten `compile_standard_builtin`** — 203 of its 402 arms hold 38,309 lines
  inline. Extract the bodies into family files, leaving one-line delegations;
  keep the match flat and exhaustive so "you forgot to implement this builtin"
  stays a compile error.

Until those land, treat builtin lanes as *coordinated* rather than independent:
one lane at a time in `builtins.rs` and `standard.rs`. Property installation is
no longer a contention point.

### How to run an extraction like this safely

The `intrinsics/` split is the template for the two remaining ones:

1. **Capture a golden baseline first**, under `target/golden/before`.
2. **Pilot one small arm**, verify byte-identity, and only then fan out. Proving
   the shape on 16 lines turns the 4,700-line run into a mechanical repeat.
3. **Dry-run the extraction** before writing. The dry run here caught 7
   interspersed no-op or-patterns that a naive pass would have mangled.
4. **Move bodies verbatim.** Re-bind shared context by destructuring a struct
   into the original identifier names rather than rewriting call sites — that is
   what makes byte-identity a meaningful claim.
5. **Re-run golden and diff.** Empty diff or it did not land.
6. **Add the new boundary to `check-module-boundaries.sh`**, and confirm the
   check fails when a module is removed rather than trusting that it would.
