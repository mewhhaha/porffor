# b4 RUNNER findings (incremental; written after EVERY rung)

Start: 2026-08-10 03:34 UTC. HEAD `677075b9e` ("WIP checkpoint: batch 4 resumed after restart"),
branch `claude/test-driven-rust-opus-pp6giw`. Machine 4 CPU / 15 GiB.

## Rung -1 — inherited state, verified not assumed  [3 facts that change the plan]

1. **There is no `./target/release/porf`.** All three lane `verify_prefix` lines call it. `target/debug/porf`
   exists (149 MB, mtime 02:35) and is what batch 3 measured with. A release build of this workspace is not
   affordable inside a ~1 h container window alongside the sweep; I use the debug binary, as batch 3 did, and
   say so at every number.
2. **The debug binary is STALE.** 7 `crates/**/*.rs` are newer than it, including the three files this batch's
   lanes changed: `builtins/errors.rs` (03:24), `code_sink.rs` (03:19), `builtins/temporal.rs` (03:16), plus
   `planning.rs`, `lowering.rs`, `data.rs`, `intrinsics/temporal.rs`. **Every test262 number produced before a
   rebuild describes batch 3's compiler, not batch 4's.** Rebuild is a prerequisite, not an optimisation.
3. **The background sweep is crash-looping, not progressing.** `target/test262-scratch/sweep-supervisor.sh`
   re-launches `report-all` up to 40 times; `baseline-sweep.log` shows
   `attempt 1 -> exit 134 (fatal runtime error: stack overflow)` at 03:32:05 after reaching 230/250 cases of a
   node, then `attempt 2 -> exit 134` after **86 seconds**, then attempt 3 at 03:33:51. Two consecutive
   stack-overflow aborts, the second one immediate, means the resumed case is deterministically overflowing —
   the supervisor will burn its remaining attempts on the same case. I have NOT killed it (brief says do not),
   but it is consuming ~2 of 4 CPUs to no purpose and every timing below is measured under that load.
   **New finding, owner = next batch's integrator: `report-all --resume` has a poison-case failure mode** —
   a case that aborts the process is retried forever because resume state only advances on completion.

## Rung 0 — HARVEST of prior attempts' banked work (free; read from disk)

### PlainMonthDay full node — batch 3 REMAINING #10, now largely CLOSED
`target/test262-scratch/b3/b3r-pmd-node-8014157493499151608.{json,txt}`, written 02:29 by batch 3's queued
`b3r-chain2.sh` and abandoned when that container died. Verbatim:

```
total=150 pass=150 fail=0
Parser=0 EarlyError=0 Lowering=0 Runtime=0 WasmBackend=0 HostHarness=0 Unsupported=0
outcome_Success=150 outcome_NotImplemented=0 outcome_Crash=0 outcome_Bug=0
```

**150 of the 199 `built-ins/Temporal/PlainMonthDay` cases ran and 150 passed. Zero failures.** The batch-2
baseline for this node was 197/199 with exactly two failures (`from/fields-string.js`,
`prototype/equals/argument-string-invalid.js`), both of which batch 3 fixed and verified individually.
This is a resume checkpoint, so the remaining 49 cases are still unrun — recorded below as remaining with the
snapshot name to resume from (`b3r-pmd-node`).

### batch 3's queued ZDT chain — 12/12 completed, all banked
`target/test262-scratch/b3/b3r-zdt-{1..12}-*.json` all exist. `b3r-chain2.log` shows
`ZDT ALL DONE 23:55:51`. Batch 3's findings already record 10 of these; cases 11 and 12
(`intl402/PlainDateTime/from/calendar-invalid-era.js`, `intl402/PlainDate/from/calendar-not-supporting-eras.js`)
completed and are **0/1 Bug/Runtime** — i.e. **the full 12-case ZDT/era chain is 0 passed / 12**, unchanged
from batch 2. Batch 3's REMAINING #8 stands.

### Not harvestable
`target/lane-notes/b3-runner-auto-results.md` is **0 bytes** — the 60 s harvester died with its container
before writing. `target/watched/b3r-fake.log` is 164 bytes (10 of 190 cases, as batch 3 recorded).

## Rung 1 — `cargo xc`  [GREEN]
`cargo xc` (fresh, under `run-watched`, log `target/watched/b4r-xc.log`): **0 lines matching `^error`**.
Warning totals identical to the fixer's baseline: `porffor-aot-wasm` 26 (lib) / 21 (lib test, all duplicates),
`porffor-ir` 6 (lib) / 5 (lib test), `porffor-test262` 1. No new warnings, no new errors.

## Rung 1a — REBUILD (prerequisite, not a rung)
`cargo build -p porffor-cli` → `Finished dev profile in 1m 12s`. `target/debug/porf` now 03:37, and
`find crates -name '*.rs' -newer target/debug/porf` returns **0**. Every measurement below this line is at
batch-4 code. Everything above it is not.

## Rung 2 — unit gates for all three lanes  [ALL GREEN]
Log `target/watched/b4r-unit.log`, chained by `target/watched/b4r-unit.sh`.

| step | filter | result | target size |
|---|---|---|---|
| A | `-p porffor-engine --lib -- render_wasmtime wasm_backend_characterization` | **2 passed / 0 failed**, 185.55 s | engine lib = 673 tests |
| B | `-p porffor-test262 --lib -- detail_hash` | **4 passed / 0 failed**, 0.00 s | test262 lib = 263 tests |
| C | `-p porffor-aot-wasm --lib -- code_sink::` | **9 passed / 0 failed**, 0.14 s | aot-wasm lib = 267 tests |
| D | `-p porffor-cli --test cli -- throw_propagation::` | **2 passed / 0 failed**, 35.03 s | **cli = 593 tests** |

Notes that matter, each read rather than assumed:

- **Step A included the test the fixer widened the filter for.** `render_wasmtime` alone matches only the new
  pure unit test; `wasm_backend_characterization_matrix_locks_public_surface_and_outcomes` is the assertion that
  can see an inert Half B, and it is in the run and green. The fixer's widening was necessary and is now
  exercised, not merely applied.
- **Step B's four are the Half-A tests by name**, including
  `detail_hash_identity_erases_the_address_but_the_raw_detail_is_preserved` and
  `detail_hash_ignores_the_heap_address_in_a_wasm_aot_throw`.
- **Step D is lane B's headline, and it is the whole point of the lane.**
  `run_wasm_backend_propagates_a_throwing_property_read_out_of_a_loop` and
  `..._out_of_a_switch` both PASS. I read both fixtures: the loop test asserts
  `stdout.matches("iteration\n").count() == 1` (the pre-repair binary printed it ~560,812 times and then
  trapped) and both assert the full `caught TypeError: thrown from a prototype accessor`, message included, so
  neither can pass on a different TypeError. **The loop back-edge spin and the switch silent-discard are both
  repaired**, measured by execution.
- **CLI target size moved 590 -> 593.** Batch 3 measured 590 (`1 passed; 589 filtered out`); this run reports
  `2 passed; 591 filtered out`. +3 compiled tests this batch.

## Rung 2b — Half A on the REAL corpus, counted both sides  [the lane's own before/after]

The lane asked for `porf test262 failure-details 'intl402/Collator' --snapshot-name <name>` before and after.
**That command cannot run here**: it requires a complete 498-node aggregate for the manifest hash, and the only
aggregate on disk (`baseline-wasm-aot-b2-aggregate-…`, 15 nodes: annexB + Array) does not cover Intl. It exits
`missing aggregate snapshot …; no compatible wasm-aot aggregate snapshot for manifest hash 2666282911900143411`.

So I measured it directly off the banked node snapshots instead, replicating
`group_failures_by_detail_identity`'s key exactly as the source spells it — `(hash_detail(detail), outcome,
kind, origin)` where `hash_detail` hashes `FailureDetailIdentity::of(detail)`, i.e. `erase_volatile_handles`
over the closed prefix list `["handle@", "symbol@"]`, digits only, replaced by `<addr>`
(`crates/porffor-test262/src/lib.rs:23615` and `:25156-25190`). BEFORE = group by the `detail_hash` **stored in
the snapshot** (computed by the pre-batch-4 binary); AFTER = group by the erased detail. Same failure list on
both sides, so this isolates the grouping change and nothing else.

| node (banked snapshot) | failures | handle-bearing | groups BEFORE | singletons BEFORE | groups AFTER | singletons AFTER |
|---|---|---|---|---|---|---|
| `intl402/Collator` (`b2-outline-collator`) | 63 | 50 | **47** | 39 | **7** | 3 |
| `intl402/DateTimeFormat` (`post-dtf`) | 65 | 30 | **48** | 41 | **21** | 14 |
| `intl402` Intl rooting (`b2-intl-rooting-after`) | 48 | 16 | **26** | 22 | **14** | 9 |

The Collator line is the lane's prediction confirmed and slightly bettered: the brief said "~48 singleton
groups today for what source-reading shows is one cause"; measured 47 groups / 39 singletons, collapsing to
**7 groups / 3 singletons**, with **one group of 50** carrying
`uncaught throw: TypeError: wasm-aot completion: object(handle@<addr>)`.

**The honest caveat, which is also the reason Half B is not optional.** Half A groups by *signature*, and the
lane itself supplies the counter-example: `built-ins/Temporal/ZonedDateTime/prototype/era/prop-desc.js` and
`eraYear/prop-desc.js` carry the identical signature from a completely unrelated cause (a missing accessor).
So the single group of 50 above is a group of 50 *renderings*, not 50 instances of one defect, and Half A on
its own makes that conflation harder to see rather than easier. Only Half B — putting the real message in the
detail — makes the group boundary mean something. Measured next.

## Rung 3 — STOP THE LINE. A batch-4 Crash-class regression that blocks the sweep and two of the three lanes.

### What I saw
The first corpus run of the batch died before producing a single case result:

```
=== b4-zdt-bi-era  built-ins/Temporal/ZonedDateTime/prototype/era  START 03:43:49 ===
thread '<unknown>' (29907) has overflowed its stack
fatal runtime error: stack overflow, aborting
=== b4-zdt-bi-era EXIT=134 END 03:44:23 ===
```

Identical to the abort that has been killing the background sweep since 03:25 (`baseline-sweep.log`,
supervisor attempts 1-6, all `exit 134`).

### Minimised, deterministic, four steps
Reproduced three times in a row on the same input; ~9.6 GiB free, so this is stack depth, not memory pressure.

| probe | result |
|---|---|
| `porf test262 report built-ins/Array/isArray/proxy.js` | **Success 1/1** |
| `porf test262 report built-ins/Array/prototype/map/15.4.4.19-5-21.js` | **stack overflow** (x3) |
| `porf run --execution-backend wasm` on that case's 4 lines | **stack overflow** |
| `porf run` on `var g = this; print(typeof g);` | **stack overflow** |
| `porf run` on `print(typeof globalThis);` | **stack overflow** |
| `porf run` on `print(typeof Temporal.ZonedDateTime);` | **stack overflow** |
| `porf run` on `print(typeof Temporal.PlainDateTime);` | **stack overflow** |
| `porf run` on `print(typeof Temporal.PlainDate);` / `Temporal` / `Temporal.Instant` / `Temporal.Now` | fine |
| `porf run` on `new Temporal.PlainDate(2020,1,1,"gregory").era` | prints `ce` |
| **`porf build wasm`** on `print(typeof Temporal.ZonedDateTime);` | **stack overflow, no `.wasm` produced** |

`build wasm` alone reproduces it, so this is the **compiler**, not execution. `RUST_MIN_STACK` does not help
because the compile runs on an engine worker thread with an explicit `ENGINE_WORKER_STACK_SIZE = 64 * 1024 *
1024` (`crates/porffor-engine/src/lib.rs:154`). **64 MiB of stack is being consumed by a one-line program.**

### Root cause, read in the source and confirmed by `git diff`
`crates/porffor-aot-wasm/src/planning.rs`:

- `fn require_standard_builtin(&mut self, builtin)` at **:1231** begins
  `self.standard_roots.insert(builtin);` — **the returned `bool` is discarded, so there is no re-entry guard.**
- The `TemporalPlainDateTime*` arm (pattern begins :1853, `TemporalPlainDateTimeConstructor` is the first
  alternative) calls `self.require_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor)` at
  **:1901**. This is **pre-existing**.
- The `TemporalZonedDateTime*` arm (pattern begins :2042, `TemporalZonedDateTimeConstructor` is the first
  alternative) calls `self.require_standard_builtin(StandardBuiltinId::TemporalPlainDateTimeConstructor)` at
  **:2087**. `git diff 091487732 HEAD -- crates/porffor-aot-wasm/src/planning.rs` shows this line prefixed
  `+`: **it is added by batch 4, by the `zdt-era-and-prototype` lane.**

So batch 4 closed a cycle in an unguarded recursive walk:
`require(ZonedDateTime…) -> require(PlainDateTimeConstructor) -> require(ZonedDateTimeConstructor) -> …`
until 64 MiB of stack is gone. The lane's own comment at :2069 anticipates a *size* consequence of that new
edge and explicitly does not anticipate a *termination* one.

### Why this is the most important result of the batch
- **It is the sweep's poison case.** The sweep died inside
  `built-ins/Array/prototype@chunk-0006-of-0012` after 240 completed cases; reconstructing the node's file
  order puts the next pending case at `built-ins/Array/prototype/map/15.4.4.19-5-2x.js`, and
  `15.4.4.19-5-21.js` — whose second line is `var global = this;` — aborts on demand. `report-all --resume`
  then retries the same case forever, because resume state only advances on completion. That is a second,
  separate defect worth a row: **`--resume` has no poison-case circuit breaker.**
- **Blast radius is not Temporal.** Top-level `this` and `globalThis` root every global, so they reach the
  cycle. Any test262 case containing `var global = this;` aborts the process.
- It also invalidates the naive reading of lane C's own verify prefix: the four `porf test262 run …/era…`
  commands cannot pass, or even report, until this is fixed.

### Fix applied by me (round 1 of max 3), and the reason for this shape rather than the obvious one
The obvious one-liner — `if !self.standard_roots.insert(builtin) { return; }` — is **wrong here**, and it took
reading the arms to see why. Several arms add dependencies with a bare `self.standard_roots.insert(dep)`
instead of a recursive `require`. Under that guard, a later genuine `require(dep)` would find `dep` already in
`standard_roots` and skip its arm, silently dropping every root that arm would have added. That converts a
crash into a wrong answer, which is worse.

So the guard goes on a separate "have I already walked this one" set, leaving `standard_roots` as the answer
and the new set as the recursion's visited-marker. The walk then runs each arm exactly once and terminates;
every effect in these arms is a monotone set-insert or a `= true` flag, so running an arm once instead of
n times yields the identical fixpoint.

### The repair, and its verification  [GREEN]
`crates/porffor-aot-wasm/src/planning.rs`:
- new private field `walked: BTreeSet<StandardBuiltinId>` on `RuntimeBootstrapPlan`, documented with the cycle,
  the blast radius, and the reason the guard may not live on `standard_roots`;
- `require_standard_builtin` now roots unconditionally and *walks* at most once
  (`if !self.walked.insert(builtin) { return; }`);
- two new unit tests: `a_cyclic_rooting_dependency_terminates_and_roots_both_ends` (enters the cycle from all
  five entry points — both constructors, both new era getters, `toPlainDateTime` — and asserts BOTH ends stay
  rooted and `temporal_object` is set, which is exactly the property the naive one-line guard would have
  broken) and `requiring_a_builtin_twice_is_idempotent`.

Rebuilt (`cargo build -p porffor-cli`, 34.26 s) and re-ran every probe. **All six previously-aborting programs
now compile and run:**

| probe | before | after |
|---|---|---|
| `print(typeof this);` | stack overflow | `object` |
| `print(typeof globalThis);` | stack overflow | `object` |
| `print(typeof Temporal.ZonedDateTime);` | stack overflow | `function` |
| `print(typeof Temporal.PlainDateTime);` | stack overflow | `function` |
| `new Temporal.ZonedDateTime(0n,"UTC").era` | stack overflow | **`undefined`** |
| `var g=this; function f(v){return this===g;} [11].map(f,this)[0]` | stack overflow | `true` |

The fifth row is worth its own line: `undefined` is exactly what
`built-ins/Temporal/ZonedDateTime/prototype/era/basic.js` asserts for an iso8601 ZonedDateTime, and the lane
flagged that file as one of two **vacuous passes to correct** — passing today only because the property did not
exist. It now answers `undefined` because the getter exists and iso8601 has no eras.

## Rung 4 — Lane A (handle cluster), the 6+1 named Intl cases  [Half B CONFIRMED, and it REFUTES the brief's lead]

All 7 ran to completion at batch-4 code (log `target/watched/b4r-t262.log`, snapshots
`target/test262-scratch/b4-handle/`). Every one is still 0/1 — no new passes — but **the detail changed, and
that is the entire deliverable of Half B**:

| case | batch 3 detail | batch 4 detail | outcome |
|---|---|---|---|
| `intl402/Collator/prototype/resolvedOptions/basic.js` | `object(handle@5297280)` | `object(handle@5303480: **target is not a constructor**)` | Bug |
| `intl402/Collator/taint-Object-prototype.js` | — | `object(handle@5346240: **target is not a constructor**)` | Bug |
| `intl402/Collator/this-value-ignored.js` | `object(handle@5265392)` | `object(handle@5271592: **target is not a constructor (Testing with Collator.)**)` | Bug |
| `intl402/Collator/test-option-usage.js` | `object(handle@5272672)` | `object(handle@5278872: **target is not a constructor**)` | Bug |
| `intl402/DateTimeFormat/this-value-ignored.js` | `object(handle@5265496)` | `object(handle@5271688: **target is not a constructor (Testing with Collator.)**)` | Bug |
| `intl402/DateTimeFormat/prototype/resolvedOptions/basic.js` | `object(handle@5397552)` | `object(handle@5403752: **RegExp.prototype.exec unsupported pattern**)` | **NotImplemented** |
| `intl402/DateTimeFormat/taint-Object-prototype.js` | — | `object(handle@5399096: **RegExp.prototype.exec unsupported pattern**)` | **NotImplemented** |

**The brief's lead (A) — "likely ONE shared Intl bootstrap/constructor-path defect" — is REFUTED by
measurement, and refuting it is the payoff.** The six-case cluster is **two** unrelated causes:

1. **`target is not a constructor`** — 5 cases (4 Collator + `DateTimeFormat/this-value-ignored.js`). This is
   the real shared Intl constructor-path defect the brief predicted, and it is one lead.
2. **`RegExp.prototype.exec unsupported pattern`** — 2 DateTimeFormat cases. Not an Intl constructor defect at
   all; a missing RegExp feature reached through DTF's pattern parsing. Note the harness re-bucketed these
   from `Bug` to **`NotImplemented`** on the strength of the message alone — a triage-class change that the
   old `object(handle@N)` rendering could not have produced.

So Half A's group of 50 was never one defect, Half B is what makes the group boundary mean anything, and the
two must be judged together. With Half B's text in place the same erasure now yields **3 distinct groups**
across these 7 cases instead of 1.

Third independent proof that the bare signature groups nothing, exactly as the lane predicted: the ZonedDateTime
`era`/`eraYear` `prop-desc.js` failures carried the identical `object(handle@N)` signature from a missing
accessor — a third unrelated cause.

### Consequence for the background sweep: it is DEAD, and the regression killed it
`ps` shows no `sweep-supervisor.sh` and no `report-all`. `baseline-sweep.log` contains **47** `supervisor
attempt` lines and ends `=== exit 134 at 03:58:28 ===`: the supervisor's 40-retry budget was consumed entirely
by the same poison case, at roughly 40 s per attempt, with zero cases completed after the first abort. The
aggregate is frozen at **15 nodes / 2,640 cases / 2,134 passed**. Nothing else has been killed — it exhausted
itself. With the `walked` guard in place a re-launch of the identical command would resume from the checkpoint;
I have not started one (the brief forbids new sweeps), so that is a hand-off, not a result.

### Post-fix unit gate  [GREEN]
`cargo test -p porffor-aot-wasm --lib -- planning:: tests::a_cyclic tests::requiring_a_builtin`:
**36 passed / 0 failed / 1 ignored**, 76.08 s. That is every existing `planning::` test plus the two new ones,
so the guard changes no rooting answer that any existing test pins. aot-wasm lib target 267 -> **269** tests.

## Rung 4 — Lane C (zdt-era-and-prototype), getter half: **8 of 8 files GREEN**  [lane payoff, counted]

Snapshots `target/test262-scratch/b4-zdt/`, log `target/watched/b4r-post.log`. Run at batch-4 code **with the
`walked` fix in**; without it none of these could produce a result at all.

| node | result |
|---|---|
| `built-ins/Temporal/ZonedDateTime/prototype/era` | **3 / 3 pass, 0 fail** |
| `built-ins/Temporal/ZonedDateTime/prototype/eraYear` | **3 / 3 pass, 0 fail** |
| `intl402/Temporal/ZonedDateTime/prototype/era` | **1 / 1 pass** |
| `intl402/Temporal/ZonedDateTime/prototype/eraYear` | **1 / 1 pass** |

Counted before -> after, per file, against the lane's own measured baseline:

| file | before (lane's cited snapshot) | after |
|---|---|---|
| `built-ins/…/era/branding.js` | RED — `TypeError object(handle@1506640)` | **PASS** |
| `built-ins/…/era/prop-desc.js` | RED — `TypeError object(handle@1488168)` | **PASS** |
| `built-ins/…/eraYear/branding.js` | RED — `TypeError object(handle@1506680)` | **PASS** |
| `built-ins/…/eraYear/prop-desc.js` | RED — `TypeError object(handle@1488168)` | **PASS** |
| `built-ins/…/era/basic.js` | vacuous pass (property absent) | **PASS for the right reason** |
| `built-ins/…/eraYear/basic.js` | vacuous pass (property absent) | **PASS for the right reason** |
| `intl402/…/era/basic.js` | never measured (lane's prediction) | **PASS** — `…"gregory").era === "ce"` |
| `intl402/…/eraYear/basic.js` | never measured (lane's prediction) | **PASS** — `.eraYear === 1970` |

**4 measured red files flipped green. 2 vacuous passes were corrected rather than celebrated** — the direct
probe `new Temporal.ZonedDateTime(0n,"UTC").era` now prints `undefined` because the getter exists and iso8601
has no eras, not because the property is missing. **2 predicted-but-unmeasured files were run and are green.**
The lane's refutation of the analyst's "zero files flip" is confirmed: eight files flip or become honest, and
none of them needed a named IANA zone.

This also retires batch 3's REMAINING #8 ("ZDT era repair itself"), which was measured 0/12 and unchanged
across two batches.

---

# b4 RUNNER, attempt 2 (container restart at ~04:45 killed attempt 1)

Start 2026-08-10 04:57 UTC. HEAD `5bb66a35a` ("WIP checkpoint: batch 4 runner rebuilding, sweep overflow
identified") — attempt 1's `planning.rs` `walked` cycle-guard IS committed. Working tree **clean**.
`target/debug/porf` mtime 04:07; `find crates -name '*.rs' -newer target/debug/porf` returns **0**, so the
binary contains every batch-4 edit including the cycle guard. There is still no `target/release/porf`;
all numbers below are the debug binary, as in batch 3 and attempt 1.

## Rung -1 — inherited state (verified, not assumed)
- **No process is running.** `ps` shows no `report-all`, no `sweep-supervisor.sh`, no `cargo`. The background
  sweep is dead (attempt 1 recorded it exhausting its 40 supervisor retries on the poison case) and attempt 1's
  own runs died with the container. **All 4 CPUs are free for the first time this batch.**
- `target/watched/b4-cli.log` — attempt 1's rung 1c, started 04:11, killed 04:32. Reached **44 of 593 tests,
  0 FAILED**. Its header reads `capped: CPUs 0-1 of 4 (50%)`: `scripts/run-watched.sh` routes through
  `scripts/capped.sh`, which pins to half the CPUs by default. So that 44-in-21-minutes rate (28.6 s/test) was
  measured on **2** CPUs, not 4, and it extrapolates to ~4.7 h for the suite. `PORFFOR_CPU_PERCENT=100`
  overrides the cap; that knob is the single most useful thing to know on this box now the sweep is gone.

## Rung 1c — RESTRUCTURED so it can actually complete, and it is running  [the batch's hard deadline]

Rung 1c has never completed on this tree (batch 3 REMAINING #1; two batch-3 attempts and one batch-4 attempt
all died mid-run). The reason is structural, not bad luck: the run is ~2.4 h at best on this box, libtest has
**no resume**, and the container restarts ~hourly. A fourth serial attempt would have died at the same place.

So I run it as **resumable per-module chunks** — `target/watched/b4c-rung1c.sh`, launched `setsid`/`disown`:

- one `cargo test -p porffor-cli --test cli -- --test-threads=3 <module>::` per area module, 16 chunks;
- `--test-threads=3` (never 1 — under `--test-threads=1` every test runs on the thread named `main`,
  `known_failures::execution_path` loses the per-test name it routes on, and all 593 fall back to spawning a
  cold `porf` child; batch-workflow.md §rung-1c);
- `PORFFOR_CPU_PERCENT=100` so the chunk gets all 4 CPUs;
- each chunk appends a verdict line, plus its `failures:` block if red, to
  **`target/lane-notes/b4-rung1c-chunks.md`**, and records itself in `target/watched/b4c-done`, so a restart
  loses at most the chunk in flight and the next attempt re-invokes the identical script to resume.

**Every test still runs exactly once**, with the same per-test-name execution-path routing, so the union of the
chunks is the suite and the ledger's `should_panic` enforcement is unaffected. `array::` is a *substring*
filter and also matches `typed_array::`, so the array chunk carries `--skip typed_array::`; nothing is counted
twice. What chunking gives up, stated plainly: whole-suite ordering/interference effects, and a single
`test result:` line. Neither is what the T03 row needs — it needs the failing set.

### Chunk results so far (live file: `target/lane-notes/b4-rung1c-chunks.md`)
| chunk | tests | result |
|---|---|---|
| `known_failures::` | 4 | **4 passed / 0 failed**, 0.96 s |
| `throw_propagation::` | 2 | **2 passed / 0 failed**, 40.73 s |

`known_failures::` is not a formality: it contains `ledger_is_well_formed`, the assertion that fails the moment
`CURRENT_BATCH` reaches `unfilled-allowed-until: batch-4`. It is green **because `CURRENT_BATCH` is still 3**
(`crates/porffor-cli/tests/cli/known_failures.rs:91`). The deadline is unmet, not met — see T03 below.

`throw_propagation::` green here is lane B's headline re-confirmed at this commit on a clean tree: both
`run_wasm_backend_propagates_a_throwing_property_read_out_of_a_loop` and `..._out_of_a_switch` pass.

## Side chain (nice 15, pinned to CPU 3) — `target/watched/b4c-side.sh`
Rung 1c's 3 test threads leave one CPU; the side chain uses it at `nice -n 15` so it can never slow the
deadline run. 9 single-case `porf test262 report` runs, each banking its own snapshot under
`target/test262-scratch/b4-side/` and a done-marker, log `target/watched/b4c-side.log`:
lane C's 3 unmeasured prototype-method files (`since`, `until`, `from`), lane B's 4 corpus cases, lane A's 2
regression-watch cases. Selection reasoning is in the script.

## Rung 4 — Lane C prototype-method half: 2 of 5 measured, and Half B changed the answer
Harvested from attempt 1's `target/test262-scratch/b4-zdt/` (runs completed 04:23 and 04:28, after the
container had stopped reporting):

| file | batch 3 detail | batch 4 detail | result |
|---|---|---|---|
| `intl402/…/prototype/add/era-boundary-gregory.js` | `TypeError object(handle@1820208)` | `TypeError object(handle@1827888: **value is not callable**)` | 0/1 Bug |
| `intl402/…/prototype/subtract/era-boundary-gregory.js` | `TypeError object(handle@1820208)` | `TypeError object(handle@1827888: **value is not callable**)` | 0/1 Bug |

Two things are counted here rather than assumed. (i) **Half B works on this lane too** — the bare
`object(handle@N)` now carries the cause. (ii) The two files still share one handle **byte for byte**
(`1827888`), which is the lane's own reading confirmed: add/subtract are near-textually-identical, so the
shared handle is an interned-pool layout artifact, **not** evidence of a shared defect. The lane's correction
to the brief ("one lead, not two") survives contact with batch-4 code.
Note the measured message is `value is not callable`, **not** the `undefined is not a function` the lane
predicted. Same class of defect, different wording — record the measured string, not the predicted one.

## Rung 4 — Lane C prototype-method half, 3rd file measured; and the cleanest Half-A+B proof of the batch
`intl402/…/prototype/since/era-boundary-gregory.js` (05:01, `target/test262-scratch/b4-side/`):
0/1 Bug/Runtime, detail `TypeError: wasm-aot completion: object(handle@1879624: **value is not callable**)`.

Put beside `add`/`subtract` (both `handle@1827888`), this is the sharpest measured statement of why Half A
needs Half B and vice versa, and it needs no argument — only the four strings:

| file | handle | cause text (Half B) |
|---|---|---|
| `prototype/add/era-boundary-gregory.js` | `1827888` | `value is not callable` |
| `prototype/subtract/era-boundary-gregory.js` | `1827888` | `value is not callable` |
| `prototype/since/era-boundary-gregory.js` | `1879624` | `value is not callable` |

- **The bare handle splits one defect into two groups.** `1827888` vs `1879624` is exactly batch 3's
  "add/subtract share one handle, since/until another — two leads". Measured now, that split is an artifact.
- **Half A alone would still have split them**, because `erase_volatile_handles` erases the *digits* and keeps
  everything else; with no cause text there is nothing left to join on.
- **Half B is what merges them**: all three (and, by the pairing, `until`) carry the identical cause string, so
  after erasure they are **one group**. The lane's correction to the brief — "one lead, not two" — is now
  measured rather than argued, and the batch brief's item (B) "add/subtract share one defect handle,
  since/until another — two leads" is **REFUTED**.

## T24 — checked ahead of the `language::` chunk, because the lane called it a same-patch hazard  [CLEAN]
The lane warned that when Half B lands, `language::run_wasm_backend_gives_a_runtime_error_a_message_distinct_
from_its_name` flips to `test did not panic as expected` unless the `#[should_panic]`, the `const _` line and
the ledger row are retired in the same patch. Verified all three, by reading:
- `crates/porffor-cli/tests/cli/language.rs:1849` — the attribute is **gone**; the only `#[should_panic]` left
  in `crates/porffor-cli/tests/` is `binary_data.rs:549` (`expected = "porf run exceeded"`, the declared T17
  hang). The test now asserts positively: `stdout.contains("string(message-differs)")`.
- `known_failures.rs` — exactly **two** `const _` lines (`:544` atomics-wait, `:545` heap page-boundary).
- `known-failures.tsv` — no T24 row.
The retirement is complete and consistent. This was the single likeliest cause of a red `language::` chunk and
it is not present.

## Conformance failures recorded with an owner and a reason (AGENTS.md requirement)
Lane C asked that the 6 calendar-blocked files be recorded rather than left silent. I verified the blocking
claim in source rather than repeating it: `crates/porffor-aot-wasm/src/builtins/temporal_plain_date.rs`,
`enum TemporalCalendarId` has exactly two variants (`Iso8601`, `Gregory`) and `pub(crate) const ALL: [Self; 2]`.
The doc comment on `ALL` also records a second, subtler blocker: adding a lunisolar calendar invalidates
`emit_temporal_month_day_string_reference_year`'s unconditional 1972 constant.

| files | owner | reason |
|---|---|---|
| `ZonedDateTime/from/{calendar-invalid-era, calendar-not-supporting-eras, non-positive-single-era-year}.js` | **T22** (`built-ins/Temporal`) / **T23** (`intl402`) per `test262/backlog/ownership-map.tsv` | Blocked on a two-variant calendar set. They need 13-15 calendars with real arithmetic; `TemporalCalendarId::ALL` is `[Iso8601, Gregory]`. Not a defect in this batch's work and NOT counted against lane C. |
| `PlainDate/from/{calendar-invalid-era, calendar-not-supporting-eras}.js` | same | same |
| `PlainDateTime/from/calendar-invalid-era.js` | same | same |
| `intl402/…/prototype/{add,subtract,since,until}/era-boundary-gregory.js` + `from/era-boundary-gregory.js` | **T23** | ONE defect, measured: `value is not callable` before any era assertion. Needs `toPlainDateTime` / `since` / `until` / `withCalendar` on ZonedDateTime. 3 of 5 measured at batch-4 code, 2 in flight. |

### The 4-file ZDT prototype-method set, COMPLETE and counted (05:05)
`until` finished: `object(handle@1879624: value is not callable)` — **byte-identical to `since`**.

| file | handle | cause text |
|---|---|---|
| `prototype/add/era-boundary-gregory.js` | `1827888` | `value is not callable` |
| `prototype/subtract/era-boundary-gregory.js` | `1827888` | `value is not callable` |
| `prototype/since/era-boundary-gregory.js` | `1879624` | `value is not callable` |
| `prototype/until/era-boundary-gregory.js` | `1879624` | `value is not callable` |

All four 0/1 Bug/Runtime at batch-4 code. **Two handles, one cause.** This closes the question the brief opened:

- **Batch brief item (B) is REFUTED.** "add/subtract share one defect handle, since/until another — two leads"
  describes the *handles* correctly and the *defects* incorrectly. The handle pairing tracks textual identity
  of the fixtures (interned-string-pool layout), not causation.
- **Lane C's correction is CONFIRMED by measurement**: one lead, not two.
- **This is the batch's cleanest joint proof of Half A + Half B.** Half A (erase the digits) leaves two groups
  because the handles genuinely differ. Half B (put the cause in the detail) collapses them to one. Neither
  half alone gets the right answer here; the pair does. Nothing about this is visible by eye in the old
  rendering, which was four distinct `object(handle@N)` strings.

## Side chain COMPLETE (05:12) — 9 cases, and one of them is a NEW PASS nobody predicted

| case | total | passed | failed | verdict |
|---|---|---|---|---|
| `intl402/…/ZonedDateTime/prototype/since/era-boundary-gregory.js` | 1 | 0 | 1 | `value is not callable` |
| `intl402/…/ZonedDateTime/prototype/until/era-boundary-gregory.js` | 1 | 0 | 1 | `value is not callable` |
| **`intl402/…/ZonedDateTime/from/era-boundary-gregory.js`** | 1 | **1** | 0 | **NEW PASS** |
| `language/statements/try/S12.14_A12_T1.js` | 1 | 1 | 0 | pass |
| `language/statements/try/S12.14_A12_T2.js` | 1 | 1 | 0 | pass |
| `language/statements/switch/S12.11_A1_T1.js` | 1 | 1 | 0 | pass |
| `language/statements/switch/S12.11_A4_T1.js` | 1 | 1 | 0 | pass |
| `built-ins/Error/prototype/message/prop-desc.js` | 1 | 1 | 0 | pass |
| `built-ins/NativeErrors/EvalError/proto-from-ctor-realm.js` | 1 | 0 | 1 | **NotImplemented / Unsupported**, see below |

### `ZonedDateTime/from/era-boundary-gregory.js`: red -> green, and it is NOT a vacuous pass
Batch 3 measured this file **0/1** (`TypeError object(handle@1817560)`), twice, and lane C listed it among the
five prototype-method files expected to die on a missing `toPlainDateTime`. It **passes at batch-4 code**.
I read the whole file before counting it, because a Temporal file that suddenly passes is exactly where a
vacuous pass hides. It is the opposite of vacuous — four constructions, each asserted through
`TemporalHelpers.assertPlainDateTime` on **twelve** fields plus era and eraYear:

```js
Temporal.ZonedDateTime.from({era:"ce", eraYear:0,  monthCode:"M01", day:1, …, calendar:"gregory"}, {overflow:"reject"})
  .toPlainDateTime()  ->  year 0,  "bce", 1     // CE 0  resolves to BCE 1
{era:"ce",  eraYear:-1} -> year -1, "bce", 2
{era:"bce", eraYear:0}  -> year  1, "ce",  1
{era:"bce", eraYear:-1} -> year  2, "ce",  2
```

So it exercises three things at once and all three must be right: the **era -> year** property-bag direction,
**`ZonedDateTime.prototype.toPlainDateTime`**, and the **era/eraYear getters** on the result. This is a fifth
ZDT era file flipped by this batch, on top of lane C's eight getter files — **9 of the 13 ZDT/era files this
batch touched are now green**, and the 4 that remain are the single `value is not callable` defect.

### Lane B, rung 4 on the corpus: 4/4 green, chosen to hit the repaired paths
Not arbitrary cases. `S12.14_A12_T1/T2` are titled *"Loop inside try Block, where throw exception"* — a `throw`
crossing a loop back-edge out to a `catch`, which is precisely the path that spun ~560,812 times and trapped
before the repair. `S12.11_A1_T1` and `S12.11_A4_T1` are switch-with-throw, including nested switch. All four
pass, so the `Br` depth immediate is right on real corpus control flow, not only on the lane's two fixtures.
This is the corpus evidence the lane wanted; the full `language/statements/try` (250+ cases) and `/switch` (48)
nodes remain unrun for cost, recorded below.

### Lane A regression watch: no regression, and the one red is a declared non-defect
- `built-ins/Error/prototype/message/prop-desc.js` **passes** — `.message` still has its correct property
  shape after Half B rewired `emit_runtime_error_object` to define `message` from its message argument.
- `built-ins/NativeErrors/EvalError/proto-from-ctor-realm.js` fails as **outcome NotImplemented, bucket
  Unsupported (1)** — `feature 'Function constructor dynamic code generation' is not implemented`. It needs
  `$262.createRealm()`. AGENTS.md ("Compiler Contract") explicitly classifies dynamic code generation as a
  tracked Wasm-AOT unsupported case rather than a defect, and the harness bucketed it that way on its own.
  **Not a Half B regression** — it never reaches error-message code. Owner T-harness/realm, reason: realm
  creation requires the Function constructor.

## Rung 1c chunk 5 — `date::` **16 passed / 0 failed**, 607.87 s
Note the cost: **38 s/test**, ~5x the `dynamic::`/`heap::` rate, because these compile Date/Intl-heavy
fixtures. Per-module cost varies by more than 5x across this suite; do not extrapolate one module's rate to
the whole, which is the mistake the batch-workflow ladder warns about at rung 1b.

## Lane A Half B — the end-to-end CLI gate, run out of order because it is the batch's key assertion  [GREEN]
`language::run_wasm_backend_gives_a_runtime_error_a_message_distinct_from_its_name` sits inside the 105-test
`language::` chunk, which is last-but-one in the queue and may not be reached before the container restarts.
It is also the single assertion that decides whether Half B actually works end to end, so I ran it alone
against the already-built test binary (`target/debug/deps/cli-986abd5f02521ed6`, nice 15 on the free CPU, so
the queued chunk was not slowed):

```
test language::run_wasm_backend_gives_a_runtime_error_a_message_distinct_from_its_name ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 592 filtered out; finished in 26.41s
```

`592 filtered out` independently re-confirms the target holds **593** tests.

**Counted before -> after on the same assertion:** batch 3 ran this test and read its stdout verbatim as
`wasm-aot completion: string(message-equals-name)` — the fixture's defect-present branch, which is why the T24
ledger row was green *as a declared failure*. It now passes on the defect-absent branch.

I checked it is not vacuous by reading the fixture
(`crates/porffor-cli/tests/fixtures/wasm_runtime_error_message_is_not_its_name.js`). It is a four-way
discriminator, not a boolean: `no-throw` / `wrong-error-kind` / `message-equals-name` / `message-differs`. To
print `message-differs` the program must (a) actually throw on `null.x`, (b) throw a real `TypeError`, and
(c) have `e.message !== e.name`. A stubbed or absent message lands on one of the other three.
**Half B is confirmed at the CLI level, not merely at the unit level.**

## Rung 1c chunks 6-7, and THE FIRST RUNG-1C FAILURES EVER MEASURED ON THIS TREE

| chunk | result |
|---|---|
| `iterator::` | **FAILED — 26 passed / 4 failed**, 488.89 s |
| `regexp::` | 33 passed / 0 failed, 356.63 s |

The four:
```
iterator::run_wasm_backend_succeeds_for_iterator_prototype_every_fixture
iterator::run_wasm_backend_succeeds_for_iterator_prototype_find_fixture
iterator::run_wasm_backend_succeeds_for_iterator_prototype_reduce_fixture
iterator::run_wasm_backend_succeeds_for_iterator_prototype_some_fixture
```
All four panic identically at `assertion failed: output.status.success()` — a bare assert, so the log carries
no cause. I reproduced it by hand rather than guessing:

```
$ ./target/debug/porf run --execution-backend wasm \
    crates/porffor-cli/tests/fixtures/wasm_iterator_prototype_some.js
uncaught throw: wasm-aot completion: string(callback throw)
```

`"callback throw"` is the fixture's own sentinel, thrown at `if (!callbackThrew) { throw "callback throw"; }`.
The block above it is:

```js
try { new ClosingIterator().some(function () { ++callbackCalls; throw new SomeSentinelError(); }); }
catch (error) { callbackThrew = true; … }
```

So a callback throw inside `Iterator.prototype.some` **never reaches the user `catch`** — it is silently
discarded. That is exactly the symptom class of batch-3 fixer finding 21 ("switch -> silent discard").

### Regression, or newly observed? — DISCRIMINATED BY MEASUREMENT, one program, four probes
This matters more than the failure itself: batch 4 changed the depth immediate of every `Br` in the tree, so
"lane B broke the iterator helpers" is the hypothesis that has to be killed or confirmed. One program:

| probe | result |
|---|---|
| **A** — throw from a callback inside **`Iterator.prototype.some`** (builtin iterator helper) | **`false` — DISCARDED** |
| **B** — throw from a callback inside **`Array.prototype.some`** (builtin array helper) | `true` — caught |
| **C** — throw inside a **user-level `for` loop** in `try` (lane B's repaired path) | `true` — caught |
| **D** — throw from a callback inside **`Array.prototype.forEach`** | `true` — caught |

A global `Br`-depth regression cannot produce this shape: it would break B, C and D too, and all three are
green. The defect is **confined to the `Iterator.prototype.*` helper family**. Lane B's repair is measured
working (probe C) and measured not to have broken sibling builtin-callback paths (B, D).

### Localised in source, so the next batch does not repeat the search
`crates/porffor-aot-wasm/src/builtins/standard.rs`, the `StandardBuiltinId::IteratorPrototypeSome` arm at
**line 26936**, 319 lines. Counted occurrences inside that arm:

| symbol | count |
|---|---|
| `Instruction::Loop` | 1 |
| `Instruction::Br(` | 1 |
| `COMPLETION_KIND_THROW` | 3 |
| **`emit_propagate_throw*`** | **0** |
| **`active_throw_target`** | **0** |

The arm **hand-rolls its own loop and its own throw handling** and never calls the shared propagation helper
(`control_flow.rs:588 emit_propagate_throw_from_locals_if_needed`) that lane B repaired. That is both the
reason lane B's fix does not reach it and the reason the defect survived: three `COMPLETION_KIND_THROW`
compares that do not end in a branch to the active throw target. The same shape should be checked in the
`Every` / `Find` / `Reduce` arms, which fail identically.

**Honest limit on the claim.** I cannot *prove* non-regression without a pre-batch-4 binary, and building one
costs a full cold rebuild plus a rung-1c re-run that this box has no window for. What is measured is that the
blast radius is the Iterator-helper family alone and that every path lane B actually touched is green. I state
it as "confined and consistent with pre-existing", not as "proven pre-existing".

**Owner: T15** (`built-ins/Iterator` in `test262/backlog/ownership-map.tsv`). **Reason:** `Iterator.prototype`
helper arms in `standard.rs` hand-roll throw handling and never branch to the active throw target, so a
callback throw is discarded instead of propagating. **Repro:** the four-probe program above, and
`porf run --execution-backend wasm crates/porffor-cli/tests/fixtures/wasm_iterator_prototype_some.js`.

I did **not** attempt the repair. It is four ~300-line hand-rolled arms, and a speculative fix I could not
re-gate with a full rung 1c in the remaining window would be worse than an exact lead.

## HOW TO RESUME RUNG 1C (one command; this is the batch's most valuable hand-off)

```sh
./target/watched/b4c-rung1c.sh          # setsid it; re-run verbatim, it resumes
```

It skips every chunk already recorded in `target/watched/b4c-done` and continues with the next. Progress and
verdicts accumulate in `target/lane-notes/b4-rung1c-chunks.md`; per-chunk logs are `target/watched/b4c-<mod>.log`.
Do not "clean up" `target/watched/b4c-done` — it *is* the resume state.

### Rung 1c coverage as of this attempt
| chunk | tests | verdict |
|---|---|---|
| `known_failures::` | 4 | 4 pass |
| `throw_propagation::` | 2 | 2 pass |
| `dynamic::` | 11 | 11 pass |
| `heap::` | 12 | 11 pass, 1 ignored (declared, T05) |
| `date::` | 16 | 16 pass |
| `iterator::` | 30 | **26 pass, 4 FAIL** |
| `regexp::` | 33 | 33 pass |
| **measured** | **108 of 593** | **4 failures, all in `iterator::`** |
| `language::` (1 test, run alone) | 1 | T24 pass — see above |

Remaining chunks, in the script's order: `object`(35), `string`(36), `data_view`(38), `functions`(45),
`frontend`(46 of 54; 8 are `#[cfg(feature = "spec-exec-oracle")]`), `typed_array`(58), `array`(84),
`language`(105), `binary_data`(38). **485 tests.**

Arithmetic that proves the chunking is a complete rung 1c and not a subset: `awk` exact-line count over
`crates/porffor-cli/tests/cli/*.rs` = **601** `#[test]`; minus the **8** `spec-exec-oracle`-gated in
`frontend.rs` = **593** compiled, which is exactly what every chunk's `N passed + filtered out` sums to
(e.g. `11 + 582 = 593`, `1 + 592 = 593`). The 16 chunk filters partition those 593 with no overlap
(`array::` carries `--skip typed_array::` because libtest filters are substrings) and no gap.

**Budget for the next attempt, measured here rather than estimated:** per-module cost spans **6.8 s/test**
(`heap`) to **38 s/test** (`date`); 108 tests took ~30 min with all 4 CPUs. The 485 remaining are ~2 h, plus
**900 s of dead wall-clock** in `binary_data::` for the declared T17 `Atomics.wait` hang (`HANG_TIMEOUT`,
`tests/cli/main.rs:66`). Budget ~2.5 h across 3 container windows. Use `PORFFOR_CPU_PERCENT=100`: without it
`scripts/capped.sh` silently halves the machine, which is what made batch 4's first attempt look 2x slower.

## T03 — the ledger row. NOT filled, and deliberately NOT force-closed.
`known-failures.tsv` still carries `cli / UNFILLED / unfilled / T03` and
`known_failures.rs:91` still reads `const CURRENT_BATCH: u32 = 3`, so `ledger_is_well_formed` is green
(measured, in the `known_failures::` chunk: 4 passed).

I did **not** bump `CURRENT_BATCH` to 4 and did **not** delete the row. The row's own text says the runner
"fills this in from its own run", and a fill is only honest once the failing set is complete. 108 of 593 tests
are measured; the other 485 could add rows. Bumping now would either (a) redden `ledger_is_well_formed` for a
reason unrelated to any lane's work, or (b) if I deleted the row too, silently declare "no expected failures"
on a suite I have measured 18% of. Both are worse than an unmet deadline that is stated out loud.

**What the next attempt does, in order:** finish the 9 chunks; then, for whatever set has failed, add one row
per test with owner + reason + evidence, the matching one-line `#[should_panic(expected = "…")]` and
`pub(crate)`, and a `const _: fn() = crate::<module>::<name>;` line in `known_failures.rs`; delete the
`UNFILLED` row; set `CURRENT_BATCH = 4`; re-run the `known_failures::` chunk to confirm green.

**The four rows already earned by measurement** (message substring `assertion failed: output.status.success()`
is stable and identical for all four):

| target | test | state | owner | reason |
|---|---|---|---|---|
| cli | `iterator::run_wasm_backend_succeeds_for_iterator_prototype_every_fixture` | fail | T15 | Iterator helper discards a callback throw |
| cli | `iterator::run_wasm_backend_succeeds_for_iterator_prototype_find_fixture` | fail | T15 | ditto |
| cli | `iterator::run_wasm_backend_succeeds_for_iterator_prototype_reduce_fixture` | fail | T15 | ditto |
| cli | `iterator::run_wasm_backend_succeeds_for_iterator_prototype_some_fixture` | fail | T15 | ditto |

Evidence column must cite a tracked path (the hygiene test rejects `target/`): use
`crates/porffor-cli/tests/fixtures/wasm_iterator_prototype_some.js`.
A better outcome than four ledger rows is fixing the arm — the rows are the fallback, not the goal.

## RUNG 1C — FINAL COVERAGE FOR THIS ATTEMPT (05:51)
Chunks 8-10 all green: `object::` 35/35 (402.51 s), `string::` 36/36 (371.17 s), `data_view::` 38/38 (198.14 s).
`functions::` was in flight when the window closed.

| # | chunk | tests | verdict |
|---|---|---|---|
| 1 | `known_failures::` | 4 | 4 pass (incl. `ledger_is_well_formed`) |
| 2 | `throw_propagation::` | 2 | 2 pass (lane B headline) |
| 3 | `dynamic::` | 11 | 11 pass |
| 4 | `heap::` | 12 | 11 pass + 1 declared ignore (T05) |
| 5 | `date::` | 16 | 16 pass |
| 6 | `iterator::` | 30 | **26 pass / 4 FAIL** |
| 7 | `regexp::` | 33 | 33 pass |
| 8 | `object::` | 35 | 35 pass |
| 9 | `string::` | 36 | 36 pass |
| 10 | `data_view::` | 38 | 38 pass |
| — | `language::` T24 only | 1 | pass |
| | **TOTAL MEASURED** | **217 of 593 (36.6%)** | **213 pass, 4 fail, 1 declared ignore** |

**This is the furthest rung 1c has ever got on this tree.** Prior high-water marks: batch 3 = 15 tests
(`b3-cli.log`), batch 4 attempt 1 = 44 tests (`b4-cli.log`). Both were single monolithic invocations that lost
everything when the container died; this one banks each chunk and resumes.

`functions::` (45) was in flight at the cutoff. Remaining after it: `frontend`(46), `typed_array`(58),
`array`(84), `language`(105), `binary_data`(38) = **331 tests** plus whatever `functions::` did not finish.
Re-run `./target/watched/b4c-rung1c.sh` verbatim; it resumes from `target/watched/b4c-done`.

## Regressions found by this attempt: ZERO
Nothing measured moved backwards.
- 213 of 217 CLI tests green; the only 4 failures are localised to the `Iterator.prototype` helper family and
  discriminated by measurement as *not* caused by lane B's tree-wide `Br`-depth change (probes B/C/D green).
- Lane B corpus rung 4: 4/4 green on real try-with-loop-throw and switch-with-throw cases.
- Lane A regression watch: `built-ins/Error/prototype/message/prop-desc.js` green; the one red is a declared
  `Unsupported` dynamic-code-generation case, not a defect.
- Lane C: 9 of 13 ZDT/era files green, including one (`from/era-boundary-gregory.js`) that was red in batch 3.
- No case measured this attempt is worse than its batch-3 baseline.

## REMAINING, each with owner and reason
| # | Item | Owner | Reason it did not run |
|---|---|---|---|
| 1 | Rung 1c chunks 11-16 (`functions` tail, `frontend`, `typed_array`, `array`, `language`, `binary_data`) — 331+ tests | next runner | Container window. Resumable: `./target/watched/b4c-rung1c.sh`. ~2 h incl. 900 s dead time for the declared T17 hang. |
| 2 | T03 ledger row | next runner | Needs the complete failing set from item 1. Deliberately not force-closed; 4 rows already earned, drafted above. `CURRENT_BATCH` left at 3 so `ledger_is_well_formed` stays honest. |
| 3 | `Iterator.prototype.{some,every,find,reduce}` callback-throw discard | **T15** | Newly found by this attempt. Four hand-rolled ~300-line arms in `standard.rs` (Some at :26936) with 3 `COMPLETION_KIND_THROW` compares and 0 branches to the active throw target. Repro + 4-probe discriminator recorded above. Not a runner-sized repair. |
| 4 | `cargo test -p porffor-ir --lib` tests 298-626 | next runner | Batch-3 debt. All 4 CPUs were committed to rung 1c, which is the deadline item. |
| 5 | `cargo test -p porffor-aot-wasm --lib` in full (269 tests) | next runner | Same. Attempt 1 ran `planning::` (36) and `code_sink::` (9) green; the emit-heavy remainder is unrun. |
| 6 | Fake wasm-safe suite, cases 11-190 | next runner | Batch-3 debt; measured at ~46 s/case on this box (~2.5 h), and the ladder's "10-60 s" is a 16-CPU figure. |
| 7 | `built-ins/Temporal/PlainMonthDay` cases 151-199 | next runner | 150/199 banked green by batch 3's queued chain (`b3r-pmd-node`, resumable). ~2 min/case here. |
| 8 | Rung G (golden diff) for lane B | next integrator | Requires parking the change to capture a "before"; the brief forbids git stash/commit, and each side is ~10 min plus a clean window. Lane B's note says an EMPTY diff here would *disprove* the repair — so it must be run deliberately, not opportunistically. |
| 9 | Full `language/statements/{try,switch,for}` nodes | next runner | 250+/48/101 cases at 30 s-5 min each. 4 representative cases run instead, all green. |
| 10 | Coercion regression watch (`Object/prototype/toString` 41, `expressions/{addition,equals}`, `Array/prototype/join`, `Symbol.toPrimitive` x2) | next runner | Batch-3 debt, multi-hour here. Rung 1c is its cheapest real gate and is 37% done with 0 coercion failures so far. |
| 11 | Re-launch of the background sweep | next integrator | Attempt 1 found it dead, having burned all 40 supervisor retries on the `planning.rs` cycle poison case; that cycle is now fixed and committed, so the identical command would resume from the 15-node checkpoint. The brief forbids starting new sweeps, so this is a hand-off. |
| 12 | `report-all --resume` poison-case circuit breaker | next integrator | Attempt 1's finding: resume state advances only on completion, so a process-aborting case is retried forever. Still true; the cycle that triggered it is fixed but the failure mode is not. |

### Late addition (05:51): `functions::` completed green
`functions EXIT=0  test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 548 filtered out;
finished in 390.41s` (45 + 548 = 593, so the partition still checks out).
**Revised total measured: 262 of 593 (44.2%), 258 pass, 4 fail, 1 declared ignore.** `frontend::` started
05:51:33. Remaining after it: `typed_array`(58), `array`(84), `language`(105), `binary_data`(38) = 285.
