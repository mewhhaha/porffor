# b3 RUNNER findings (incremental; written after each rung)

Commit at start: 28b9766cf (working tree CLEAN; predecessors committed WIP checkpoints).
Machine: 4 CPU / 15 GiB. Background full sweep (pid from batch, `lila test262 report-all`) ALIVE throughout, ~2 CPUs.
Start of this attempt: 2026-08-09 22:49 UTC.

## Inherited state from killed predecessor attempts (verified, not assumed)
- `target/test262-scratch/b3/` is EMPTY -> no rung-4/6/7 test262 result was ever banked at this commit.
- `target/watched/b3-cli.log` last written 20:57, mid-run at `running 590 tests` -> rung 1c was STARTED and KILLED. Not a result.
- `target/test262-scratch/b3-lane2-*-before.*` are the DEAD baselines the fixer already flagged (20/199 and 60/87). Not results.
- `./target/debug/lila` mtime 22:10 and **0** `crates/**/*.rs` newer than it -> the binary already contains all batch-3 edits. Rung 3 (`dev.sh build`) is therefore satisfied without a rebuild; every test262 number below is from the post-batch-3 binary.

## Rung 1 — `cargo xc`  [GREEN]
`cargo xc` exit 0, 1m55s, 0 errors. Warning counts unchanged from the fixer report:
`lila-aot-wasm` 26 (lib) / 21 (lib test, duplicates), `lila-test262` 1. No new warnings.

## Rung 2 (partial — the NEW size/report tests only)  [GREEN]
`cargo test -p lila-aot-wasm --lib -- --test-threads 2 <4 filters>`, 73.86 s, **4 passed / 0 failed**:
- `tests::emitted_function_bodies_stay_under_budget` — **PASSES**. This is the lane's headline prediction confirmed
  at the unit level: `js::probe#f0` (dynamic index `x = A[i];`) is now <= 30,000 bytes, where the analyst measured
  87,101 before the split, AND the relational half holds — the dynamic body exceeds the static-index control
  (`x = A[0];`) by <= 10,000 bytes, so neither seam is half-fired. (The test prints the two numbers only on failure,
  so the exact post-split byte count is not in this log; measured separately below.)
- `tests::typed_report_row_count_matches_the_code_section` — pass.
- `tests::the_size_report_file_is_the_same_traversal_as_the_typed_report` — pass (the fixer's new sink test).
- `emitted_function::tests::the_report_and_the_typed_summaries_are_one_traversal` — pass.
Filter line reports `250 filtered out` => the `lila-aot-wasm` lib target now holds **254** tests (batch-2 baseline 248, +6).
Full lib suites deferred to the end of the ladder (see Remaining).

## Rung 5 — THE HEADLINE CHECK: is "Code for function is too large" gone?  [GREEN — CONFIRMED]

### Baseline (batch 2, counted from the snapshots, not estimated)
`target/test262-scratch/post-dtf-*.json` (DTF 183/248), `b2-outline-collator-*.json` (Collator 2/65),
`b2-intl-rooting-after-*.json` (Intl 18/66). Cases whose `detail` contained `too large`:
**17 DTF + 10 Collator + 27 Intl = 54**, every one of them the identical
`Code for function is too large: 3615449 bytes, 154 locals`.

### After (this commit)
Two independent bodies of evidence, both from the post-batch-3 binary:

**(a) Banked by a killed predecessor attempt at 19:49-20:01 today** — 9 single-case `report` runs whose
snapshots are committed under `test262/snapshots/latest-*.json`. Verbatim outcomes:

| case | before | after |
|---|---|---|
| intl402/DateTimeFormat/test-option-hour12.js | too large | **PASS** |
| intl402/DateTimeFormat/test-option-localeMatcher.js | too large | **PASS** |
| intl402/DateTimeFormat/this-value-ignored.js | too large | Bug/Runtime `TypeError ... object(handle@5265496)` |
| intl402/DateTimeFormat/prototype/resolvedOptions/basic.js | too large | Bug/Runtime `TypeError ... object(handle@5397552)` |
| intl402/DateTimeFormat/taint-Object-prototype.js | too large | Bug/Runtime `TypeError ... object(handle@5392904)` |
| intl402/DateTimeFormat/date-time-options.js | too large | Bug `string(Function toLocaleString did not return expected string ...)` |
| intl402/Collator/this-value-ignored.js | too large | Bug/Runtime `TypeError ... object(handle@5265392)` |
| intl402/Collator/prototype/resolvedOptions/basic.js | too large | Bug/Runtime `TypeError ... object(handle@5297280)` |
| intl402/Collator/test-option-usage.js | too large | Bug/Runtime `TypeError ... object(handle@5272672)` |

**(b) Re-run by me at this exact commit** (binary mtime 22:10, after the last source edit), one case per
`lila test262 report --threads 1 --jobs 2 --snapshot-dir target/test262-scratch/b3`:
- `intl402/Collator/test-option-usage.js`: 0/1, `Bug | Runtime | TypeError: wasm-aot completion: object(handle@5272672)`.
  **bucket `WasmBackend: 0`, bucket `Runtime: 1`.** Byte-identical detail to the predecessor's 20:01 run,
  so (a) and (b) agree and the intervening `lila-ir/modules/dynamic.rs` edit did not move these.
  Cost: 2 min 43 s per case on this box under the sweep.
- remaining 7 of my 8 still running (log `target/watched/b3r-headline.log`).

### The claim, stated exactly
`Code for function is too large` is **GONE** from every case examined: 9 (predecessor) + 1 (mine, re-confirmed)
= 9 distinct cases out of the 54, drawn from all three nodes. The failures that remain are honest *runtime*
failures at a later stage — they now compile and execute and fail on Intl semantics.
**2 of the 9 are new passes** (`test-option-hour12`, `test-option-localeMatcher`). I do **not** claim 54 new passes;
I claim the too-large emit failure is retired and the residue re-bucketed from `WasmBackend` to `Runtime`.
The dominant residual detail (`TypeError ... object(handle@N)`) is one shared defect across 6 of the 9 —
next batch should treat it as a single lead, not six.

## Rung 6 — PlainMonthDay: the two inherited batch-2 REGRESSIONS  [GREEN at file level]

Batch-2 baseline `target/test262-scratch/b2-era-bi-pmd-8014157493499151608.json`: **197/199**, failures exactly
`from/fields-string.js` and `prototype/equals/argument-string-invalid.js` (invalid ISO PlainMonthDay strings threw
a generic Error where RangeError is required).

All **8** files the lane nominated were run individually as `report` at the post-batch-3 binary by a killed
predecessor attempt (20:05-20:23 today; snapshots committed under `test262/snapshots/latest-*.json`) and every one
is **1/1 PASS**:

| file | result |
|---|---|
| from/fields-string.js | **PASS** (was the regression) |
| prototype/equals/argument-string-invalid.js | **PASS** (was the regression) |
| prototype/equals/argument-string.js | PASS |
| from/argument-string-calendar-annotation.js | PASS |
| from/argument-string-calendar-invalid-iso-string.js | PASS |
| from/argument-string-calendar-case-insensitive.js | PASS |
| from/options-read-before-algorithmic-validation.js | PASS |
| from/observable-get-overflow-argument-string-invalid.js | PASS |

`from/fields-string.js` is not a token test: it drives all 10 strings of
`TemporalHelpers.ISO.plainMonthDayStringsInvalid()` (`"11-18junk"`, `"11-18[u-ca=gregory]"`, `"11-18[U-CA=iso8601]"`,
`"-999999-01-01[u-ca=chinese]"`, ...) through `assert.throws(RangeError, ...)`, plus the full valid-string list.
So the RangeError conversion is exercised across the whole invalid-string domain, not one example.

**Both inherited regressions are CLOSED at file level.** The node-level 199/199 claim is NOT yet counted — the full
199-case node is queued behind rungs 5/7 in `target/watched/b3r-chain.log` (resumable, snapshot `b3r-pmd-node`);
see Remaining for the partial count actually reached.

## Rung 5b — the emit-size report, actually measured (the sink now works)

The batch-2 refutation was that `LILA_WASM_TRACE_DUMP` was unreachable so `LILA_EMIT_SIZE_REPORT` printed
nothing. The new `LILA_EMIT_SIZE_REPORT_PATH` sink is honoured **inside `emit()`**, and it produces real output:

**testIntl-shaped probe** (`scratchpad/size-report-after.txt`, 508 emitted bodies) — largest bodies now:

| bytes | function |
|---|---|
| 796,875 | `lila::main` |
| 300,720 | `builtin::Object.defineProperty` |
| 253,488 | `builtin::Reflect.defineProperty` |
| 147,000 | `js::isCanonicalizedStructurallyValidLanguageTag#f12` |
| **104,259** | **`js::canonicalizeLanguageTag#f46`** |

`js::canonicalizeLanguageTag#f46` is the function batch 2 identified as the single body reported at
`3,615,449 bytes, 154 locals` in all 54 too-large cases. It is now **104,259 bytes — a 34.7x reduction**, and
nothing in the module is above 796,875. That is the mechanism behind the rung-5 result above, measured rather
than inferred.

**Budget probe** (`function probe(k,i,j){ var A=k.split('-'); var x=''; x=A[i]; return x; }`):

| probe | `js::probe#f0` |
|---|---|
| pre-split (analyst, batch 3 review) | 87,101 |
| post-split, dynamic index `A[i]` | **16,519** |
| post-split, static-index control `A[0]` | **14,943** |
| delta attributable to the dynamic key | **1,576** |

The outlined helpers are present in the same report: `helper::value_to_primitive_number` 70,339,
`helper::value_to_primitive_string` 70,339, `helper::value_to_property_key` 1,027 — i.e. the ~72,528-byte
composite now exists once per helper instead of once per call site. The 1,576-byte delta is close to the
"~1,000 once counted" the test's doc comment predicted and nowhere near the 72,528 a half-fired seam would
leave, so **both** seams fired. (Recommendation for the next batch: `DYNAMIC_KEY_MARGIN_BYTES` can be tightened
from 10,000 to ~2,500 now that the delta is counted at 1,576.)

**Re-measured by me at this commit** (not inherited): `LILA_EMIT_SIZE_REPORT_PATH=... lila build wasm` on both
probes reproduces the table exactly — dynamic `js::probe#f0` **16,519**, static control **14,943**, delta **1,576** —
and the report lists **four** outlined helper bodies, `helper::value_to_primitive_default` /
`value_to_primitive_number` / `value_to_primitive_string` at **70,339 each** plus `helper::value_to_property_key`
at **1,027**. Both `probe#f0` and its `$exact_helper_context$0` twin carry the same size, so the seam fired on
both bodies.

### Rung 5 re-run, cases 1-3 of 8 (mine, at this commit)
| case | bucket | outcome | detail |
|---|---|---|---|
| intl402/Collator/test-option-usage.js | Runtime 1 / WasmBackend 0 | Bug | `TypeError: wasm-aot completion: object(handle@5272672)` |
| intl402/Collator/this-value-ignored.js | Runtime 1 / WasmBackend 0 | Bug | `TypeError: wasm-aot completion: object(handle@5265392)` |
| intl402/Collator/prototype/resolvedOptions/basic.js | Runtime 1 / WasmBackend 0 | Bug | `TypeError: wasm-aot completion: object(handle@5297280)` |
All three reproduce the predecessor's 19:58-20:01 details byte for byte. `Code for function is too large` appears
in none of them; the `WasmBackend` bucket is empty in every snapshot.

### Whole-module effect on the testIntl probe (counted, both sides)
| | before (lane note §1.1, same probe) | after (`size-report-after.txt`) |
|---|---|---|
| emitted functions | 504 | 508 (+4 = the outlined helpers) |
| total code-section body bytes | 22,278,896 | **8,293,785** (−62.8%) |
| largest body | `js::canonicalizeLanguageTag#f46` 3,615,449 / 154 locals | `lila::main` 796,875 |
| `js::canonicalizeLanguageTag#f46` | 3,615,449 | **104,259** (−97.1%) |
The single body that produced the identical `3615449 bytes, 154 locals` detail in all 54 too-large cases is now
34.7x smaller and is no longer the largest function in the module.

## In flight at the time of writing (logs, so a killed attempt still reports)
- `target/watched/b3r-headline.log` — my 8-case rung-5 re-run (3 Collator + 5 DTF), ~2 min/case.
- `target/watched/b3r-chain.log` — chained after it: 12 ZDT/era + calendar-invalid-era cases (rung 7), then the
  resumable full `built-ins/Temporal/PlainMonthDay` node (rung 6 at node level), snapshot `b3r-pmd-node`.
- `target/watched/b3r-ir-lib.log` — `cargo test -p lila-ir --lib` (rung 2, baseline 626).
- `target/watched/b3r-fake.log` — rung 4, the fake wasm-safe suite.
- `target/watched/b3r-clitest.log` — the single unrun CLI test the fixer flagged
  (`language::run_wasm_backend_gives_a_runtime_error_a_message_distinct_from_its_name`, the T24 ledger row).
Judge each by log growth, not elapsed time.

**Live results table:** `target/lane-notes/b3-runner-auto-results.md` is regenerated every 60 s by
`target/watched/b3r-harvest.py` (setsid, detached) from every snapshot in `target/test262-scratch/b3/` plus the
tails of all five logs, with an explicit `'too large'?` column. It survives this agent being killed; read it for
the final counts of anything listed as in flight above.

## Rung 2 (full lib suites) — PARTIAL, stopped deliberately
- `cargo test -p lila-ir --lib -- --test-threads 2`: reached **297 of 626 tests, 0 FAILED, 0 panics** before
  I killed it at 23:06. I killed it, it did not die: load average was 7.64 on 4 CPUs with `kswapd0` active and
  11.9 GiB of 16 GiB resident, and the priority rung-5/rung-7 `lila` process was being starved down to 53% of one
  core. No IR test failed in the 297 that ran. Log: `target/watched/b3r-ir-lib.log` (`IR_EXIT=101` is my SIGTERM,
  not a test failure).
  **Remaining, owner = next batch's runner:** the other 329 IR tests. Re-run as the FIRST thing on an idle box —
  it is minutes there.
- `cargo test -p lila-aot-wasm --lib` in full (254 tests) was **not** run; only the 4 new size/report tests were
  (green, above). Owner = next batch's runner. Reason: the 4 that matter for this batch's headline claim ran, and
  the emit-heavy remainder costs more than the remaining window.

## Rung 4 — fake wasm-safe suite: NOT COMPLETED, and the cost is the finding
`lila test262 run --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262
--execution-backend wasm --threads 1 --jobs 2` reached **10 of 190 cases in 7 min 40 s** (~46 s/case) before I
stopped it at 23:10 to stop it starving rung 5. No failure appeared in those 10. The ladder in
`batch-workflow.md` bills this rung at "10-60 s **warm**, 190 cases"; on this 4-CPU box with the sweep running it
is a ~2.5 hour job, i.e. **~150x the documented cost**, not the ~20x the batch brief assumed.
`docs/rust-rewrite/batch-workflow.md` should carry that per-box figure — an agent picking rung 2 as "the cheap
one" here will lose its whole window. **Remaining, owner = next batch's runner** (or any run on the 16-CPU box,
where it is genuinely a minute).

## The fixer's one unverified runtime assertion (finding 23 / T24) — RUN, and it is honest  [GREEN]
`cargo test -p lila-cli --test cli -- --exact --nocapture
language::run_wasm_backend_gives_a_runtime_error_a_message_distinct_from_its_name`
→ `test result: ok. 1 passed; 0 failed; **589 filtered out**` in 46.62 s (so the `cli` target holds **590** tests,
matching the fixer's recount).

The fixer asked the integrator to read this test's real stdout rather than accept the pass, because a
`#[should_panic]` can match its `expected` substring for the wrong reason. Read:

```
panicked at crates/lila-cli/tests/cli/language.rs:1869:5:
runtime error message equals its name (T24, emit_runtime_error_object ignores its `_message`):
run outcome: RunOutcome { backend_used: WasmAot, note: "wasm-aot completion: string(message-equals-name)" }
```

The completion value is `string(message-equals-name)` — the fixture's own defect-present branch, not a fallback
or an unrelated failure. So the row is green for exactly the declared reason, and it will flip to
`test did not panic as expected` the moment the `errors.rs` + `data.rs` repair lands. Fixer item 4 is closed.
Note this also confirms lane 4's manual reproduction (`e.message === e.name` is `true` today) without needing the
separate probe.

## Regressions found by this runner: NONE so far
Nothing I ran moved anything backwards:
- 0 failures in the 297 `lila-ir` lib tests that executed; 0 in the 4 aot-wasm size/report tests; 0 in the
  single CLI test; 0 in the 10 fake-suite cases that executed.
- No test262 case I ran is worse than its batch-2 baseline. Every changed cell moved from
  `WasmBackend`/too-large to `Runtime`, or from red to green.
- The batch-2 PlainMonthDay regression pair is fixed, not merely relabelled (`from/fields-string.js` exercises
  all 10 invalid strings).
Caveat, stated plainly: the coercion-heavy regression watch the outline lane asked for
(`built-ins/Object/prototype/toString` 41, `language/expressions/addition`, `.../equals`,
`built-ins/Array/prototype/join`, `built-ins/Symbol/prototype/Symbol.toPrimitive`,
`built-ins/Date/prototype/Symbol.toPrimitive`) was **not run** — at ~46-160 s per case on this box those nodes are
multi-hour. The env-forwarding edit the fixer landed at both ToPrimitive seams is unvalidated by execution;
rung 1c is its cheapest real gate. Owner = next batch's runner. Reason: window, not judgement.

## Rung 5 FINAL — my 8-case re-run completed 23:13:38 (8/8 ran, none skipped)
| case | batch-2 detail | this commit |
|---|---|---|
| intl402/Collator/test-option-usage.js | too large 3615449/154 | Bug/Runtime `TypeError object(handle@5272672)` |
| intl402/Collator/this-value-ignored.js | too large | Bug/Runtime `TypeError object(handle@5265392)` |
| intl402/Collator/prototype/resolvedOptions/basic.js | too large | Bug/Runtime `TypeError object(handle@5297280)` |
| intl402/DateTimeFormat/this-value-ignored.js | too large | Bug/Runtime `TypeError object(handle@5265496)` |
| intl402/DateTimeFormat/prototype/resolvedOptions/basic.js | too large | Bug/Runtime `TypeError object(handle@5397552)` |
| intl402/DateTimeFormat/test-option-hour12.js | too large | **PASS** |
| intl402/DateTimeFormat/test-option-localeMatcher.js | too large | **PASS** |
| intl402/DateTimeFormat/legacy-regexp-statics-not-modified.js | too large | **PASS** |

Union with the predecessor's 9: **10 distinct cases of the 54 verified**, drawn from both Collator and DTF.
- `Code for function is too large`: **0 of 10**. The string does not occur in any snapshot under
  `target/test262-scratch/b3/`, and every snapshot reports `bucket: WasmBackend (0)`.
- **3 outright new passes**, counted, not extrapolated: `test-option-hour12`, `test-option-localeMatcher`,
  `legacy-regexp-statics-not-modified`.
- 7 now fail at runtime. 6 of the 7 share one shape, `TypeError: wasm-aot completion: object(handle@N)` — one
  lead, not seven. The 7th (`date-time-options.js`, predecessor) is a `toLocaleString` output mismatch.
**Verdict: the lane's headline claim is CONFIRMED and the batch-2 REFUTATION is itself refuted** — the size
report is now reachable (`LILA_EMIT_SIZE_REPORT_PATH`), the outlining did fire, and the failure detail that
was byte-identical across 54 cases is gone.

## Rung 7 — ZDT era: NO CHANGE from batch 2 (not a regression, still red)
First 2 of 12 cases, against the batch-2 singles in `target/test262-scratch/sgl-era-focus-*.json` (10:37-11:02):
| case | batch 2 | this commit |
|---|---|---|
| ZonedDateTime/from/era-boundary-gregory.js | 0/1 `TypeError object(handle@1817424)` | 0/1 `TypeError object(handle@1817560)` — same shape |
| ZonedDateTime/from/canonicalize-era-codes.js | 0/1 `string('ad' is accepted as alias for 'ce' ...)` | 0/1 **identical string** |
This is exactly the batch-2 verdict "gregory era green outside ZDT" holding unchanged: the era work landed for
PlainDate / PlainDateTime / PlainYearMonth (all green in the batch-2 singles) and **ZonedDateTime was never
fixed**. Nothing regressed; nothing improved. Owner: the temporal-gregory-era lane, next batch. The remaining
10 of my 12 are still running — see `b3-runner-auto-results.md` for the final table.

### Rung 7 update — 5 of 12 ZDT/era cases done, all matching their batch-2 baseline exactly
| case | batch 2 | this commit |
|---|---|---|
| ZDT/from/era-boundary-gregory.js | TypeError object(handle@1817424) | TypeError object(handle@1817560) |
| ZDT/from/canonicalize-era-codes.js | string("'ad' is accepted as alias for 'ce'") | identical |
| ZDT/from/non-positive-single-era-year.js | RangeError object(handle@1841152) | RangeError object(handle@1841288) |
| ZDT/from/calendar-not-supporting-eras.js | RangeError object(handle@1527928) | RangeError object(handle@1528064) |
| ZDT/prototype/add/era-boundary-gregory.js | TypeError object(handle@1820072) | TypeError object(handle@1820208) |
Five for five, same outcome and same error class; only the heap handle addresses moved (by 136 bytes, uniformly —
a bootstrap layout shift, not a semantic one). **ZDT era is unchanged: 0 fixed, 0 regressed.**

### Rung 7, 6 of 12 (23:42): `ZDT/prototype/since/era-boundary-gregory.js` also unchanged
batch 2 `TypeError object(handle@1871808)` -> now `TypeError object(handle@1871944)`. Six for six, every handle
exactly +136. Cases 7-12 still running; the harvested table has the rest.

## REMAINING, each with an owner and a reason (nothing here is "unknown")
| # | Item | Owner | Reason it did not run |
|---|---|---|---|
| 1 | Rung 1c, the whole CLI suite (590 tests) with the new known-failures baseline | next batch's runner | Est. 1 h 45 m at `--test-threads=2` on the fast box; this box is 2-5x slower again and the container restarts ~hourly. Two prior attempts died mid-run (`target/watched/b3-cli.log` stopped at 15 of 590 with 0 failures). This is the gate for the fixer's two broad-blast-radius edits (`HANG_TIMEOUT` 120->900 s and the bounded in-process worker thread, which touches all 590 tests' execution path) and for the outline lane's env-forwarding change. Run it FIRST on an idle box. |
| 2 | The `cli / UNFILLED / unfilled / T03` ledger row | T03, next batch | It can only be filled from a completed rung-1c run. `CURRENT_BATCH = 3` and `unfilled-allowed-until: batch-4`, so it does not fail `ledger_is_well_formed` today — it fails the moment `CURRENT_BATCH` becomes 4. That is a hard deadline, not a nag. |
| 3 | `cargo test -p lila-ir --lib` tests 298-626 | next batch's runner | I killed it at 297/626 with 0 failures to stop it starving rung 5 (load 7.64 on 4 CPUs, 11.9/16 GiB, `kswapd0` active). |
| 4 | `cargo test -p lila-aot-wasm --lib` in full (254 tests) | next batch's runner | Only the 4 new size/report tests were run (green). The rest are emit-heavy. |
| 5 | Rung 4, fake wasm-safe suite cases 11-190 | next batch's runner | Measured at ~46 s/case here (~2.5 h). The ladder's "10-60 s" figure is a 16-CPU number. |
| 6 | Rung G (golden capture + `diff -r`) | next batch's integrator | Never started; it is ~10 min per side on the fast box and both sides need a clean window. Note the fixer's finding 22: an EMPTY rung-G diff is **zero evidence** for lane 4's `objects.rs` hunk, and the outline lane's Part 2 plus the env-forwarding edit both diff non-empty **by design**. Rung G cannot adjudicate branch-target depth. |
| 7 | The coercion regression watch (`Object/prototype/toString` 41, `expressions/addition`, `expressions/equals`, `Array/prototype/join`, `Symbol.toPrimitive` x2) | next batch's runner | Multi-hour at this box's per-case cost. |
| 8 | ZDT era repair itself | temporal-gregory-era lane, next batch | Measured unchanged, 6/6 identical to batch 2. PlainDate / PlainDateTime / PlainYearMonth era are green; ZonedDateTime never was. |
| 9 | The `TypeError: wasm-aot completion: object(handle@N)` cluster now exposed under Intl | new lane, next batch | It is ONE defect surfacing in 6+ of the 10 previously-too-large cases I verified, and it was invisible while those cases died at emit. Triage it as one lead. |
| 10 | Full `built-ins/Temporal/PlainMonthDay` node count (199) | next batch's runner | Queued last in `b3r-chain2.sh` with `--resume` and snapshot `b3r-pmd-node`; at this box's per-case cost the node is hours. All 8 nominated files, including both regression files, are individually green. |
| 11 | Findings 20 (branch depth for `depth_to >= 1`) and 21 (loop/switch throw propagation) from the fixer report | recorded unowned in the lane-4 note §1.4 / §5.5 | I did not run them; the fixer deliberately did not repair them. §5.5's defect is Crash/timeout-class and is live in the sweep today. |

### Rung 7 FINAL-ish (10 of 12 at 23:49): **0 passed / 10**, every one matching its batch-2 baseline
`ZDT/from/{era-boundary-gregory, canonicalize-era-codes, non-positive-single-era-year,
calendar-not-supporting-eras, calendar-invalid-era}`, `ZDT/prototype/{add,since,subtract,until}/era-boundary-gregory`,
`PlainDate/from/calendar-invalid-era`. Same outcome, same error class, handles uniformly +136 bytes.
`prototype/subtract` and `prototype/add` share handle@1820208; `since` and `until` share handle@1871944 — i.e.
add/subtract are one defect and since/until are another, two leads rather than four.
The last 2 cases (`PlainDateTime/from/calendar-invalid-era`, `PlainDate/from/calendar-not-supporting-eras`) and
then the resumable PlainMonthDay node continue in `target/watched/b3r-chain2.log`; the harvester keeps
`b3-runner-auto-results.md` current without me.
