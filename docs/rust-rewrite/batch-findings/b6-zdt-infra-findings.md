# b6 ZDT+INFRA analyst findings

Started 2026-08-10, branch `claude/test-driven-rust-opus-pp6giw`. Read-only lane: no cargo, no `porf`
runs, nothing CPU-bound. The sweep was left running throughout. Every number below is `grep`/`ls`/`sed`
over the checkout, so it is countable rather than measured-by-execution, and I say which is which.

---

# (C) The ZDT era-boundary quartet — DIAGNOSED. The methods do not exist.

## The one-line answer

`Temporal.ZonedDateTime.prototype` has no `add`, no `subtract`, no `since`, no `until`. The property
lookup yields `undefined` and the call site throws the `TypeError` whose handle b5 recorded. This is a
missing-registration defect, not a mis-emission and not an unrooted internal.

## How that is established without running anything

`crates/porffor-aot-wasm/src/intrinsics/temporal.rs:938-1130`
(`install_temporal_zoned_date_time_constructor_intrinsics`) is the *only* place ZDT prototype members
are installed. Read end to end it installs exactly 23 things:

- 18 accessor getters, in one `for (name, builtin)` loop (`:948-1030`): `epochMilliseconds`,
  `epochNanoseconds`, `offset`, `offsetNanoseconds`, `timeZoneId`, `calendarId`, `era`, `eraYear`,
  `year`, `month`, `monthCode`, `day`, `hour`, `minute`, `second`, `millisecond`, `microsecond`,
  `nanosecond`;
- 4 data methods, each its own `emit_object_define_function_data` call: `equals` (`:1055`),
  `toInstant` (`:1069`), `withTimeZone` (`:1084`), `toPlainDateTime` (`:1101`);
- `Symbol.toStringTag` = `"Temporal.ZonedDateTime"` (`:1113`).

The membership is mirrored — deliberately, per the comment at `:977-987` — by
`porffor-ir/src/lowering.rs:1666` `temporal_zoned_date_time_prototype_shape()`, which inserts the same
18 accessors and the same 4 `ObjectShapeProperty::Data` methods (`:1764-1791`) and nothing else. The
two lists agree, so there is no shape/prototype disagreement to chase: **both sides agree the methods
are absent.**

The enum settles it independently of either list. Every `TemporalZonedDateTime*` identifier in the whole
workspace (`grep -rho 'TemporalZonedDateTime[A-Za-z]*' crates/ --include=*.rs | sort -u`) is 25 names:
the 22 prototype members above, plus `TemporalZonedDateTime`, `TemporalZonedDateTimeConstructor`,
`TemporalZonedDateTimeFrom`. **There is no `TemporalZonedDateTimePrototypeAdd` variant to register, no
emitter arm to route, and no `AddZonedDateTime` internal to root.** By contrast the sibling types all
have theirs — `TemporalPlainDateTimePrototype{Add,Subtract,Since,Until,WithCalendar,Round,With}`,
`TemporalPlainDatePrototype{Add,Subtract,Since,Until,WithCalendar,With}`,
`TemporalPlainTimePrototype{Add,Subtract,Since,Until,Round,With}`,
`TemporalPlainYearMonthPrototype{Add,Subtract,Since,Until,With}`,
`TemporalDurationPrototype{Add,Subtract,Round,With}`. ZonedDateTime is the one Temporal type whose
arithmetic surface was never written; there is no `builtins/temporal_zoned_date_time_methods.rs`
beside the five `temporal_*_methods.rs` files that do exist.

So the brief's second hypothesis ("an internal `AddZonedDateTime`/`DifferenceZonedDateTime` dependency
that was never rooted") is **disproved**: `grep -rn 'AddZonedDateTime\|DifferenceZonedDateTime\|
add_zoned_date_time\|difference_zoned_date_time\|AddDurationToZonedDateTime\|GetPlainDateTimeFor\|
GetEpochNanosecondsFor' crates/` returns **zero hits**. Nothing is unrooted because nothing exists.
(It becomes a real hazard *after* the fix — see "the rooting edge" below.)

## The control that makes this airtight: the passing `from/` sibling

The brief asks what the failing files call that the passing getter files do not. There is a near-perfect
control in the same subtree, and the delta is one token.

`intl402/Temporal/ZonedDateTime/from/era-boundary-gregory.js` (36 lines) does:

```js
const ce0 = Temporal.ZonedDateTime.from({ era: "ce", eraYear: 0, monthCode: "M01", day: 1,
                                          hour: 12, minute: 34, timeZone: "UTC", calendar }, options);
TemporalHelpers.assertPlainDateTime(ce0.toPlainDateTime(), 0, 1, "M01", 1, 12, 34, 0,0,0,0, "...", "bce", 1);
```

`.../prototype/add/era-boundary-gregory.js` (68 lines) does the *same* `from()` call with the *same*
`{era, eraYear, monthCode, day, hour, minute, timeZone, calendar}` bag and the *same*
`{overflow:"reject"}` options, and the *same* `toPlainDateTime()` + `assertPlainDateTime` tail. The only
construct it adds is `.add(duration)` between them (`add:18`). `assertPlainDateTime`
(`test262/vendor/test262/harness/temporalHelpers.js:235`) reads only
`datetime.{calendarId,era,eraYear,year,month,monthCode,day,hour,minute,second,millisecond,microsecond,
nanosecond}` off a **PlainDateTime**, never off the ZDT — so the helper cannot be the missing callable.

For `since`/`until` the delta is two constructs, both missing: `.since`/`.until` (`since:58`, reached
first) and `.withCalendar("iso8601")` (`since:65-66`). `assertDuration` / `assertDurationsEqual`
(`temporalHelpers.js:143`, `:185`) read only `Temporal.Duration` getters, which exist.

## Correction to the brief: the handle pairing is NOT evidence of two callables

The brief infers "add/subtract share handle `1827888`, since/until share `1879624`, so there are exactly
two missing callables." That inference does not hold, and a fix lane that believes it will look for one
shared symbol per pair and not find it.

`diff add/era-boundary-gregory.js subtract/era-boundary-gregory.js` is **9 hunks, all cosmetic**: the
`esid` line, the sign of the two `Temporal.Duration` literals, and `add` -> `subtract` at 8 call sites.
The allocation sequence before the throw is byte-for-byte identical, so the `TypeError` object lands at
the same heap address. `diff since until` is the same story (sign-flipped expectation tuples plus
`since` -> `until`). **`handle@N` is the address of the thrown `TypeError`, not of the callee or the
receiver** — `object(handle@...)` is how the runner renders a thrown object. The pairing is a
consequence of the two files being isomorphic, and it would be identical if all four names were
distinct missing methods. Which they are: **four missing callables, not two** (`add`, `subtract`,
`since`, `until`), plus a fifth, `withCalendar`, that `since`/`until` reach at line 65 and that is also
absent.

Falsifiable prediction, cheap for the next runner with CPU to spend: the other 24 era-boundary cases
(`{add,subtract,since,until}/era-boundary-{ethiopic,islamic-civil,islamic-tbla,islamic-umalqura,
japanese,roc}.js`) must carry the *same* `value is not callable` label at *different* handles, because
those files differ from the gregory ones in their field bags. If any of them fails differently, my
diagnosis is incomplete.

## Full scope: what ZonedDateTime.prototype is missing

Present: the 18 getters + `equals`, `toInstant`, `withTimeZone`, `toPlainDateTime`.

Missing getters (10): `dayOfWeek`, `dayOfYear`, `weekOfYear`, `yearOfWeek`, `daysInWeek`, `daysInMonth`,
`daysInYear`, `monthsInYear`, `inLeapYear`, `hoursInDay`.

Missing methods (16): **`add`**, **`subtract`**, **`since`**, **`until`**, **`withCalendar`**, `with`,
`withPlainTime`, `round`, `startOfDay`, `getTimeZoneTransition`, `toPlainDate`, `toPlainTime`,
`toString`, `toJSON`, `toLocaleString`, `valueOf`.

## Blast radius, counted with `ls` (upper bound on cases a fix may move)

| method | `built-ins/Temporal/ZonedDateTime/prototype/` | `intl402/...` | total |
|---|---|---|---|
| `add` | 43 | 76 | 119 |
| `subtract` | 42 | 76 | 118 |
| `since` | 99 | 67 | 166 |
| `until` | 97 | 66 | 163 |
| **quartet** | **281** | **285** | **566** |
| `withCalendar` (needed by since/until) | 16 | 7 | 23 |

566 cases sit under a ZDT method that does not exist — **20x** the 28 era-boundary cases b5 sized, and
84x the 4 the brief names. This is an upper bound (some cases test branding/argument-coercion and may
fail for further reasons after the methods land), not a predicted delta.

## Where a fix lane writes, named

Adding one ZDT builtin is a **19-site** edit. Counted, not estimated: the site list for the existing
`TemporalZonedDateTimePrototypeToPlainDateTime` is

```
porffor-ir/src/lowering.rs:1788, 6950, 27541
porffor-ir/src/builtins.rs:1131, 1706, 2902, 4422, 5988, 6932, 7976, 8723
porffor-aot-wasm/src/intrinsics/temporal.rs:1103     (the install call)
porffor-aot-wasm/src/module.rs:1559
porffor-aot-wasm/src/builtins/standard.rs:37109      (the compile arm)
porffor-aot-wasm/src/builtins/bootstrap.rs:1046
porffor-aot-wasm/src/planning.rs:234, 2168, 2216, 6122   (:234 is the cycle doc, :6122 is arity)
```

and `TemporalPlainDateTimePrototypeSince` has the same shape at 17 sites. Both lists are one
`grep -rn <Variant> crates/ --include=*.rs` away and should be regenerated rather than copied.

The bodies to model on already exist and are `pub(crate)`:

- `builtins/temporal_plain_date_time_methods.rs:1618`
  `emit_temporal_plain_date_time_add_or_subtract(subtract: bool, function)`
- `builtins/temporal_plain_date_time_methods.rs:2246`
  `emit_temporal_plain_date_time_until_or_since(since: bool, function)`
- `builtins/temporal_plain_date_time_methods.rs:1526` `emit_temporal_plain_date_time_with_calendar`
- `builtins/temporal_plain_date_time_methods.rs:2967` `emit_temporal_plain_date_time_to_zoned_date_time`
  — the PDT -> ZDT direction, i.e. the second half of the spec's round trip
- `builtins/temporal.rs:3050` `emit_temporal_zoned_date_time_to_plain_date_time` — the ZDT -> PDT
  direction, i.e. the spec's `GetPlainDateTimeFor`, already written
- `builtins/temporal.rs:2345` `emit_alloc_temporal_zoned_date_time` — allocating the result
- `builtins/temporal.rs:3384` `emit_temporal_zoned_date_time_with_time_zone` — the template for a
  ZDT-in/ZDT-out prototype method (argument handling, branding, result allocation)

So `AddZonedDateTime` decomposes into pieces that all exist: ZDT -> PlainDateTime -> calendar date
arithmetic -> PlainDateTime -> ZDT. The lane's real work is the wiring plus the disambiguation
semantics, not new date maths. Note the two directions are *not* symmetric in the spec (add does
date-then-time in the calendar, then re-derives epoch nanoseconds with `compatible` disambiguation), so
"call `toPlainDateTime`, call PDT `add`, call `toZonedDateTime`" is the right *skeleton* and the wrong
*answer* for DST-crossing cases. It is correct for these 28 era-boundary cases, all of which are
`timeZone: "UTC"`.

## The rooting edge — where the brief's hypothesis becomes true prospectively

`planning.rs:1988-2050` is a single or-pattern arm covering **all 22** ZonedDateTime members (its own
comment at `:2005` says "all 24", which is stale by two). That arm does
`require_standard_builtin(TemporalPlainDateTimeConstructor)` and then inserts all 22 into
`standard_roots`. It does **not** require `TemporalDurationConstructor`.

The PlainDateTime arm at `:1980` does, with the reason spelled out at `:1998`: "`until`/`since` hand
back a `Temporal.Duration` and `add`/`subtract` read one". **A ZDT lane that adds `add`/`since` to the
existing or-pattern without adding `require_standard_builtin(TemporalDurationConstructor)` to that arm
gets exactly the failure mode the brief guessed** — an emitter reading a prototype global nothing
bootstrapped. That is the one line most likely to be forgotten, so it is worth stating as the lane's
first invariant.

Two further consequences, both already documented in the tree and both live for this lane:

1. The ZDT arm and the PDT arm already `require_standard_builtin` **each other**
   (`:2022` requires `TemporalPlainDateTimeConstructor`; `:2001` requires
   `TemporalZonedDateTimeConstructor`). That is precisely the cycle whose unbounded recursion killed 40
   sweep processes in batch 4, now contained by `RuntimeBootstrapPlan.walked`
   (`planning.rs:1161`, `:1334`) with the regression test
   `planning::tests::a_cyclic_rooting_dependency_terminates_and_roots_both_ends`. Adding Duration to
   the ZDT arm widens that cycle (Duration's `round`/`total` take a `relativeTo` that may be a ZDT).
   **The lane must re-run that test, and should extend it to enter the cycle from a ZDT arithmetic
   member.** This is the single highest-risk part of the change, because its failure mode is a
   stack-overflow SIGABRT in a whole-suite sweep rather than a red unit test.
2. The comment at `:2005-2021` already flags that this arm roots the whole PlainDateTime family for any
   program touching `zdt.hour`, and calls the fix "split this arm". Adding four arithmetic members that
   genuinely need Duration is the moment that split stops being optional — otherwise `zdt.hour` starts
   rooting `Temporal.Duration` too, and the emit-size cost is real and unmeasured against batch 3's
   budget.

## Recommended lane shape

Smallest honest slice that turns the quartet green: `add`, `subtract`, `since`, `until`, `withCalendar`
— five members, ~95 registration sites, one new
`crates/porffor-aot-wasm/src/builtins/temporal_zoned_date_time_methods.rs` following the five sibling
`_methods.rs` files, one `planning.rs` arm edit (with the Duration root), and a `check-module-boundaries.sh`
entry for the new module. Verify at rung 4 with
`porf test262 run intl402/Temporal/ZonedDateTime/prototype/add` etc. — but budget for it: b5 measured
**~300 s per cold case** in this subtree, so 566 cases is not a rung-4 the lane can run whole. Run the
28 era-boundary cases as the gate and the rest as a hand-off.

Do **not** scope this lane as "the quartet". The quartet is 4 files of a 566-case hole, and the four
files are the cheapest possible witness of it.

---

# (D) Rung-1c completion infrastructure

Read at HEAD `4bb813639`, tree otherwise clean. Recounted here rather than quoted:
`awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++}' crates/porffor-cli/tests/cli/*.rs` = **615**,
`grep -c 'cfg(feature = "spec-exec-oracle")' .../frontend.rs` = **8**, so **607 executing**,
`frontend.rs` 54 compiled / 46 executing. b5's arithmetic still holds at this head.

## D1 — isolating the 8.7 GiB frontend test

### The test, and why it is not on either bounded path

`crates/porffor-cli/tests/cli/frontend.rs:1345-1370`
`frontend::test262_wasm_backend_runs_supported_fixture_subset`. It is ungated (no
`spec-exec-oracle` cfg) and it does **not** go through the crate's `Command` wrapper — it calls
`ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))` directly (`ProcessCommand` is
`std::process::Command`, aliased in `main.rs:31`). Consequence worth stating on its own: this test is on
**neither** of `main.rs`'s two bounded paths. `HANG_TIMEOUT` (900 s, `main.rs:67`) is applied by
`Command::output` (`main.rs:153`), and this call site bypasses it. 9 call sites in the target do the
same — `frontend.rs` 6, `language.rs` 2, and `main.rs:216` (which *is* the guarded path). So 8 tests in
the suite are unbounded by wall clock, and the most memory-expensive test in rung 1c is one of them.

### `-- --exact` CANNOT be the spelling. The hygiene test forbids it.

b5 recorded the fix as "a separate chunk for that one test via `-- --exact <name>`, which needs a
`run_chunk` line and would have to keep `rung_1c_chunks_cover_every_cli_area_module` green". Read the
test (`known_failures.rs:1464-1587`) and the parser it uses (`rung_1c_chunks()`, `:1044-1120`): that
combination is impossible without editing the test. Three assertions block it, and each is load-bearing:

1. `:1074-1080` — the chunk's second argument must equal `format!("{name}::")` **exactly**. A line
   `run_chunk frontend_exact --exact frontend::test262_...` fails on the second token before anything
   else is considered.
2. `:1083-1090` — the only arguments permitted after the filter are `--skip <module>::` pairs, and the
   skip target must end in `::` and must be an area module (`:1560-1568`). `--exact` is rejected by
   name; `--skip frontend::test262_wasm_backend_runs_supported_fixture_subset` is rejected for not
   ending in `::` *and* for not being a module.
3. `:1526-1541` — the chunk-key set and the module-stem set must be equal **in both directions**. A
   chunk named `frontend_exact` with no `tests/cli/frontend_exact.rs` fails
   "there is no tests/cli/{chunk}.rs".

There is also a libtest fact that kills the decomposition independent of the hygiene test: libtest's
`--exact` applies to `--skip` as well as to the filter (both go through the same `matches_filter`), so
`--exact frontend:: --skip <one test>` cannot express "the frontend module minus one test".

### The design that works: give the test its own area module

The hygiene test already understands exactly one form of isolation — a module. Use it.

1. **New file** `crates/porffor-cli/tests/cli/frontend_test262_subset.rs`, opening `use crate::*;`
   (that is what `frontend.rs:3` does, and it is why `ProcessCommand` resolves: a private `use` in the
   crate root is visible to descendant modules, so the glob picks it up). Move the whole of
   `frontend.rs:1345-1370` into it, unchanged.
2. **`main.rs`**: add `mod frontend_test262_subset;` — required by `:1518-1523`, and the check exists
   precisely because `iterator_helpers.rs` once shipped a file and a chunk with no `mod` line and banked
   a chunk that measured nothing.
3. **`scripts/rung1c-chunks.sh`**: add `run_chunk frontend_test262_subset frontend_test262_subset::`.
   Place it **immediately before** `run_chunk frontend`, so a short container window banks the
   chokepoint first — the whole point is that `frontend` has failed to bank in three consecutive
   windows and is scheduled 13th of 17.

**No `--skip` is needed and no hygiene-test edit is needed.** Verified against the overlap rule at
`:1554-1559` (`format!("{other}::").ends_with(&format!("{chunk}::"))`): `"frontend_test262_subset::"`
does not end with `"frontend::"` (the character after `frontend` is `_`, not `:`), and
`"frontend::"` does not end with `"frontend_test262_subset::"`. The existing `array`/`typed_array` pair
remains the only overlap. **The naming constraint the next person must respect**: a new module stem may
not have another module's stem as a `::`-suffix. `frontend_heavy`, `frontend_test262_subset`,
`wasm_suite` are all fine; `subset_frontend` would not be.

Arithmetic after the move, so the partition proof survives: 615 compiled and 607 executing are
**unchanged** (a test moved, none added); `frontend` becomes 53 compiled / 45 executing and the new
module is 1/1; every chunk's `ran + filtered_out` still sums to 607.
`MINIMUM_RUNG_1C_CHUNKS` is 10 and the count goes 17 -> 18, so that floor is untouched.

Ledger impact: **none**. No `known-failures.tsv` row names any `frontend::` test, so no `const _`
assertion and no `should_panic` moves with it, and `execution_path` (`known_failures.rs:680`) routes
only declared names — the renamed test was and remains on the default path.

### What isolation buys, and what it does not

Isolation makes the chunk *bankable*: 1 test, so `--test-threads=3` schedules nothing beside it and the
peak is 8.7 GiB + libtest, which fits 15 GiB on an idle box. It does **not** make rung 1c
sweep-compatible — 8.7 GiB plus a `--threads 2 --jobs 2` sweep still does not fit. Combine with D2 if
that is wanted.

## D2 — the 8.7 GiB, and why `PORFFOR_*_CACHE_LIMIT_BYTES` is the wrong lever

**The brief's premise is wrong, and acting on it would waste a window.** b5 recorded the 8.7 GiB as "a
compile-cache sizing question (`PORFFOR_*_CACHE_LIMIT_BYTES` are set for the sweep but not for the CLI
test child)". The three tiers are **on-disk** caches: `FunctionCache` implements wasmtime's `CacheStore`
with `fs::read` / `fs::write` against a cache directory (`porffor-engine/src/cache.rs:220-240`,
`:163`), and the module and program tiers are directories too (`MODULE_CACHE_DIR`,
`PROGRAM_CACHE_DIR`, `:90-92`). Those limits bound **bytes on disk**, not process RSS. Setting them on
the CLI test child would not have prevented one of the three OOM kills.

### What actually holds 8.7 GiB, traced

`test262_wasm_backend_runs_supported_fixture_subset` spawns `porf test262 run language/wasm/pass
--execution-backend wasm` with **no `--threads` and no `--jobs`**. Both defaults are high:

- `SuiteConfig::default().worker_count = available_parallelism().min(4)`
  (`porffor-test262/src/lib.rs:233`) = **4** on this box.
- Those workers are **scoped threads in one process**, not child processes:
  `execute_cases` does `thread::scope` + `thread::Builder::spawn_scoped` per slot
  (`lib.rs:20727-20752`) with `TEST262_WORKER_STACK_SIZE = 64 MiB` (`lib.rs:60`).
- The child-runner (one process per case) is **off by default**: `case_runner_bin` is `None`
  (`lib.rs:236`) and the CLI sets it only under `PORFFOR_TEST262_FORCE_CASE_RUNNER=1`
  (`porffor-cli/src/main.rs:2886-2890`).
- `--jobs` defaults to half the logical CPUs (`main.rs:175-177`) = **2** Cranelift threads.

So: **187 cases, 4 concurrent cold Wasm-AOT compiles, one process, 2 Cranelift threads.** That is a
complete explanation of b5's `ps` sample (one process, 8.4-8.7 GiB, ~230 % CPU — 230 % is exactly what 4
workers sharing 2 compiler threads on 4 CPUs looks like), and it explains why the RSS was *flat*: it is
4 workers' steady-state working set, not a leak. Stacks are 4 x 64 MiB = 256 MiB and are not the story.

### The levers, ranked by directness

1. **`--threads 2` (or `1`) on that one `ProcessCommand`** — `frontend.rs:1347`, one `.arg()` pair.
   Halves (or quarters) the concurrent compile count and therefore the peak. This is the lever that
   matches the measured cause. Cost: the test serialises proportionally; budget for the chunk growing
   from b5's ~330 s. `--threads 2` is the recommended setting — ~4.4 GiB fits beside a running sweep,
   where `--threads 1` buys little more RSS and doubles the wall clock again.
2. **`--jobs 1`** — bounds Cranelift's own 2 threads. Secondary; smaller effect than (1).
3. **`PORFFOR_TEST262_FORCE_CASE_RUNNER=1`** in the child's env — one process per case, so RSS is
   bounded by construction. Rejected: it pays a process re-exec + prelude reload + fresh Wasmtime
   `Engine` bootstrap **187 times**, which the comment at `main.rs:2875-2881` says is exactly what the
   in-process default exists to avoid.
4. **`PORFFOR_{FUNCTION,MODULE,PROGRAM}_CACHE_LIMIT_BYTES`** — bounds disk, not RSS. Worth setting
   anyway for a different, real reason: the CLI suite runs hundreds of distinct sources and the
   program/module tiers are source-keyed pure churn (`cache.rs:27-37`), so the same asymmetric shape the
   sweep uses keeps the CLI suite from thrashing the shared cache directory. It is a hygiene
   improvement, **not** the OOM fix, and it must not be reported as one.

### If the cache env is set anyway, set it in the script — never in a test

Every limit is memoised in a `OnceLock` resolved from `std::env::var` on first use
(`cache.rs:44-66`). Both the in-process CLI path and any spawned `porf` child read it, and a child
inherits the parent's environment, so **one `env` on the `cargo test` invocation covers both paths**.
A `std::env::set_var` inside a test is useless (the `OnceLock` may already be initialised by an earlier
test in the same process) and unsound under libtest's threads. The correct place is
`scripts/rung1c-chunks.sh`, on the same line that already carries `PORFFOR_CPU_PERCENT=100`
(`:149`), which is inherited by `cargo test` -> the test binary -> every `porf` child:

```sh
PORFFOR_CPU_PERCENT=100 \
PORFFOR_FUNCTION_CACHE_LIMIT_BYTES=8589934592 \
PORFFOR_MODULE_CACHE_LIMIT_BYTES=536870912 \
PORFFOR_PROGRAM_CACHE_LIMIT_BYTES=536870912 \
./scripts/run-watched.sh --label "rung1c-$name" --stall 900 -- \
  cargo test -p porffor-cli --test cli -- --test-threads=3 "$@"
```

(Defaults if unset: total 1 GiB, function tier = total, module and program tiers = total/2 —
`cache.rs:13`, `:50-66`.) Note this changes no chunk's verdict and is invisible to the hygiene tests, so
it can land independently of D1.

## D3 — the `CURRENT_BATCH` bump and the T03 fill, with the coupling stated both ways

### The tripwire, re-read at this head

- `known_failures.rs:137` `const CURRENT_BATCH: u32 = 3;`
- `crates/porffor-cli/tests/known-failures.tsv:41` `# unfilled-allowed-until: batch-4`
- `known_failures.rs:1233-1250`: the assertion runs **only if at least one `unfilled` row exists**
  (`if unfilled_rows > 0`), and is `CURRENT_BATCH < ledger.unfilled_allowed_until`.

So `3 < 4` passes today and `4 < 4` reddens `ledger_is_well_formed` the instant anyone bumps while
row 67 lives. That is the designed deadline and it is currently **vacuous** — the gate that would catch
a stale ledger has never been allowed to bite.

Three, and only three, end states are legal:

| end state | edits | when it is honest |
|---|---|---|
| **A. bump + delete the row + no new cli rows** | `CURRENT_BATCH = 4`; delete tsv:67 | a **complete** rung 1c came back with **zero** failures outside the existing rows |
| **B. bump + delete the row + N real rows** | `CURRENT_BATCH = 4`; delete tsv:67; N x (tsv row + `should_panic` + `pub(crate)` + `const _`) | a complete rung 1c came back with N reds that are **not** being repaired this batch |
| **C. bump + extend the header** | `CURRENT_BATCH = 4`; tsv:41 -> `batch-5` (or later); row 67's reason rewritten to today's evidence | rung 1c is still incomplete, or every red belongs to a lane repairing it now |

Deleting the row makes the whole `unfilled` assertion inert, so A and B carry no deadline risk. C is the
only one that keeps a live deadline, and `known_failures.rs:1244-1249` says in its own panic message
that C is "possible, but it is a visible edit to the header, which is the point". **C is not a cheat.**
It is the documented escape, and it is dishonest only if the header is extended without the reason
column being rewritten to the evidence that justifies it.

### The fill is a four-place edit per test, not a tsv append

Read out of the assertions, so nobody discovers it at the bump. For each row with state `fail` or
`hang`:

1. **tsv row**, 6 tab-separated columns (`LEDGER_COLUMNS = 6`, `:148`), sorted ascending and unique on
   `(target, test)`, every column non-empty and untrimmed-clean. `target` is `cli`; the `test` column
   is the libtest name **without** a `cli::` prefix (tsv header comment, `:49-53`). `owner_task` must
   be `T<NN>` backed by a real `tasks/<NN>-*.md` — `tasks/15-generators-iterators-resource-management.md`
   exists, so `T15` is valid for the iterator reds.
2. **evidence column**: its first whitespace token must resolve under the repo root and must **not**
   start with `target/` (`:1209-1231`). Citing `target/lane-notes/rung1c-chunks.md` or any
   `target/watched/*.log` is a hard red. Cite the fixture
   (`crates/porffor-cli/tests/fixtures/wasm_iterator_helper_class_receiver_some.js`) or the test file.
3. **`#[should_panic(expected = "...")]`** on the named test, non-empty (`:1307-1319`; bare and empty
   are both rejected by name). One physical line, exactly that spelling — `scan_source` asserts any line
   starting `#[` also ends `]`, and the `\`-continued form is already idiomatic elsewhere in this tree,
   so it is a live trap.
4. **`pub(crate)`** on the test fn, plus `const _: fn() = crate::<module>::<test>;` in
   `known_failures.rs`. Both directions are enforced (`:1254-1276`): a row with no assertion is red, an
   assertion with no row is red. The existing example is `binary_data.rs:548-550`
   (`#[should_panic(expected = "porf run exceeded")]` + `pub(crate) fn ...`).

### Row content designed from the banked verdicts

b5 banked **12 of 17 chunks / 276 of 607 executing tests**, with **13 reds, all in the two iterator
modules**, and every other banked chunk `0 failed`. The only non-pass non-fail outcome in 276 tests is
`heap` 1 ignored, which already has a row. So *from the banked half*, the failing set is exactly:

- `iterator::run_wasm_backend_succeeds_for_iterator_prototype_{some,every,find,reduce}_fixture` (4),
  `iterator.rs:{387,411,435,459}`
- `iterator_helpers::run_wasm_backend_calls_iterator_prototype_{some,every,find,reduce,map,filter,take}_on_a_class_receiver`,
  `..._chains_take_and_to_array_on_a_class_receiver`,
  `..._gives_identical_results_for_static_and_computed_helper_keys` (9)

If option B is taken, these are the `expected` substrings to use — chosen to be stable, i.e. free of
handle addresses and free of values read from unrelated memory:

| tests | measured message | recommended `expected` | why |
|---|---|---|---|
| `iterator::*_{some,every,find}_fixture` | `stderr=uncaught throw: wasm-aot completion: string(callback throw)` | `string(callback throw)` | the fixture's own guard label; stable text |
| `iterator::*_reduce_fixture` | `string(reducer throw)` | `string(reducer throw)` | same |
| helpers `filter`, `map`, `take`, `chains_take_and_to_array` | `TypeError: ... object(handle@1483824: value is not callable)` | `value is not callable` | **must exclude the handle** — `1483824`/`1483832`/`1485040`/`1479696` are addresses and move |
| helpers `some`, `every`, `find` | `string(type-object;value;calls-0;caught-no-throw;...)` | `calls-0` | `type-object` is **not** stable (session 1 measured the same call as `number`); `calls-0` names the defect (callback never invoked) rather than the corrupted value |
| helpers `reduce`, `gives_identical_results_for_static_and_computed_helper_keys` | `wasmtime execution trapped: error while executing at wasm backtrace:` | `wasmtime execution trapped` | a trap message, stable |

Do **not** use the fixture filename as the substring. It appears in every assertion message in
`assert_helper_fixture_is_ok` (`iterator_helpers.rs:50-71`), so a row pinned on it would go green on any
failure of that test — the same vacuity the bare-`should_panic` ban exists to prevent.

### The coupling with the iterator lane, stated in both directions

**If the iterator lane closes all 13 in this batch:** those tests go green, so writing rows for them
would be actively harmful — libtest reports a passing `should_panic` test as `test did not panic as
expected`, i.e. the ledger itself turns rung 1c red. The infra lane must then take **option A**: bump
`CURRENT_BATCH` to 4, delete tsv:67, add nothing. This is only legal once the 5 remaining chunks
(`frontend` 46, `typed_array` 58, `array` 84, `language` 105, `binary_data` 38 = **331 tests**) have
come back to a verdict — `binary_data` is expected to contribute exactly one red, the already-declared
T17 hang.

**If the iterator lane does not close them:** the 13 reds become the known-failure set, and the infra
lane takes **option B** with the 13 rows above (13 tsv rows + 13 `should_panic` + 13 `pub(crate)` +
13 `const _`). Note the friction b5 flagged: three of those rows (`some`/`every`/`find`, class ii) pin
on a symptom that is a wrong-typed value read from unrelated memory, so even `calls-0` is a bet on the
defect's *shape* not changing. If the lane is mid-repair rather than parked, **option C** is more
honest than B: extend `# unfilled-allowed-until:` to `batch-5` with a rewritten reason, so the deadline
survives and the ledger does not acquire 13 rows that will all need deleting next batch.

**If the iterator lane closes some but not all:** option B over exactly the survivors. The infra lane
must read the *re-measured* `iterator` and `iterator_helpers` chunk verdicts at post-fix code, not b5's
— `rung1c-chunks.sh` will skip both as already banked, so **delete their two lines from
`target/watched/rung1c-done`** before resuming. That is the one legitimate edit to the done-file, and it
is the exact opposite of the script's property 3 ("do not clean it up between runs"), so it needs saying
out loud: property 3 forbids *wholesale* deletion, not a targeted invalidation of chunks whose subsystem
changed. The script's own header already makes the same argument about the batch-4 seed
(`rung1c-chunks.sh:96-112`).

**The ordering constraint that binds all three cases:** the bump and the fill are ONE patch. Bumping
first leaves rung 1c red for everyone; filling first leaves the deadline vacuous. And neither can be
written from b5's numbers alone, because 331 of 607 tests have still never produced a verdict at any
head.

## Sequencing recommendation for the infra lane

D1 and D2 are independent of everything else and should land **before** the next rung-1c resume — they
are what stops `frontend` failing to bank a fourth time. D3 cannot start until that resume completes.
Concretely: land D1 (3 files: new module, `main.rs`, `rung1c-chunks.sh`) + D2 lever 1 (one `.arg` pair),
verify at rung 0 with `cargo test -p porffor-cli --test cli -- known_failures::` (5 tests, 0.02 s in b5
— it is the cheapest possible check that the chunk/module bijection still holds), then resume
`./scripts/rung1c-chunks.sh` verbatim.
