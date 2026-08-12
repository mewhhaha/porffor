# b5 RUNNER findings

Start 2026-08-10 ~09:30 UTC. HEAD `1939975ad`. Branch claude/test-driven-rust-opus-pp6giw. 4 CPU / 15 GiB.
LILA_CPU_PERCENT=100 used throughout.

## Rung 0 — cargo xc
`LILA_CPU_PERCENT=100 ./scripts/run-watched.sh --label b5r-xc --stall 900 -- cargo xc` -> **EXIT=0**, 9.95 s
(warm). Warnings: aot-wasm lib 25 / lib-test 20, test262 lib-test 1. Matches the fixer's reported counts exactly.
No release `lila` exists (still true from batch 4); all numbers below use `target/debug/lila`.

## Rung 0b — debug `lila` rebuild (prerequisite, not an optimisation)
`target/debug/lila` was STALE again: **17** `crates/**/*.rs` newer than the binary, including all three lanes'
files (`lila-test262/{lib,attempt_journal}.rs`, `lila-aot-wasm/{functions,objects,heap}.rs`, 6 `lila-ir`
files, 5 `lila-cli/tests/cli` files). Rebuilt: `cargo build -p lila-cli --bin lila` -> **Finished in 1m43s**,
new binary 150,251,472 bytes at 09:05:03Z (was 149,892,272 at 04:07:54Z). Every `lila` number below is post-rebuild.

## Rung 1 — `cargo test -p lila-test262 --lib`  [the fixer's #1 risk: all six unrun tests now RUN]
`LILA_CPU_PERCENT=100 run-watched --label b5r-t262lib --stall 900 -- cargo test -p lila-test262 --lib`
-> **278 passed / 1 failed / 0 ignored / 0 filtered out**, 161.25 s. Log `target/watched/b5r-t262lib.log`.

The whole quarantine surface is GREEN, by name (counted from the log, not the lane note):
- `attempt_journal::tests::` 8/8 ok, including the fixer's new
  `attempt_journal_retire_forgives_the_strike_of_a_case_that_then_completed`.
- `tests::a_case_left_in_the_attempt_journal_is_charged_a_strike_and_forgiven_when_it_completes` ok
- `tests::a_quarantined_case_keeps_the_node_snapshot_resumable` ok
- `tests::an_empty_journal_is_the_normal_exit_state` ok
- `tests::execute_cases_runs_the_suspect_phase_serially_and_first` ok  (the rewritten two-suspect test)
- `tests::the_child_runner_path_journals_exactly_once` ok
- `tests::the_attempt_journal_sits_beside_its_snapshot_and_is_not_a_json_file` ok
- `tests::a_case_at_the_strike_limit_is_recorded_as_crash_and_not_run` — present at lib.rs:36314, ran ok
- All four EXISTING must-stay-green: `execute_cases_resume_reuses_case_checkpoint_snapshot`,
  `execute_cases_first_run_writes_periodic_checkpoint`, `execute_cases_runs_wasm_aot_cases_on_persistent_workers`,
  `execute_cases_resume_child_runner_enforces_preemptive_timeout` — all ok.
The lane's `verify_prefix` (`-- attempt_journal quarantine execute_cases_ validate_resume`) does NOT select
`a_quarantined_case_...` or `an_empty_journal_...` or `the_child_runner_path_...`. Run the whole `--lib` target.

### The one red, and it is NOT a batch-5 regression
`tests::typed_array_literal_helper_contract_covers_all_319_vendored_bodies`, lib.rs:27804:
`assert_eq!(representative_source_bytes, (2_930, 17_298))` -> measured `(2_930, 17_633)`.
Only the second member moved (+335 bytes): the materialised source of
`built-ins/TypedArray/prototype/find/predicate-may-detach-buffer.js`. `/some/length.js` is exact.
Evidence it predates batch 5, not argued but read off the diff: `git diff 77505818a..HEAD -U0 --
crates/lila-test262/src/lib.rs` has exactly two hunks below line 20000 and both are `use` lines
(`@@ -4,0 +5 @@`, `@@ -19,0 +21,6 @@`). `materialize_test`/`load_preludes`/the `detachArrayBuffer.js`
fingerprint (lines ~2169-2241, ~3713-3733) are untouched by this batch. The vendored harness is untouched too
(`test262/vendor/test262/harness/*` all mtime Aug 8 15:35, `git status test262/` clean,
`test262/harness-wasm-aot.js` last changed in `682d200b1`). The constant was written in `885b924b1` ("smth smth")
and this lib target has not been run since — batch 4's `b4r-unit.sh` ran only `-- detail_hash` (4 passed,
259 filtered out), so nobody has executed it. **Owner: integrator. It is a stale locked constant or a real
prelude drift; do not attribute it to any batch-5 lane.**

## Rung 3 — THE SWEEP (lead A). It is ALIVE: the poison node completed 250/250.
Relaunched verbatim: `setsid nohup target/test262-scratch/sweep-supervisor.sh` at 09:11:52Z, post-rebuild.
Prior state (batch 4): 40 consecutive `exit 134` "fatal runtime error: stack overflow", every one of them
dying inside the node that had reached **230/250** cases; supervisor budget fully burnt, zero cases banked.

Measured now, `target/test262-scratch/baseline-sweep.log`:
- `=== supervisor attempt 1 09:11:52 ===`, then the resumed node ran straight to **`test262 checkpoint:
  250/250 cases`** — past the 230/250 death point — and the sweep moved on to the NEXT node
  (`10/250 ... 50/250`) inside the same process.
- **Zero `exit 134`. Zero `test262 quarantine:` lines. One `lila test262 report-all` process alive.**

Read that carefully, because it decides ownership of lead A's two deliverables:
1. The **recursion fix landed and is sufficient** — the case that killed 40 processes now COMPLETES. The
   quarantine never had to fire, so its end-to-end arm is unexercised by this run *and did not need to be*.
2. The **quarantine is proven at the unit level instead** (12 tests, listed under Rung 1, all green), which is
   the right level: it is a supervisor-convergence property, not a conformance property. Do not claim an
   end-to-end quarantine observation that did not happen — the honest statement is "the sweep no longer needs it".
There was also no `.attempts` journal to inherit: the 40 batch-4 deaths predate the journal, so attempt 1 was a
clean re-attempt of the case rather than a strike-charged one. The first genuine end-to-end quarantine evidence
will come from the next process death, whenever that is.

## Rung 1b — ITERATOR (lead B). The lane did NOT fix it, and the defect is NOT what the brief says.
All runs `./target/debug/lila run --execution-backend wasm <file>`, post-rebuild binary, one process each.

### The four lane fixtures are still red, with their labels now recoverable
`crates/lila-cli/tests/fixtures/wasm_iterator_prototype_{some,every,find,reduce}.js` each end
`uncaught throw: wasm-aot completion: string(callback throw)` / `string(reducer throw)` — i.e. the fixture's own
`throw "callback throw"` guard at `some.js:100`, reached because `callbackThrew` stayed false. So
`iterator::run_wasm_backend_succeeds_for_iterator_prototype_{some,every,find,reduce}_fixture` are still FAILING.

### The real defect, narrowed to ONE source-level variable by a controlled A/B
The brief (and the lane) call this "silently discards a callback throw". It is worse and more specific: **the
helper is not executed at all, and returns a value of the wrong TYPE.** The discriminator is not the receiver's
kind, not the call frame, and not the key form — it is **whether the `class X extends Iterator` declares an
explicit constructor.** Identical `next()` bodies, one variable:

| receiver | `.some(cb)` returns | typeof | callback invocations |
|---|---|---|---|
| `class NoCtor extends Iterator { next(){...} }` | `0` | **number** | **0** |
| `class WithCtor extends Iterator { constructor(){super();} next(){...} }` | `true` | boolean | 2 |

`NoCtor.forEach(cb)` on the SAME instance calls back 3 times and returns `undefined` — correct. So the instance
is fine; the defect is confined to the `some/every/find/reduce` emission block.

Full no-constructor table (`n` closure counter, `next()` yields 1,2,3 then done):
`some ret=0 t=number calls=0 | every ret=0 t=number calls=0 | find ret=0 t=number calls=0 |
 reduce ret=0 t=number calls=0 | forEach ret=undefined t=undefined calls=3`
With `constructor(){super();this.n=0;}` and `this.n`, ALL FIVE are correct:
`some ret=true/2 calls | every ret=true/3 | find ret=2/2 | reduce ret=6/3 | forEach undefined/3`.

`typeof (new NoCtor().some)` is `"function"` — the method resolves; it is the CALL that does nothing.
With no `try`/`catch` anywhere, `new NoCtor().some(boom)` evaluates to a **string containing the class's own
source text** (`"class C extends Iterator { next() { ... } }"`). A wrong-typed value drawn from unrelated memory
is a corruption-class wrong answer, not a lost exception. Rank it accordingly.

### The lane's differential test WILL be red, and it is right to be
`x.some(cb)` (static key) -> `NOT-CAUGHT ret=undefined`;  `x['some'](cb)` (computed key) -> `CAUGHT`.
The computed-key generic tail is the correct oracle exactly as the lane claimed. The fast path is still wrong.
Two more measured forms, both wrong, both static-key: receiver held in a `let` (`recv.some(boom)`), and receiver
arriving through a function parameter (`function f(x){return x.some(boom);}` — a `Dynamic` receiver). So the
fast path's selection is NOT limited to a statically-known Iterator subclass.

### One frame-shaped caveat the fixer should not trip over
`try { new NoCtor().some(boom) } catch` **inside a function body** catches correctly; the same `try` in the
top-level script frame does not. That is a consequence of the above (in the function-frame probe the throw
crossing the boundary is the callback's, from a differently-emitted call site), not a second defect — but it
means any repro written inside a helper function will falsely look green. Write repros at top level.

### `take`/`toArray` — the lane's fourth new test
`new NoCtor().take(1).toArray()` -> **`TypeError: value is not callable`**, as the lane predicted. Also
`new NoCtor().map(cb).toArray()` -> `uncaught throw: TypeError: ... value is not callable`. So `toArray` after a
lazy helper is broken independently of the some/every/find/reduce block.

---

# b5 RUNNER — session 2 (container restart recovery)

Resumed 2026-08-10 ~09:44 UTC at HEAD **`05c3b010d`** ("Theory-first round 4: Property Descriptor
lattice, IteratorClose obligations"), i.e. **two commits past the `1939975ad` all the measurements above
were taken at**. Session 1 was killed by the hourly container restart at ~09:42 UTC.

## Rung 0 (re-measured at the NEW head, because r4 touched the compiler)
`LILA_CPU_PERCENT=100 run-watched --label b5r2-xc --stall 600 -- cargo xc` -> **EXIT=0 in 15 s**.
r4's diff vs `1939975ad` is `lila-aot-wasm/{heap,objects}.rs` + 4 `lila-ir` files (+266/-71), so
this was not a formality. `target/debug/lila` is CURRENT: `find crates -name '*.rs' -newer
target/debug/lila` = **0** files (binary 150,251,488 B, 09:28Z). No rebuild needed this session.

## THE SCHEDULING CONSTRAINT that cost session 1 two chunks — read this before launching anything
Session 1 ran the sweep (`--threads 2 --jobs 2`) and `rung1c-chunks.sh` (`--test-threads=3`)
**concurrently on 4 CPUs / 15 GiB**. Both rung-1c chunks that ran under that overlap died:

- `rung1c-dynamic`: `EXIT=101`, `process didn't exit successfully: ... (signal: 9, SIGKILL: kill)`
  after 5 of 11 tests, 4 of them simultaneously "running for over 60 seconds".
- `rung1c-heap`: killed mid-chunk at the container restart, 3 tests in flight.

`SIGKILL` with no `test result:` line is **not** the stall guard (that reports 124) — it is the OOM
killer. Five concurrent cold Wasm-AOT compiles do not fit in 15 GiB. The script did the right thing
and refused to bank either chunk (the fixer's finding #18 fix, working on its first real outing).
**Do not run the sweep and rung 1c at the same time on this box.** I am running rung 1c alone.

## Lead C — ZDT era boundary: REPRODUCED, exact label
`./target/debug/lila test262 run intl402/Temporal/ZonedDateTime/prototype/add/era-boundary-gregory.js
--execution-backend wasm` ->
`[Bug:Runtime] [origin:boa-runtime] uncaught throw: TypeError: wasm-aot completion:
object(handle@1827888: value is not callable)` — the brief's `1827888` add/subtract handle, exactly.
Scope correction for whoever fixes it: this is **not 4 files**. Each of `add`, `subtract`, `since`,
`until` carries **7** era-boundary files (ethiopic, gregory, islamic-civil, islamic-tbla,
islamic-umalqura, japanese, roc) = **28 cases**, counted with `ls`, not estimated.

## Lead A — the poison case is NAMED, by the quarantine journal, on its first real outing
`test262/snapshots/latest-8842212995299038775.attempts` (committed into the tree by the mid-session
checkpoint) contains:
```json
{"version":1,"in_flight":[{"worker_slot":0,
  "case_path":"intl402/Temporal/ZonedDateTime/prototype/subtract/era-boundary-gregory.js"}],
 "strikes":{}}
```
This is the artefact batch 4 spent 40 supervisor attempts failing to produce. Two honest caveats:
1. The process that left it was killed by the **container restart**, not by a stack overflow — the
   sweep log ends mid-node at `70/250` with no `=== exit 134 ===` and no supervisor attempt 2.
   So this entry is a container-kill, and one strike for it is the design working (a bystander is
   forgiven on completion).
2. It is nevertheless the same *family* as lead C's defect and the same subtree the sweep was in.
   Whether the stack-overflow case is one of these 28 is measured below, not asserted here.

## Lead C — the era-boundary quartet, all four measured, both handles confirmed
One `lila test262 run --execution-backend wasm` per case, one process each, at HEAD `05c3b010d`:

| case | outcome | label |
|---|---|---|
| `.../add/era-boundary-gregory.js` | Bug:Runtime | `object(handle@1827888: value is not callable)` |
| `.../subtract/era-boundary-gregory.js` | Bug:Runtime | `object(handle@1827888: value is not callable)` |
| `.../since/era-boundary-gregory.js` | Bug:Runtime | `object(handle@1879624: value is not callable)` |
| `.../until/era-boundary-gregory.js` | Bug:Runtime | `object(handle@1879624: value is not callable)` |

The brief's handle pairing is exact: add/subtract share `1827888`, since/until share `1879624`.
`origin:boa-runtime` on all four. **`Crash: 0` on all four** — so whatever kills the sweep process,
it is not these, and lead C is a plain wrong-answer defect.

Cost, measured because it changes what the sweep can be expected to do: **~300 s per case**
(`subtract` 09:48:47->09:53:49, `since` 09:53:49->09:59:30, `until` 09:59:30->10:04:29), against
`add`'s ~60 s in a separately warmed process. A 250-case Temporal node at that rate is hours.

## Lead B — iterator: RE-MEASURED at `05c3b010d`, still red, unchanged by theory r4
Theory round 4 landed `IteratorClose as an obligation ...` and touched `lila-aot-wasm/objects.rs`
between session 1's measurements and now, so this needed re-running rather than quoting.
`./target/debug/lila run --execution-backend wasm crates/lila-cli/tests/fixtures/wasm_iterator_prototype_<h>.js`:

| fixture | rc | output |
|---|---|---|
| `..._some.js` | 1 | `uncaught throw: wasm-aot completion: string(callback throw)` |
| `..._every.js` | 1 | `uncaught throw: wasm-aot completion: string(callback throw)` |
| `..._find.js` | 1 | `uncaught throw: wasm-aot completion: string(callback throw)` |
| `..._reduce.js` | 1 | `uncaught throw: wasm-aot completion: string(reducer throw)` |

Identical to session 1's labels. **The four `iterator::run_wasm_backend_succeeds_for_iterator_prototype_*_fixture`
tests are still FAILING at the current head.** Session 1's A/B (the discriminator is whether the
`class X extends Iterator` declares an explicit constructor; `.some()` returns `0`/number with zero
callback invocations) stands as the characterisation — nothing in r4 moved it.

Deliberately NOT run: `lila test262 run built-ins/Iterator/prototype/{some,every,find,reduce}`.
Started, then killed after ~1 min. It answers "how many test262 cases are affected", which is a
sizing question, and it competes for the same 4 CPUs as rung 1c, which is the deadline item. Owner:
next runner, once rung 1c is banked. This is a deliberate omission, not a silent one.

### One iterator hypothesis CHECKED AND DISPROVED — do not spend the round on it
`emit_iterator_prototype_helper_method_call` (`functions.rs:8878`) builds the callee key with
`self.strings.payload(helper.property_name())`, while the neighbouring
`emit_object_read_number_slot_to_i64_local` (`objects.rs:2572`) builds its key with
`self.strings.static_builtin_property_key_payload(key)`. Two different accessors for the same
argument slot of the same `emit_object_read` is exactly the shape of a payload-namespace mismatch,
and a mismatched namespace would produce session 1's measured symptom (a value drawn from unrelated
memory that reads as the class's own source text).

It is not the defect. `data.rs:3988`: `static_builtin_property_key_payload` returns
`property_key_symbol_payload(value)` **only** when `value.starts_with("Symbol.")` and otherwise
returns `self.payload(value)` verbatim. All ten `IteratorHelper::property_name()` values are plain
ASCII names, so the two calls are the same i64. Structurally the emission also looks right: the
`[[Receiver]]` pair is the object pair (the same idiom `emit_object_read_number_slot_to_i64_local`
uses), the callee is read into its own local pair, and the destination is written last.

Reported as a negative because the next agent will otherwise find the asymmetry and chase it.

## Rung 1c — chunked run, banked verdicts (script `scripts/rung1c-chunks.sh`, run VERBATIM)
`git check-ignore -v scripts/rung1c-chunks.sh` -> **exit 1** (tracked, not swallowed by the bare
`*.txt` rule). Compiled test count is **607**, not the 593 the brief and `batch-workflow.md` still
say — the new `iterator_helpers` module added 14. Every chunk's `ran + filtered_out` is checked
against 607 below; that arithmetic is the proof the chunked run is a complete rung 1c.

| chunk | exit | ran | filtered | sum | result | wall |
|---|---|---|---|---|---|---|
| `known_failures` | 0 | 5 | 602 | **607** | 5 passed | 0.02 s |
| `throw_propagation` | 0 | 2 | 605 | **607** | 2 passed | 45.8 s |
| `dynamic` | 0 | 11 | 596 | **607** | 11 passed | 156.8 s |
| `heap` | 0 | 12 | 595 | **607** | 11 passed, 1 ignored | 100.7 s |
| `date` | 0 | 16 | 591 | **607** | 16 passed | 783.3 s |
| `iterator` | 101 | 30 | 577 | **607** | **26 passed, 4 FAILED** | 652.3 s |

`known_failures` is green **before** the `CURRENT_BATCH` bump, which is the vacuous state the lane
exists to close: `known_failures.rs:137` still reads `CURRENT_BATCH: u32 = 3`, the ledger header
still reads `# unfilled-allowed-until: batch-4`, and the assertion is
`CURRENT_BATCH < ledger.unfilled_allowed_until` — so 3 < 4 passes today and 4 < 4 fails the moment
anyone bumps it while the `UNFILLED` row is alive.

### Lead B — the four iterator failures, named and with their messages recoverable
`iterator::run_wasm_backend_succeeds_for_iterator_prototype_{some,every,find,reduce}_fixture`,
`iterator.rs:{387,411,435,459}`. The fixer's finding-#14 change earned itself immediately: the panic
now carries the child's streams, so the failure is
`stdout= stderr=uncaught throw: wasm-aot completion: string(callback throw)` (`string(reducer throw)`
for reduce) instead of a bare `assertion failed`. **These are the only red in 96 tests measured so
far**, and they are the fixture's own `throw "callback throw"` guard firing because `callbackThrew`
stayed false — i.e. the lane did NOT flip them green.

### Lead B is THREE defects across SEVEN helpers, not one defect across four
The lane's new `iterator_helpers` module is the most valuable thing measured this batch: **13 ran,
4 passed, 9 FAILED** (`sum = 13 + 594 = 607`). It does exactly what it was written to do — the four
pre-existing fixtures cover four helpers, and the brief's framing ("some/every/find/reduce silently
discard a callback throw") survives contact with none of it.

Correct on a `class X extends Iterator` receiver with NO explicit constructor (4):
`drop`, `flatMap`, `forEach`, `toArray`.

Broken (7 helpers + 2 structural tests), grouped by symptom, which is the grouping a fix lane wants:

**(i) `TypeError: value is not callable`** — the callee acquisition genuinely fails:
- `filter` `object(handle@1483824: ...)`
- `map` `object(handle@1483832: ...)`
- `take` `object(handle@1485040: ...)`
- `chains_take_and_to_array` `object(handle@1479696: ...)`
Note `1483824` and `1483832` are **8 bytes apart** — adjacent slots of one table, which is a much
sharper lead than the symptom alone. (These are the same *shape* of label as lead C's
`handle@1827888` / `1879624`, which is worth someone checking for a common cause.)

**(ii) silent wrong-typed value, callback never invoked** — the corruption class:
- `some`  `string(type-object;value;calls-0;caught-no-throw;throw-calls-0;)`
- `every` `string(type-object;value;calls-0;all-value;all-calls-0;caught-no-throw;throw-calls-0;)`
- `find`  `string(type-object;value;calls-0;missing-type-object;missing-calls-0;caught-no-throw;throw-calls-0;)`
`calls-0` and `caught-no-throw` in the same string is the whole story: the callback is never called,
so there is no throw to discard. "Silently discards a callback throw" is a mis-description of the
defect — nothing is discarded because nothing is invoked. `type-object` also refines session 1,
which measured `typeof` as `number` for `.some()`; the wrong type is not stable, consistent with a
value read from unrelated memory rather than a wrong-but-deterministic result.

**(iii) hard Wasm trap** — strictly worse than a wrong answer, and previously unreported:
- `reduce` `wasmtime execution trapped: error while executing at wasm backtrace:`
- `gives_identical_results_for_static_and_computed_helper_keys` (the differential) — same trap.
The differential test is red, and correctly so: session 1 measured `x.some(cb)` and `x['some'](cb)`
diverging. It now traps rather than merely diverging.

Owner: the iterator lane. Three symptom classes, at least seven helpers; scope this as a lane, not a
one-line fix. Do NOT write ledger rows for these expecting the current messages to be stable — (ii)
is reading unrelated memory and its exact string can move without the defect changing.

### Rung 1c continued (all green, all sum=607)
| chunk | exit | ran | filtered | result | wall |
|---|---|---|---|---|---|
| `regexp` | 0 | 33 | 574 | 33 passed | 503.7 s |
| `object` | 0 | 35 | 572 | 35 passed | 541.8 s |
| `string` | 0 | 36 | 571 | 36 passed | 528.8 s |
| `data_view` | 0 | 38 | 569 | 38 passed | 314.7 s |
| `iterator_helpers` | 101 | 13 | 594 | **4 passed, 9 FAILED** | 128.4 s |

11 of 17 chunks banked, **231 of 607 tests measured**, 13 red and all 13 in the two iterator
modules. Remaining: `functions`, `frontend`, `typed_array`, `array`, `language`, `binary_data`.

---

# b5 RUNNER — session 3 (second container restart recovery)

Resumed 2026-08-10 12:59 UTC at HEAD **`37893ef93`** ("WIP checkpoint: batch 5 runner verifications
continue"). Sessions 1 and 2 above were each killed by the hourly restart. Machine idle on arrival:
**0 `lila`, 0 `cargo`, 0 sweep processes**, 14 GiB of 15 free, 4 CPUs.

## Rung 0 — `cargo xc` at `37893ef93`  [GREEN]
`LILA_CPU_PERCENT=100 run-watched --label b5r3-xc --stall 600 -- cargo xc` -> **EXIT=0 in 15 s**
(log `target/watched/b5r3-xc.log`). `lila-aot-wasm` lib 25 warnings, unchanged.

## Rung 0b — the debug binary is CURRENT; no rebuild this session
`find crates -name '*.rs' -newer target/debug/lila` -> **0 files**. Binary 150,251,488 B, 09:28Z.
The two commits since session 2's measurements (`25f4894f8`, `37893ef93`) touch only
`test262/snapshots/*.json` and `*.attempts` — **no `crates/**` file**, verified with `git log --stat`.
So every session-2 `lila` measurement above is still at the current compiler and does not need re-running.

## Rung 1c — resumed. State inherited, and the ONE chunk that had died
`target/watched/rung1c-done` carries **12 banked chunks** (`known_failures throw_propagation dynamic
heap date iterator iterator_helpers regexp object string data_view functions`). Session 2's note stops at
11; `functions` completed after it was written:
`functions EXIT=0 ran=45 filtered_out=562 sum=607  45 passed`, 542.18 s (11:11:36 -> 11:20:51Z).

**`frontend` produced NO VERDICT and was correctly not banked.** `frontend EXIT=101 ran=46
filtered_out=0 sum=46`, and the log's last lines are the OOM signature, not the stall guard:

```
test frontend::run_wasm_backend_succeeds_for_supported_param_fixture has been running for over 60 seconds
test frontend::test262_run_exits_unsuccessfully_when_a_case_fails has been running for over 60 seconds
test frontend::test262_wasm_backend_runs_supported_fixture_subset has been running for over 60 seconds
error: test failed ... (signal: 9, SIGKILL: kill)
```

39 of its 46 tests had already printed `ok`. `typed_array` then started at 11:32:03Z and was cut off by
the container restart 3 tests in. So the remaining set is **5 chunks / 331 tests**: `frontend`(46),
`typed_array`(58), `array`(84), `language`(105), `binary_data`(38).

Relaunched `./scripts/rung1c-chunks.sh` verbatim at 13:02Z (log `target/watched/b5r3-rung1c-driver.log`).
It skipped all 12 banked chunks and entered `frontend` — the resume worked, and the fixer's
finding-#18 "bank only on a verdict" change is what made the retry possible at all: under the batch-4
script `frontend` would have been banked on EXIT=101 and skipped forever with 0 of its tests measured.

### DELIBERATE SCHEDULING DECISION: the sweep stays DOWN while rung 1c runs
The brief ranks the sweep restart above rung 1c. I am inverting that, and this is the reason, measured
rather than argued:
- Session 2 measured `rung1c-dynamic` and `rung1c-heap` **both killed by the OOM killer** (SIGKILL, no
  `test result:` line — the stall guard reports 124, not 9) while the sweep held 2 of 4 CPUs.
- `frontend` has now ALSO been SIGKILLed, with 4 tests concurrently over 60 s.
- 5 concurrent cold Wasm-AOT compiles do not fit in 15 GiB. `free -g` on arrival: 14 GiB free, idle box.
- Lead A's *conformance* question is already **answered** (session 1: the poison node ran 250/250, zero
  `exit 134`). Restarting the sweep now buys sweep progress on a ~15 h job that cannot finish in this
  window, at the cost of re-killing the deadline item.
So: **rung 1c runs alone.** The sweep is a hand-off with a one-line relaunch, recorded in REMAINING.

## Sizing answers banked WITHOUT spending CPU (counted with `ls`, not run)
Session 2 deferred `lila test262 run built-ins/Iterator/prototype/...` as a sizing question competing
with rung 1c. The blast-radius bound is a file count, so it costs nothing:

| helper | test262 cases | measured CLI verdict (session 2) |
|---|---|---|
| some 33, every 33, find 32, reduce 30 | 128 | all 4 broken (3x silent wrong value, reduce traps) |
| map 36, filter 37, take 33 | 106 | all 3 broken (`value is not callable`) |
| flatMap 44, drop 34, forEach 27, toArray 18 | 123 | all 4 correct |
| | **357 across the 11 helpers; 373 in `built-ins/Iterator/prototype` overall** | |

**234 test262 cases sit under a helper measured broken.** That is an upper bound on the blast radius,
not a failure count — it is what a fix lane is allowed to claim it might move, and it is 3.6x the
"4 helpers" the brief scopes lead B to.

`intl402/Temporal/ZonedDateTime/prototype/{add,subtract,since,until}/era-boundary-*.js` recounted:
**7 + 7 + 7 + 7 = 28 cases**, confirming session 2's correction to the brief's "4 files".

## Lane-3 precondition re-verified at this head
`git check-ignore -v scripts/rung1c-chunks.sh` -> **exit 1**, and `git ls-files --error-unmatch` says
TRACKED. The bare `*.txt` on `.gitignore` line 3 has not eaten it.

## Compiled-test-count arithmetic, recounted at this head
`awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++}' crates/lila-cli/tests/cli/*.rs` = **615**;
`frontend.rs` carries **8** `spec-exec-oracle` gates; **615 - 8 = 607**, which is what every banked
chunk's `ran + filtered_out` sums to. `docs/rust-rewrite/batch-workflow.md` and the T03 ledger row both
still say 593 — stale by the 14 tests the `iterator_helpers` module added.

---

# b5 RUNNER — session 4 (third container restart recovery)

Resumed 2026-08-10 13:52 UTC at HEAD **`37893ef93`** (unchanged from session 3 — no commit landed while
session 3 was down). Machine on arrival: **0 `lila`, 0 `cargo`, 0 sweep processes**, 14 GiB of 15 free.
Session 3 was killed by the container restart at **13:06Z**, i.e. its window was only ~4 minutes of chunk
time. `target/watched/rung1c-done` still carries the same **12 banked chunks**.

## What session 3's dying `frontend` chunk actually measured (read off the log, not inferred)
`target/watched/rung1c-frontend.log` (13:06Z) ends with **45 of 46 tests printed `ok`** and the 46th,
`frontend::test262_wasm_backend_runs_supported_fixture_subset`, "has been running for over 60 seconds".
The driver log stops at the same instant with no `rc`, so this was the **container restart**, not the
OOM killer (session 3's earlier 11:2x attempt was OOM: four tests concurrently over 60 s + `signal: 9`).
Two useful facts fall out and neither costs a run:
- `frontend` is not intrinsically red. Its 45 completed tests were all `ok`, including
  `frontend::inspect_reports_phase_eighteen_global_ir_shape` — the test the T03 ledger row and the
  `CURRENT_BATCH` doc comment both still describe as asserting `global_bindings=64` against a measured 65.
  **The integrator's one-token fix landed and is measured green.** That sentence of the T03 row is now
  stale in the good direction.
- The chunk is ~330 s of which one test is >60 s; a container window of 5 minutes cannot bank it, and
  three consecutive windows have now failed on exactly this chunk. It is the resume's chokepoint.

## Rung 1c relaunched IMMEDIATELY on arrival (13:53Z), alone
`setsid nohup ./scripts/rung1c-chunks.sh > target/watched/b5r4-rung1c-driver.log` — verbatim, no flags.
It skipped the 12 banked chunks and re-entered `frontend`. **The sweep stays down**, for session 3's
measured reason (5 concurrent cold Wasm-AOT compiles do not fit in 15 GiB; the sweep holding 2 of 4 CPUs
is what SIGKILLed `dynamic`, `heap` and `frontend`). Nothing else CPU-bound runs in this session.

## Ledger state re-read at this head (cheap, no CPU)
- `known_failures.rs:137` `const CURRENT_BATCH: u32 = 3;`
- `crates/lila-cli/tests/known-failures.tsv:41` `# unfilled-allowed-until: batch-4`
- The `UNFILLED` row is `tsv:67`, owner **T03**. Four real rows follow it (`binary_data::...atomics_wait`
  hang/T17, `heap::...page_boundary_stress` ignored/T05, two `perf` ignored/T25).
- Assertion at `known_failures.rs:1242` is `CURRENT_BATCH < ledger.unfilled_allowed_until`, so `3 < 4`
  passes today and `4 < 4` reddens the moment anyone bumps while row 67 is alive. Unchanged, still vacuous.

## Partition arithmetic, recounted per module at this head (pure `awk`, no CPU cost)
`awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++}' crates/lila-cli/tests/cli/*.rs` = **615**;
`frontend.rs` carries **8** `cfg(feature = "spec-exec-oracle")` gates; **615 - 8 = 607** executing.
Per module, which is what makes the resume auditable instead of a single opaque total:

| module | tests | banked? |
|---|---|---|
| language | 105 | no |
| array | 84 | no |
| typed_array | 58 | no |
| frontend | 54 compiled / **46** executing | no (in flight) |
| functions | 45 | yes |
| data_view | 38 | yes |
| binary_data | 38 | no |
| string | 36 | yes |
| object | 35 | yes |
| regexp | 33 | yes |
| iterator | 30 | yes (4 FAILED) |
| date | 16 | yes |
| iterator_helpers | 13 | yes (9 FAILED) |
| heap | 12 | yes (1 ignored) |
| dynamic | 11 | yes |
| known_failures | 5 | yes |
| throw_propagation | 2 | yes |

**Banked = 276 executing tests of 607. Remaining = 331** (`frontend` 46, `typed_array` 58, `array` 84,
`language` 105, `binary_data` 38). 276 + 331 = 607 exactly — the resume has lost nothing.
Corrigendum to session 3: `iterator_helpers` is **13** tests, not 14, so the +14 delta from the
historical 593 is 13 new `iterator_helpers` tests plus one test added elsewhere in batch 5.

## Chunk/module partition verified BY HAND (the thing `rung_1c_chunks_cover_every_cli_area_module` asserts)
`grep '^mod ' crates/lila-cli/tests/cli/main.rs` = **17** declarations; `grep -c '^run_chunk '
scripts/rung1c-chunks.sh` = **17**; `ls crates/lila-cli/tests/cli/*.rs` = **18** files (the 18th is
`main.rs`, which declares 0 tests). Sets are equal in both directions by inspection, `iterator_helpers`
included. The fixer's finding-#19 rewrite of that test therefore has a true precondition — but note the
test itself is **still unrun**; it lives in the `known_failures` chunk, which was banked at 13:00Z on
**5 passed**, and `known_failures.rs` declares exactly 5 tests, so it did run. Green.

## Doc drift found while counting (owner: integrator, one-line each)
`docs/rust-rewrite/batch-workflow.md` says **593** in three places — line 38 (the rung table's
"593 compiled / 592 executing"), line 65, line 367 ("**593 compile**"). All three are stale by 14.
The T03 ledger row at `known-failures.tsv:67` repeats "262 of 593".

## The `binary_data` 900-vs-900 race, RESOLVED on paper before it costs a window
`scripts/rung1c-chunks.sh` flags it and does not resolve it: `main.rs:67` `HANG_TIMEOUT = 900 s` (which
converts the declared T17 `Atomics.wait` hang into an ordinary libtest failure) sits exactly on
`run-watched.sh --stall 900` (which SIGKILLs on 900 s of *no log growth* and returns 124 — a NO-VERDICT,
so the chunk would never bank and rung 1c could never complete). If the stall guard won that race,
`binary_data` would be an unbankable chunk and the whole run permanently INCOMPLETE.

It cannot win, and the margin is measurable rather than argued. libtest prints
`test <name> has been running for over 60 seconds` **exactly once per test** — counted, not assumed:
across the 14 completed chunk logs the warning appears 0-11 times and **every occurrence is a distinct
test name** (`sort | uniq -c` on `rung1c-date.log` gives five lines, each count 1). So the hanging test
itself grows the log at `hang_start + 60`. The stall guard therefore cannot reach 900 s of quiet before
`hang_start + 960`, while `HANG_TIMEOUT` fires at `hang_start + 900`. **HANG_TIMEOUT wins by ~60 s** and
`binary_data` produces a `test result:` line, i.e. a bankable verdict with one declared failure.
This holds regardless of where in the chunk the hang test is scheduled, because the 60 s warning is
emitted relative to that test's own start. Nobody needs to lower `--stall` for this chunk.

## Lead A — sweep state re-read at 13:5xZ (read-only; the sweep is deliberately DOWN)
`target/test262-scratch/baseline-sweep.log`: **48 `supervisor attempt` lines, 40 `exit 134` lines, and
every one of the 40 is at or before `03:58:28` — batch 4.** The b5 restart block
(`=== b5 sweep restart 2026-08-10T09:11:52Z ===`, attempt 1) shows `250/250 cases` on the poison node and
then `10/250 ... 70/250` on the next, and the log ends there: that is the 09:42Z container restart, not a
crash. **Zero `exit 134` and zero `test262 quarantine:` lines since the recursion fix.** Session 1's
conclusion stands unchanged at this head, and the sweep has simply been down for ~4 h.

### A leftover journal that is NOT the sweep's, and would be misread as one
`test262/snapshots/latest-2697367116994329042.attempts` (10:07Z) currently holds **four** in-flight
entries, all `built-ins/Iterator/prototype/some/*` (`this-plain-iterator`, `this-non-object`,
`result-is-boolean`, `this-non-callable-next`) on worker slots 0-3, `"strikes":{}`. This is the residue of
session 2's deliberately-killed `lila test262 run built-ins/Iterator/prototype/some` sizing probe, not a
sweep death — the sweep runs `--threads 2` and would leave at most two. Session 2's
`latest-8842212995299038775.attempts` (the ZDT `subtract/era-boundary-gregory.js` entry) is **gone**,
consistent with the fixer's `discard()` on clean exit.

Consequence the next runner must not trip over: whoever next resumes that snapshot will see **four**
suspects charged a strike at once, and the narrowing step will serialise all four. That is the design
working on a container-kill, but it is *not* evidence about a crashing case, and a `quarantine:` line
arising from it would be a false positive for lead A. The journal is per-snapshot; deleting this one file
before the next `built-ins/Iterator` probe is legitimate and loses nothing.

## Lane 2's "flip green WITHOUT editing `iterator.rs`" constraint — AUDITED, and it holds
`iterator.rs` *was* edited in batch 5 (`8a649799f`, +44/-12), so the constraint needed checking rather
than assuming. Read the diff: all four hunks are the identical shape — `assert!(output.status.success())`
gains a `"...: stdout={stdout} stderr={stderr}"` message and the two `stdout.contains(...)` asserts gain
`"stdout={stdout}"`. **The set of asserted conditions is byte-for-byte unchanged** (`status.success()`,
`contains("backend_used: WasmAot")`, `contains(...)`); only the failure *report* changed. The tests were
not weakened, and the measured red is a real red. The fixer's finding-#14 claim is confirmed by reading
the diff, not by trusting the report. `git status --porcelain` is **empty** — the tree is fully committed.

## Rung 1c banked failing set as it stands (13 tests, both modules, names exact)
From `target/lane-notes/rung1c-chunks.md`, not retyped from a prose note:
`iterator::run_wasm_backend_succeeds_for_iterator_prototype_{some,every,find,reduce}_fixture` (4) and
`iterator_helpers::run_wasm_backend_{calls_iterator_prototype_{some,every,find,reduce,map,filter,take}_on_a_class_receiver, chains_take_and_to_array_on_a_class_receiver, gives_identical_results_for_static_and_computed_helper_keys}` (9).
Every other banked chunk is `0 failed`. The only non-failure non-pass outcomes in 276 tests are
`heap` 1 ignored (a declared T05 ledger row) — so the ledger's existing rows are all still accurate.

## What filling the T03 row ACTUALLY costs — read out of the assertions, so nobody discovers it at the bump
Lane 3 is written as "bump `CURRENT_BATCH` to 4 and fill the row". Read `known_failures.rs`, and filling
it is a **three-place edit per declared test**, not a one-line tsv append. For each failing test a row
declares with state `fail`/`hang`:
1. a tsv row with 6 tab-separated columns, whose `evidence` column's **first whitespace-token must be a
   path that exists in this checkout and does not start with `target/`** (`:1209-1231`) — so citing
   `target/lane-notes/rung1c-chunks.md` or any `target/watched/*.log` is a hard red. Cite the fixture or
   the test file.
2. `should_panic` with a **non-empty** `expected` substring on the named test itself, and
3. a compile-time existence assertion in `known_failures.rs` spelled exactly
   `const _: fn() = crate::<module>::<test>;` (`:171`, `:618-619` are the two live examples). Both
   directions are enforced (`:1253-1270`): a row with no assertion is red, an assertion with no row is red.

Applied to the 13 measured reds that is 13 tsv rows + 13 `should_panic` attributes + 13 const assertions.
**Two of the three symptom classes will not survive it:**
- class (ii), `some`/`every`/`find` — the panic text is a value read from unrelated memory
  (`string(type-object;value;calls-0;...)`, and session 1 measured `typeof` as `number` for the same
  call). A `should_panic(expected = ...)` substring over an unstable string is a test that goes red for
  the wrong reason on the next unrelated allocation change.
- class (i), `value is not callable` — the message embeds a raw handle address (`handle@1483824`). The
  `expected` substring must be chosen to exclude the address, e.g. `"value is not callable"` alone.
Class (iii) (`reduce`, and the differential) is a wasm trap: `"wasmtime execution trapped"` is stable.

The assertion at `:1236-1249` names the legitimate alternative in its own message: extend
`# unfilled-allowed-until: batch-N` in the header, which is "possible, but it is a visible edit to the
header, which is the point". **Runner's recommendation, from measurement rather than preference:** the
13 reds are all one lane's live work (lead B), the ledger explicitly must not carry rows that libtest will
report as "test did not panic as expected" the moment the repair lands, and the repair has not landed at
this head. If lead B does not close in batch 5, the honest move is the visible header extension to
`batch-6` **plus** rows for anything red that lead B does NOT own — which, across 276 measured tests, is
currently the empty set. Owner of the decision: integrator; it is one edit either way and both are honest.

## The 4 PASSING `iterator_helpers` tests are NOT vacuous — checked, because a green here would be the
## most expensive kind of wrong
Every test in the module routes through `assert_helper_fixture_is_ok` (`iterator_helpers.rs:50`), which
asserts three things: the child exited 0, `stdout` contains `backend_used: WasmAot`, and **`stdout`
contains `string(ok)`** — the fixture's own discriminator, which it emits only when its internal
`failures` accumulator is empty. A fixture that silently did nothing would print some other value and go
red. Read one passing fixture end to end (`wasm_iterator_helper_class_receiver_drop.js`, 60 lines): it
checks `typeof helper === "object"`, `values.length === 2`, `values.join(",") === "2,3"`, **and** that
`new ThrowingSource().drop(1).toArray()` propagates a `next()` throw to a user `catch` as an
`instanceof Sentinel`. So the green for `drop` is a real green over both halves.

### And that green is itself the sharpest lead in lead B
`drop(1).toArray()` **works**; `take(1).toArray()` throws `TypeError: value is not callable`
(`chains_take_and_to_array`, `handle@1479696`). Same receiver shape, same chained `toArray`, opposite
outcome. The fixture's own header records the structural difference: **`drop`'s fast-path guard is
`receiver_is_iterator || !receiver_is_array`, where the other helpers use plain `receiver_is_iterator`.**
`drop`, `flatMap`, `forEach`, `toArray` are exactly the four that pass. That is a 4/4 correlation between
"passes" and "does not use the plain `receiver_is_iterator` guard", on 11 helpers — the fix lane should
start there rather than at the callee-acquisition sequence. Reported as a correlation on 11 data points,
not as a mechanism; I did not read the emitter.

## Lead A's FIRST deliverable — the unbounded recursion — is fixed, and I can now NAME the mechanism
The brief guesses "likely lowering/emitter recursion on deeply-nested source". It is not. Read
`crates/lila-aot-wasm/src/planning.rs` at this head:

- `RuntimeBootstrapPlan` carries a `walked: BTreeSet<StandardBuiltinId>` field (`:1161`) and
  `require_standard_builtin` opens with `if !self.walked.insert(builtin) { ... }` (`:1334`). The field's
  own doc (`:1137-1153`) states the subtlety that made the naive fix wrong: **a builtin can be *rooted*
  without having been *walked***, so the guard cannot be `if !standard_roots.insert(..) { return }`.
- The cycle is named in the regression test's doc (`:207-266`):
  `Temporal.PlainDateTime`'s arm requires `TemporalZonedDateTimeConstructor` (its `toZonedDateTime`
  returns one) and `Temporal.ZonedDateTime`'s arm requires `TemporalPlainDateTimeConstructor` (its
  `toPlainDateTime` returns one). The rooting walk recursed between them until the 64 MiB worker stack
  was gone -> SIGABRT. Reachable from `print(typeof Temporal.ZonedDateTime)`, from
  `print(typeof globalThis)`, and **from any test262 case containing `var global = this;`** — which is
  why it killed a whole-suite sweep rather than one case.
- The regression test is `planning::tests::a_cyclic_rooting_dependency_terminates_and_roots_both_ends`,
  which enters the cycle from five distinct builtins and asserts *both* ends stay rooted from each.

**Provenance, which changes who gets credit and what batch 5 still owes:** this landed in
`5bb66a35a` — a **batch-4** commit ("WIP checkpoint: batch 4 runner rebuilding, sweep overflow
identified", 04:13Z, +104 lines to `planning.rs`), plus `677075b9e` (+15 more). So lead A's recursion
half was already closed before batch 5 opened; batch 5's contribution to lead A is the **quarantine**
alone. Session 1's end-to-end measurement (poison node 250/250, zero `exit 134`) is the confirmation.
This also explains the shape of everything else: the sweep died in a Temporal node, the leftover journal
entry session 2 found was `ZonedDateTime/prototype/subtract/era-boundary-gregory.js`, and lead C is in
the same subtree. **They are not the same defect** — lead C is `Crash: 0` on all four cases (session 2).

## Lead C — NO fix has landed at this head, stated from the file list rather than from a re-run
`git log --name-only 77505818a..HEAD` over `crates/` touches exactly 33 files: `lila-aot-wasm`
{errors, functions, heap, objects, planning}, 5 `lila-cli/tests/cli` files, 13 new iterator fixtures,
`known-failures.tsv`, 6 `lila-ir` files, and `lila-test262` {attempt_journal, lib}. **No Temporal
source file of any kind.** Session 2's four measured `value is not callable` labels therefore still
describe the current head exactly, and lead C is untouched work, not work whose fix needs verifying.

## WHY `frontend` has failed to bank three times running — MEASURED, and it is not the container restart
Sessions 2 and 3 attributed the repeated `frontend` deaths to "the OOM killer, 5 concurrent cold
Wasm-AOT compiles". That is the right family but the wrong cause, and the difference matters because the
stated cause implies "run it alone and it is fine", which has now failed once more.

Sampled `ps -o rss` on the live child during this session's run, every 10 s while the box was otherwise
**idle** (no sweep, nothing else): a **single** process,
`lila test262 run language/wasm/pass --execution-backend wasm` — the child of
`frontend::test262_wasm_backend_runs_supported_fixture_subset` (`frontend.rs:1346`, asserts
`total: 187` / `passed: 187`) — holds **8.4-8.7 GiB RSS** for minutes at a stretch, at ~230 % CPU:

```
etime 05:13 rss 8672484 KiB   mem_used 14459 MiB  avail 1615 MiB
etime 05:28 rss 8367624       mem_used 14152      avail 1923
etime 05:58 rss 8677244       mem_used 14453      avail 1621
etime 06:18 rss 8677244       mem_used 14471      avail 1603
```

That is **~57 % of this 15 GiB box in one test**, and it is flat rather than growing, so it is a working
set and not a leak. Under `--test-threads=3` the other two libtest workers run their own cold Wasm-AOT
compiles beside it, and `avail` at 1.6 GiB is the entire margin. This is the whole explanation for the
three failed `frontend` chunks, and it also retro-explains session 2's `dynamic`/`heap` SIGKILLs: the
sweep was not the only heavyweight in the room.

**Consequences the next agent should not have to rediscover:**
- `frontend` is the single most memory-expensive chunk in rung 1c and it is scheduled 13th of 17, i.e.
  after ~1 h of banked work — so it is exactly where a resume keeps landing.
- Running the sweep concurrently with rung 1c is not merely unwise, it is arithmetically impossible while
  this chunk is in flight (sweep `--threads 2 --jobs 2` plus 8.7 GiB does not fit).
- If it fails a fourth time, the fix is a **scheduling** change, not a test fix: give `frontend` its own
  `--test-threads=1`-equivalent isolation. Note that literally lowering `--test-threads` is banned
  (`rung1c-chunks.sh` property 1: `known_failures::execution_path` routes on the libtest thread name).
  The supported spelling is a separate chunk for that one test via `-- --exact <name>`, which needs a
  `run_chunk` line and would have to keep `rung_1c_chunks_cover_every_cli_area_module` green. **Owner:
  integrator** — it is a script + hygiene-test edit, outside a runner's remit.
- Whatever makes 187 fixture cases cost 8.7 GiB in one process is itself worth an owner. It is a
  compile-cache sizing question (`LILA_*_CACHE_LIMIT_BYTES` are set for the sweep but **not** for the
  CLI test child), not a conformance question. **Owner: unassigned; recorded here so it stops being
  invisible.**
