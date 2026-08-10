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
| 1c | the whole CLI suite, run as 18 resumable chunks by `scripts/rung1c-chunks.sh` (617 `#[test]` attributes at batch 6 → **609 compiled**, 608 executing; see below) | **~26 min** at `--test-threads=8` on 16 CPUs; ~2.5 h at `--test-threads=3` on 4 CPUs | end-to-end CLI behaviour | conformance beyond the fixture corpus |
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

### Rung 1c terminates, and checks its own expectations

On a machine that can hold the whole suite in one process lifetime, run it
exactly like this. No `--skip`:

```sh
./scripts/run-watched.sh --label b3-cli --stall 900 -- \
  cargo test -p porffor-cli --test cli -- --test-threads=2
```

On a container that restarts hourly, that invocation cannot finish, and the
supported form is `./scripts/rung1c-chunks.sh` — the same suite as 18 resumable
per-module chunks, banked one verdict at a time. It is tracked precisely because
every batch used to re-derive it from a lane note. Its own header carries the
four properties that must not be "simplified", and
`known_failures::rung_1c_chunks_cover_every_cli_area_module` fails if its chunk
set stops partitioning the suite.

Raise `--test-threads` on a machine with spare cores; the suite is CPU-bound and
scales close to linearly. **Never lower it to 1.** Under `--test-threads=1`
libtest runs every test on the thread named `main`, the per-test name that
`known_failures::execution_path` routes on is unavailable, and every test falls
back to spawning a cold `porf` child process instead of the warm in-process call
the ~26 minute estimate is built on. It is correct and terminating, just far
slower. For one test use `-- --exact <name>`, not a lower thread count.

Keep `--stall` at 900 regardless: on a 4-CPU box with a sweep holding two of
them, a single cold Wasm-AOT compile can exceed 300 s of log silence, and the
300 s default then kills a perfectly healthy run with exit code 124. As always,
judge a long run by whether its **log is still growing**, never by elapsed time
against an estimate.

**Do not compare the result against a document.** The expected non-green
outcomes are tracked in `crates/porffor-cli/tests/known-failures.tsv` and the
suite enforces them itself, so a green rung 1c means "exactly the declared
outcomes, for the declared reasons" and a red one means something moved. Seven
kinds of drift are failures rather than notes someone has to remember:

| Drift | How it fails |
|---|---|
| new failure | ordinary red test |
| declared failure starts passing | libtest: `test did not panic as expected` |
| declared failure fails for a different reason | `should_panic` message mismatch |
| declared test renamed or deleted | `cargo xc`: E0425/E0603 on a `const _` line |
| ledger row with no test, or test with no row | `known_failures::*` hygiene tests |
| `#[ignore]` added with no owner | `known_failures::every_ignored_test_is_declared` |
| **hang in an undeclared test** | `porf run exceeded ... in process` after the hang timeout |

That last row is the one the table used to be missing, and its absence was not
cosmetic. `execution_path` routes only *declared* hangs to the guarded
subprocess; every undeclared test takes the in-process path, so a new hang could
never produce the guarded path's "this is a NEW hang" message under the
documented `--test-threads=2` invocation. The in-process path is now bounded by
the same timeout, on a worker thread that is leaked rather than killed — so the
suite terminates whichever path the hang appears on. A `fail`-state row whose
test later starts hanging is covered by the same bound.

Neither bound can distinguish "blocked" from "pathologically slow": the declared
hang's fixture prints nothing before it blocks. The timeout is calibrated (900 s,
the same headroom as `--stall 900`) so that a cold Wasm-AOT compile on a loaded
4-CPU box finishes well inside it; treat a timeout as "hung *or* very slow" and
investigate before adding a row.

`binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture` used to
hang the suite forever near the end of the run, which is why the old invocation carried
`--skip atomics_wait_core` and why rung 1c was never actually a gate. It is now
a declared hang (owner T17): `tests/cli/main.rs` runs it as a real child process
and kills it after the hang timeout, so it terminates as a bounded, expected
failure. The underlying defect is still open — the ledger row carries the lead.

Two naming traps, both of which have cost time:

- **libtest names carry no target prefix.** `cli` is the cargo *target*, so the
  name is `binary_data::run_...`. `cli::binary_data::run_...`, which this
  document used to print, matches nothing as a filter.
- **`--skip` and filters are substring matches**, not exact names. Prefer
  `-- --exact <name>` when you mean one test.

#### Adding a row

1. Get the names: `sed -n '/^failures:$/,/^test result:/p' target/watched/b3-cli.log`.
2. Get the message per failure: `grep -n '^---- .* stdout ----' -A6 target/watched/b3-cli.log`.
3. Add a ledger row (target, test, state, owner from
   `test262/backlog/ownership-map.tsv`, reason, evidence).
4. Add `#[should_panic(expected = "<stable substring of that message>")]` and
   `pub(crate)` to the test, and a `const _: fn() = crate::<module>::<name>;`
   line in `tests/cli/known_failures.rs`.

   **The attribute must stay on one physical line and use the exact
   `expected = "..."` spelling.** `known_failures::scan_source` asserts that any
   line starting with `#[` also ends with `]`, and the `should_panic` parser
   accepts only that spelling. A wrapped or `\`-continued attribute fails the
   hygiene tests with a pointed message — and the `\`-continued form is already
   idiomatic in this tree for long `#[ignore = "..."]` reasons
   (`crates/porffor-aot-wasm/src/planning.rs`), so a contributor following local
   convention will trip it. Shorten the substring instead of wrapping the line.
5. Re-run until green.

A bare `#[should_panic]`, or an empty `expected`, is rejected by the hygiene
tests: it passes on any panic and would turn the next genuine defect in that
test green.

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

### Before adding any tracked data file, run `git check-ignore -v`

`.gitignore` line 3 is a bare `*.txt`. It is not scoped to a directory, so it
swallows any `.txt` anywhere in the tree, `git add -A` reports nothing, and the
file simply never exists for anyone else.

```sh
git check-ignore -v <path>   # exit 1 = tracked. exit 0 prints the rule eating it.
```

This has already cost this repository two files. `benchmarks/wasm-aot-20.txt` is
machine-local for exactly this reason (see the comment in
`crates/porffor-cli/tests/perf.rs`), and an earlier
`crates/porffor-cli/tests/known-failures.txt` was silently dropped while this
document and `README.md` went on citing it for three batches — nobody noticed,
because the suite that would have used it could not be run.

Use `.tsv` for hand-maintained tables; that is already the convention
(`test262/backlog/ownership-map.tsv`, `test262/backlog/shortcut-allowlist.tsv`,
`crates/porffor-cli/tests/known-failures.tsv`). Do **not** "fix" this with a `!`
negation in `.gitignore`: `*.txt` has real users, and the next such file walks
into the same trap. Better still, give the file a consumer that fails without it
— `known-failures.tsv` is an `include_str!`, so its absence is a compile error.

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
assignment, or property installation order can leave every CLI test green
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
- `crates/porffor-cli/tests/cli.rs` split into `tests/cli/` — **617** `#[test]`
  attributes (recounted by this session at the head of batch 6) across the 18
  area modules plus the `known_failures.rs` hygiene module, so feature lanes no
  longer all append to one 10,709-line file. 8 of them sit behind
  `#[cfg(feature = "spec-exec-oracle")]` in `frontend.rs`, so **609 compile**
  under default features — that is the number every chunk's
  `ran + filtered_out` must sum to — and **608 actually run**, because one of
  the 609 is `#[ignore]`d (in `heap.rs`). Ignored is not the same as not
  compiled: `--list` counts the ignored test, and the 8-test gap between 617 and
  609 is the `cfg` gates alone.
  This number moves every batch (593 at batch 3, 607 at batch 5, 617 now), so
  **recount it rather than citing this line.** Use the **exact-line** form — the
  same one the hygiene scanner itself uses (`known_failures.rs`), not a
  substring grep — and settle the compiled/executing split with `--list`, the
  only form that resolves `cfg`:

  ```sh
  awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++} END{print n}' \
    crates/porffor-cli/tests/cli/*.rs
  cargo test -p porffor-cli --test cli -- --list | tail -1
  ```

  This line has been wrong four times. `grep -h '#\[test\]' … | wc -l` — which
  this document used to print as the recount recipe — is a *substring* match and
  currently returns **619** against the true 617, because it also matches prose
  lines inside `known_failures.rs` that name the attribute. Do not trust either
  the number or a substring recount; run the `awk` form.

- **`crates/porffor-cli/tests/known-failures.tsv`** — the tracked ledger of
  expected non-green outcomes for this crate's three test targets, enforced by
  `tests/cli/known_failures.rs` at compile time (file existence via
  `include_str!`, test existence via `const _`) and by libtest at run time
  (`should_panic` with a required non-empty `expected`). This is what makes
  rung 1c a gate instead of a reading exercise.

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
