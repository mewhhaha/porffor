# b7 runner findings

Runner for batch 7. Written incrementally: every rung appends here before the
next rung starts. Counts are counted, never estimated. No git.

Machine: 4 CPUs, 15 GiB. Sweep (pid 10024) and FRONTIER driver (pid 17689) both
alive at session start; scheduling law respected (heavy CLI chunks do not
coexist with the sweep).

---

## Rung 0 — `cargo xc` (check --workspace --all-targets)

`cargo xc -j 2` → **EXIT=0**, `Finished dev profile in 0.44s` (fully cached from
the fixer's run). Log: `target/watched/b7r-xc.log`, 399 lines.
Warning set identical to the fixer's report (`porffor-ir` lib-test 5 warnings,
4 duplicates, etc.). Confirms the worktree the fixer left is the worktree I run.

Worktree state at start (uncommitted, the fixer's 25 fixes):

```
 M crates/porffor-aot-wasm/src/data.rs
 M crates/porffor-aot-wasm/src/expressions.rs
 M crates/porffor-cli/tests/cli/known_failures.rs
 M crates/porffor-cli/tests/cli/language.rs
 M crates/porffor-cli/tests/cli/main.rs
 M crates/porffor-cli/tests/fixtures/wasm_intl_date_time_format_subclass.js
 M crates/porffor-cli/tests/fixtures/wasm_regexp_runtime_pattern_invalid.js
 M crates/porffor-cli/tests/known-failures.tsv
 M crates/porffor-engine/src/lib.rs
 M crates/porffor-ir/src/lowering.rs
 M crates/porffor-ir/src/lowering_helpers.rs
 M crates/porffor-test262/src/lib.rs
 M docs/rust-rewrite/batch-workflow.md
 M scripts/rung1c-chunks.sh
```

HEAD `c8ca03832` "WIP checkpoint: batch 7 integrate/fix".

---

## Item (A) SWEEP TRIAGE — the banked baseline, recounted this session

Source: the 23 node JSONs under `target/test262-scratch/baseline/` that carry a
`failures[]` array (`aggregate` and `matrix-cache` files excluded). Grouped from
the JSON `failures[]` arrays, **never** the truncated `.txt` sidecars.
Machine-readable dump written to `target/lane-notes/b7-sweep-families.json`.

**The sweep has advanced since both planners read it.** They reported 3,640
banked cases with exactly one Bug-outcome failure. As of this session:

| | count |
|---|---|
| nodes with a `failures[]` array | 23 |
| cases | **4,220** |
| passed | **3,711** |
| failure records | **509** |
| distinct `detail_hash` families | **7** |

Outcome split: `NotImplemented` 506, **`Bug` 3**. Kind split: `Unsupported` 506,
`Runtime` 3.

### All seven families, counted

| n | outcome | family (detail, truncated) |
|---:|---|---|
| **471** | NotImplemented | `eval dynamic source evaluation` is not implemented |
| **17** | NotImplemented | `$262.evalScript dynamic source evaluation` is not implemented |
| **14** | NotImplemented | `Function constructor dynamic code generation` is not implemented |
| 2 | NotImplemented | `function or class declaration` (the generator-declaration diagnostic) |
| 2 | NotImplemented | `async for-of with a body await requires an array iterable and a plain binding` |
| **2** | **Bug** | `uncaught throw: wasm-aot completion: string(Expected SameValue(object,object, boolean,boolean) to be true)` |
| 1 | **Bug** | `uncaught throw: Error: wasm-aot completion: object(handle@1515984)` |

### The headline the triage produces

**502 of 509 failures (98.6%) are the dynamic-code-generation policy family** —
`eval`, `$262.evalScript`, `Function` constructor. AGENTS.md declares these
explicit Wasm-AOT unsupported dynamic-code-generation cases, not defects. They
are not a fix lane and cannot be turned into one.

That leaves **7 addressable failures in 4,220 cases = 1.66 per 1,000**. Every
one of the seven is already owned by a batch-7 lane:

- 1 → RE-RT gate (`annexB/built-ins/RegExp/prototype/compile/duplicate-named-capturing-groups-syntax.js`)
- 2 → IR-SHAPES item 2 (`built-ins/Array/fromAsync/asyncitems-{async,}iterator-not-callable.js`)
- 2 → IR-SHAPES item 3 / RE-RT item 3 (`annexB/built-ins/RegExp/RegExp-control-escape-russian-letter.js`, `RegExp-invalid-control-escape-character-class.js`)
- **2 → UNOWNED, NEW SINCE PLANNING** (below)

### NEW: a second Bug family, unowned

```
built-ins/Array/prototype/toLocaleString/primitive_this_value.js
built-ins/Array/prototype/toLocaleString/primitive_this_value_getter.js
detail_hash 4872481042840822955, outcome=Bug, kind=Runtime
[origin:unknown] uncaught throw: wasm-aot completion:
  string(Expected SameValue(object,object, boolean,boolean) to be true)
```

Both planners described the RegExp `compile` case as "the ONLY Bug-outcome
failure in 3,640 banked sweep cases". At 4,220 cases that is no longer true:
there are three Bug failures in two families, and this second family arrived
with nodes banked after the planners read the directory. It is a genuine
wrong-answer class (an assertion inside a Test262 case observed the wrong
value), it is in no lane's ownership, and it is the single largest *addressable*
family that batch 7 did not plan for — 2 of the 7 addressable failures, 29%.

**Conclusion for item (A): the sweep's broad failure data does not support new
fix lanes.** The top three families by size are policy, by an enormous margin
(502 vs 7). The correct reading is that the sweep is confirming the lanes batch
7 already chose, plus one new two-case family that needs an owner. Turning "the
top families into fix lanes" would mean implementing `eval`, which AGENTS.md
forbids.

### Prefix before-counts (from the banked snapshots, for the after-comparison)

| lane verify_prefix | node | completed | passed | failed |
|---|---|---:|---:|---:|
| `annexB/built-ins/RegExp/prototype/compile` | annexB | 23 | 22 | **1** |
| `built-ins/Array/fromAsync` | built-ins_Array_fromAsync | 95 | 93 | **2** |

The annexB node reports `total=1086 passed=589` and carries exactly 497 records
in `failures[]` — re-confirming the b7 planner's check that the JSON is complete
and the `.txt` is a head.

---

# SESSION 2 (container restarted; sweep relaunched 04:53, pid 2209)

State harvested at start: HEAD `167add6a9` "WIP checkpoint: batch 7 runner
mid-ladder", worktree **clean** (the harness checkpoint-committed the fixer's and
the write lanes' edits). `target/debug/porf` was 04:21 and therefore **stale**
against `crates/porffor-ir/src/lowering.rs` (05:31) — the fixer's IR-SHAPES and
RE-RT edits were not in it. Session 1's `cargo test -p porffor-ir` was killed
mid-run by the restart (`target/watched/b7r-ir.log` ends on two
"has been running for over 60 seconds" lines).

Banked and re-usable: the 9 FRONTIER node snapshots under
`target/test262-scratch/frontier/` (758 cases), the 23-node sweep baseline
triage in the section above, and `target/watched/rung1c-done` (15 banked chunk
rows).

## Rung 0 (re-run) — `cargo xc`

`cargo xc -j 2` → **EXIT=0**, `Finished dev profile in 0.21s`.
Warning set unchanged (`porffor-ir` lib-test 5 warnings, 4 duplicates).
Log: `target/watched/b7r2-chain.log`.

## Rung 0.5 — debug `porf` rebuilt against the current head

`cargo build -j 2 -p porffor-cli --bin porf` → **EXIT=0** in `2m 29s`.
`target/debug/porf` now 05:42. Everything below this line is measured with that
binary.

**Sweep hazard, recorded not hidden.** `sweep-supervisor.sh` runs
`./target/debug/porf`; the live sweep process (pid 2209, started 04:53) keeps the
old inode, but its *next* supervisor attempt will pick up the new binary. The
`baseline-wasm-aot-b2` snapshot therefore spans two compiler revisions. It only
matters for cases whose verdict the b7 semantic changes move (DTF subclassing,
async for-of diagnostics, RegExp runtime-table lookups); the 502-case
dynamic-code policy family is unaffected.

## Rung 1 — the two post-build probes the lane notes owed (measured)

Both are the discriminators the write lanes filed for the integrator/runner to
run *after* the build; both had never been run against a binary containing the
fix.

| probe | expectation on record | measured now |
|---|---|---|
| RE-RT §1, the six-line wrapped-call probe (`re-rt-b7-integration.md` line ~137). Literal `"(?<n>a)"` appears ONLY as a call argument inside a `function`, with a separate seed literal to keep `runtime_regexp_candidate_literals` non-empty | pre-fix `THREW TypeError: RegExp.prototype.exec unsupported pattern`; post-fix `true` | `(?<n>a)` then **`true`**, RC=0 |
| IR-SHAPES item 1, `porf inspect` on `class D extends Intl.DateTimeFormat {} new D();` | pre-fix `result=undefined` (that is the whole defect); post-fix `result=object` | **`result=object`**, with `class_extends=1 constructs=2 classes=1` |

The RE-RT probe is the stronger of the two: `(?<n>a)` is a named group, which the
`emit_regexp_exec_simple_from_locals` fallback matcher cannot execute, so `true`
can only come from a compiled program installed out of the runtime table at a
**function-wrapped** call site. That is the collection arm the lane added, and it
closes the residual uncertainty the RE-RT note flagged as "the first thing to
check after the build".

## Rung 2 — scheduling: the sweep was PAUSED at 05:54:13 UTC

State recorded before the kill, so the restart is auditable:
`baseline-sweep.log` at `=== supervisor attempt 52 ===`, mid-node at
`test262 checkpoint: 60/104 cases`; **27** node jsons in
`target/test262-scratch/baseline/` (up from the 23 the triage above counted).
`pkill -f sweep-supervisor.sh` then `kill 2209`; `MemAvailable` went 4 GiB → 10 GiB.
The attempt journal makes this safe — the in-flight cases of the killed node are
charged one strike each and re-selected on resume, and a node's json is written
only on completion, so nothing banked is lost.

First measured consequence of the pause: with the sweep live the prefix run was
also being pinned to CPUs 0-1, because `run-watched.sh` routes through
`capped.sh`, whose default is `PORFFOR_CPU_PERCENT=50` → `taskset -c 0-1`. Setting
it on the **outer** `run-watched.sh` invocation (as `frontier-driver.sh` does),
not inside the wrapped script, is what makes it `CPUs 0-3 of 4 (100%)`. The first
launch of this session's prefix chain had it inside and was killed and relaunched.

## Rung 3 — verify_prefix, counted before → after

Before-counts are read from the banked sweep snapshot
(`baseline-wasm-aot-b2-annexB-…json`, `…built-ins_Array_fromAsync-…json`) and the
FRONTIER node json; after-counts are fresh runs of this session's binary into
`target/test262-scratch/b7r2/`.

### RE-RT — `annexB/built-ins/RegExp/prototype/compile`

| | before (sweep baseline) | after (05:50-05:57, this binary) |
|---|---|---|
| total | 23 | **23** |
| passed | 22 | **23** |
| failed | **1** (`Bug`/`Runtime`) | **0** |

`duplicate-named-capturing-groups-syntax.js` — the single Bug-outcome failure the
whole RE-RT lane was written around, and one of only 3 Bug failures in the 4,220
banked sweep cases — now passes. Outcome histogram after: `Success 23`,
`NotImplemented 0`, `Crash 0`, `Bug 0`.

## Rung 4 — IR-SHAPES item 1: the DTF fixture, and the size question §1.5 owed

### The fixture, run as a real libtest (not a probe)

```
./target/debug/deps/cli-986abd5f02521ed6 --exact \
  date::run_wasm_backend_succeeds_for_intl_date_time_format_subclass_fixture
→ test result: ok. 1 passed; 0 failed; 611 filtered out; finished in 47.54s
```

Incidental but useful: `1 running + 611 filtered out` = **612 compiled tests**,
which is the number `docs/rust-rewrite/batch-workflow.md` states for 620
`#[test]` attributes. Recounted this session with the exact-line `awk`:
620 attributes across `tests/cli/*.rs`, and `language 45 + language_errors 29 +
language_numerics 31 = 105`, so the batch-7 three-way split preserves the
language total exactly.

### The size cost of the rooting change — MEASURED, closing `ir-shapes` §1.5

The lane deliberately did not measure this because the sweep and FRONTIER were
both live in its window. With the sweep paused it is three one-command sides.
`PORFFOR_EMIT_SIZE_REPORT_PATH` sink, one file per side, `porf build wasm`:

| probe | source | emitted fns | total bytes | `porffor::main` | `Intl.DateTimeFormat.*` bodies |
|---|---|---:|---:|---:|---|
| A | `class D extends Intl.DateTimeFormat {}` + `new D()` | 404 | **6,009,478** | 577,211 | 7 fns / 203,759 B |
| C | `new Intl.DateTimeFormat()` — names DTF, does **not** subclass | 403 | **6,003,673** | 574,528 | 7 fns / 203,759 B |
| B | `class L extends Intl.Locale {}` + `new L("en")` — the sibling heritage, never names DateTimeFormat | 404 | **6,009,561** | 577,294 | 7 fns / 203,759 B |

**The lane's falsifiable prediction holds, on all three of its clauses.**

* *"DTF bodies present in the size report on both sides"* — they are present in
  all three, byte-identical at **203,759 B across 7 functions**, including probe
  B, whose source never writes `DateTimeFormat`. That is `planning.rs`'s
  `INTL_NAMESPACE_ROOTS` rooting the whole namespace for any program naming
  `Intl`, exactly as predicted, so the constructor_instance change cannot be
  adding builtin bodies.
* *"call-site only"* — the only structural difference A vs C is **one** function,
  `js::D#f0` (the subclass's own constructor). `diff` over the name column
  returns that single line and nothing else.
* *"low kilobytes"* — A − C = **+5,805 bytes** (+0.097% of a 6.0 MB module), of
  which `porffor::main` is +2,683 B and the new `js::D#f0` body is the remaining
  3,122 B. And A vs B — the same subclass shape over a heritage that was already
  correct — differ by **83 bytes**, i.e. subclassing DTF now costs what
  subclassing `Intl.Locale` costs.

## Rung 5 — rung 1c chunks (sweep paused for the whole of it)

`./scripts/rung1c-chunks.sh`, driver log `target/watched/b7r2-rung1c-driver.log`.
The driver selected exactly the 8 chunks the integrator predicted: it skipped 13
banked-and-unchanged modules by name, re-ran `known_failures` unconditionally,
re-ran `date` and `regexp` on the counts guard (banked 17/33, now 18/35), and ran
the 5 with no done-file row (`string`, `functions`, `language`,
`language_errors`, `language_numerics`).

Partition arithmetic per chunk: `running N` + `N filtered out` must equal the
**612** compiled tests (`--list` total; 620 `#[test]` attributes on disk).

| chunk | ran | filtered out | sum | result | wall |
|---|---:|---:|---:|---|---:|
| `known_failures` | 5 | 607 | **612** | **ok, 5 passed** | 0.02 s |
| `date` | 18 | 594 | **612** | **ok, 18 passed** | 802.02 s |

`known_failures::` green is the ledger gate for item (D): all five hygiene tests
(`ledger_is_well_formed`, `rung_1c_chunks_cover_every_cli_area_module`,
`routing_takes_the_guarded_path`, `every_expected_failure_carries_a_should_panic`,
`every_ignored_test_is_declared`) pass at `CURRENT_BATCH = 7` and with the chunk
list at 20 `run_chunk` lines over 20 modules — i.e. the three-way `language`
split still partitions the suite.

`date` at 18 is the IR-SHAPES item-1 end-to-end evidence: it carries
`run_wasm_backend_succeeds_for_intl_date_time_format_subclass_fixture`, which is
the 18th test and the one the counts guard forced this re-run for.
| `regexp` | 35 | 577 | **612** | **ok, 35 passed** | 514.72 s |
| `string` | 36 | 576 | **612** | **ok, 36 passed** | 451.58 s |

`regexp` at 35 is the RE-RT lane's CLI gate: it carries both halves of the
runtime-pattern fixture pair
(`run_wasm_backend_succeeds_for_regexp_runtime_pattern_valid_fixture`,
`…_invalid_fixture`), and both are in the passing 35.
| `functions` | 45 | 567 | **612** | **ok, 45 passed** | 536.74 s |
| **`language`** | **45** | **567** | **612** | **ok, 45 passed** | 647.21 s |

**Item (B) is closed for the first of the three sub-chunks, and this is the
headline of rung 5.** `language::` had never produced a single-invocation
verdict: at 105 tests it OOM-SIGKILLed three times, and batch 6 could only reach
105/105 by *union* of a chunk run and a tail run. At 45 tests in one process it
banks in 647 s with peak `MemAvailable` never below 5 GiB (sampled through the
run: 6, 5, 5, 5, 5 GiB) on a box whose whole allowance is 15 GiB. No union, no
tail run, no `--skip`.
| `language_errors` | 29 | 583 | **612** | **ok, 29 passed** | 428.98 s |
| `language_numerics` | 31 | 581 | **612** | **ok, 31 passed** | 384.51 s |

`rung1c: all chunks done` at 07:08:56 UTC. **All 20 chunks now hold a verdict at
this head**, and the three `language_*` chunks each banked from ONE invocation.
Every one of the eight chunks that executed printed `sum = 612`, which is the
compiled-test total, so no chunk silently selected a subset. Zero failures, zero
ignored, across 244 executed tests.

Chunk-set arithmetic, recounted rather than cited: 20 `^run_chunk ` lines, 20
`tests/cli/*.rs` modules, 620 `#[test]` attributes by the exact-line `awk`, 612
compiled (`--list`). The 8-byte gap between 620 and 612 is unchanged from the
figure `batch-workflow.md` records.

## Rung 6 — a NEW scheduling fact, measured by an OOM kill (regression of my own making)

I launched `cargo test -p porffor-ir` and the resumed test262 prefix chain
concurrently, on the theory that the scheduling law only covers "the sweep vs
heavy CLI chunks". It does not, and the kernel said so:

```
oom-kill: ... task=porffor_ir-16cb, pid=7024
Out of memory: Killed process 7024 (porffor_ir-16cb)
  total-vm:9267376kB, anon-rss:9098260kB
error: test failed ... (signal: 9, SIGKILL: kill)
```

**`cargo test -p porffor-ir` alone is a ≥9.1 GiB job** at its default
`--test-threads` (4 on this box), for 645 tests. It is in the same weight class
as the `date::` chunk (11.48 GiB) and it does **not** coexist with a
`porf test262 report` node (~2-4 GiB), let alone with the sweep. Session 1's run
of the same command was killed by the container restart before it finished, so
this is the first time the crate's lib-test peak has been attributed.

Extend the scheduling law as measured: **sweep, heavy CLI chunks, AND
`cargo test -p porffor-ir` are mutually exclusive.** Re-run below, alone, with
`--test-threads=2`.

Partial result harvested from the killed run before it died (383 of 645 tests
printed, alphabetical through `i`): both of IR-SHAPES' new/changed unit cases had
already reported —

```
test tests::a_refused_generator_declaration_reports_its_yield_shape ... ok
test tests::allocates_disjoint_for_await_states_when_the_body_never_suspends ... ok
```

`tests::rejects_async_loop_awaits_with_no_resumable_shape` (the one carrying the
new positive array-walk case) sorts after the kill point and was NOT reached.

## Rung 7 — the IR-SHAPES anti-vacuity fixture, measured (it stays unwired, and now for a measured reason)

`crates/porffor-cli/tests/fixtures/wasm_async_for_of_closure_capture.js` exists on
disk with **zero** references anywhere in `crates/` or `scripts/` (grepped). That
is deliberate per `ir-shapes-b7-integration.md` §2.5, and I confirmed the reason
rather than taking it:

```
./target/debug/porf run --execution-backend wasm \
  crates/porffor-cli/tests/fixtures/wasm_async_for_of_closure_capture.js
→ unsupported in porffor wasm-aot first slice: async for-of with a body await
  cannot give the loop binding a fresh per-iteration environment record, and a
  closure in the body captures it; the iterable is an array and the head binds
  one plain name.
```

Two things are measured here at once. The fixture is genuinely **red** at this
head, so wiring it into `functions.rs` would have to be as a `should_panic` row —
i.e. it is correctly unwired for batch 7. And the refusal now names the premises
the classifier actually tested ("the iterable is an array and the head binds one
plain name") instead of the old text's false claim that the shape "requires an
array iterable and a plain binding" — which was wrong for exactly this input,
since this input *has* both. That message change is IR-SHAPES item 2's landed
half, and this is its end-to-end evidence.

## Rung 3 (continued) — `built-ins/Array/fromAsync`, in progress at 08:04 UTC

Resumed run (`b7r2-fromasync`, `--resume`), 80/95 completed. The node is far
slower per case than any other measured this session: `annexB/.../compile` ran
23 cases in 389 s (17 s/case) while `fromAsync` is averaging ~35 s/case with
bands of 9.5 minutes for ten cases, at 223% CPU and 7.6 GiB RSS in one process.
That is why it, not the CLI chunks, is the long pole of this session.

Sequencing cost recorded honestly: my first launch of this node (05:57-06:09) was
killed by me to free the box for the rung-1c chunks, which charged
`sync-iterable-with-thenable-then-method-err.js` and
`sync-iterable-with-thenable-sync-mapped-callback-err.js` one crash strike each
(`strike 1 of 2`, printed on resume). Neither reached the limit, so neither is
recorded as a false `Crash`; a third kill of the same cases would do exactly
that, which is why this run is being left alone to finish.

### IR-SHAPES — `built-ins/Array/fromAsync` (finished 08:07:13)

| | before (sweep baseline) | after (this binary) |
|---|---|---|
| total | 95 | **95** |
| passed | 93 | **93** |
| failed | 2 | **2** |
| outcome | `NotImplemented` ×2, one `detail_hash` | `NotImplemented` ×2, one `detail_hash` |
| `detail_hash` | `10438609855492019567` | **`10831672115949215379`** |

Counts identical, hash moved — which is precisely the claim IR-SHAPES made for
item 2: **the diagnostic landed, the semantics are filed.** Both records read out
of the node json's `failures[]` (not the `.txt`) are the two expected cases,
`asyncitems-asynciterator-not-callable.js` and `asyncitems-iterator-not-callable.js`,
and both now carry

> `async for-of with a body await cannot give the loop binding a fresh
> per-iteration environment record, and a closure in the body captures it; the
> iterable is an array and the head binds one plain name.`

This is the anti-vacuity point made in the negative: a lane that had "fixed" this
by hoisting one slot would have turned these two green and the message would have
disappeared. It did not, so nothing has been vacuously banked here. `Crash 0`,
`Bug 0` — the two strike-1 cases from my earlier kill completed normally.

### FRONTIER — `language/destructuring` (08:07-08:10)

| | before (FRONTIER banked node) | after (this binary) |
|---|---|---|
| total | 19 | **19** |
| passed | 17 | **17** |
| failed | 2 (`Bug`/`Runtime` 1, `NotImplemented`/`Unsupported` 1) | **2**, same split |

No movement, which is the wanted answer: it is the lane's smoke node and none of
batch 7's changes touch destructuring. It is the only one of the nine banked
FRONTIER nodes re-measured against the post-fix binary, so it is the regression
evidence for that lane's 758-case corpus, not a verdict on all of it.

## Rung 3 summary — every lane's verify_prefix, before → after

| lane | prefix | before | after | delta |
|---|---|---|---|---|
| RE-RT | `annexB/built-ins/RegExp/prototype/compile` | 22/23 | **23/23** | **+1 pass, the batch's only closed `Bug`** |
| IR-SHAPES | `built-ins/Array/fromAsync` | 93/95 | **93/95** | 0; `detail_hash` moved (diagnostic landed, semantics filed) |
| FRONTIER | `language/destructuring` | 17/19 | **17/19** | 0 |
| RUNG1C | `cargo test … known_failures::` | 5 passed | **5 passed** | 0, now at `CURRENT_BATCH = 7` |

Total across the three test262 prefixes: **133 cases, 133 → 133 attempted,
132 → 133 passed, 5 → 4 failed.** No case that passed before fails now.

## Rung 8 — sweep restarted at 08:22:11 UTC

`nohup ./target/test262-scratch/sweep-supervisor.sh &` — supervisor pid 26287,
worker pid 26291, `=== supervisor attempt 53 ===` in
`target/test262-scratch/baseline-sweep.log`, log growing. The pause cost exactly
what the journal design says it costs: two cases
(`built-ins/AsyncDisposableStack/prototype/disposeAsync/Symbol.asyncDispose-method-not-async.js`,
`built-ins/AsyncDisposableStack/prototype/defer/throws-if-onDisposeAsync-not-callable.js`)
were in flight at the kill and are charged `strike 1 of 2`; neither is at the
limit, so neither is recorded as a false `Crash`.

Total pause: **05:54:13 → 08:22:11, 2 h 28 min**, spent on 8 rung-1c chunks
(244 tests), 133 test262 prefix cases, the DTF size measurement and the
`porffor-ir` runs. The sweep now runs the **post-fix** binary, so the
`baseline-wasm-aot-b2` snapshot spans two compiler revisions from node
`built-ins/AsyncDisposableStack` onward.

## Rung 9 — `cargo test -p porffor-ir`, alone, `--test-threads=2`: 644/645, ONE RED

```
test result: FAILED. 644 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 2482.72s
failures:
    tests::lowers_array_spread_with_unshaped_concat_input_as_dynamic_elements
```

Peak was well inside the box at 2 threads (~4.1 GiB against 9.1 GiB at the
default 4), which is the setting to use from here.

The three cases the IR-SHAPES lane owed all passed, and they are the reason this
run mattered:

```
test tests::rejects_async_loop_awaits_with_no_resumable_shape ... ok      (line 784)
test tests::a_refused_generator_declaration_reports_its_yield_shape ... ok
test tests::allocates_disjoint_for_await_states_when_the_body_never_suspends ... ok
```

### The red one — REPORTED, NOT PAPERED OVER, and not attributable to batch 7

```
crates/porffor-ir/src/lib.rs:1420
  panicked: an unshaped concat input must not become an empty array shape
  source: let source = [].concat({ length: 1 }); let copy = [...source]; copy[0];
  assertion: copy_init.heap_shape.is_none()
```

Three things are measured about it, and the third is why I did not "fix" it:

1. **It is not new in batch 7 and no batch-7 lane touched array-literal spread
   shape inference.** The last completed `porffor-ir` run in `target/watched/` is
   `b2-ir-lib.log` (2026-08-09 07:41) where this test is `... ok`. Batches 3-6
   never completed a full crate run — every later log is a partial killed by a
   restart, including this session's first attempt. So the regression window is
   the whole of batches 3-6 plus 7, and this session is simply the first time
   anything ran far enough to see it.
2. **The runtime answer is correct.** Measured, `--execution-backend wasm`, on
   the test's own source extended with prints: `copy.length` → **1**,
   `typeof copy[0]` → **object**, `copy[0] === undefined` → false,
   `source.length` → **1**. So this is not the wrong-answer class the assertion's
   message fears; nothing miscompiles today.
3. **Which means the two available "fixes" are opposite and I cannot choose
   between them from outside.** Either the shape now attached to `[...source]` is
   a *sound improvement* and the assertion is stale, or the shape is unsound and
   only survives because no consumer reads it for length yet. Deleting the
   assertion because the program happens to run is exactly the vacuous pass this
   batch is supposed to catch. `array_concat_result_info` (`lowering.rs:28223`)
   is *not* the culprit — its `default` is `unshaped_array_result_info`
   (`lowering.rs:28193`), which returns `heap_shape: None`, and a non-array
   argument takes that path. The shape is being introduced by the array-literal
   spread lowering downstream of it.

**Owner: the `porffor-ir` shape-inference owner. Reason: needs a decision on
whether `[...unshaped]` may carry an `ArrayShape`, which is a soundness question
about every consumer of `heap_shape`, not a test edit.**

## Rung 10 — close

* `cargo xc` at close: **EXIT=0**, `Finished dev profile in 0.68s`, warning set
  unchanged from the fixer's.
* Sweep restarted for the last time at **08:56:05 UTC** — supervisor pid 7255,
  worker pid 7259, `baseline-sweep.log` growing.
* `target/watched/rung1c-done` now carries **20** rows — every chunk in the
  partition, including all three `language_*` — and
  `target/lane-notes/rung1c-chunks.md` ends `ALL CHUNKS DONE 2026-08-11T07:08:41Z`.

### What remains, with an owner and a reason

| item | owner | why it is not done |
|---|---|---|
| `porffor_ir::tests::lowers_array_spread_with_unshaped_concat_input_as_dynamic_elements` | `porffor-ir` shape-inference owner | soundness decision about `heap_shape` on `[...unshaped]`, not a test edit (rung 9) |
| FRONTIER tier 1 tail: `language/statements/let` (145 cases) unrun, `language/block-scope` banked PARTIAL at 120/145 | FRONTIER lane | the driver is resumable and was not restarted this session; the box was spent on rung 1c and the 133 prefix cases |
| FRONTIER tier 2 (792 cases: `switch` 111, `try` 201, `function-code` 217, `arguments-object` 263) | FRONTIER lane | gated on tier 1 completing |
| RE-RT F4/F7 deltas: `built-ins/RegExp/prototype` (487 cases) for the false-`InvalidSyntax` risk, emit-size attribution for the +12.5% row growth | RE-RT lane | 487 cases at this node's measured rate is a multi-hour run; it does not fit beside a sweep restart |
| the second Bug family, `built-ins/Array/prototype/toLocaleString/primitive_this_value{,_getter}.js` (2 cases, `detail_hash 4872481042840822955`) | UNOWNED — filed in the item (A) triage above | no batch-7 lane covers it; it is 2 of the 7 addressable failures in the 4,220-case baseline |
| wiring `wasm_async_for_of_closure_capture.js` | IR-SHAPES / batch 8 | measured red (rung 7); wiring it needs a ledger row + `should_panic`, which is RUNG1C's file |
