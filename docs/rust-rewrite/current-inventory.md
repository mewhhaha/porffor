# Current project inventory — 2026-10-23

This is the maintained status entry point. Results belong to their measured
source and executable, not automatically to today's checkout. The
[earlier inventory](inventory-history-20261021.md) retains all detailed historical
counts and links. [Workflow](batch-workflow.md) owns operating instructions;
[maintenance hills](../maintenance-hills.md) owns cleanup status.

## Current work

### Latest performance diagnostic — paired repeatability, 2026-10-23

P005 now has empirical same-source ranges and adjacent effects for all sixteen
preserved ABBA cases. No execution reran and no earlier optimization decision is
retroactively validated. [Analysis and reproduction](performance-repeatability-20261023.md).
The current ledger is **26 terminal / seven partial / 127 unresolved**.

### Latest affected checkpoint — cached string hashes, 2026-10-23

Source **`97036e5a8`**, runtime layout **16**, passes **4,083 native tests**
across twelve affected release scopes, eighteen compiled semantic families and
277 fast source guards. Initial stale test expectations are retained in the
failed receipt; complete engine-lib/runtime-cache reruns use the test-only
repair `7efa13482`, while the other ten native targets and measured
product retain source53. The **13,104-case original Test262 replay** gives
**13,100 Success / four known Bug / zero Crash / zero NotImplemented**.
This closes P023/P112 with explicit string-storage and retention timing costs;
[measurements and exact evidence](v8-baseline.md#cached-string-hash-checkpoint--published-2026-10-23).
That checkpoint brought the ledger to **25 terminal / seven partial / 128 unresolved**. Broader acceptance
remains open. The full-suite chart and V8 comparison below remain source50;
source52 and earlier checkpoint details are historical evidence.

### Completed diagnostic — preserved row scaling, 2026-10-23

P003 is complete with [analysis of 21 preserved cases](performance-row-scaling-20261023.md):
geometric row/array-width controls, normalized cost plots and the observed
capacity transition. These historical source cohorts are not a new current-product
benchmark or an isolated GC measurement. No original benchmark was rerun;
P001/P002 execution attribution remains open.

Previous accepted product: **`5c63c6597`**, the
[prepared property-definition checkpoint](v8-baseline.md#prepared-property-definition-checkpoint--published-2026-10-23).
It closes **P020** with **3,893 native passes / zero failures / zero ignored**
across nine complete affected targets and seventeen compiled semantic families.
The **7,174 original affected Test262 executions** return **7,170 Success /
four known Bug / zero Crash / zero NotImplemented**. No new failures appear.
All source/linker fences and independent runtime-payload identities pass.
The stale language source-count guard is repaired and its entire 151-test target
rerun; the original failure, storage deferrals and rejected continuation launch
remain in [portable evidence](performance-property-definition-20261023.json).

Missing long/short-key definitions have paired medians 60.357→56.084 ms and
28.359→26.802 ms. Existing-key updates are flat; object/retained-row rounds
overlap. Cold startup rises 1,035.635→1,062.143 ms; the CLI grows 893,104 bytes
and process peak RSS roughly 1–1.5 MiB. Slower initial screens remain visible.
These measurements do not replace the separate source50 V8 table or full-suite
chart. Layout15 and the helper ABI are unchanged.

The early source check now covers both structural targets: **277 tests across
94 complete source-only modules in about ten seconds**. Eight typed modules
remain in the mandatory full Cargo targets; nine tooling controls pass.
At that preceding checkpoint, counts were **22 terminal / seven partial / 131 unresolved**.
Broader release/all-feature, retention and remaining performance gates stay open.

Previous affected native checkpoint: **`b9ecfe64b`**, with
[portable observations and exact source boundaries](performance-hot-reads-20261022.json).
Seven complete runtime targets pass **3,576 tests**. The full structural target
then passes **166/166** after repairing one stale assertion about trim-policy
visibility, giving **3,742 native passes / zero failures / zero ignored** across
eight disjoint targets. The original 165/166 failure remains in the evidence.
Production compiler inputs are unchanged by the test/doc/tool repair; all
seventeen compiled semantic families and seven independent product/test runtime
identities pass. These are affected scopes, not a default/all-feature sweep.

The README now uses the source50 ordinary Wasmtime/V8 report: cold/cached startup
1,084.901/801.288 ms, integer 6.505 ms, objects 15.040 ms and retained rows
1,287.305 ms. V8 is faster in each workload. Paired source48→50 confirmation
gives indexed reads 173.015→152.961 ms, charAt/at 43.131→41.011 ms, single-unit
strings 77.580→74.219 ms and ASCII iteration 15.740→14.845 ms. Initial slower
cohorts and overlapping integer/Unicode rechecks remain; no isolated or
statistical gain is claimed. Startup is roughly flat. Layout15 and the existing helper ABI stay.

The earlier source-only early check ran **143 guards in about five seconds**,
without Cargo's compiler dependency graph. Six tooling controls pass; all four
typed IR/Wasm modules remain in the mandatory complete Cargo structural target.
Tracked `.cargo/config.toml` is restored and checked before that diagnostic:
its accidental sparse-checkout absence had forked dependency fingerprints.
The focused Cargo recheck explicitly retains empty encoded Rustflags to reuse
the unchanged preceding dependency cohort; subsequent default builds use the
restored config. [Commands and limits](batch-workflow.md#fast-source-only-guards).

The [fresh full Test262 campaign](test262-current.md) finished on **2026-10-23**:
**102,990 Success / 4 known Bug / 0 Crash / 0 NotImplemented**, all **748 nodes /
102,994 original executions**. The isolated checkout remained at `4aac260e7`,
using the exact source50 product above. No new failure IDs appeared; the twelve
previously repaired failures stay fixed. The native canonical generation and
README graph now use this full run, with no historical failure replay mixed in.
The 429 out-of-scope executions all passed through the native host; this unchanged
scope partition does not hide failures or establish standalone dynamic eval.

The campaign resumed the same checkpoints at a read-only barrier with 16
sequential nodes per invocation, preserving four workers, case budgets,
isolation and every leaf checkpoint. Native publication exited zero. The
sampled whole-cgroup peak was **34,264,653,824 bytes**, with zero OOM events;
this includes shared files/tmpfs and is not JS heap memory. The original pause
and all raw logs, provenance, aggregate/leaf snapshots and completion records
remain portable in the [full-run receipt](test262-full-20261023.json).

The full run retained **21 terminal / eight partial / 131 unresolved**; the
new property-definition checkpoint closes one more hill. This full run closes
the outstanding conformance measurement for source50, not every
performance hypothesis or release gate. A historical availability audit finds
twenty `/dev/shm` archive paths absent here; their old binaries are no longer
claimed restorable. Current products/runtime payloads and retained observations
are verified. Do not use shared memory as persistent history storage.

## Earlier checkpoints

The records below retain their original source and acceptance boundaries.
References to unchanged graph/full-suite counts describe those earlier batches;
the current full publication is the 2026-10-23 campaign above.

Previous numeric/collection acceptance source: **`37ab9fa44`**, with [4924 portable evidence files](performance-prepared-numeric-20261022.json).
The [numeric/collection checkpoint](v8-baseline.md#numeric-and-collection-checkpoint--published-2026-10-22)
passes **3,575 release native tests / 0 failed / 0 ignored** across seven complete
affected targets, twelve compiled semantic families and dynamic-call controls.
Source/linker fences and seven independent product/test runtime identities match.
The **3,235 original affected Test262 executions** give **3,231 Success / 4 Bug /
0 Crash / 0 NotImplemented**, exactly the same tracked failures at original budgets.
Full-suite, default/all-feature acceptance and chart counts remain unchanged.

That checkpoint's measurements remain bound to **`bf681edcb`**. Its acceptance removes only one
optional Rust annotation; all 21 emitted P modules match exactly, R native bytes
and keys match, and authenticated source/package framing is checked explicitly.
Its former README table used the measured source47 ordinary V8 report. Compared with
diagnostic45's separate ordinary report, cold startup and retained rows are
slower; cached startup, integer and object medians are lower;
these separate cohorts are not causal comparisons.

P081's exact-arity numeric Math.imul extension is verified after actual callee,
code and Realm proof, with generic fallbacks. Paired direct/alias medians improve
24.1%/53.8%. P020 now spans computed callbacks, groupBy, cross-owner Set and
deletion, but remains **partial** for ordinary define-own lookup/append hashing
and broader multi-pass work. Slower initial Map/object controls and the targeted
overlapping/flatter rechecks are retained. Status is **21 terminal, eight
partial/open, 131 proposed** of 160 hills; this is not completion of all hills.
Earlier wider native/Test262 acceptance remains bound to its original sources.
The product build takes 3m42s with full runtime-image cache hits. Retired bytes
are restore-only SHA-verified archives; current dependencies/cache budgets stay.

The later [NativeHost compression experiment](v8-baseline.md#rejected-nativehost-storage-experiment--2026-10-22)
is **rejected and removed**. It saves 59.3 MB per product but makes application-cold
startup 6.6% slower and cached startup 7.1% slower, with higher child peak RSS.
Its 21 exact programs and eight runtime-file checks pass; no native/Test262
acceptance is assigned to that experiment. Accepted48 code and Portable storage
compression remain. The hill counts and full-suite chart are unchanged.

The [opt-in CLI chunk scheduler](verification-cli-queue-20261022.json) passes 74
tooling controls and an exact four-way real 858-ID inventory check. It preserves
two live libtests and default policy. Its [complete 858-body observation](verification-cli-queue-observation-20261022.json)
passes but takes 783.161s versus the earlier two-chunk 757.216s. Sampled group
peaks are 31.45 versus 32.00 GiB; cache/file-page state differs. This single
comparison establishes no speed or causal memory benefit, so the default stays.

The first eight maintenance hills are complete: obsolete product routes are
retired, runner setup is clearer, LDML parsing has one owner, and current guidance
is separated from history. Implementation is in `ab7093a46`, `30065bbb5` and
`e7532de91`; [verification and exact source limits](maintenance-first-wave-20261021.md)
are recorded with [artifact identities](maintenance-first-wave-20261021.json).
The following [tooling/documentation wave](maintenance-tooling-wave-20261021.md)
completes 28 more hills. The [CLI/reporting wave](maintenance-cli-wave-20261021.md)
completes nine more. The [conformance/publication wave](maintenance-conformance-wave-20261021.md)
completes another 16. The [compiler/runtime/Intl wave](maintenance-compiler-intl-wave-20261021.md)
completes the final 39: **all 100 authorized maintenance hills are done**.
The final checkpoint covers all 109 all-feature native scopes after two complete
test-owner replays, plus 685 tooling controls and all 15 Intl identities.
107 default and 109 all-feature planning scopes remain unchanged. Maintenance
adds no published Test262 passes and does not establish release acceptance.

The [performance backlog](../performance-hills.md) contains 160 investigations
and optimization experiments. Its first verified execution batch ships batched
concat (P091) and matching optimized Portable runtime images (P153); the numeric
Array experiment (P040) was rejected and removed. Execution improves in the
measured corpus, with an explicit 116.01 MiB executable and startup-time cost.
[Results, exact source limits and evidence](v8-baseline.md#first-execution-batch--published-2026-10-22)
record 2,687 passing ordinary native tests and a 186-execution Test262 replay:
182 Success / the same four Bug results. This is not a fresh full Test262 or
all-feature workspace checkpoint; published conformance counts are unchanged.

The [second performance batch](v8-baseline.md#second-execution-batch--published-2026-10-22)
is accepted at `865189051` for the measured corpus and **3,567 scoped native
tests**, with zero failures/ignored tests and unchanged source. Product and test
runtime image bytes match. It retains shared immutable empty storage, bulk
repeat/pad/trim/slice operations, shorter array-length/normal-call routes,
identical-operand descriptor snapshots and transient decoded Portable images.
The CLI shrinks by 57.12 MiB against the preceding accepted batch; retained-row
invocation process RSS is 373.68 MiB and the separate whole-child peak is
392.80 MiB. These are process measurements, not live GC bytes. That seven-sample
Wasmtime/V8 report is retained as the preceding baseline.

Its [final 9,207-execution Test262 replay](performance-second-wave-replay-20261022.json)
returns **9,203 Success / 4 Bug / 0 Crash / 0 NotImplemented**, preserving exactly
the original four upstream-tracked failures. No new regression appears in the
campaign. The intermediate F replay is historical; the final affected-case
campaign does not refresh the full-suite graph. Rejected layout-16 binding and numeric-branch
experiments remain removed, with their evidence retained. Layout remains 15 and
bindings retain immutable StoredValue snapshots. Performance work remains open.

Startup host sampling identifies SHA-256 as roughly 64–74% of sampled user CPU.
A separately identified debug-stripped copy is prepared with allocated ELF
sections verified equal, full executable hashing preserved and separate caches.
The [completed ABBA diagnostic](performance-startup-strip-20261022.json) measures
identity binding 376.835→221.256 ms, cold start 1,171.645→1,018.480 ms and cached
start 923.743→739.889 ms, with unchanged process RSS. The next batch keeps
release CLI/example DWARF stripping. The combined P017/P038 literal experiments
were built, passed six compiled semantic families and were then removed after
screen/ABBA measurements showed no reliable throughput gain. [Exact evidence](performance-literal-decisions-20261022.json)
retains the failures of the performance hypothesis and all raw controls. New
semantic fixtures remain; layout and the emitter behavior return to accepted
`865189051`. The [packaging checkpoint](v8-baseline.md#release-packaging-checkpoint--published-2026-10-22)
is accepted at `b59303ebe`: **3,568 native passes**, no failures/ignored tests,
unchanged source/linker, seven identical product/test image payloads/manifests
and six compiled semantic families. Its 277.03 MiB CLI and actual seven-sample
Wasmtime/V8 report previously supplied the README table. The expanded **9,411-case
Test262 replay returns 9,407 Success / 4 Bug / 0 Crash / 0 NotImplemented**,
with exactly the original four failures. [Final replay evidence](performance-kept-packaging-replay-20261022.json)
supplements the immutable native/measurement publication; full-suite counts
remain unchanged.

The [replacement checkpoint](v8-baseline.md#replacement-assembly-checkpoint--published-2026-10-22)
is accepted at `1e2978361`: **3,569 native passes / 0 failed / 0 ignored** across
seven affected targets, six compiled semantic families and exact catchable overflow.
Four original passed scopes remain in their receipt; a same-source continuation
covers only three storage-deferred scopes. Product/test runtime image bytes match.
The **410-case affected replay returns 406 Success / 4 Bug / 0 Crash /
0 NotImplemented**, with exactly the original four failures.
[933 verified portable evidence files](performance-replacement-20261022.json)
retain source identities, raw observations and all comparison cohorts. No fresh
full-suite/all-feature claim or graph-count change is made.

P107's eight-root collector retains large original-baseline gains and removes the
observed short two-match obstacle. Some intermediate collector cases are slower;
those tradeoffs remain explicit. P004 completes checked empty/loop/scalar-expression/
scalar-return controls, without subtracting setup from raw scores. Fresh Wasm
Realm/builtin initialization runs inside timed main. Its earlier ordinary report
previously supplied README values, including its slower cold-start median; its
balanced startup diagnostic remains separate. The earlier collector and bounded-prefix revision remain
[rejected](performance-replacement-probe-20261022.json)
[diagnostics](performance-replacement-prefix-probe-20261022.json).

At that replacement checkpoint the ledger had **20 terminal decisions, seven
partial/open outcomes and 133 proposed/unresolved hills**. Its planned insertion
hash/argv work is now the measured key/argument checkpoint above. Remaining
value-only getOrInsert scope and static/repeated key hashing stay open; the
opt-in four-chunk queue does not establish a benefit in its first complete observation.

The CLI/reporting wave passes 478 Test262 library tests, 89 CLI unit tests and
14 affected CLI integration controls after correcting two batch test/diagnostic
issues and ten preexisting rename-related expectations. Its record retains the
earlier full 856-ID CLI checkpoint and adjacent checks with their source limits.

The conformance/publication wave passes 493 Test262 library tests, all 856 CLI
integration tests, 91 CLI unit tests and 29 adjacent native tests across its
corrected source-bound checkpoints. It also passes 669 tooling controls and 53
structure checks, and admits all 53,910 pinned Test262 files (102,994 execution
IDs). Failed attempts and the test-only changes between checkpoints are retained
in its record; that earlier selected checkpoint implied no complete workspace
or conformance refresh. The final maintenance checkpoint is recorded below.

## Test262: current complete run and history

| Evidence | Source / scope | Observed result |
| --- | --- | --- |
| **Current complete pinned suite, 2026-10-23** | **Compiler `b9ecfe64b`; all 102,994 executions** | **102,990 Success, 4 Bug, 0 Crash, 0 NotImplemented** |
| Original complete suite, 2026-10-20 | Compiler `c5053c9d4`; 102,994 executions | 102,978 Success, 16 Bug, 0 Crash, 0 NotImplemented |
| Original failure replay, 2026-10-21 | Compiler with global-prelude repair `cbb07c6cf`; 16 exact IDs | 12 Success, 4 Bug |
| Remaining-failure recheck | Four original IDs and ten nearby controls | Four failures retained; all ten controls pass |
| Maintenance revalidation | Default compiler `e7532de91`; original 16 IDs, then four plus ten controls | 12 Success/4 Bug; then 10 Success/4 Bug; zero Crash/NotImplemented |

[Current full publication](test262-current.md) and [historical focused replay](failed-case-replay-20261021.md)
bind exact pins, compiler fingerprints, executable hashes and commands. The
current canonical publication includes every original execution on source50.
Its scoped graph partitions those 102,994 IDs as **102,561 in-scope passes,
four failures, zero NotImplemented and 429 out of standalone Wasm scope**.
The complete 2026-10-20 baseline and its replay reconciliation remain historical.

**User decision: keep all four original disagreements as failures.** The README
lists reasons and [upstream tracking](../../test262/errata/proposals/upstream-status-20261021.md):
arguments has an exact open issue/PR; Johnston has related open discussion;
labelled functions have related discussion but no exact issue found in the search.
Local amended-fixture and specification proposals are held. They do not change
original results, and no report was posted upstream. The
[resolution evidence](final-four-resolution-20261021.md) remains historical.

## What the product can compile and run

Scriptw compiles JavaScript through parsing, early errors, spec IR, lowering IR
and Wasm code generation. Wasm-AOT is the default; the feature-gated interpreter
is an engineering oracle. C/native JavaScript output and a REPL are out of scope.

Verified representative programs include closures and private fields, object
properties/Proxy/Reflect, collections, typed arrays/BigInt/Unicode/JSON, Promise
jobs and generators, filesystem ES modules with top-level await, and selected
Intl/Temporal operations. The CLI provides `print`; it does not supply Node or
browser globals such as `console`.

`run file.mjs` and `run --module file.js` use the Module goal. `build wasm file.js`
compiles a Script and can emit a program/runtime Wasm pair through
`SCRIPTW_WASM_DUMP`; a dedicated module-artifact build command is not exposed.

The ordinary default-feature debug CLI at `e7532de91` builds without the oracle
and preserves the original failure-replay results. The default-feature optimized
release at `cbbd9b170` builds and completes the new performance/memory corpus;
its recorded timings, child RSS, compilation
phases and emitted Wasm are in the [current measurement](v8-baseline.md).
The earlier optimized release at `c5053c9d4` also builds without the oracle.
Three synchronous React 19.3.1 SSR outputs match Node at that earlier source;
the public browser entry fails. [SSR evidence](react-ssr.md) and
[local Wasmtime/V8 measurements](v8-baseline.md)
are focused demonstrations, not full compatibility or calibrated T25 acceptance.

The host requires the [documented Wasm features and Scriptw imports](runtime-requirements.md).
Portable Wasm does not bundle a parser/VM for general eval or Function constructors.
Finite prepared sources and native-host compilation are distinct supported paths.
The chart now has an audited 429-execution dynamic-source subset; the scope
inventory is conservative and does not claim to classify every incidental
mention of eval or Function.

## Native verification and release gates

The final [compiler/runtime/Intl checkpoint](maintenance-compiler-intl-wave-20261021.md)
covers every one of the 109 all-feature native scopes: **9,712 passed, zero
remaining failures, zero ignored**. The frozen workspace run passed 107 scopes
and exposed two stale source-layout assertions. Only those two test files changed;
complete frontend (181) and engine structure (33) replays then passed with the
same dependency closures. The zero-test feature anchor adds no tests.
[Exact receipts and source boundaries](maintenance-compiler-intl-wave-20261021.json)
retain both original failures and all inventories.

This includes all 973 engine unit tests, 299 Engine Intl integration tests,
856 CLI integration tests and the differential campaigns at their original
budgets. The final full tooling checkpoint passes all 685 controls with zero
skips under TMPDIR=/tmp. It includes three new mutation controls for standalone
source guards corrected after the native replays; no compiler or native-test
source changed. Earlier failed boundary checks are retained in the record.
Locked all-target/all-feature Cargo check and all 15 Intl identity checks pass.
Generated semantic payloads remain byte-identical. This is all-feature debug
native verification, not default/release or a complete refreshed Test262 run.

The stopped pre-rename campaign at `cbf7cc71f` remains historical: 49/113 scopes,
7,560 passes, one failure and zero ignored tests, with 64 scopes unfinished.
It is superseded for current all-feature coverage by the complete 109-scope
checkpoint above, not silently resumed or relabelled. Its original failure was:

`scriptw-test262:differential_generated_campaign::builtin_scenarios_use_actual_selected_workers_for_all_six_stateful_families`

The seed-0 array-buffer Wasm worker exceeded its original 5,000 ms deadline.
That control now passes at the unchanged deadline in the new complete native
checkpoint. This does not prove the old timeout's cause; its failed executable
and receipts remain in the [historical source-bound inventory](current-inventory-20261020.json).

Outstanding gates include:

- Complete current-source default native and ordinary optimized release
  acceptance; current all-feature native coverage is complete as recorded above.
- Original 230-case/prior 219-case retention and real serial/shard/resume checks.
- Complete debug/optimized differential and robustness campaigns, idle samples,
  supported-host reproducibility and calibrated T25 performance.
- A second fresh complete current-pin Wasm-AOT family and deterministic backlog
  generation; the first fresh family and canonical publication are now complete
  at `b9ecfe64b`, with all four non-successes visible. Green acceptance remains open.
- Historical 5,365-failure replay: its original aggregate is absent from this
  cloud checkout and was likely local/unpushed. Do not manufacture that input.

## Task acceptance

T00, T27, T28 and T29 remain recorded complete; T01–T25 remain in progress and
T26 remains blocked. These are permanent technical areas, distinct from the
maintenance hills. [Ownership](conformance-ownership.md) and
[acceptance criteria](conformance-acceptance.md) are authoritative.

Keep CONTINUE and tasks until every criterion passes. The decision to retain
upstream failures documents their status; it does not establish a green suite
or waive release acceptance. The [continuation queue](../../CONTINUE.md) records
next actions without duplicating this inventory.
