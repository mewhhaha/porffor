# b6 RUNNER findings

Start 2026-08-10 17:15 UTC. HEAD `e708f1f22` ("WIP checkpoint: batch 6 fix stage") + the fixer's
uncommitted working tree (11 modified files). Branch `claude/test-driven-rust-opus-pp6giw`.
4 CPU / 16 GiB. `LILA_CPU_PERCENT=100` throughout.

## State harvested on arrival (nothing re-derived)

- Sweep **was alive**: supervisor pid 4128, `lila test262 report-all --snapshot-name
  baseline-wasm-aot-b2 --threads 2 --jobs 2 --resume` pid 4131, 231 % CPU, 4.4 GiB RSS, up 2 h 38 m.
- `target/test262-scratch/baseline/` held **22 node `.json` snapshots**, log at `80/250` inside the
  23rd node (`built-ins/Array/prototype` chunk 10 of 12).
- `target/watched/rung1c-done` carried **9** banked chunks (not b5's 12): the integrator removed
  `iterator`, `iterator_helpers` (compiler-stale) and `known_failures` (its subject, the chunk/module
  partition, went 17 -> 18). `rung1c-done-counts` seeded from b5's per-module counts, so `date`
  (banked 16, now declares 17) re-runs. Verified by reading both files.

## Rung 0 — `cargo xc`  [EXIT=0, 15 s]

`target/watched/b6r-xc.log`. Warnings by target, counted from the `generated N warnings` lines:
`lila-ir` lib **6** / lib-test **5**; `lila-aot-wasm` lib **25** / lib-test **20**;
`lila-test262` lib-test **1**. **Identical to the b5 baseline and to the integrator's and
fixer's reports.**

## Rung 0b — the debug binary WAS stale, again (third batch running)

`find crates -name '*.rs' -newer target/debug/lila` = **21** files on arrival (binary 09:28Z,
150,251,488 B). Rebuilt: `cargo build -p lila-cli --bin lila` -> **75 s**, 150,288,056 B.
Rebuilt a second time after the data.rs fix below -> 150,288,760 B at 17:43Z.

## Counts recounted at this head (exact-line `awk`, not a substring grep)

`awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++}' crates/lila-cli/tests/cli/*.rs` = **617**.
`frontend.rs` carries **8** `spec-exec-oracle` gates (`frontend_test262_subset.rs` carries 0), so
**609 compile** and **608 execute** (one `#[ignore]` in `heap.rs`). `batch-workflow.md:38` already
says exactly this — the fixer's F17 edit is correct at this head.

Per module (this is the partition every chunk's `ran + filtered_out` must sum against):

| module | tests | | module | tests |
|---|---|---|---|---|
| language | 105 | | object | 35 |
| array | 84 | | regexp | 33 |
| typed_array | 58 | | iterator | 30 |
| frontend | 53 compiled / 45 exec | | date | **17** (was 16) |
| functions | 45 | | iterator_helpers | **14** (was 13) |
| binary_data | 38 | | heap | 12 |
| data_view | 38 | | dynamic | 11 |
| string | 36 | | known_failures | 5 |
| **frontend_test262_subset** | **1** (new) | | throw_propagation | 2 |

`grep -c '^run_chunk ' scripts/rung1c-chunks.sh` = **18**; `grep -c '^mod ' tests/cli/main.rs` = **18**.

## Scheduling decision — the sweep is DOWN for the duration of this session

Killed supervisor 4128 then report-all 4131 at 17:16Z. Reason is b5's measurement, not preference:
five concurrent cold Wasm-AOT compiles do not fit in 15 GiB, and one frontend test alone holds
8.7 GiB. Everything on my priority list from step 3 down is a CLI chunk or a Temporal test262 case.
The kill left the expected artefact and it is worth recording because it is the quarantine's design
working on a non-crash death:

```
baseline/...Array_prototype@chunk-0010-of-0012-....attempts
{"version":1,"in_flight":[{"worker_slot":0,"case_path":"built-ins/Array/prototype/some/15.4.4.17-7-b-12.js"},
                          {"worker_slot":1,"case_path":"built-ins/Array/prototype/some/15.4.4.17-7-b-11.js"}],
 "strikes":{}}
```

Two in-flight entries, `--threads 2`, exactly as the supervisor header predicts. On resume both are
charged one strike and forgiven when they complete. **A `test262 quarantine:` line arising from these
two would be a false positive for lead A** — they were killed by me, not by a crash.

---

# BLOCKER FOUND AND FIXED — batch 6 shipped a compile-time panic in every full bootstrap

## `cargo test -p lila-aot-wasm --lib` -> **245 passed / 24 FAILED / 1 ignored**, 740.5 s

`target/watched/b6r-aotunit.log`. This target was **not run by the fixer or the integrator** (neither
ran any test), and b5 never ran it either. Every one of the 24 carries the same panic:

```
thread '...' panicked at crates/lila-aot-wasm/src/data.rs:3975:32:
string `Temporal.ZonedDateTime until and since require the same calendar` must exist in pool
```

with the identical frame chain
`StringPool::payload -> builtins/errors.rs:494 -> temporal_plain_date.rs:888
(emit_temporal_require_same_calendar) -> temporal_zoned_date_time_methods.rs:490 ->
standard.rs:37131 -> emit.rs -> lib.rs`.

The 24, by name:

```
debug_dump_attributes_the_largest_emitted_function      operations_emits_get_spec_operation
full_bootstrap_emits_without_proto_source_reference     operations_emits_get_v_spec_operation
map_cross_realm_new_target_modules_validate             operations_emits_has_own_property_spec_operation
operations_emits_create_data_property_or_throw_...      operations_emits_has_property_spec_operation
operations_emits_delete_property_or_throw_...           operations_emits_set_spec_operation
operations_emits_get_method_spec_operation              operations_emits_to_boolean_spec_operation
proxy_revoked_cross_realm_call_module_validates         runtime_helper_count_is_derived_not_asserted
set_cross_realm_new_target_modules_validate             temporal_now_builtins_emit
temporal_zoned_date_time_civil_accessors_and_equals_emit    the_size_report_file_is_the_same_traversal_...
temporal_zoned_date_time_from_builtin_emits             typed_report_row_count_matches_the_code_section
temporal_zoned_date_time_from_property_bags_emit        typedarray_get_own_property_descriptor_module_validates
temporal_zoned_date_time_offset_accessors_emit          temporal_zoned_date_time_with_time_zone_emits
```

Note the shape: **most of them are not Temporal tests.** `operations_emits_*`, the proxy/cross-realm
module validators and the size-report traversals all emit a *full bootstrap*, so any builtin whose
body reads an uninterned pool string takes them all down together. That is why the blast radius is
24 and not 5.

### Mechanism, read out of the source rather than inferred

`StringPool::payload` (`data.rs:3971-3981`) is `refs.get(value).unwrap_or_else(|| panic!(...))` — it
panics rather than degrading. The pool is built by a long series of
`if compiled_standard_builtins.contains(&StandardBuiltinId::X) { for value in [...] { intern } }`
blocks. Batch 6's ZDT lane added `emit_temporal_zoned_date_time_difference`
(`temporal_zoned_date_time_methods.rs:490`, `:522`) with **two new message strings** and no matching
intern block. The three sibling families already have theirs, together, at `data.rs:2011-2013`:
`Temporal.{PlainDate,PlainDateTime,PlainYearMonth} until and since require the same calendar`.

**Nothing in the type system catches this** — the message is a `&str` parameter of
`emit_temporal_require_same_calendar` and the pool is a runtime `HashMap`. It is a textbook case for
the AGENTS.md "code invariants before test invariants" list, and I have recorded it as such below
rather than pretending the fix closes it.

### Fix applied (`crates/lila-aot-wasm/src/data.rs`, after the `TemporalZonedDateTimeFrom` block)

A gated block in the established shape, keyed on the two builtins that emit the messages:

```rust
if compiled_standard_builtins.contains(&StandardBuiltinId::TemporalZonedDateTimePrototypeUntil)
    || compiled_standard_builtins.contains(&StandardBuiltinId::TemporalZonedDateTimePrototypeSince)
{
    for value in [
        "Temporal.ZonedDateTime until and since require the same calendar",
        "Temporal.ZonedDateTime until and since require the same time zone",
    ] { pool.intern_string(value); }
}
```

Both strings, not only the one that panicked: `:522`'s `require the same time zone` is the *next*
`payload` call on the same path and would have panicked the moment the first was fixed. I enumerated
every string literal in the new module and every `self.strings`/`payload(` call in it (there are no
others) rather than fixing the one the backtrace named.

### Verified after the fix (two runs, filtered, because the full target is 740 s)

| run | filter | result |
|---|---|---|
| `b6r-aotfix1` | `temporal_zoned_date_time full_bootstrap operations_emits runtime_helper_count typed_report_row_count the_size_report_file debug_dump_attributes temporal_now_builtins` | **40 passed, 0 failed**, 192.7 s |
| `b6r-aotfix2` | `cross_realm typedarray_get_own_property_descriptor iterator` | **15 passed, 0 failed**, 62.5 s |

Between them the two filters cover **all 24** by name. 0 red.

## The lanes' own new unit tests — all green, all actually selected

| test | target | result |
|---|---|---|
| `planning::tests::a_cyclic_rooting_dependency_terminates_and_roots_both_ends` | `--lib` | **ok** |
| `planning::tests::reflected_call_entry_points_root_proxy_dispatch` | `--lib` | ok |
| `a_class_receiver_dispatches_take_the_same_way_as_drop` | `--test iterator_helper_dispatch` | **ok** |
| `a_class_receiver_dispatches_map_the_same_way_as_flat_map` | same | **ok** |
| `a_class_receiver_helper_call_emits_a_valid_module` | same | **ok** (85 s; >60 s warning) |

`iterator_helper_dispatch` is **3 tests, 0 filtered out** when run as a bare target. Trap worth
recording: a `-- iterator` filter selects **0** of its 3 (none of the names contain `iterator`), and
a `-- dispatch` filter selects 2 of 3. Run it unfiltered.

## Rung 1c chunk — `known_failures::`  [5 passed, 0.01 s]

`ran=5 filtered_out=604 sum=609` — matches the recounted compiled total exactly.

```
ledger_is_well_formed ... ok
routing_takes_the_guarded_path_whenever_the_test_name_is_unknown ... ok
rung_1c_chunks_cover_every_cli_area_module ... ok
every_expected_failure_carries_a_should_panic ... ok
every_ignored_test_is_declared ... ok
```

So at this head, **measured rather than reasoned**: the 18-chunk / 18-module bijection holds after
the `frontend_test262_subset` move; `CURRENT_BATCH = 6` against `# unfilled-allowed-until: batch-7`
holds (`6 < 7`); and the integrator's ledger edits parse. The tripwire is no longer vacuous — the
b5 state was `3 < 4` and had been true for three batches.

---

# LANE 1 — iterator helpers: CLOSED. 13 of 13 batch-5 reds are green.

## `iterator_helpers::` — **14 passed / 0 failed**, 128.3 s  (b5: 4 passed / 9 FAILED of 13)

`ran=14 filtered_out=595 sum=609`. `target/watched/b6-iterator-helpers.log`. Every name, so the
before/after is per-test rather than per-total:

| test (`run_wasm_backend_…`) | b5 | b6 |
|---|---|---|
| `…_calls_iterator_prototype_some_on_a_class_receiver` | FAIL (ii) wrong-typed value | **ok** |
| `…_every_on_a_class_receiver` | FAIL (ii) | **ok** |
| `…_find_on_a_class_receiver` | FAIL (ii) | **ok** |
| `…_reduce_on_a_class_receiver` | FAIL (iii) wasm trap | **ok** |
| `…_map_on_a_class_receiver` | FAIL (i) `value is not callable` `handle@1483832` | **ok** |
| `…_filter_on_a_class_receiver` | FAIL (i) `handle@1483824` | **ok** |
| `…_take_on_a_class_receiver` | FAIL (i) `handle@1485040` | **ok** |
| `…_chains_take_and_to_array_on_a_class_receiver` | FAIL (i) `handle@1479696` | **ok** |
| `…_gives_identical_results_for_static_and_computed_helper_keys` | FAIL (iii) wasm trap | **ok** |
| `…_for_each_…`, `…_flat_map_…`, `…_drop_…`, `…_to_array_…` (control group) | ok | **ok** |
| `…_propagates_abrupt_completions_from_helper_dispatch` | (did not exist) | **ok** |

The control group is the one that mattered: the integrator flagged that the `lila-ir`
`constructor_instance` fix makes `forEach` on these receivers lower to `CallMethod` for the first
time, so `…_for_each_…` was the test that would catch a bad reroute. It is green.

## `iterator::` — **30 passed / 0 failed**, 526.7 s  (b5: 26 passed / 4 FAILED)

`ran=30 filtered_out=579 sum=609`. All four legacy fixtures
`run_wasm_backend_succeeds_for_iterator_prototype_{some,every,find,reduce}_fixture`, whose b5 message
was `uncaught throw: wasm-aot completion: string(callback throw)` / `string(reducer throw)`, are
**ok**. `failures:` block is empty.

## The green is NOT vacuous — checked at the source level, one process, top-level frame

The b5 characterisation was that `.some()` on a `class X extends Iterator` with **no explicit
constructor** returned `0`/number with **zero** callback invocations. I re-ran that exact
discriminator directly (`lila run --execution-backend wasm`, top-level statements, no wrapping
function — b5's frame caveat), and every b5 symptom is gone:

```
some ret=true typeof=boolean calls=2      <- b5: ret=0 typeof=number calls=0
throw YES:boom                            <- b5: NOT-CAUGHT (callback throw discarded)
take+toArray=1,2                          <- b5: TypeError: value is not callable
map+toArray=10,20,30                      <- b5: TypeError: value is not callable
filter=2,3                                <- b5: TypeError: value is not callable
computed=true                             <- static and computed keys now agree
```

`RC=0`, `backend_used: WasmAot`. So all three b5 symptom classes — (i) callee acquisition, (ii)
wrong-typed value with no invocation, (iii) wasm trap — are closed on the same receiver shape that
produced them, not merely on the fixtures.

## Lane 1's new unit target

`cargo test -p lila-aot-wasm --test iterator_helper_dispatch` -> **3 passed, 0 filtered out**,
85.2 s. The third test warns past 60 s and finishes at 85 s. The byte-slack assertions the
integrator flagged as "calibrated but not verified post-repair" hold at this head.

---

# LANE 2 — ZDT era-boundary quartet: ALL FOUR GREEN

`./target/debug/lila test262 run intl402/Temporal/ZonedDateTime/prototype/<m>/era-boundary-gregory.js
--execution-backend wasm`, one process per case, sequential:

| case | b5 | b6 |
|---|---|---|
| `add/era-boundary-gregory.js` | Bug:Runtime `object(handle@1827888: value is not callable)` | **total 1 / passed 1**, Success 1, Bug 0, Crash 0 |
| `subtract/era-boundary-gregory.js` | same handle `1827888` | **passed 1** |
| `since/era-boundary-gregory.js` | `object(handle@1879624: …)` | **passed 1** |
| `until/era-boundary-gregory.js` | same handle `1879624` | **passed 1** |

`NotImplemented 0`, `Crash 0`, `Bug 0` on all four; `FAILED` marker count in the log is **0**.
Wall clock for all four: ~390 s total (~100 s/case), against b5's ~300 s/case — the difference is a
warm function-cache tier, not a code change, so do not read it as a speedup claim.

## The batch's falsifiable prediction is SUPERSEDED, and I stopped its check deliberately

The brief predicted the other 24 era-boundary files carry the same `value is not callable` at
different handles. With the quartet green that premise no longer describes the head, so the live
question became "do the other 24 also pass" — a conformance-breadth question, not a lane gate.

I started it (`lila test262 run intl402/…/<m>/era-boundary` per directory, 7 cases per process) and
**killed it after 9 m 22 s on the first directory**, measured reasons:

- the prefix form holds **6.2 GiB RSS** at 196 % CPU in one process and prints **nothing** until the
  whole directory finishes (the `test262 checkpoint:` line is every 10 cases; a 7-case node never
  emits one), so each directory is a silent block that races `run-watched --stall 900` and would be
  killed at 124 with no verdict;
- at the measured ~100 s/case that is ~40 min of exclusive box time, directly against the rung 1c
  deadline item.

Recorded as REMAINING with an owner, not silently dropped.

---

# LANE 3 — frontend isolation: the 8.7 GiB is now **4.46 GiB**, measured

`ps -o rss` sampled every 25 s against the live child of the isolated chunk, box otherwise idle
(sweep down), 12 samples:

```
18:24:36 etime 00:12 cpu 184%  rss 3,896,560 KiB   mem_used 5056 MiB  avail 11018 MiB
18:25:26 etime 01:02 cpu 196%  rss 4,379,876      mem_used 5363      avail 10712
18:27:56 etime 03:33 cpu 232%  rss 4,455,852      mem_used 5442      avail 10632
18:28:46 etime 04:23 cpu 234%  rss 4,455,852      mem_used 5439      avail 10635   <- flat plateau
```

Full child command line, read from `ps`, so the flag is observed rather than assumed:

```
lila test262 run language/wasm/pass --suite-root …/fake_test262/vendor/test262
  --snapshot-dir /tmp/lila-cli-test262-2525 --snapshot-name cli-wasm-fixture
  --execution-backend wasm --threads 2
```

**Peak 4.46 GiB against b5's measured 8.4-8.7 GiB for the same test** — the lane's `--threads 2`
claim ("roughly halves it"), which the integrator explicitly flagged as an inference from source
reading rather than an observation, is now an observation and it is right. `avail` never drops
below 10.6 GiB, i.e. the margin went from b5's 1.6 GiB to ~10.6 GiB. Flat plateau, so a working set
and not a leak, consistent with b5.

## The isolated chunk BANKED — first time in five attempts across two batches

```
frontend_test262_subset  EXIT=0  ran=1  filtered_out=608  sum=609
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 608 filtered out; finished in 1813.91s
```

b5 tried this test inside the `frontend` chunk **four** times and banked a verdict **zero** times
(two OOM SIGKILLs, two container restarts). Isolated, it passes in one attempt.

### F13's stall ceiling: the real number is now MEASURED, and 900 s was not survivable

The fixer set `chunk_stall()` to 3600 s for this chunk and wrote, honestly, that "3600 is headroom,
not a measurement". Here is the measurement, off this log:

- libtest emits its one and only `has been running for over 60 seconds` line at **t+60 s**.
- The next byte written to that log is the `... ok` at **t+1814 s**.
- So the log is silent for **1,754 s**.

`run-watched --stall 900` kills on 900 s of no growth. **The old ceiling would have killed this run
at ~t+960 s, 854 s short of the finish** — precisely the b5 arithmetic F13 reconstructed from
`b5r4-rung1c-driver.log`, now confirmed prospectively rather than inferred. 3600 s is a 2.05x margin
over the measured 1,754 s. That is the right kind of headroom and the number should not be lowered
below ~2,600 s.

### Memory: the chunk fits with ~2x margin, and this is the number that decides sweep co-scheduling

38 `ps -o rss` samples over the full 30 min, box otherwise idle:

| phase | RSS | `avail` |
|---|---|---|
| ramp (0-1 min) | 3.81 -> 4.28 GiB | 11.0 -> 10.7 GiB |
| plateau (4-23 min) | **4.87 - 5.03 GiB** | 9.9 - 10.0 GiB |
| observed peak | **5.55 GiB** | 9.05 GiB |

Against b5's 8.4-8.7 GiB for the same test with no `--threads` flag. `avail` never fell below
**9.05 GiB**, against b5's **1.6 GiB**. Two consequences worth stating plainly:

1. Lane 3's design goal is met and then some. The chunk is no longer the reason a rung 1c dies.
2. **The b5 scheduling law can be relaxed for THIS chunk specifically, but I did not test that** —
   the sweep is ~4.4 GiB and 5.55 + 4.4 = 9.95 GiB against 15 GiB total, which fits on paper. It is
   an arithmetic claim, not a measurement, and the b5 law was itself an arithmetic claim that turned
   out to be right for the wrong chunk. Recorded as a hypothesis for batch 7, not acted on.

---

# RUNG 1C — the chunked run, at this head (`scripts/rung1c-chunks.sh`, launched verbatim 18:24Z)

Every row is `ran + filtered_out` against the **609** compiled tests recounted above, which is the
arithmetic that makes a chunked run a complete rung 1c rather than a pile of filters.

| chunk | exit | ran | filtered | sum | result | wall |
|---|---|---|---|---|---|---|
| `known_failures` | 0 | 5 | 604 | **609** | 5 passed | 0.01 s |
| `frontend_test262_subset` | 0 | 1 | 608 | **609** | **1 passed** | **1813.9 s** |
| `date` | 0 | 17 | 592 | **609** | **17 passed** | 664.4 s |
| `iterator` | 0 | 30 | 579 | **609** | **30 passed** (b5: 4 FAILED) | 521.7 s |
| `iterator_helpers` | 0 | 14 | 595 | **609** | **14 passed** (b5: 9 FAILED) | 118.9 s |
| `frontend` | 0 | 45 | 564 | **609** | **45 passed** | **273.2 s** |
| `typed_array` | 0 | 58 | 551 | **609** | 58 passed | 715.7 s |
| `array` | 0 | 84 | 525 | **609** | 84 passed | 1134.6 s |

plus the nine chunks banked at b5's head and deliberately retained by the integrator
(`throw_propagation` 2, `dynamic` 11, `heap` 12 = 11 + 1 ignored, `regexp` 33, `object` 35,
`string` 36, `data_view` 38, `functions` 45).

`date` re-ran because the counts sidecar saw 16 banked against 17 declared — the integrator's
decision to keep the lane's out-of-spec sidecar is what made the ZDT lane's only affordable
regression test actually execute. It contains
`date::run_wasm_backend_succeeds_for_temporal_zoned_date_time_era_fixture`, **ok**.

`frontend` at **273.2 s / 45 passed** is the other half of the isolation result: b5 could not bank
this chunk in four attempts and estimated it at ~330 s *including* the 8.7 GiB test. With that test
moved out, the remainder is a 4.5-minute chunk that banks first try.

**Zero failures in 254 executing tests measured at this head.** No `failures:` block in any chunk log.

---

# b6 RUNNER — SESSION 2 (post-fixer verification)

Start 2026-08-10 21:36 UTC. HEAD `a0f411eaf` ("WIP checkpoint: batch 6 runner final rungs")
+ the FINDINGS-FIXER's uncommitted tree (**10** modified files). 4 CPU / 15.7 GiB,
`LILA_CPU_PERCENT=100`.

## State harvested on arrival (nothing re-derived)

- **The sweep is DOWN.** `ps aux | grep -E 'sweep-supervisor|report-all'` returns nothing. Session 1
  killed it at 17:16Z and never restarted it. It has therefore been down ~4 h 20 m, and restarting
  it is my step 8 — see the closing section for what I actually did.
- `target/watched/rung1c-done` carries **15** banked chunks. Missing from the 18: `language`,
  `binary_data`, and `known_failures`. The first two were never reached by session 1; the third is
  absent **by design** after the fixer's finding 13 (the script now deletes its done-row every run,
  because its assertions read three inputs its own `#[test]` count cannot see). Read out of both
  `rung1c-done` and `rung1c-done-counts`; the counts sidecar still holds `known_failures 5`, which
  the script ignores for a chunk with no done-row.
- Debug binary was **stale**: `find crates -name '*.rs' -newer target/debug/lila` = **14** files
  (binary 17:43Z). Fourth batch running. Rebuilt below.

## What the fixer actually changed, read as a diff rather than from the report

`git diff --stat` = 10 files, +354/-145. Three of them are **not** comment-only, so the fixer's
"every code edit is byte-neutral" claim is a claim about emission, not about the diff:

| file | change | byte-neutrality argument |
|---|---|---|
| `lila-ir/src/lowering.rs` | -43: four `properties.insert` literals (`equals`, `toInstant`, `withTimeZone`, `toPlainDateTime`) deleted, folded into the shared table loop | `properties` is a `BTreeMap`, so insertion order is not observable in the shape |
| `lila-ir/src/names.rs` | +41: those four prepended to `TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_METHODS`, **ahead of** batch 6's five | install order preserved exactly (`equals, toInstant, withTimeZone, toPlainDateTime`, then the five) |
| `lila-aot-wasm/src/intrinsics/temporal.rs` | -88/+~25: four hand-written `emit_object_define_function_data` blocks deleted, the loop now covers all nine | same order, same emitter, same args |
| `data.rs` +15, `builtins.rs` +18 | comments only | — |
| `iterator_helper_dispatch.rs` +85 | new negative-control test + 4 renames | test-only |
| `rung1c-chunks.sh` +78, `check-module-boundaries.sh` +37 | script logic | not emission |

I did not run rung G (10 min/side, and I have three unbanked chunks plus a dead sweep to pay for).
The byte-neutrality of the ZDT fold is therefore **argued, not measured** — recorded as REMAINING.
The nine members are exercised end to end by the `date` chunk and the ZDT quartet, both re-run below,
which is evidence about behaviour and not about bytes.

## Rung 0 — `cargo xc`  [EXIT=0, 15 s]  `target/watched/b6r2-xc.log`

`grep -c '^error'` = **0**. Warnings by target, from the `generated N warnings` lines:
`lila-ir` lib **6** / lib-test **5**; `lila-aot-wasm` lib **25** / lib-test **20**;
`lila-test262` lib-test **1**. **Identical to session 1, to b5, and to the fixer's report.**

## Rung 0b — debug binary rebuilt

`cargo build -p lila-cli --bin lila` -> 90 s, EXIT=0, 150,286,880 B at 21:39Z
(was 150,288,760 B at 17:43Z; the 1,880-byte shrink is the four deleted literal install blocks in
`intrinsics/temporal.rs`, i.e. Rust code size, not emitted Wasm).

## `cargo test -p lila-aot-wasm --test iterator_helper_dispatch` — **4 passed / 0 failed**, 96.6 s

`target/watched/b6r2-iterdispatch.log`. `0 filtered out`, so the target really is 4 tests now.

| test | status |
|---|---|
| `iterator_helper_dispatch_differential_separates_two_emitters` | **ok** — the fixer's new negative control, **never run before this** |
| `iterator_helper_take_dispatches_like_drop_on_a_class_receiver` | ok |
| `iterator_helper_map_dispatches_like_flat_map_on_a_class_receiver` | ok |
| `iterator_helper_class_receiver_call_emits_a_valid_module` | ok (warns past 60 s, finishes ~96 s) |

The negative control green is the load-bearing result: the fixer's worry was that the two
differentials had converged (`"abc".take(1)` taking the tail's `ValueKind::String` arm vs
`"abc".drop(1)` taking dispatch), which would have made the two positive tests vacuous. They diverge
by more than the slack. Session 1's recorded trap is also closed by the renames: all four names now
contain `iterator_helper`, so `-- iterator` selects **4 of 4** instead of 0 of 3.

## Blast radius of the fixer's tree, established by reading the diff (this drives what I re-ran)

`git diff crates/lila-aot-wasm/src/functions.rs | grep -vE '^[-+]\s*//'` (after dropping the
`+++/---` header lines) is **empty** — lane 1's file moved by **comments only**. Same for `data.rs`
and `lila-ir/src/builtins.rs`. So the ONLY behavioural surface the fixer touched is
`Temporal.ZonedDateTime.prototype` install + shape (`names.rs` table membership,
`lowering.rs` shape fold, `intrinsics/temporal.rs` installer loop).

Consequences I acted on rather than assumed:

- the `iterator` / `iterator_helpers` chunks banked by session 1 are **not** compiler-stale in any
  behavioural sense, so I did not spend 640 s re-running them;
- the `date` chunk **is** the targeted revalidation of the fold and I re-ran it (below);
- `crates/lila-cli/tests/fixtures/wasm_temporal_zoned_date_time_arithmetic.js` also changed, but
  its diff is a 20-line comment block retracting a vacuous ordering claim. **No JS statement moved.**

### Coverage gap the fold opens, counted rather than asserted

The fold moved four members (`equals`, `toInstant`, `withTimeZone`, `toPlainDateTime`) from
hand-written installer blocks into the shared table. `methodNames` in the arithmetic fixture is
`["withCalendar","add","subtract","until","since"]` — the five NEW ones. Grepping both ZDT fixtures
for the four folded names:

| folded member | called in a CLI fixture? |
|---|---|
| `toPlainDateTime` | **yes** — 16 call sites across both fixtures, plus a `typeof`/`length`/brand block in `…_era.js:94-101` |
| `equals` | **no** |
| `toInstant` | **no** |
| `withTimeZone` | **no** |

So a fold that dropped or misordered `equals`/`toInstant`/`withTimeZone` would be invisible to
rung 1c. That is not a claim the fold is wrong — the diff preserves install order literally, and
`toPlainDateTime` (the one that IS covered) is the last of the four, so a truncation of the block
would have been caught. It is a claim that the ledger's evidence for three of the nine members is
test262, which I did not run for them. **REMAINING, owner T09/ZDT lane.**

## `cargo test -p lila-aot-wasm --lib` — **269 passed / 0 failed / 1 ignored**, 691.0 s

`target/watched/b6r2-aotlib.log`, `0 filtered out`. Session 1 measured **245 passed / 24 FAILED /
1 ignored** on this target and fixed the cause (the two uninterned ZonedDateTime guard strings in
`data.rs`). The totals reconcile exactly: 245 + 24 + 1 = 270 = 269 + 1, so the 24 turned green and
**no test disappeared** — which is the check that matters, because a filter typo or a deleted test
would also produce "0 failed".

This is the second time this target has been run by anyone. It is worth stating why it kept being
skipped and why it should not be: it is the only cheap gate that emits a **full bootstrap**, so a
builtin that reads an uninterned pool string takes 24 tests down at once while every CLI chunk stays
green (a CLI fixture only bootstraps what it touches). 691 s buys that.

# RUNG 1C — completing it. Driver launched 21:55Z, `setsid`+`disown`, `RUNG1C_STALL=1800`

Sweep already down, so nothing to kill. Two deliberate departures from a bare re-run:

1. **I retired `date` from the done-file by hand** (`target/watched/rung1c-done.bak-b6r2` holds the
   pre-edit copy). This is the script's documented property-3 judgement: the counts fingerprint
   cannot see a compiler change, and the fixer's fold is exactly a compiler change to the surface
   `date` covers. `date` banked at 17 tests and still declares 17, so it would otherwise have been
   skipped.
2. `RUNG1C_STALL=1800` rather than the 900 default, because `binary_data::` carries the declared T17
   `Atomics.wait` hang whose `HANG_TIMEOUT` is **900 s** — equal to the default stall. The script's
   own comment at `:403` names that as sitting "directly on that boundary". 1800 moves it off the
   boundary in the safe direction; it cannot mask a real hang, because the hang is bounded by
   `HANG_TIMEOUT` inside the test, not by the stall guard.

## `known_failures::` — **5 passed / 0 failed**, 0.01 s. `ran=5 filtered_out=604 sum=609`.

```
ledger_is_well_formed .. ok       rung_1c_chunks_cover_every_cli_area_module .. ok
routing_takes_the_guarded_path_whenever_the_test_name_is_unknown .. ok
every_expected_failure_carries_a_should_panic .. ok   every_ignored_test_is_declared .. ok
```

So at the fixer's head the 18-chunk / 18-module bijection still holds
(`grep -c '^run_chunk '` = **18** = `grep -c '^mod ' tests/cli/main.rs`), the ledger parses, and
`CURRENT_BATCH = 6 < unfilled-allowed-until = 7` still holds — the T03 row is alive and legal, and
this is the last batch at which that is true.

**One correction to the fixer's finding 13.** The new branch is
`if grep -qx "$name" "$DONE" && [ "$name" = "known_failures" ]`, so the "RE-RUN unconditionally"
message fires only when the chunk is *already banked*. On this run it was **not** in the done-file
(session 1 had hand-deleted it), so the fresh-run path took it and the message never printed. Same
verdict, but the branch the fixer added is **still unexercised at this head** — it will first execute
on the next driver invocation, now that this run banks the row. Recorded so nobody reads today's
green as evidence that guard works.

## Prior-run forensics found in the results file

`target/lane-notes/rung1c-chunks.md` ends session 1 with `=== language START 2026-08-10T19:52:26Z ===`
and **no matching END**. That is the container restart taking the `language` chunk mid-flight, and it
is why `language` has never produced a verdict at any head — matching the T03 row's own claim.

## RESUME PLAN (written before the results, so a container restart does not lose it)

1. driver `target/watched/b6r2-rung1c-driver.log` is running detached (`setsid`, pid 26862).
   Remaining chunks when it started: `date` (retired by hand), `language`, `binary_data`.
   Re-launch with the identical command; the done-file resumes.
2. then `sh target/watched/b6r2-zdt.sh` under `run-watched --label b6r2-zdt --stall 900`
   (four ZDT era-boundary cases, one process each, `sh -n` clean).
3. then the T03 ledger decision (see below), then re-run `known_failures::` to prove it closes.
4. then restart the sweep: `setsid target/test262-scratch/sweep-supervisor.sh ... & disown`.
   Sweep state harvested at 21:36Z, so before/after is checkable: **22** node `.json` files under
   `target/test262-scratch/baseline/`, log inside the 23rd node at `80/250`,
   `grep -c 'test262 quarantine:' baseline-sweep.log` = **0**.

## `date::` re-run after the ZDT fold — **17 passed / 0 failed**, 608.7 s

`ran=17 filtered_out=592 sum=609`. This is the targeted revalidation of the fixer's fold, and it is
the only CLI evidence for it. Both ZDT fixtures are in this chunk
(`…_temporal_zoned_date_time_era_fixture`, `…_arithmetic_fixture`), and between them they assert
`Object.getOwnPropertyDescriptor` writable/enumerable/configurable on all five new members plus
`typeof`/`length`/brand on `toPlainDateTime`. Green.

### Memory: `date::` is now the heaviest rung-1c chunk, not `frontend_test262_subset`

`ps -o rss` sampled every 15 s against the live `cli-986abd5f02521ed6` child
(`target/watched/b6r2-rss-samples.txt`), box otherwise idle, sweep down:

| chunk | peak RSS | min `avail` |
|---|---|---|
| `date::` (17 tests, `--test-threads=3`) | **11.48 GiB** | **3.81 GiB** |
| `frontend_test262_subset` (session 1, 1 test) | 5.55 GiB | 9.05 GiB |

**This inverts the assumption lane 3 was built on.** The isolation work targeted the 8.7 GiB
frontend test; measured at this head, that test is 5.55 GiB and `date::` is 11.48 GiB — 2.1x the
chunk that got its own `RUNG1C_STALL_FRONTEND_SUBSET=3600` and its own preflight guard. `avail` fell
to 3.81 GiB, so `date::` alone leaves less headroom than the isolated chunk does, and the b5
scheduling law ("the sweep and heavy 1c chunks do not fit together") is *more* true for `date` than
for the chunk it was written about. The sweep is ~4.4 GiB; 11.48 + 4.4 = 15.9 GiB against 15.7 GiB
total, i.e. co-scheduling `date` with the sweep does **not** fit, arithmetically.

Session 1's hypothesis "the b5 law can be relaxed for the isolated chunk" survives (5.55 + 4.4 fits);
the generalisation "1c chunks now fit beside the sweep" does not. **Do not restart the sweep while a
rung 1c is running.** I did not, and the ordering below reflects that.

Two caveats stated so this is not over-read: 17 tests at `--test-threads=3` means up to three cold
Temporal Wasm-AOT compiles in flight, which is where the 11.48 GiB lives — it is a concurrency
property of the chunk, not one test; and I have no b5 `date` sample to compare against, so this is a
first measurement, not a regression.

---

# b6 RUNNER — SESSION 3 (container-restart recovery)

Start 2026-08-10 22:42 UTC. HEAD **`adf35046f`** ("WIP checkpoint: batch 6 runner rung-1c chunks").
`git status --porcelain` **empty** — session 2's fixer tree is now COMMITTED, so everything session 2
measured against an uncommitted tree is measured against this head. 4 CPU / 15.7 GiB free 14.6 GiB.

## State harvested on arrival (read, not re-derived)

- **Sweep DOWN.** `ps` shows zero `lila` / `cargo` / `sweep-supervisor` / `report-all` processes.
  Down since session 1 killed it at 17:16Z, i.e. ~5 h 26 m. Restarting it is my last step.
- `target/watched/rung1c-done` = **16** banked chunks. Remaining: **`language`, `binary_data`** only.
  (`known_failures` IS in the file now — session 2 banked it — but the script deletes its row every run.)
- `target/lane-notes/rung1c-chunks.md` ends `=== language START 2026-08-10T22:06:22Z ===` with no END:
  session 2's driver was taken by the container restart mid-`language`, the **second** time that chunk
  has been cut off (19:52Z and 22:06Z). `rung1c-language.log` (22:21Z) shows it had printed a long run
  of `ok` lines with three tests concurrently past 60 s — no `failures:` block, no verdict.
- Debug binary **CURRENT**: `find crates -name '*.rs' -newer target/debug/lila` = **0**
  (150,286,880 B, 21:39Z, built after the fixer's edits). No rebuild this session — first session in
  four batches where step 2 is a no-op.

## Rung 0 — `cargo xc` at `adf35046f`  [EXIT=0, 15 s]  `target/watched/b6r3-xc.log`

`grep -c '^error'` = **0**. Warnings by target: `lila-ir` lib **6** / lib-test **5**;
`lila-aot-wasm` lib **25** / lib-test **20**; `lila-test262` lib-test **1**.
**Identical to sessions 1 and 2, to b5, and to the fixer's report** — the commit of the fixer tree
moved nothing.

## Rung 1c driver relaunched 22:42Z, detached (`setsid`+`disown`), `RUNG1C_STALL=1800`

Same two deliberate settings as session 2: `RUNG1C_STALL=1800` keeps the stall guard off the 900 s
`HANG_TIMEOUT` boundary that `binary_data`'s declared T17 `Atomics.wait` hang sits on. It skipped all
16 banked chunks by name and entered `language`.

### The fixer's finding-13 branch is now EXERCISED — session 2's caveat is closed

Session 2 recorded that the new unconditional-re-run branch had never executed (the row was absent, so
the fresh-run path took it). This run's driver log opens with the branch's own message:

```
rung1c: RE-RUN known_failures unconditionally -- it holds the chunk/module partition check, whose
inputs (main.rs `mod` list, tests/cli/*.rs, this file's run_chunk lines) the counts fingerprint cannot see.
```

`known_failures EXIT=0 ran=5 filtered_out=604 sum=609`, **5 passed**, 2.08 s. So the guard works on its
first real outing, and the 18-chunk/18-module bijection + `CURRENT_BATCH = 6 < unfilled-allowed-until = 7`
hold at the committed head.

## Counts and partition re-verified at `adf35046f` (pure `awk`/`grep`, no CPU)

`awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++}' crates/lila-cli/tests/cli/*.rs` = **617**;
`frontend.rs` carries **8** `spec-exec-oracle` gates -> **609 compile**, **608 execute** (one
`#[ignore]` in `heap.rs`). `grep -c '^run_chunk ' scripts/rung1c-chunks.sh` = **18** =
`grep -c '^mod ' crates/lila-cli/tests/cli/main.rs`. Unchanged from session 1.

Banked-chunk arithmetic, so the resume is auditable rather than a single opaque total:
`known_failures 5 + frontend_test262_subset 1 + throw_propagation 2 + dynamic 11 + heap 12 (11 exec)
+ date 17 + iterator 30 + iterator_helpers 14 + regexp 33 + object 35 + string 36 + data_view 38 +
functions 45 + frontend 45 exec + typed_array 58 + array 84` = **466 compiled / 465 executing**.
Remaining `language 105 + binary_data 38` = **143**. `465 + 143 = 608` exactly — the T03 row's own
figure, and the resume has lost nothing across three container restarts.

## The T03 close is a DELETION, not a fill — read out of the assertion, before spending the run

`known_failures.rs:1253-1270`: the `unfilled` deadline assertion is inside
`if unfilled_rows > 0 { … }`. With the placeholder row deleted the check is not merely satisfied, it
is **not evaluated**, so `CURRENT_BATCH = 6` (already bumped by the integrator) needs no further move
and `# unfilled-allowed-until: batch-7` becomes inert rather than wrong. The row's own text asks for
"replace this row with the surviving set" — if `language` and `binary_data` land with no undeclared
failure, the surviving set is **empty** and the correct edit is to delete row 67 and nothing else.
Bumping `CURRENT_BATCH` to 7 while the row is alive would be `7 < 7` = red; that is the designed
deadline and it must not be tripped by a well-meaning bump.

# RUNG 1C — `language`: FIRST VERDICT EVER at any head, and it contains a REAL RED

`language` has never produced a per-test result in batches 3-6 (b5 never reached it; b6 session 1 and
session 2 were both taken by the container restart mid-chunk). This run got 105 of 105 started and
**66 printed `ok`** before the child was **SIGKILLed** at t+1200 s:

```
error: test failed, to rerun pass `-p lila-cli --test cli`
  process didn't exit successfully: `.../cli-986abd5f02521ed6 --test-threads=3 'language::'`
  (signal: 9, SIGKILL: kill)
```

`language EXIT=101 ran=105 filtered_out=0 sum=105` -> **NO-VERDICT, correctly NOT banked.** Signal 9
with no `test result:` line is the OOM killer, not the stall guard (which reports 124) — the same
signature b5 measured on `frontend`. Sampled `free`: `avail` fell from 8.5 GiB at t+2 min to
**3.56 GiB at t+13 min**. So `language` joins `date` (11.48 GiB peak, session 2) as a chunk that does
not fit beside anything else, and the b5 scheduling law holds for a third chunk.

## The red, named, with its line and its assertion (this is the batch's first undeclared failure)

```
test language::run_wasm_backend_succeeds_for_aggregateerror_iterable_to_list_fixture ... FAILED
```

`crates/lila-cli/tests/cli/language.rs:1369`. It runs
`lila run --execution-backend wasm crates/lila-cli/tests/fixtures/wasm_aggregateerror_iterable_to_list.js`
and asserts three things: `status.success()`, `stdout` contains `backend_used: WasmAot`, and `stdout`
contains `number(123`. **The message is not recoverable from this log** — libtest prints captured
output in the `failures:` block at the end, and the SIGKILL came first. Its immediate neighbours
(`…_aggregateerror_constructor_properties_fixture`, `…_cross_realm_newtarget_fixture`,
`…_newtarget_prototype_fixture`) all printed `ok`, so this is one fixture, not the AggregateError
surface.

Deliberately NOT reproduced yet: `binary_data` is in flight and one more concurrent cold Wasm-AOT
compile is exactly what killed `language`. Reproduction is queued behind it.

## The `language` red DIAGNOSED to a one-token stale fixture — the compiler is the correct party

Reproduced directly, one process:
`./target/debug/lila run --execution-backend wasm crates/lila-cli/tests/fixtures/wasm_aggregateerror_iterable_to_list.js`
-> `uncaught throw: wasm-aot completion: string(iterator getter wrong value)`, RC=1. That is the
fixture's own `assertThrowsValue(..., "iterator getter")` guard: it caught *an* error whose value was
not the string `"iterator getter"`.

Two probes named the mechanism (`scratchpad/probe1.js`, `probe2.js`, both `lila run --execution-backend wasm`):

| form | result |
|---|---|
| `Object.defineProperty(o, "Symbol.iterator", {get})` — the fixture's spelling | caught `TypeError: AggregateError errors input must be iterable`; the getter never ran |
| `Object.defineProperty(o, Symbol.iterator, {get})` — the real symbol | caught **`iterator getter`** — correct |
| `o[Symbol.iterator] = fn` | caught `direct` — correct |
| `o["Symbol.iterator"] = fn` | `TypeError: … must be iterable` |
| `typeof Symbol.iterator` | **`symbol`** |

So the string `"Symbol.iterator"` is no longer an alias for the well-known symbol, and that is
**spec-correct**: a string-keyed property of that name is not @@iterator, so `AggregateError` must
throw a TypeError. The fixture encodes the old Lila-ism. Under AGENTS.md ("old JavaScript code can
be a reference and oracle, but it is not a constraint") the fixture is the stale party, not the
compiler.

**Fixed** (`crates/lila-cli/tests/fixtures/wasm_aggregateerror_iterable_to_list.js`): the key is now
`Symbol.iterator` rather than `"Symbol.iterator"`, with the measurement recorded in a comment beside
it. Re-run of the whole 127-line fixture: `wasm-aot completion: number(123)`, **RC=0** — which is
exactly the `stdout.contains("number(123")` the test asserts, so the corrected form exercises the
entire body rather than short-circuiting.

### One vacuous pass in the same fixture, reported and NOT changed

`assertTypeError(function() { new AggregateError({ "Symbol.iterator": 1 }); }, "non-callable iterator")`
uses the same string spelling. It still passes, but for the wrong reason: the TypeError it observes is
"no @@iterator at all", not "@@iterator is not callable". Rewriting it to `{[Symbol.iterator]: 1}`
would be the honest form, but computed-symbol keys in an object literal are a path I have not measured
and a new red there costs a 25-minute chunk. **REMAINING, owner: language/T-frontend lane.**

## `language` OOMs at `--test-threads=3` on this box — TWICE, at the same wall clock, and it is not the red

| attempt | env | tests `ok` before death | reds | death |
|---|---|---|---|---|
| 22:43Z | default caches | 66 | 1 (`aggregateerror_iterable_to_list`) | SIGKILL at t+1200 s |
| 23:29Z | `LILA_{FUNCTION,MODULE,PROGRAM}_CACHE_LIMIT_BYTES` = 256/64/64 MiB | **75** | **0** | SIGKILL at t+1200 s |

The second run is the brief's **D2** item actually executed — per-tier cache limits set for the CLI
test children for the first time. **It did not save the chunk.** `avail` fell 8.5 -> 3.6 GiB by
t+10 min in both runs, i.e. the same trajectory, so the resident cost is the three concurrent cold
Wasm-AOT compiles themselves, not the engine's cache tiers. D2 is therefore ANSWERED IN THE NEGATIVE
for `language`; it should not be re-tried as a memory fix, and b5's "the 8.7 GiB is compile-cache
sizing" hypothesis is disconfirmed for this chunk. (It was never tested against
`frontend_test262_subset`, which was solved by `--threads 2` inside the test instead.)

`LILA_CPU_PERCENT` is not a lever here: it is consumed by `scripts/capped.sh` as a CPU *share*,
and `rung1c-chunks.sh:381` hardcodes `LILA_CPU_PERCENT=100` inside `run_chunk`, so an outer export
is overridden. There is no `--jobs` equivalent for `lila run`.

Both deaths land at the same alphabetical frontier (`lexical_super_home_object` / `spec_get_v`), which
is not a single poison test — the second run simply got 9 further.

### Complementary tail run, so `language` is measured even though it cannot bank

`comm` of the 75 `ok` names against the 105 `fn` names in `tests/cli/language.rs` leaves **30**
un-measured, all in the `o…v` tail. libtest accepts multiple filters, verified with `--list`:
`cargo test -p lila-cli --test cli -- <30 names> --list` selects **exactly 30 tests**. Running that
set alone is a legitimate rung-1b measurement of the remainder; it is NOT a banked chunk and I have
not written one.

### The real fix is the one lane 3 already demonstrated — and it is an integrator edit

`language` needs what `frontend` got: its heaviest tests moved into their own module + `run_chunk`
line, keeping `known_failures::rung_1c_chunks_cover_every_cli_area_module` green. Lowering
`--test-threads` remains banned (property 1). **Owner: integrator.** Until then rung 1c is 17 of 18
chunks; `language` is the only chunk that has never banked at any head in four batches.

## `language` COMPLETE by union, 105/105, zero failures

`cargo test -p lila-cli --test cli -- --test-threads=3 <the 30 tail names>` (log
`target/watched/b6r3-language-tail.log`) -> **`test result: ok. 30 passed; 0 failed; 579 filtered
out`**, 498.2 s. `30 + 579 = 609`, the compiled total, so the filter selected what it claimed.

**75 (second chunk attempt) + 30 (tail run) = 105 = every `fn` in `tests/cli/language.rs`**, and the
one red among them is the fixture fixed above. So `language` is measured green in full — but by two
complementary runs, **not** by a banked chunk, and I have written no done-row for it. The distinction
matters: `rung1c-done` is the artefact the next session resumes from, and a hand-written row there
would claim a verdict the script never saw.

---

# RUNG 1C — `binary_data`: BANKED, and it carries the batch's second drift

`binary_data EXIT=101 ran=38 filtered_out=571 sum=609 -> test result: FAILED. 37 passed; 1 failed`,
870.2 s. **Banked** — correctly: it produced a `test result:` line, which is a verdict, and the
script banks verdicts rather than exit codes. First `binary_data` verdict at any head.

```
---- binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture stdout ----
note: test did not panic as expected at crates/lila-cli/tests/cli/binary_data.rs:550:15
```

That is drift row 2 of `batch-workflow.md`'s table — **a declared failure has started passing.** The
T17 `Atomics.wait` hang, which cost rung 1c its `--skip` for three batches, no longer hangs: the
guarded child returns, and the test's asserts (`status.success()`, `backend_used: WasmAot`,
`stdout.contains("number(901")`) all hold. Confirmed alone under `--exact`: **146.2 s**, same message.

## T17 CLOSED — four coordinated deletions, then re-verified on BOTH routing paths

The test's own doc comment specified the close ("the day it does … delete the ledger row, the
attribute and this comment together"). Done:

1. `crates/lila-cli/tests/known-failures.tsv` — the `hang`/T17 row deleted (73 -> 72 lines).
2. `crates/lila-cli/tests/cli/binary_data.rs:548` — `#[should_panic(expected = "lila run
   exceeded")]` removed, `pub(crate) fn` -> `fn`, doc comment rewritten to record the measurement.
3. `crates/lila-cli/tests/cli/known_failures.rs:638` — the `const _: fn() = …atomics_wait_core…`
   existence assertion removed (`ledger_is_well_formed` rejects an assertion with no row).
4. the module doc's "**1** `should_panic` attribute" -> **0**.

### The close was not free, and the tripwire that caught it is worth recording

`cargo xc` EXIT=0, but `known_failures::` then went **4 passed / 1 FAILED**:
`routing_takes_the_guarded_path_whenever_the_test_name_is_unknown` carried
`assert!(!hangs.is_empty(), "no cli hang rows, so the loop below would assert nothing. If the hang is
genuinely fixed, delete this assertion along with the row.")` — an explicit **anti-vacuity guard**
that fires the moment the last hang row goes. It is the best-behaved assertion I have met in this
tree: it failed with the instruction for its own removal. Removed, with a comment stating that the
loop is now vacuous *by design* and that the three unconditional `execution_path` assertions above it
are what keep the test meaningful.

Re-verified after all five edits:
- `cargo xc` -> **EXIT=0**, `grep -c '^error'` = 0, warning counts unchanged (ir 6/5, aot-wasm 25/20,
  test262 1).
- `known_failures::` -> **5 passed / 0 failed**, `ran=5 filtered_out=604 sum=609`.
- `binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture` under `--exact` -> **1 passed**,
  135.6 s. This one matters more than it looks: deleting the row moves the test from the guarded-child
  path to the **bounded in-process** path (`execution_path` routes on ledger membership), so the
  passing result had to be re-established on the path it will actually take from now on. It was.

---

# LANE 2 — ZDT era-boundary quartet RE-VERIFIED at the committed head, post-fold: ALL FOUR GREEN

Session 1 measured these four green at `e708f1f22` + the *iterator/ZDT lanes'* tree. The
FINDINGS-FIXER then folded four ZDT prototype members (`equals`, `toInstant`, `withTimeZone`,
`toPlainDateTime`) out of hand-written installer blocks into the shared table
(`lila-ir/{lowering,names}.rs`, `intrinsics/temporal.rs`), and session 2 re-ran only the `date::`
CLI chunk against it. This is the test262-level re-run at `adf35046f`, one process per case
(`target/watched/b6r3-zdt.log`):

| case | outcome | wall |
|---|---|---|
| `intl402/…/ZonedDateTime/prototype/add/era-boundary-gregory.js` | total 1 / **passed 1**, Success 1, Bug 0, Crash 0, NotImplemented 0 | 312 s |
| `…/subtract/era-boundary-gregory.js` | **passed 1**, Bug 0, Crash 0 | 299 s |
| `…/since/era-boundary-gregory.js` | **passed 1**, Bug 0, Crash 0 | 292 s |
| `…/until/era-boundary-gregory.js` | **passed 1**, Bug 0, Crash 0 | 293 s |

b5's labels for these were `object(handle@1827888: value is not callable)` (add/subtract) and
`handle@1879624` (since/until). Both handle clusters are gone at this head **after** the fold, so the
fold did not regress the surface it touched. ~300 s/case here against session 1's ~100 s/case: that is
a cold function-cache tier (this session ran ~1 h of CLI chunks that evicted it), not a code change —
do not read either number as a performance claim.

The brief's falsifiable prediction (the other 24 era-boundary files carry the same label at different
handles) remains **superseded and unmeasured**: with the quartet green the premise is gone, and
session 1 measured the prefix form at 6.2 GiB RSS with no output until a directory completes, i.e.
~40 min of exclusive box time that races the stall guard silently. **REMAINING, owner: T09/ZDT lane.**

## `language` OOMed a THIRD time — 66 / 75 / 75 `ok`, all three at t+1200 s

Third attempt (00:30Z driver, default caches again since the tier limits made no difference):
`language EXIT=101 ran=105 filtered_out=0 sum=105`, **75 `ok`, 0 FAILED**, SIGKILL. `free` showed
`avail` at **1.14 GiB** two minutes before the kill. Three attempts, three OOMs, two of them at
exactly 75 completed tests — this is a reproducible capacity limit, not a flake, and no runner-level
knob moves it (cache tiers: no effect, measured; `LILA_CPU_PERCENT`: overridden inside
`run_chunk`; `--test-threads`: banned by property 1).

The third attempt also re-confirms the fixture repair: the run that previously carried
`aggregateerror_iterable_to_list … FAILED` now carries **zero** failures over the same 75 tests.

# RUNG 1C AT THE END OF SESSION 3 — 17 of 18 chunks banked, 503 of 608 executing tests

`target/watched/rung1c-done` (17 rows): `throw_propagation dynamic heap regexp object string data_view
functions frontend_test262_subset iterator iterator_helpers frontend typed_array array date
known_failures binary_data`. Only **`language`** has no banked verdict.

| chunk | this session | note |
|---|---|---|
| `known_failures` | **5 passed**, 0.02 s | re-ran 3x (script deletes its row every run); green after the T17 close |
| `binary_data` | **38 passed / 0 failed**, 881.9 s | re-banked after the T17 close; was 37/1 |
| `language` | **NO VERDICT** x3 (SIGKILL) | 105/105 measured green by union of two runs; see above |

Banked executing-test arithmetic: the 465 carried into this session **+ 38 (`binary_data`)** = **503**.
Remaining unbanked = `language` 105. `503 + 105 = 608`. The two chunks the T03 row named as "never
produced a verdict at any head" have both now produced one; `binary_data` banked, `language` did not.

## Sweep restarted 01:20Z (step 8), state recorded so the next session can diff it

Killed by session 1 at 17:16Z, down **8 h 04 m**. Relaunched verbatim
(`setsid nohup target/test262-scratch/sweep-supervisor.sh … & disown`, marker
`=== b6r3 sweep restart … ===` appended to `baseline-sweep.log`). Alive: supervisor + one
`lila test262 report-all --snapshot-name baseline-wasm-aot-b2 --snapshot-dir
target/test262-scratch/baseline --threads 2 --jobs 2 --resume`, 77 % CPU.

State at restart, so before/after is checkable rather than remembered:
**22** node `.json` snapshots under `target/test262-scratch/baseline/`;
`grep -c 'test262 quarantine:' target/test262-scratch/baseline-sweep.log` = **0**.

The sweep was deliberately down for the whole of sessions 1-3 while rung 1c ran. This session
produced the third and fourth independent confirmations of the b5 scheduling law: `language` OOMs on
an **idle** box, and `date` was measured at 11.48 GiB in session 2. Do not co-schedule.

## Sweep health check (the closing step), measured twice

- `ls target/test262-scratch/baseline/*.json | wc -l` = **22** (unchanged from the restart — the node
  in flight has not completed yet).
- Log **is growing**: 19,991 -> 20,025 bytes over 150 s, last line `test262 checkpoint: 100/250 cases`.
- One `report-all` process, `etime 06:07`, `time 00:12:03`, **196 % CPU**, 4.34 GiB RSS.
- The journal fired on startup exactly as session 1 predicted:
  `test262 attempt journal: built-ins/Array/prototype/some/15.4.4.17-7-b-{11,12}.js was in flight when
  a previous process died (strike 1 of 2)`. **These two strikes are session 1's deliberate kill, not a
  crash** — they will be forgiven on completion. A `quarantine:` line naming either would be a false
  positive for lead A. `grep -c 'test262 quarantine:'` is still **0**.

## SESSION 3 CLOSING STATE

Tree: my four edits were committed by the orchestrator as `be2f4edc6` / `e0c61f428`;
`git status --porcelain` is empty and each edit verified present at HEAD (`Symbol.iterator` in the
fixture, zero `atomics_wait_core` rows in the tsv, zero `#[should_panic]` attributes in
`binary_data.rs`). Gate re-run after the last edit: `cargo xc` **EXIT=0**, 0 errors, warning counts
unchanged; `known_failures::` **5 passed**.

REMAINING, each with an owner and a reason:
1. **`language` cannot bank** — needs its heaviest tests split into their own module + `run_chunk`,
   the treatment `frontend` got. **Owner: integrator.** Everything else is measured green.
2. **The other 24 ZDT era-boundary files** — 6.2 GiB, silent, ~40 min. **Owner: T09/ZDT lane.**
3. **`{ "Symbol.iterator": 1 }` vacuous pass** in the AggregateError fixture. **Owner: language lane.**
4. **Rung G was not run** this batch, so the ZDT fold's byte-neutrality is still argued, not measured
   (session 2's note). **Owner: integrator.**
5. **`equals` / `toInstant` / `withTimeZone`** have no CLI fixture coverage after the fold (session 2's
   count). **Owner: T09/ZDT lane.**
