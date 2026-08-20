# Batch 6 — INTEGRATOR notes (serial, `cargo check`/`cargo xc` only)

Session start 2026-08-10 ~16:35 UTC, HEAD `7f690152c` ("WIP checkpoint: batch 6
review/integrate"), branch `claude/test-driven-rust-opus-pp6giw`.
Machine 4 CPU / 15 GiB. **The sweep was left running throughout** (supervisor pid
4128, `lila test262 report-all --threads 2 --jobs 2`, ~4.4 GiB RSS): `cargo check`
is not rung 1c and does not contend with it the way the 8.7 GiB frontend test does.
No test, build, test262 or git command was run by this session.

## State on arrival — the three lanes were already COMMITTED and already compile

`git status --porcelain` empty; the three lanes' files are all inside `7f690152c`.
So "apply the notes lane by lane with `cargo check` between" starts from a tree that
already checks:

| gate | on arrival |
|---|---|
| `cargo check -p lila-ir` | 0 errors, **6** warnings (b5 baseline: 6) |
| `cargo check -p lila-aot-wasm` | 0 errors, **25** warnings (b5 baseline: 25) |
| `cargo xc` (workspace, all targets) | **EXIT=0**, 39 s cold-ish |
| `cargo fmt --all -- --check` | ONE diff (`planning.rs:331`) |
| `./scripts/check-module-boundaries.sh` | **RED** — the ZDT lane's §4 finding |

Warning counts per target, against b5 session 3's measured baseline
(`target/watched/b5r3-xc.log`): `lila-ir` lib 6 / lib-test 5,
`lila-aot-wasm` lib 25 / lib-test 20, `lila-test262` lib-test 1. **Identical.**
Batch 6 added ~2,300 lines and no warning.

So the integration work is the notes' explicit integrator patches, not error triage.

## Lane 1 — iterator-helper-static-key-call-on-a-class-receiver (ship-with-fixes)

### Applied: the §3 "PATCH FOR THE INTEGRATOR" — the root cause, in `lila-ir`

`crates/lila-ir/src/lowering.rs`, `standard_builtin_signature`'s
`StandardBuiltinId::IteratorConstructor` arm: `constructor_instance` was
`ValueInfo::undefined()`, now
`with_instance_prototype(fresh_constructed_instance_info(), Some(iterator_prototype_shape()))`.

The lane could not edit this file (owned by the ZDT lane) and wrote the patch out
instead. Three things I checked by reading before applying it, because the lane
could not compile and the patch as written in the note does not compile:

1. **The note's spelling is wrong by one `Some`.** `with_instance_prototype` takes
   `Option<Box<HeapShape>>` (`lowering.rs:7668`) and `iterator_prototype_shape()`
   returns `Box<HeapShape>` (`:4445`). Applied with the `Some(...)`.
2. **The mechanism is real and is exactly as described.** `lower_class` binds
   `inherited_instance` from the heritage signature's `constructor_instance` and
   keeps it whenever `possible_kinds != KindSet::EMPTY` (`:19294`) — and
   `ValueInfo::undefined()` is non-empty, so it was inherited verbatim. That is
   what typed `new S()` as `undefined` for `class S extends Iterator {}`.
3. **The prototype argument is not load-bearing for the class case** and I said so
   in the comment rather than implying more than the change does: `lower_class`
   immediately overwrites the instance's prototype with the subclass prototype
   (`:19295`), whose own prototype is already `Iterator.prototype` via
   `heritage_prototype` (`:19218`) — `IteratorConstructor`'s static shape carries
   `prototype -> iterator_prototype_shape()` (`:3942`). The layered prototype only
   matters for a direct `new Iterator()`.

`cargo check -p lila-ir` after: 0 errors, 6 warnings (unchanged).

**What this changes downstream, stated because it is a behaviour change I cannot
test:** `receiver_shape_targets_iterator_helper` now resolves for these receivers,
so the seven repaired helper blocks reach the shared dispatch through their
*original* guard rather than through the lane's new fall-back predicate — both
routes end in the same emitter. It also makes lowering's own `receiver_is_iterator`
probe (`lowering.rs:21281`, which reads `forEach` off the receiver shape) true, so
`forEach` on such a receiver now lowers to `CallMethod` and takes the emitter's
`forEach` block instead of the property-read path. That block is the same
`emit_iterator_prototype_helper_method_call` that `drop`/`flatMap` — measured green
throughout batch 5 — already used. `iterator_helpers::run_wasm_backend_calls_iterator_prototype_for_each_on_a_class_receiver`
is the test that decides it; **run the `iterator_helpers` chunk first.**

### Filed, deliberately NOT applied: §3(3)

`lowering.rs` still types the result of *calling* `Iterator()` as the `Iterator`
constructor function value, and the result of `it.toArray()` as the `toArray`
function value (the two `standard_builtin_value_info` arms the lane names). Both are
wrong; neither is load-bearing for the thirteen tests (`toArray` reaches the generic
tail and works), and changing `toArray`'s static kind to `Array` moves every
downstream lowering decision about that value. It needs its own lane with a run,
not a ride-along in a session that cannot run anything.

### Also not applied: the audit of the other 582 `constructor_instance` arms

Recorded by the lane as an unclaimed deliverable; still unclaimed. It is a real one —
the same one-token mistake in any other constructable builtin's arm produces exactly
this defect class, and nothing in the type system stops it.

## Lane 2 — zdt-arithmetic-surface (ship-with-fixes)

### Applied: the §4 finding — `check-module-boundaries.sh` was RED at HEAD

```
check-module-boundaries: crates/lila-ir/src/lib.rs has 169 non-test lines; expected at most 140
```

The lane verified it is pre-existing (`git show HEAD:...` counts 169 too) and handed
it over. The interesting part is *why*, and it changes the fix: measured with `awk`,
those 169 lines are 8 blank + 21 whole-line comments + **140 lines of code — exactly
the budget.** Every line over the limit is a doc comment added by the theory rounds
to point a re-exported contract type at its `docs/rust-rewrite/contracts/` file. The
file had not grown; documenting it had.

So the fix is the metric, not the number: `non_test_lines()` now skips blank lines
and whole-line comments, and both budgets (`lila-ir` 140, `lila-aot-wasm` 180)
stay where they are. `lila-ir/src/lib.rs` = 140/140, `lila-aot-wasm/src/lib.rs`
= 101/180. Raising the number would have ratcheted a budget for a file that had not
gained a line of code — and this budget exists to keep implementation out of a crate
root, which comments cannot be.

Verified, including the negative control the lane asked for: script green; moving
`builtins/temporal_zoned_date_time_methods.rs` aside makes it print
`missing file: ...` and exit 1; restored, green again.

### Read and accepted without change

The five new members are registered at all seven sites in the shape the existing
`TemporalZonedDateTimePrototypeWithTimeZone` uses; `standard.rs` dispatch, `module.rs`
`None` arm, `bootstrap.rs` no-op arm, `intrinsics/temporal.rs` install loop,
`planning.rs` arm + `standard_builtin_length` = 1. The `self.functions.get(...)`
borrow in the new install loop is fine because `functions` is a borrowed field with
the builder's `'a` lifetime, not a borrow of `self` — the sibling
`to_plain_date_time_meta` above it does the same. The new module's
`from` -> `HEAP_OBJECT_BOXED_PAYLOAD_OFFSET` read is the same idiom
`emit_temporal_zoned_date_time_equals` (`temporal.rs:3200-3213`) already uses.

## Lane 3 — rung1c-frontend-isolation (rework)

### The one thing I changed in the lane's own work: the cache limits, 8 GiB -> 1 GiB

`scripts/rung1c-chunks.sh` set `LILA_FUNCTION_CACHE_LIMIT_BYTES=8589934592`
(8 GiB) with module/program at 512 MiB, copying the shape of the sweep invocation
in `batch-workflow.md`. The lane's reasoning about *what* the limits do is correct
and I verified it independently — `lila-engine/src/cache.rs` implements all
three tiers over `fs::read`/`fs::write` (`impl CacheStore for FunctionCache`,
`cache.rs:220-235`), so they bound disk, not RSS, and they are indeed not the OOM
fix. Two measurements say the *values* are wrong for this box:

1. **The cache is one shared directory with no per-process keying.**
   `lila_cache_root()` = `$LILA_CACHE_DIR` else `~/.cache/lila`
   (`cache.rs:341-357`). The supervisor actually running on this box right now
   uses **1 GiB / 64 MiB / 64 MiB** — read out of
   `target/test262-scratch/sweep-supervisor.sh` *and* confirmed live in
   `/proc/4131/environ` — not the doc's 32 GiB. A chunk that let the shared
   directory grow to 8 GiB hands the next sweep process 7+ GiB to prune back to
   70 % of its own 1 GiB budget. Two budgets over one directory undo each other.
2. **The disk is a fixed per-session allowance.** Measured now: 19 GiB available,
   `target/debug` 7.3 GiB, `~/.cache/lila` **948 MiB** — i.e. the function tier
   is already sitting at the sweep's cap. Spending a third of the remaining
   allowance on a cache to make rung 1c warm is a bad trade when "no space left on
   device" mid-chunk presents as anything but a disk problem.

Now 1 GiB / 64 MiB / 64 MiB, identical to the sweep. The real hygiene win survives
and is the part that was always true: the module and program tiers are keyed by
source text and never serve a hit across distinct fixtures, so they go **down**
from the 512 MiB default to 64 MiB. The comment in the script carries both
measurements so the next person does not re-raise it blind.

### Kept, after checking it myself: the counts sidecar (the lane's out-of-spec addition)

The lane flagged this as the thing it would accept being reverted. I kept it, and
it is the reason the ZDT lane's only affordable regression test will actually run:
`date` banked at 16 tests in batch 5 and `tests/cli/date.rs` now declares 17, so
the sidecar makes that chunk re-run instead of being skipped by a done-file that
cannot see the difference.

I extracted `module_test_count` / `banked_test_count` / `record_test_count` and ran
them against a synthetic tree rather than trusting the lane's simulation: a rewrite
replaces a row instead of duplicating it, an unknown chunk reads back empty, an
unreadable module reads 0 (the fail-safe the skip predicate keys on), and rows
survive the comment header. `sh -n` on both edited scripts is clean. `set -u` and
no `set -e`, so `grep -v` returning 1 on a first write does not abort the run.

### Checked and accepted: the module move, and the hygiene test's view of it

`grep -c '^run_chunk '` = 18, `mod` declarations in `main.rs` = 18, `*.rs` under
`tests/cli/` = 19 (`main.rs` declares no tests). The overlap rule the move depends
on is `format!("{other}::").ends_with(&format!("{chunk}::"))`
(`known_failures.rs:1554`): `"frontend_test262_subset::"` does not end with
`"frontend::"`, and libtest's substring filter `frontend::` does not select
`frontend_test262_subset::…` either, so neither chunk needs a `--skip` and
`array`/`typed_array` stays the only overlapping pair. `MINIMUM_RUNG_1C_CHUNKS` is
10. The lane's claim that `-- --exact` cannot express this is correct as written.

### The done-file question the lane handed over — answered with evidence, not a coin toss

The lane left "batch 6 rewrote the compiler under nine banked chunks; invalidating
them costs ~45 min" as the integrator's call. Measured instead of judged: the
emitter change only reaches a `CallMethod` with a static key in
{`find`,`reduce`,`take`,`map`,`every`,`some`,`filter`} on a receiver whose kind set
is contained in `{Object, Function} ∪ NULLISH` — and lowering only builds that
`CallMethod` when the receiver is **not** an array (`lowering.rs:21290-21305`).

- Fixtures calling any of those methods: 7 `wasm_iterator_prototype_*`,
  ~11 `wasm_iterator_helper_class_receiver_*`, 5 `wasm_typedarray_*`,
  8 `wasm_array_*`, 2 `wasm_string_*`. Of the banked chunks only `string` appears,
  and both of its receivers are `Array.from(...)` and an array literal — array
  kind, so lowering never builds the `CallMethod` and the path is unreachable.
- Fixtures containing `extends Iterator` — the receiver the `lila-ir` typing fix
  moves: **18, every one of them `wasm_iterator_*`.**

Both populations land entirely in `iterator` and `iterator_helpers`, which the lane
already removed from the done-file. So the nine banked chunks stay banked, and that
is a measurement rather than optimism. `date` re-runs on the counts guard, and
`known_failures` re-runs because it was removed *and* because I edited it.

## Cross-lane: the ledger deadline (batch inheritance item D3)

Rung 1c is **not** complete, so the T03 `UNFILLED` row cannot be honestly replaced:
`frontend` (46), `typed_array` (58), `array` (84), `language` (105) and
`binary_data` (38) have never produced a verdict at any head. But leaving
`CURRENT_BATCH = 3` in batch 6 makes the tripwire inert — `3 < 4` has passed for
three batches — which is the vacuous state the mechanism exists to prevent.

So I took the alternative the assertion names in its own failure message:

- `known_failures.rs`: `CURRENT_BATCH` 3 -> **6**, with the batch-5-era narrative in
  its doc comment replaced by what was actually measured (12 of 17 chunks, 276 of
  607 tests; `frontend::inspect_reports_phase_eighteen_global_ir_shape` is fixed and
  green and is no longer a candidate row; the 13 reds are all the iterator lane's
  and two of the three symptom classes carry panic text no `should_panic` substring
  can pin).
- `known-failures.tsv`: `# unfilled-allowed-until: batch-4` -> **`batch-7`**, and the
  `UNFILLED` row's `reason` rewritten to the current measurement.

Both are visible one-line diffs and `6 < 7` holds, so `ledger_is_well_formed` passes
and the deadline is a real one-batch deadline again instead of one set three batches
in the past. I re-validated every row against the parser's rules by hand (6 columns,
no padded or empty column, reason >= 20 chars, evidence's first token an existing
path outside `target/`, no `:<digit>` anywhere in evidence, sort order unchanged).

## Type strengthening (AGENTS.md "code invariants before test invariants")

Two, both compile-checked, neither changing emitted behaviour:

1. **`ZonedDateTimeArithmetic` / `ZonedDateTimeDifference`** replace the `subtract:
   bool` / `since: bool` parameters of the two new ZDT emitters, and the delegate
   is now a total function on the closed set rather than an `if` inside the body.
   The two call sites are adjacent arms of one `match` in `standard.rs`, spelled
   `(false, function)` and `(true, function)`: a transposition compiled, formatted
   and type-checked, and made `zdt.add(d)` subtract. Re-exported from
   `builtins/mod.rs` the way `UnvalidatedEpochNanoseconds` and
   `TemporalCalendarId` already are.
2. **The ten `Iterator.prototype` helper guards** in `emit_method_call` matched a
   bare literal (`name == "take"`) while the block dispatched
   `IteratorHelper::Take` — two independent sources for one fact, in a family that
   has already shipped this exact defect twice, and batch 6 added a *second*
   emission site to seven of those blocks. They now read
   `name == IteratorHelper::Take.property_name()`, so the guard and the emission
   come off one enum. `IteratorHelper::property_name`'s doc says why.

## Gate status at the end of this session

| gate | result |
|---|---|
| `cargo check -p lila-ir` | **0 errors**, 6 warnings (baseline 6) |
| `cargo check -p lila-aot-wasm` | **0 errors**, 25 warnings (baseline 25) |
| `cargo check -p lila-cli --all-targets` | **0 errors**, 0 warnings |
| `cargo xc` (workspace, all targets) | **EXIT=0** — ir lib 6 / lib-test 5, aot-wasm lib 25 / lib-test 20, test262 lib-test 1: **identical to the b5 baseline, no new warning** |
| `cargo fmt --all -- --check` | **clean** (one pre-existing diff in `planning.rs` fixed) |
| `./scripts/check-module-boundaries.sh` | **ok**, and its negative control fires |
| `sh -n` on both edited scripts | clean |

## What the runner must do, in this order — and what I could NOT verify

Nothing in this session was executed: no test, no build, no `lila`, no test262, no
git. Every behavioural claim below is unverified by construction.

1. **Rebuild `target/debug/lila`.** It is from 09:28Z on 2026-08-10 and the compiler
   has changed twice since. b5 measured it stale twice.
2. **`iterator_helpers::` first** (13 -> 14 tests). It decides both halves of lane 1:
   the emitter routing *and* the `lila-ir` typing fix, whose one behavioural
   surprise is that `forEach` on these receivers now lowers to `CallMethod`.
   `run_wasm_backend_calls_iterator_prototype_for_each_on_a_class_receiver` is the
   test that catches it if that is wrong.
3. **`iterator::`** (30 tests, 4 of them the batch-5 reds), then
   `cargo test -p lila-aot-wasm --test iterator_helper_dispatch` (3 tests, new,
   never run — its 8-byte slack against a 69-77 byte signal is calibrated but not
   verified post-repair; read the printed byte counts before widening it).
4. **`known_failures::`** (5 tests, 0.02 s) — it is the only check of the 18-chunk
   partition, of the ledger edits above, and of `CURRENT_BATCH = 6` against
   `batch-7`.
5. **`date::`** (17 tests) for the ZDT fixture, then the 4-case era-boundary gate.
   If it fails on *values* rather than on `value is not callable`, the defect is in
   the PlainDateTime delegate, not in the delegation.
6. `frontend_test262_subset::` alone, **with the sweep down**, sampling
   `ps -o rss` — the lane's "`--threads 2` roughly halves 8.7 GiB" is an inference
   from source reading, not an observation, and it is the one number that decides
   whether a chunked rung 1c can ever share this box with the sweep.

Unclaimed and still open, recorded so they stop being invisible:

- The audit of the other 582 `constructor_instance` arms of
  `standard_builtin_signature` for the same one-token mistake lane 1 found in one.
- `lowering.rs`'s two wrong `standard_builtin_value_info` results (`Iterator()` and
  `it.toArray()`), filed by lane 1 and deliberately not ridden along.
- The eight CLI tests with no wall-clock bound (lane 3's side finding); the obvious
  fix moves 8.7 GiB into the libtest process, so it needs its own design.
- The ZDT lane's four named divergences (DST round trip, default `largestUnit`,
  over-strict `TimeZoneEquals`, `GetDifferenceSettings` ordering) — all bounded,
  all written at their sites, none reaching the 4-case gate.

---

# Batch 6 — INTEGRATOR session 2 (serial, `cargo check`/`cargo xc` only)

Session start 2026-08-10 ~20:05 UTC, HEAD `002756629` ("WIP checkpoint: batch 6
runner ladder"), branch `claude/test-driven-rust-opus-pp6giw`. 4 CPU / 15 GiB.
**The sweep is DOWN** — the b6 runner killed supervisor 4128 / report-all 4131 at
17:16Z and did not restart it; `ps` on arrival shows 0 `lila`, 0 `cargo`, 14.2 GiB
free. No test, build, `lila`, test262 or git command was run by this session.

This session follows the runner, not the lanes: the three lanes and integrator
session 1 are inside `002756629`, and `git status --porcelain` was empty on
arrival. So "apply the notes lane by lane" starts from a tree where every
explicit integrator patch in the three notes is already in, and the work is the
items those notes left owned-but-unapplied plus the type strengthening the
runner's blocker asks for by name.

## Gate state on arrival — all green, warning counts unchanged since b5

| gate | on arrival |
|---|---|
| `cargo xc` (workspace, all targets) | **EXIT=0**, 54 s cold |
| `lila-ir` | lib **6** / lib-test **5** warnings |
| `lila-aot-wasm` | lib **25** / lib-test **20** warnings |
| `lila-test262` | lib-test **1** warning |
| `cargo fmt --all -- --check` | clean |
| `./scripts/check-module-boundaries.sh` | ok |

Identical to the b5 baseline (`target/watched/b5r3-xc.log`) and to the runner's
`b6r-xc.log`. Baseline log kept at `b6i2-xc-baseline.log` in this session's
scratchpad.

## The one type strengthening this batch actually earned — `TemporalDifferenceGuard`

The runner's blocker (`b6-runner-findings.md`, "batch 6 shipped a compile-time
panic in every full bootstrap") ends with: *"Nothing in the type system catches
this ... It is a textbook case for the AGENTS.md 'code invariants before test
invariants' list, and I have recorded it as such rather than pretending the fix
closes it."* The runner's fix was a second hand-written intern block. This
closes it.

**The shape of the defect, restated so the fix can be judged against it.** The
`DifferenceTemporal*` guards throw a RangeError whose message is a *pool string*.
`StringPool::payload` (`data.rs:3993`) is `refs.get(value).unwrap_or_else(||
panic!(...))` — it panics rather than degrading. The message reached the emitter
as a bare `&str` parameter and the pool was a hand-written list in `data.rs`: two
independent sources for one fact, with nothing at compile time able to compare
them. The ZDT lane spelled two new literals; `cargo test -p lila-aot-wasm
--lib` went **24 red**, and 22 of the 24 are not Temporal tests at all — they
emit a full bootstrap, so any builtin body that reads an uninterned pool string
takes them all down together.

**What landed** (all `cargo check`-verified, no behaviour change):

1. `crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs` — new closed
   enum `TemporalDifferenceGuard` with five variants (`PlainDate`,
   `PlainDateTime`, `PlainYearMonth`, `ZonedDateTime` same-calendar, plus
   `ZonedDateTime` same-time-zone), an exhaustive `message()`, an exhaustive
   `emitting_builtins()` naming the `until`/`since` pair that reads each message
   back, `ALL`, and a `const _: () = { ... }` block asserting `ALL[i].index() ==
   i` for every position.
2. `emit_temporal_require_same_calendar` takes the guard instead of `message:
   &str`. **A guard message is no longer spellable at a call site.**
3. All five emission sites now name a variant — the four
   `emit_temporal_require_same_calendar` callers
   (`temporal_plain_date_methods.rs`, `temporal_plain_date_time_methods.rs`,
   `temporal_plain_year_month_methods.rs`, `temporal_zoned_date_time_methods.rs`)
   and the inline `TimeZoneEquals` throw in the ZDT difference emitter.
4. `data.rs` interns by walking `TemporalDifferenceGuard::ALL -> message()`,
   gated per guard on `emitting_builtins()` — the same construction as the
   `TemporalCalendarId::ALL -> eras() -> spellings()` walk twenty lines above it,
   and for the same stated reason.
5. `builtins/mod.rs` re-exports the enum for `data.rs`, in the shape
   `TemporalCalendarId` already uses.

`grep -rn "until and since require" crates/ --include=*.rs` now returns **5
lines, all in `temporal_plain_date.rs`** — one source for all five strings,
against 10 lines across 6 files before.

**Why this is byte-identical rather than "probably fine", which matters because
rung G cannot be run from this session.** Pool offsets are assignment-ordered
(`intern_string`, `data.rs:3726`: `offset = STATIC_DATA_OFFSET + bytes.len()`),
so *reordering* the pool moves every later string. Therefore:

- The three plain-family messages keep their exact position inside the existing
  `Temporal.PlainDate` array; only the literal was replaced by a `const fn` call
  returning the same `&'static str`. Deleting them in favour of the walk is
  correct and is what I would do with a golden capture; it is a pure reordering
  and it needs rung G, so it is **not** done here. The comment at the site says
  so.
- The new walk is idempotent: `intern_string` returns early on a hit. For every
  program that compiles today it therefore interns nothing new, and the emitted
  data section is unchanged. The only case where it adds a string is a program
  whose `until`/`since` builtin is compiled while the old gate did not fire —
  i.e. a program that *panics* at emit today, which has no bytes to preserve.

Consequence for the banked rung-1c chunks, stated as a measurement rather than
as optimism: **no chunk needs re-running for this change.** See "Run state"
below for the one chunk that does.

**What it does not enforce**, written into the enum's doc rather than left for
the next reader to discover: `ALL` is a hand-written array. The const assertion
rejects a duplicate, a reordering, and a variant dropped from the middle; a
variant *appended* to the enum and left off the end of `ALL` still compiles.

## Lane 1 — iterator-helper-static-key-call-on-a-class-receiver

The §3 root-cause patch was applied in session 1 and the runner measured it:
`iterator_helpers` 14/14 and `iterator` 30/30, all 13 batch-5 reds green,
including the `forEach` control that session 1 flagged as the one that would
catch a bad reroute. Nothing further to apply.

### The unclaimed deliverable — the audit of the other 582 `constructor_instance` arms — is DONE, and it found four

Lane 1 filed this twice as "a real, unclaimed deliverable" and session 1 left it
unclaimed. It costs no CPU, so I did it: parsed all **332 arms** of
`standard_builtin_signature` (`lowering.rs:5154-7666`) and extracted the fourth
tuple member per `StandardBuiltinId`, then intersected with the **53** ids
`StandardBuiltinId::constructable()` names (`builtins.rs:7306`). Every one of the
53 has an arm; **4 of the 53 carry `ValueInfo::undefined()`**:

| builtin | reachable? | verdict |
|---|---|---|
| `IntlDateTimeFormatConstructor` | **yes** | **live defect, same class as the Iterator one** |
| `ProxyConstructor` | no | `Proxy.prototype` is `undefined`, so `class S extends Proxy {}` is a TypeError at class-definition time |
| `SymbolConstructor` | no | `new Symbol()` throws TypeError, so no instance exists to type |
| `BigIntConstructor` | no | `new BigInt()` throws TypeError, same |

The consumer is a single site — `lower_class`'s `inherited_instance`
(`lowering.rs:19304-19312`), which reads `signature.constructor_instance` only
for a class with **no explicit constructor** over a `Constructable` heritage. So
the member types `new S()` for `class S extends X {}` and nothing else, which is
why the three unreachable rows are unreachable rather than merely unlikely.

`class S extends Intl.DateTimeFormat {}` is legal and is a real test262 shape, so
that one instance is typed `undefined` today and takes exactly the path lane 1
measured: `possible_kinds ⊆ NULLISH`, so `emit_method_call`'s statically-nullish
shortcut emits no call at all for a static-key `.format(...)`-style helper on it.

**Deliberately NOT fixed here.** The patch is one token
(`Self::fresh_constructed_instance_info()`, or a shape-carrying equivalent), but
it is a semantic lowering change with no fixture in this tree, no measurement,
and no runner scheduled behind me — and applying it would make the 15 banked
rung-1c chunks compiler-stale for a defect nobody has yet observed. It is
recorded here with its exact site, its reachability argument and its one-line
patch, which is precisely the mechanism that made lane 1's own §3 patch land
cleanly this batch. **Owner: batch 7, as a lane with a run.**

Also considered and rejected: encoding the invariant as a `debug_assert!` at the
tuple destructuring (`constructable() ⇒ constructor_instance` not statically
nullish, with the four names allowlisted). It is the right shape, but
`target/debug/lila` is built *with* debug assertions and this session cannot run
a single test — a false positive would break every `lila` invocation the runner
makes, and I will not add a runtime assertion I cannot execute once. Batch 7
should add it together with the DTF fix, when it can run `cargo test -p
lila-ir` behind it.

§3(3) (the two wrong `standard_builtin_value_info` results for `Iterator()` and
`it.toArray()`) stays filed and unapplied for session 1's reason, which the
completed rung-1c chunks now strengthen: changing `toArray`'s static kind to
`Array` moves every downstream lowering decision about that value and would
invalidate 15 banked chunks.

## Lane 2 — zdt-arithmetic-surface

`check-module-boundaries.sh` is green, including the negative control session 1
verified. The runner measured all four era-boundary cases **passed 1 / Crash 0 /
Bug 0** and `date::run_wasm_backend_succeeds_for_temporal_zoned_date_time_era_fixture`
ok inside a 17/17 `date` chunk. The lane's four named divergences (DST round
trip, default `largestUnit`, over-strict `TimeZoneEquals`, `GetDifferenceSettings`
ordering) plus §5.5's operation-order divergence remain written at their sites
and unfixed — all bounded, none reaching the gate.

The type strengthening above is this lane's blast radius closed at the type
level; the two literals that took 24 tests down were its `until`/`since` pair.

## Lane 3 — rung1c-frontend-isolation

The lane's design goal is met and the numbers are now measurements rather than
inferences, so I replaced the script's own request for them with the answers.

`scripts/rung1c-chunks.sh`, `chunk_stall`'s comment block ended with "FIRST
ACTION FOR WHOEVER RUNS THIS: time that chunk alone ... and set the number from
the measurement rather than from this comment." That has now happened, so the
instruction is stale in the way this repository keeps paying for. Replaced with
the measurement, off `target/watched/rung1c-frontend_test262_subset.log`:
libtest's single "over 60 seconds" line lands at t+60 s, the next byte in that
log is the `... ok` at t+1814 s, so the log is **silent for 1,754 s** and the
900 s ceiling would have killed the run at ~t+960 s — 854 s short. 3600 stays,
as a 2.05x margin, with "do not lower below ~2600" stated. Peak RSS 5.55 GiB
(plateau 4.87-5.03) against b5's 8.4-8.7 GiB unisolated, `avail` never below
9.05 GiB, recorded at the same place because it is the number that decides sweep
co-scheduling — which is still **untested** and is flagged as such.

`sh -n` clean. `grep -c '^run_chunk '` = 18, `grep -c '^mod '` in `main.rs` = 18,
`awk` exact-line `#[test]` count = **617** (unchanged; this session added no
test).

## Cross-lane — the T03 ledger row and `CURRENT_BATCH`, refreshed against the runner's measurement

Session 1 bumped `CURRENT_BATCH` 3 -> 6 and the header `batch-4` -> `batch-7`
while rung 1c was 12-of-17. The runner then banked six more chunks, and both the
row's `reason` and the constant's doc comment became stale **in the good
direction** — which is the kind of stale this ledger exists to prevent. Both
rewritten to what is measured at this head:

- **16 of 18 chunks banked; 465 of 608 executing tests have a verdict.**
  `language` (105) and `binary_data` (38) have never produced a verdict at any
  head. 465 + 143 = 608 exactly.
- **254 of the 465 were measured at THIS head with zero failures** —
  `known_failures` 5, `frontend_test262_subset` 1, `date` 17, `iterator` 30,
  `iterator_helpers` 14, `frontend` 45, `typed_array` 58, `array` 84. The other
  211 carry batch-5 verdicts (`heap` contributes 11 executing of 12 declared,
  which is where the 466-vs-465 arithmetic reconciles).
- The 13 batch-5 reds are green, so **no row is owed for any of them**, and the
  "libtest would report `test did not panic as expected`" argument is no longer
  why this row survives. It survives because two chunks have never run.
- The only non-pass outcome across the 465 is the declared `heap` ignore (T05),
  which already has its row.

`CURRENT_BATCH = 6` against `# unfilled-allowed-until: batch-7` is unchanged, so
`6 < 7` still holds and the row is still a one-batch deadline. I re-validated the
edited row against the parser by hand: 6 tab-separated columns, no empty or
padded column, `reason` 1,099 chars (>= 20), `evidence` unchanged
(`scripts/rung1c-chunks.sh run_chunk` — first token a tracked path outside
`target/`, no `:<digit>` anywhere), sort order untouched.

## Run state — one targeted invalidation, and why the other 15 stay banked

`target/watched/rung1c-done`: removed **`known_failures`** only. Its subject is
the ledger and the chunk/module partition, and this session edited
`known-failures.tsv` and `known_failures.rs`; a banked verdict would be
validating the previous text. Cost to re-run: 0.01 s. Backup of the pre-edit file
is in this session's scratchpad. Wholesale deletion is forbidden (property 3) and
was not done; the counts sidecar was not touched, and its now-orphaned
`known_failures 5` row is ignored by design (a row with no done-file line is
inert).

The other 15 stay banked, and that is an argument rather than a preference: the
only compiler change in this session is the guard enum, which is byte-identical
for every program that compiles today (see the ordering argument above). The
remaining edits are a shell comment, a doc comment, and a ledger `reason`.

## Gate status at the end of this session

| gate | result |
|---|---|
| `cargo check -p lila-aot-wasm` | **0 errors**, 25 warnings (baseline 25) |
| `cargo check -p lila-cli --all-targets` | **0 errors**, 0 new warnings |
| `cargo xc` (workspace, all targets) | **EXIT=0** — ir lib 6 / lib-test 5, aot-wasm lib 25 / lib-test 20, test262 lib-test 1: **identical to the b5 and arrival baselines, no new warning** |
| `cargo fmt --all -- --check` | **clean** |
| `./scripts/check-module-boundaries.sh` | **ok** |
| `sh -n scripts/rung1c-chunks.sh` | clean |
| partition arithmetic | `run_chunk` 18 = `mod` 18; 617 `#[test]` attributes, unchanged |

## What the runner must do next, and what I could NOT verify

Nothing was executed here. Every behavioural claim is unverified by construction.

1. **`known_failures::`** (5 tests, 0.01 s). It is the only check of the edited
   ledger row, of the refreshed `CURRENT_BATCH` doc, and of the 18-chunk
   partition. Run it first; it is free.
2. **`cargo test -p lila-aot-wasm --lib`** (~740 s, 246 tests). This is the
   target the guard enum lives under and the one that went 24 red on the
   uninterned message. It is also the target neither the fixer nor either
   integrator has ever run. If the guard walk is wrong, this is where it says so,
   with the same ``string `...` must exist in pool`` panic.
3. **`scripts/rung1c-chunks.sh`** for the last two chunks, `language` (105) and
   `binary_data` (38), with the sweep down. `binary_data` carries the declared
   T17 `Atomics.wait` hang; b5 proved on paper that `HANG_TIMEOUT` wins the
   900-vs-900 race against `--stall` by ~60 s, and that has still never been
   observed. A complete rung 1c is what lets batch 7 delete the `UNFILLED` row
   instead of extending the header a third time.
4. **Restart the sweep** (`target/test262-scratch/sweep-supervisor.sh`), which
   has been down since 17:16Z. Its two in-flight journal entries are
   runner-killed bystanders, not crashes: a `test262 quarantine:` line arising
   from `built-ins/Array/prototype/some/15.4.4.17-7-b-{11,12}.js` is a false
   positive for lead A.

Unclaimed and still open after this session:

- **`IntlDateTimeFormatConstructor`'s `constructor_instance`** — the one live
  survivor of lane 1's audit, above, with its patch and its reachability
  argument. Owner: batch 7.
- The `debug_assert!` that would make that whole class a run-time-of-first-test
  error, which needs a session that can run `cargo test -p lila-ir`.
- `lowering.rs`'s two wrong `standard_builtin_value_info` results (`Iterator()`
  and `it.toArray()`), filed by lane 1 in batch 6 and still not ridden along.
- Deleting the three in-place plain-family intern entries in favour of the walk —
  correct, byte-moving, needs rung G.
- The eight CLI tests with no wall-clock bound (lane 3's side finding).
- The ZDT lane's five named divergences, and the other 24 era-boundary files.

## Addendum — the checkpoint commit

This session ran **no git command**. The orchestration harness checkpointed the
working tree at 21:16:50Z as `a0f411eaf` ("WIP checkpoint: batch 6 runner final
rungs") while the notes were being written, so the ten files below are already in
HEAD and `git status` is clean. Verified the commit contains exactly this
session's edits and nothing else — there is no concurrent writer, and `ps` shows
0 `lila`, 0 `cargo` on the box:

```
crates/lila-aot-wasm/src/builtins/mod.rs                         |  8 +
crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs         | 141 +-
crates/lila-aot-wasm/src/builtins/temporal_plain_date_methods.rs |  2 +-
crates/lila-aot-wasm/src/builtins/temporal_plain_date_time_methods.rs   |  2 +-
crates/lila-aot-wasm/src/builtins/temporal_plain_year_month_methods.rs  |  2 +-
crates/lila-aot-wasm/src/builtins/temporal_zoned_date_time_methods.rs   |  4 +-
crates/lila-aot-wasm/src/data.rs                                 | 65 +-
crates/lila-cli/tests/cli/known_failures.rs                      | 42 +-
crates/lila-cli/tests/known-failures.tsv                         |  2 +-
scripts/rung1c-chunks.sh                                            | 17 +-
```

All gates in the table above were measured at this content, i.e. at `a0f411eaf`.

---

# FINDINGS FIXER — batch 6, pass over the 19 findings

Entered at `a0f411eaf`, box idle (0 `lila`, 0 `cargo`, sweep down). `cargo check`/`cargo xc`
only; no git, no test, no `lila`, no test262 command.

## Outcome per finding

| # | lane | subject | outcome |
|---|---|---|---|
| 1 | iterator | `receiver_needs_dynamic_helper_dispatch` too wide | CONFIRMED, **kept deliberately**, documented as a second improvement (TypeError vs `Unreachable`) with the minimal `⊆ NULLISH` repair recorded |
| 2 | iterator | differential unfalsifiable | CONFIRMED, negative control added |
| 3 | iterator | 3rd propagate check dead | CONFIRMED, doc corrected (emission kept, byte-neutral) |
| 4 | iterator | stale `forEach` comment | CONFIRMED, rewritten |
| 5 | iterator | name-filter trap | CONFIRMED, all 4 tests renamed `iterator_helper_*` |
| 6 | iterator | note §5 probe premise | CONFIRMED unverified; recommendation retracted, b5 "inversion" shown to be a brief paraphrase |
| 7 | zdt | `data.rs` prototype keys ungated | **REJECTED on evidence** — the block at `data.rs:2226` is unconditional; coupling annotated |
| 8 | zdt | `non_test_lines` loosening | CONFIRMED, regex → state machine, ir budget 140/140 → 160 with the measurement |
| 9 | zdt | "appending keeps indices stable" | CONFIRMED (770 entries, index 480, 285 shift), rewritten |
| 10 | zdt | const covers 5 of 9 | CONFIRMED, all 9 folded in, install order preserved |
| 11 | zdt | brand-check loop passes `undefined` | CONFIRMED, comment de-claimed (a getter probe would be per-method; recorded as missing coverage) |
| 12 | zdt | note stale in 4 places | CONFIRMED, all 4 corrected |
| 13 | rung1c | `known_failures` skippable | CONFIRMED, now re-runs unconditionally |
| 14 | rung1c | counts sidecar over-claims | CONFIRMED, claim weakened (mechanism kept — `cksum` invalidates all 16 banked rows) |
| 15 | rung1c | no one-test guard | CONFIRMED, guard added (fails in ~1 ms with the reason) |
| 16 | rung1c | `chunk_stall` stale instruction | **ALREADY FIXED** at this head by integrator session 2; no change |
| 17 | rung1c | note §3 cache limits | CONFIRMED, superseded + correction 4 |
| 18 | rung1c | note §4/§189 sidecar semantics | CONFIRMED, superseded + correction 5, branch listed unverified |
| 19 | rung1c | drifted line anchors | CONFIRMED (2 of 9), re-derived |

## Run state: NOTHING invalidated, and the argument

No banked rung-1c chunk was invalidated, because no edit moves an emitted byte.
`lowering.rs` and `intrinsics/temporal.rs` replaced four literal blocks with a loop over a
const holding the same four entries: the shape's `properties` is a `BTreeMap` (key-sorted,
so insertion order is not observable there) and the installer's append order is preserved
exactly (`equals`, `toInstant`, `withTimeZone`, `toPlainDateTime`, then the five). `names.rs`
and `builtins.rs` changed a const's contents and a comment, no enum variant. `data.rs`,
`functions.rs` and the ZDT fixture are comments only. The CLI `#[test]` count is still
**617** (8 `spec-exec-oracle` gates → 609 compiled, 608 executing), so no ledger or
`batch-workflow.md` number moved. The new `iterator_helper_dispatch` test is in
`lila-aot-wasm`, not the CLI target.

`known_failures` no longer needs a hand-deleted done-file line — the script re-runs it.

## Gate at exit

`cargo xc` **EXIT=0**, 0 errors. Warnings identical to the arrival/b5 baseline:
`lila-ir` lib 6 / lib-test 5, `lila-aot-wasm` lib 25 / lib-test 20,
`lila-test262` lib-test 1. `cargo fmt --all -- --check` clean.
`./scripts/check-module-boundaries.sh` **ok** (ir 140 code lines / budget 160,
wasm 101 / 180). `sh -n scripts/rung1c-chunks.sh` and `bash -n
scripts/check-module-boundaries.sh` clean; 18 `run_chunk` = 18 `mod`.

## Owed to the runner, in order

1. `cargo test -p lila-aot-wasm --test iterator_helper_dispatch` — **UNFILTERED**.
   4 tests, 13 `emit()` calls, minutes not seconds. The new
   `iterator_helper_dispatch_differential_separates_two_emitters` is the one that has never
   run: if it reddens, the two `_dispatches_..._like_...` tests converged and are vacuous.
2. `cargo test -p lila-aot-wasm --lib` — still never run by any fixer/integrator, and it
   is the target that went 24 red on the pool-panic this batch.
3. `cargo test -p lila-cli --test cli -- --test-threads=3 date::` — the ZDT fixture
   comment changed; the fixture itself did not, so this is confirmation, not a re-measure.
4. `scripts/rung1c-chunks.sh` for `language` + `binary_data`, sweep down.
   `known_failures` will now re-run first, by design, at ~0.01 s.

## Left undone, deliberately

* `({}).map(1)` throwing `TypeError` rather than trapping — the improvement that justifies
  the wide dispatch predicate — is still unfixtured. Recorded on the predicate's doc.
* The seven fall-back blocks remain unwitnessed at this head (the `lila-ir`
  `constructor_instance` fix makes the FIRST guard fire for every class receiver). The new
  negative control is an instrument check, not coverage of those blocks.
* The counts sidecar still cannot see a same-count module edit. The `cksum` upgrade is
  written into the script's header with the reason it must wait for a batch with no banked
  verdicts to lose.
* The generic tail's own dead post-call propagate check (the twin of the one documented in
  the dispatch) was left alone: pre-existing, present in every banked verdict.
