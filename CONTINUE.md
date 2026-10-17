# Complete current Engine retention — 2026-10-17

This block supersedes older publication-time notes below.

Complete current Engine verification (2026-10-17):965/965 native tests
pass, with all961 previous passes retained and zero ignored or filtered tests.
The exact archived ELF runs from writable scratch; the earlier checkout-storage
failure is preserved in the preceding checkpoint. No Rust fixture patch was
needed. [Complete results and raw evidence](docs/rust-rewrite/serial-dispatch-complete-engine-and-storage-20261017.json) also record
lossless shared compression of30 closed native artifacts: every original byte
and original compressed frame was hash-verified before unlink, and exact restore
plus changed-bundle rejection controls pass. This recovers408MiB
for builds. The original27 single-CPU replay remains24Success/3Crash with a prior
sloppy proto success lost. A separate supported multi-CPU compiler experiment
is pending. Other current native scopes, ordinaryCargo and full conformance
remain open; keep CONTINUE.md and tasks.

Older notes follow.

# Current native verification and literal retention — 2026-10-17

This block supersedes the older publication-time notes below.

Current retention and reuse measurement (2026-10-17): original27 is
24Success/3Crash, with strict proto newly passing and the prior sloppy proto
success lost. Literal retention is still red; both class-default variants also
time out. The current Engine inventory completed964passes/1failure: its
module-policy fixture tried to write under the full checkout. The exact same
compiled binary passes that focused test from writable scratch, preserving both
module-policy controls. A complete965-test rerun from scratch is pending.
[Portable results and raw evidence](docs/rust-rewrite/serial-dispatch-retention-and-reuse-20261017.json)
include all600 exact generated Function templates. The compile-only probe
measures600 distinct templates,600 without custom metadata, and500 when only
body scalar literal values are ignored. Its512- and1024-entry LRU simulations
both have600misses; no larger-cache speedup is established. Diagnostic hashes
are never compiled or cached. Full conformance and task cleanup remain open.

Next: finish the complete current Engine965 rerun, then repair the remaining
sloppy proto and both class-default timeouts at the original60000ms deadline.
The aggregate24/27 does not retain every previous success. The prior2492-pass
native checkpoint belongs to the preceding compiler source. Remaining203,
historical5365, ordinaryCargo/features/hosts/shard-resume/fullpinned and every
task criterion still require their own passing evidence. T22 still needs
accepted normative/test authority. Keep CONTINUE.md and tasks until complete.
Older notes follow.

# Serial compiler dispatch: focused strict pass observed — 2026-10-17

CurrentCPP1a9e425ed20728e5c0fdfe90fe73af8028500a11419362abe59d64727209ee63;
nativeFPb4f121eff0513714278d34ae4e89ad214c28934c8fb3521d145034c847b87c30.
The same-profile cold proto2 replay is1Success/1Crash: strict succeeds;
sloppy still times out. All deadlines60000ms/worker1/512entries/2GiB are intact.
Four new native dispatch/stack tests pass; all24 product build stages pass.
The previous complete2492-pass native checkpoint remains scoped to its prior
source. Run full original27 literal retention and current Engine965 inventory
next; do not infer either from focused results. Keep every failure visible.

Serial jobs avoid Rayon handoff only on explicitly admitted64MiB engine stacks.
Unmarked callers and parallel jobs preserve the bounded pool. Fresh nativeR
bytes match exactly; seeding a private cache by value reduces measured bundle
production from~54s to2.18s. Runtime scopes now bind complete current CPP/NFP
sources and actual ELF/shared-image/test inputs, plus immutable build provenance,
without recursively rehashing unused historical build outputs. The byte-verified
Linux archived-test loader avoids restoring a183MB unit ELF to a full tmpfs.
It is unsuitable for integration binaries that spawn themselves by pathname.

[Portable scoped results and raw evidence](docs/rust-rewrite/serial-native-compilation-checkpoint-20261017.json).
Strict proto and both class-default timeouts, ordinaryCargo/features/hosts,
remaining203/historical5365/fullpinned/shard-resume and all task gates remain open.
T22 Apia still needs accepted normative/test authority; issue3310 and draft
PR3318/5044 do not resolve it. Do not delete CONTINUE.md or tasks yet.
Older publication-time notes follow.

# Deadline retention failed — 2026-10-17

Stop broader current-source replay until the lost success is restored. Actual
original27 replay completed23Success/4Crash, retaining23 of24 earlier successes.
The sloppy object-literal `__proto__` case now times out; both class-default modes
and strict `__proto__` still time out. All remain in the denominator, unchanged
60000ms/worker1/512entries/2GiB. Exact causal source/profile attribution is not
established; run matched old/new cold-cache samples before changing code.
[Complete literal evidence](docs/rust-rewrite/current-replay-retention-checkpoint-20261017.json)
also records the passing actualCLI entrypoint and both CLI integration targets.

The bounded actual-source trace observes582cachemiss/993cachehit; first212 paired
lookups contain94misses taking5.749s and118hits taking0.004665s. It is diagnostic,
not acceptance or proof of GCtype-prefix cost. Consider a measured coherent
optimizedIR/AOT closure (ordinaryrelease3/test2; this native diagnostic isopt1),
retaining exact source/fingerprint/typed dependency and freshRbyte checks.
The complete native2492 checkpoint remains valid for its own scope. Original
retention, ordinaryCargo/features/hosts/fullpinned/shard-resume/T22/alltasks remain
open. Do notdeleteCONTINUE/tasks. Older publication-time notes follow.

# Complete repaired-source native checkpoint — 2026-10-17

The current compiler passes2,492 native tests with0failures/ignores/filtered:
Engine961/Runtime33/AOT401/Test262435/CLI82 plus all7priority integrations580.
Literal complete inventories retain every earlier pass, repair both previous
integration failures, and add the Engine RegExp ownership regression. Evidence:
[portable complete checkpoint](docs/rust-rewrite/complete-repaired-source-native-inventories-20261017.json).
All results bind compiler1181725580ab80195edccabbec2dbd9738bdcaa84f9a8023f8dc6c3c732c08a6
and nativepolicyb4f121eff0513714278d34ae4e89ad214c28934c8fb3521d145034c847b87c30.

Next: current actualCLI entrypoint and exact original27 retention at unchanged
60000ms/worker1/512entries/2GiB, measured cache outcomes for remaining deadlines,
remaining203/historical5365, ordinaryCargo/default/allfeature/alltargets,
supportedhosts, shard/resume, two complete pinned families and all task gates.
No full-workspace or literal100%claim is made. Keep CONTINUE/tasks until the
actual criteria pass. Older notes below are historical and retain their original
source identities and publication-time snapshots.

# RegExp owner-hook repair — 2026-10-17

Currentcompiler1181725580ab80195edccabbec2dbd9738bdcaa84f9a8023f8dc6c3c732c08a6;
nativepolicyb4f121eff0513714278d34ae4e89ad214c28934c8fb3521d145034c847b87c30.
The live RegExp constructor previously bypassed its typed P-owned candidate hook.
It now validates flags, selects the exact current ProgramInstance hook, retains
cached syntax failures, and calls the real runtime compiler only on a null miss.
Completion/error precedence, GC roots and nonnull success remain checked.
The five-hook Wasm assertion remains intact and now passes. A runtime regression
covers conversion order, invalid flags/patterns, computed misses, an intervening
eval program and retained-caller matching. The Temporal source witness follows
its actual tests.rs owner, preserving every existing assertion.

Current-source runtime-link30/30 and Intl/Temporal89/89 pass; all117 previous
passes remain. The previous complete7-target baseline was578/580; other five
current-source targets still need replay. Full Engine961/961 passes; all960 previous passes are retained.
[Complete Engine evidence](docs/rust-rewrite/full-engine-regexp-owner-hook-retention-20261017.json) preserves the finished receipt and raw output.
The new exact RegExp regression and both cache regressions pass. Prior-source
Engine960/Runtime33/Test262435/CLI82 full inventories are historical acceptance,
not automatic current-source full-suite acceptance.

The fresh24-stage native build passes with10 exact unchanged typed dependencies
retained and14 fresh stages. Full source guards, all14 official generators and
whole selected Rust formatting pass. Source-composer bookkeeping incorrectly
expected a changed native policy fingerprint after all actual checks; the
explicit completion receipt preserves that failed parent assertion and verifies
the unchanged native inputs and changed compiler-source identity. No failed
source check or historical receipt was rewritten. Publication writes no separate
full Git index and reproduces all2689 compiler and1052 native inputs from the
actual Git tree. See [portable evidence](docs/rust-rewrite/regexp-owner-hook-native-checkpoint-20261017.json).

Next: run the other five integrations and full
AOT/Test262/CLI units on this closure, replay original27 retaining all24 prior
observations, then remaining203/historical5365 and actual ordinary Cargo/default/
all-feature/host/shard-resume/two full pinned-family/task gates. Three previous
60000ms timeouts remain unclaimed. Do not remove this file or tasks until real
completion. The old module-lookup trace includes validation and Cranelift work;
it does not establish GC type-prefix dominance. Rerun existing trace with actual
cache outcomes before redesigning types. Older notes below are historical.

# Compiler and retention checkpoint — 2026-10-17

The current source has a complete source-bound native compiler/runtime build.
The original27 focused Test262 executions improve from19Success/8Crash to
23Success/4Crash. All19 prior passes remain. Repaired IDs are the for-in named
class initializer, strict simple destructuring, and both regress-610026 modes;
the large empty-block tests complete within their original deadlines. Four
class/prototype staging timeouts remain. No requested ID was skipped.

The complete IR library inventory passes1598/1598, none ignored or filtered.
The full AOT library inventory also passes401/401, none ignored or filtered.
The native-source Engine group passes69/69 (891 other tests filtered).
Three test-only emission sites now select NativeHost for the same inspected
IR; they previously used the portable export profile. The original failure
diagnostic remains in the portable checkpoint. The full960-test Engine run
is still collecting results; full Engine acceptance is not claimed.
Its empty for-await assertion now checks the optimized Empty representation
and absence of a lexical frame while retaining binding and iterator checks.
The earlier1597/1 result and recorder errors remain in raw evidence. See the
[portable checkpoint](docs/rust-rewrite/compiler-source-regression-checkpoint-20261017.json).

Compiler source fingerprint25893417103a16e9ab5fa913e5c0e4338aef0ada012738e1c798b8badb568f49.
The staged Git tree reproduces all2690 compiler inputs and1052 native inputs
byte-for-byte. This is an opt1 direct build, separate from ordinary Cargo/CLI
acceptance. Full Engine and remaining integration inventories are next.

Cloud verification retains at most512 modules and the smaller of2GiB or one
eighth of inherited memory, preserving stricter explicit/local settings; all37
launcher controls pass. The compiler/cache batch changed together. A diagnostic
over1752 templates found600 unique code keys, still600 without debug names and
500 ignoring numeric values. Those diagnostic bytes were never executed or
used as cache keys; no speculative value-insensitive optimization was applied.

Next finish broad native inventories, profile/repair the four real timeouts,
then replay remaining203 and historical5365 IDs with exact pass retention.
Complete ordinary default/all-feature/CLI, supported-host/shard/resume and two
unfiltered current-pin families plus all task-specific gates. Preserve this
file/tasks until actual completion. Temporal's Apia window still has no accepted
fallback in current inspected upstream sources; full conformance is not claimed.

# Current continuation — 2026-10-15

Numeric lexer correction (2026-10-16): numeric literals reject a following Unicode IdentifierStart, and lexer checks retain the actual LegacyOctal token classification. All **311 captured native checks pass**: 300 parser checks and 11 Front checks. The same 311 IDs retain **307 previous passes** and repair the **four earlier failing assertions**.

The [checkpoint](docs/rust-rewrite/unicode-numeric-parser-checkpoint-20261017.json) preserves complete raw output and proves that the resulting 85-input normal parser/Front graph matches the compiled source. The earlier red checkpoint remains intact. This focused result does not establish ordinary Cargo, real Test262 or full-product acceptance. Generated status numbers are unchanged.

Parser correction (2026-10-16): sloppy function expressions accept `let` as a name across ordinary, generator, async and async-generator forms; strict bindings still reject it. All **five new native checks pass**, and the captured Front harness passes **11/11**.

The captured parser harness remains **red: 296 passed, four earlier lexer assertions failed**. The [checkpoint](docs/rust-rewrite/contextual-let-parser-checkpoint-20261017.json) retains all raw results and proves that the resulting 85-input normal parser/Front graph matches the compiled source. Unicode numeric-lookahead changes and broader acceptance are outside this patch. Generated status numbers are unchanged.

Private ABI9 candidate checkpoint (2026-10-16): all **six targeted real Test262 executions pass**, repairing two JSON, two ModuleSource and two Reflect cases. A same-profile 96-evaluation diagnostic reduces execution-and-completion time from **5.686 s to 1.386 s (4.10×)**; its separate cold module compilation takes 31.424 s, and the huge-eval performance target remains unmet.

The full IR unit suite remains **red: 1,577 passed, five failed**; IR integration remains **red: 732 passed, two failed**. The [portable checkpoint](docs/rust-rewrite/candidate-progress-20261016.json) preserves every failure and the [offline reader](docs/rust-rewrite/verify-candidate-progress-20261016.py) verifies its lossless raw evidence. Seven assertion repairs are authored and unrun; five other unit harnesses and 13 other integration harnesses are unrun at this checkpoint. This does not establish ordinary Cargo, combined published-source, full-suite or task acceptance. Generated status numbers are unchanged.

Unapplied repair checkpoint (2026-10-15): the fresh ABI6 runtime passes all
**four prepared same-Store ownership cases**, including conflicting function
rows, both installation orders, forced GC and typed reentry. Reentry returns
`42`; the original program remains `17` after the second program is installed.
The [captured diagnostic](docs/rust-rewrite/abi6-prepared-ownership-eb1ca63c9-20261015.json)
and [offline reader](docs/rust-rewrite/verify-abi6-prepared-ownership-eb1ca63c9-20261015.py)
preserve the two earlier driver failures and their corrections. Compilation took
6.232 seconds and native execution 62.164 seconds on one CPU, using the already
compiled candidate IR/AOT libraries. Native Function host misses are deliberately
rejected by this driver; Engine host-miss controls, ordinary builds and repaired
conformance acceptance remain pending. This diagnostic does not close a task.


Unapplied repair checkpoint (2026-10-15): all five affected default-feature
libraries pass a metadata-only compile in **81.250 seconds** end to end
(**46.173 seconds** of compiler work on one CPU). The original 12 AOT compile
errors and the five-file corrections remain in the [captured proof](docs/rust-rewrite/abi6-five-crate-metadata-eb1ca63c9-20261015.json),
with an [offline reader](docs/rust-rewrite/verify-abi6-five-crate-metadata-eb1ca63c9-20261015.py).
The complete type-fixed module-boundary check also passes. This compiles an
unapplied candidate; it performs no native code generation, unit/integration
execution, Wasm validation, stock Cargo build or conformance acceptance.
Native ownership/Function tests and broad repaired-suite verification remain pending.


Focused IR candidate checkpoint (2026-10-15): **36 actual native unit checks passed**,
including seven new and 29 existing controls; all 1,536 old unit IDs remain in the
1,543-ID inventory. This tests an unapplied IR subset, not the whole repair batch
or full IR suite. [Captured proof](docs/rust-rewrite/ir-foundation-eb1ca63c9-native36-20261015.json)
and its [offline reader](docs/rust-rewrite/verify-ir-foundation-eb1ca63c9-native36-20261015.py)
retain that scope. The full baseline was interrupted at 332/748 closed nodes by
a temporary object emitted into the checkout by the private compiler diagnostic.
Tracked source/index bytes stayed unchanged; the literal guard failure remains
recorded, and the completed nodes plus 190 interrupted checkpoint IDs are retained.
Guarded continuation and exact-ID/full-suite acceptance remain required.


## Dynamic-source foundation diagnostic — 2026-10-15

The [isolated native diagnostic](docs/rust-rewrite/dynamic-aot-same-store-eb1ca63c9-20261015.json) confirms same-Store reentrant
Wasm-GC function transport: a newly compiled program's function returns **42**
through the running program, survives forced GC, and the original call resumes.
A separate ownership control remains **red**: after a second program installs
its prepared-Function hook, the original program returns **23 instead of 17**.
General dynamic-source support therefore still requires per-program hook and
data ownership. The diagnostic reused the genuine shared precompiled runtime;
its final scratch compile/link took 4.000 seconds under concurrent suite load.
This adds no Test262 passes or product capability claim; all existing suite
failures and task gates remain recorded.

## Closed historical replay — 2026-10-15

The [exact 5,365-ID original-tree replay](docs/rust-rewrite/original-history-eb1ca63c9-20261015.json) is complete and audited:
**5,194 Success, 167 NotImplemented, 4 Bug, 0 Crash**. Against the earlier
5,142-pass checkpoint, 53 former nonpasses now pass and one former pass cannot
resolve a TXT dependency absent from its original pinned tree: net **52 more
successes**. Strict retention stays red: **5,172/5,173**. The importing case
separately passes on the current pin. A staged Annex B diagnostic confirms its
first failure at fixture line 33. A separate successful Unicode trace measures
32.027 seconds of lowering (19.095 prepass, 12.683 final body) under concurrent
load; it is not an idle benchmark. The current-pin full matrix remains active;
task closure and literal all-zero conformance remain pending.

## Verification gate map and shortcut audit — 2026-10-15

Compiler `eb1ca63c9` passes the actual checkout shortcut audit: **26 exact
entries**, exit zero in **0.964 seconds**, with unchanged source. The
[remaining acceptance map](docs/rust-rewrite/remaining-acceptance-eb1ca63c9-20261015.md) identifies required workloads already
covered by the complete default/all-features checkpoints and the distinct
campaign, idle, shard/resume, backlog and host gates. Reuse the active full
matrix for the current baseline and publication rather than starting another.
Historical/current-pin completion and task closure remain pending; retain
`CONTINUE.md` and `tasks/`.

## Verified ordinary release CLI55 — 2026-10-15

The normal default release CLI on compiler `eb1ca63c9` passes all **55 selected executions with Success**, with zero NotImplemented, Bug, Crash or InfrastructureError. All **53 preceding selected passes** are retained, and both original RegExp escape modes now pass. Fixture bytes, modes and pins are unchanged.

The complete ordinary build preparation passes in **156.575 seconds**; the actual source-bound native replay exits zero in **59.175 seconds**. It uses one worker, one thread and the unchanged 60,000 ms deadline, with the ordinary static runtime.

The [portable CLI55 proof](docs/rust-rewrite/ordinary-cli-eb1ca63c9-probe55-20261015.json) preserves exact IDs, pinned fixture bytes, actual Cargo/runtime provenance and lossless raw native evidence. It also preserves the original report-order verification error and the corrected read-only validation; the successful native run was not repeated. Verify offline using the two default checkpoints already published beside it:

```sh
python3 docs/rust-rewrite/verify-ordinary-cli55-eb1ca63c9-20261015.py \
  docs/rust-rewrite/ordinary-cli-eb1ca63c9-probe55-20261015.json
```

The historical 5,365-ID replay, complete pinned Test262 suite and remaining task gates retain their separate acceptance requirements. CONTINUE.md and tasks remain open.


## Complete all-features workspace at source eb1ca63c9 — 2026-10-15

Compiler source `eb1ca63c9` passes all **108 all-features workspace scopes: 9,269 passed, zero failed, zero ignored**. All **9,254 preceding all-features passes** are retained. The preceding default inventory is accounted for by **9,157 all-features passes plus 4 actual same-source default-only passes**. All **17 RegExp and labelled-scope regression controls** pass, including the three direct-eval activation regressions.

The [completed checkpoint](docs/rust-rewrite/all-features-workspace-eb1ca63c9-native-checkpoint-20261015.json) preserves exact native IDs, Cargo/runtime provenance, raw transcripts and the four-CPU, one-job policy. Verify it with:

```sh
python3 docs/rust-rewrite/verify-workspace-eb1ca63c9-20261015.py \
  docs/rust-rewrite/all-features-workspace-eb1ca63c9-native-checkpoint-20261015.json \
  --prior-default docs/rust-rewrite/default-workspace-8e9a8c82-native-checkpoint-20261015.json \
  --prior-all-features docs/rust-rewrite/all-features-workspace-8e9a8c82-native-checkpoint-20261015.json \
  --same-source-default docs/rust-rewrite/default-workspace-eb1ca63c9-native-checkpoint-20261015.json
```

The same source passes **547 Python tooling tests**, retaining all **546 preceding IDs**, including all **539 earlier IDs**. Formatting, module boundaries, Test262 host ABI and product/oracle dependency checks pass; [sealed raw evidence](docs/rust-rewrite/workspace-tooling-eb1ca63c9-20261015.json) records those separate checks and preserves the initial static exit1 and exact committed CI restoration.

Earlier failed and partial runs remain recorded below. This workspace checkpoint does not close the separate Test262 or task gates.


## Complete default workspace at source eb1ca63c9 — 2026-10-15

Compiler source `eb1ca63c9` passes all **107 default workspace scopes: 9,176 passed, zero failed, zero ignored**. All **9,161 preceding default passes** are retained. All **17 RegExp and labelled-scope regression controls** pass, including the three direct-eval activation regressions.

The [completed checkpoint](docs/rust-rewrite/default-workspace-eb1ca63c9-native-checkpoint-20261015.json) preserves exact native IDs, Cargo/runtime provenance, raw transcripts and the four-CPU, one-job policy. Verify it with:

```sh
python3 docs/rust-rewrite/verify-workspace-eb1ca63c9-20261015.py \
  docs/rust-rewrite/default-workspace-eb1ca63c9-native-checkpoint-20261015.json \
  --prior-default docs/rust-rewrite/default-workspace-8e9a8c82-native-checkpoint-20261015.json
```

The same source passes **547 Python tooling tests**, retaining all **546 preceding IDs**, including all **539 earlier IDs**. Formatting, module boundaries, Test262 host ABI and product/oracle dependency checks pass; [sealed raw evidence](docs/rust-rewrite/workspace-tooling-eb1ca63c9-20261015.json) records those separate checks and preserves the initial static exit1 and exact committed CI restoration.

The [source revalidation record](docs/rust-rewrite/workspace-source-revalidation-eb1ca63c9-20261015.json) preserves the original and post-run source snapshots. All prepared files, Git blobs and controls match; only Git index cache bytes changed. Its complete logical index reconstructs the committed source tree. This source revalidation followed the default native run, which was not repeated. The all-features run started after that revalidation.

Earlier failed and partial runs remain recorded below. This workspace checkpoint does not close the separate Test262 or task gates.


## Workspace verification repair — 2026-10-15

The complete `fb3a1235` default run closed with **102 passing scopes**,
**6,893 passing native tests** and **two failed structural assertions**.
Four partitioned scopes stopped before running tests because their coordinator
was restricted to one CPU by the native compilation job limit.

The coordinator now uses cloud worker affinity while native compilation,
Cargo and libtest remain serial. A regression exercises the actual workspace
launch command for all four partitioned targets with one, three and four CPUs.
The two structural expectations now name the lexical declaration selector
already used by the compiler; their assertions and test IDs remain intact.

All **45 focused Python controls** and **three source-structure Rust tests**
pass. The complete quick check took **0.453 seconds**; the standalone structural
compile took **0.149 seconds** and its tests **0.017 seconds**. These checks do
not replace complete native verification. Both full workspace modes, ordinary
CLI, historical retention and full pinned Test262 remain pending on this repair.
The failed run is preserved, and `CONTINUE.md` and `tasks/` remain open.
See the [verification commands](docs/rust-rewrite/batch-workflow.md#fast-source-structure-preflight--2026-10-15).

## RegExp source and labelled lexical scope repair batch — 2026-10-15

The compiler batch fixes RegExp source serialization after an already escaped
line terminator and preserves delimiter spelling inside character classes.
Finite source discovery and the emitted Wasm getter use the same policy, with
matcher and source round-trip regression coverage.

Block and switch lexical declaration selection stops at a labelled Statement,
as required by the pinned static semantics. Storage planning uses that same
selection, so a nonlexical chain cannot introduce an uninitialized binding
that hides an initialized outer variable from direct eval. Direct labelled
functions and top-level label-chain hoisting retain their respective rules.
The separate disputed Annex B outer-copy behavior is unchanged. See the
[lexical scope contract](docs/rust-rewrite/contracts/labelled-function-lexical-depth.md).

Source `fb3a1235` passes the exact four-scope native check: **1,556 passed,
zero failed, zero ignored**. All **1,551 prior positive IDs** remain passing,
both previously failed IDs now pass, and all three new activation-binding
controls pass.

The complete source-bound outer exits zero in **923.732 seconds**,
including compilation and orchestration. The table reports each raw libtest
duration; scheduler receipt times also include deferred completion polling.

| Native scope | Passed | Libtest seconds |
| --- | ---: | ---: |
| IR library (complete) | 1,536 | 55.37 |
| IR finite RegExp source | 4 | 3.21 |
| Wasm callable entry roles | 9 | 6.44 |
| Wasm finite RegExp source | 7 | 10.40 |

The invocation uses
`--packages lila-ir lila-engine --isolate-test-package lila-ir` to give each
Cargo producer its own package and avoid an unused Engine library test build.
`--only-targets` selects native executions; package and isolation options select
the Cargo graph. Earlier runs used different inputs and graphs, so these times
do not measure a controlled speedup.

The earlier `cc0210eb` run remains recorded as **1,551 passes and two failures**.
Its invalid RegExp literal and phantom activation-binding failures, the original
exit-1 preflight and the Cargo exit-101 build failure all remain in the
[portable focused proof](docs/rust-rewrite/focused-native-fb3a1235-20261015.json).
The proof retains raw inventories, transcripts, Cargo producers, exact IDs and
clean source-owner equality. Verify it offline:

```sh
python3 -B docs/rust-rewrite/verify-focused-native-fb3a1235-20261015.py \
  docs/rust-rewrite/focused-native-fb3a1235-20261015.json
```

The unchanged original RegExp fixture also passed both script modes through
the Wasm engine in separate diagnostics. Fresh ordinary CLI, full workspace,
historical 5,365-ID retention and full pinned Test262 verification remain
pending on the repaired source; this focused pass does not close those gates.

## Closed partial Test262 replays — 2026-10-15

Both frozen `8e9a8c82` Test262 replays are now closed, with their original
interrupted results preserved. The full-suite prefix contains **67/748 complete
nodes and 10,903 executions: 10,898 Success, 4 NotImplemented, 1 Bug, zero Crash**.
The historical replay contains **1,600 completed native executions: 1,436
Success, 161 NotImplemented, 3 Bug, zero Crash**, plus two missing-input
infrastructure errors. All 1,405 earlier historical passes reached by that
partial run remain passing; the unrun IDs remain unrun.

The [portable closure proof](docs/rust-rewrite/incomplete-publication-closure-8e9a8c82-20261015.json)
preserves the raw failures, interruptions, complete node outputs and separate
source-byte closure proof. It requires **16,071 distinct prior passing IDs** in
the later repaired-source full verification. These are retention requirements,
not claims of new passes or complete suite coverage.

Verify it with the previously published handoff files in the same directory:

```sh
python3 -B docs/rust-rewrite/verify-incomplete-publication-closure-8e9a8c82-20261015.py \
  docs/rust-rewrite/incomplete-publication-closure-8e9a8c82-20261015.json
```

The RegExp source and labelled lexical scope repairs pass the focused native
checks above. Fresh repaired-source CLI, full workspace, historical retention
and full pinned Test262 checks remain pending. CONTINUE.md and tasks remain
until their complete gates pass.

## Cloud conformance worker CPUs — 2026-10-15

`limited_verification.py --cloud --cloud-worker-cpus auto` gives explicitly
requested conformance workers the inherited CPU set bounded by the machine's
CPU quota. Native compilation, Cargo and libtest defaults remain one. This
lets three selected workers use CPUs 0–2 while keeping `LILA_JOBS=1` and
`RAYON_NUM_THREADS=1`; the existing native-compilation mode keeps its current
behavior. Worker counts, deadlines, cache caps and feature flags are preserved.

All **35 focused tooling controls** pass, including seven new controls. An
actual cloud payload confirms three allowed CPUs, one compilation job, one
Cargo/libtest thread and unchanged feature selection. See the
[verified tooling checkpoint](docs/rust-rewrite/cloud-worker-affinity-checkpoint-20261015.json)
and [worker invocation](docs/rust-rewrite/batch-workflow.md#explicit-cloud-conformance-workers--2026-10-15).

The original `8e9a8c82` historical and full-suite prefix runs are now closed
as recorded above. Their scheduling periods remain part of the preserved
observations, not a uniform CPU benchmark. Repaired-source verification and
full task acceptance remain pending.

## Genuine ordinary CLI53 checkpoint — 2026-10-15

The normal default release CLI on compiler `8e9a8c82` completes all **53 cases
with Success**, with **zero NotImplemented, Bug, Crash or InfrastructureError**.
All **43 preceding D2 successes** and **29 earlier f5 successes** are retained.
Both previously unsupported GeneratorFunction runtime modes now pass, along
with all eight selected ShadowRealm modes. Fixtures, modes and pins are unchanged.

The actual normal Cargo build takes **647.103 seconds**. The native 53-case
invocation exits zero in **47.056 seconds**; its source-bound outer invocation
also exits zero in **53.976 seconds**. Worker count remains one, with one thread
and a 60,000 ms deadline. The distinct replay worker matches the ordinary binary
byte-for-byte and uses the normal static runtime producer.

The [portable CLI53 checkpoint](docs/rust-rewrite/ordinary-cli-8e9a8c82-probe53-20261015.json)
contains exact execution IDs, fixtures, actual Cargo/static-runtime provenance,
prior-pass retention and lossless raw evidence. Verify it entirely offline:

```sh
python3 docs/rust-rewrite/verify-ordinary-cli53-8e9a8c82-20261015.py \
  docs/rust-rewrite/ordinary-cli-8e9a8c82-probe53-20261015.json
```

A prepared verifier compared the upstream Test262 commit with the content-tree
ID native snapshots record. Its original failure remains embedded. A separate
read-only audit verifies the committed commit-to-tree mapping and the existing
native results; no compiler build, replay, pin or outcome was changed.

The [matched twenty-case timing comparison](docs/rust-rewrite/runtime-retention-matched20-8e9a8c82-20261015.json)
passes all twenty cases in both runs: **2.806 seconds before, 2.857 seconds
after**. The old reused worker already retained runtime code, so this sample
establishes no meaningful retention speed gain. Formatting, module boundaries,
product/oracle dependency separation, host ABI and the refreshed shortcut
inventory checks pass. The inventory refresh changes display line numbers only;
selector fingerprints and classifications remain unchanged.

The frozen `8e9a8c82` historical and full-suite prefix runs are closed and
incomplete, as recorded above. Fresh historical 5,365-ID retention and full
pinned 102,994-execution Test262 verification remain pending on the repaired
source. `CONTINUE.md` and `tasks/` remain open.

## Complete all-features workspace checkpoint — 2026-10-15

Compiler source `8e9a8c82` passes all **108 all-features workspace scopes**:
**9,254 passed, zero failed, zero ignored**. Exact retention keeps every one
of the **9,220 preceding all-features passing IDs**. All **104 required controls**
pass. Four default-only oracle-refusal controls also pass in the same-source
default checkpoint and are accounted for separately.

The single complete run exits zero in **4153.918 seconds**, without storage
deferrals or continuations. Source, native test bodies, deadlines and cache
limits are unchanged. The complete 855-test CLI inventory, differential worker,
generated campaign and standalone fake-publisher checks pass.

The [portable all-features checkpoint](docs/rust-rewrite/all-features-workspace-8e9a8c82-native-checkpoint-20261015.json)
contains exact qualified IDs, Cargo/runtime provenance, source bindings,
lossless raw transcripts and the same-source default proof. Verify it offline:

```sh
python3 docs/rust-rewrite/verify-workspace-8e9a8c82-20261015.py \
  docs/rust-rewrite/all-features-workspace-8e9a8c82-native-checkpoint-20261015.json \
  --audit-sha256 7f93713f95e403bc5fefcfdae49d1cad289e815ff454f7b3f8640b9835e770e9
```

Together, the complete default and all-features checkpoints retain all prior
passes and add 34 controls in each mode. The genuine ordinary CLI53 checkpoint
is recorded above. Historical 5,365-ID retention, full pinned Test262 and task
acceptance remain pending.
`CONTINUE.md` and `tasks/` remain open.

## Complete default workspace checkpoint — 2026-10-15

Compiler source `8e9a8c82` passes all **107 default workspace scopes**:
**9,161 passed, zero failed, zero ignored**. Exact retention keeps every one
of the **9,127 preceding passing IDs**. All **104 required controls** pass,
including the 34 new ShadowRealm, worker-retention and prepared-constructor
controls. The unchanged GeneratorFunction runtime fixture executes successfully
in both modes. All **539 Python tooling tests** also pass.

The single complete workspace run exits zero in **3524.462 seconds**, with
unchanged source, test bodies, deadlines and cache limits. The standalone fake
publisher passes its complete 191-execution measurement and artifact checks.
Earlier failed focused attempts remain separate from this accepted run.

The [portable default checkpoint](docs/rust-rewrite/default-workspace-8e9a8c82-native-checkpoint-20261015.json)
preserves exact qualified IDs, Cargo/runtime provenance, source bindings and
lossless raw transcripts. Verify it offline with:

```sh
python3 docs/rust-rewrite/verify-workspace-8e9a8c82-20261015.py \
  docs/rust-rewrite/default-workspace-8e9a8c82-native-checkpoint-20261015.json \
  --audit-sha256 d2183481edd71c532beca6a2a5e00ab14999823c4f802146cc2cd8c300c3d017
```

The source adds bounded ShadowRealm template/String/filter and returned-callable
hints, runtime-only worker module retention, and prepared Function global
constructor grammar hints. Runtime callees, conversions, shadowing and Realms
remain live; supplemental source admission remains resource-free. See the
[prepared-constructor contract](docs/rust-rewrite/contracts/prepared-function-global-constructor-hints.md).
The prior twenty-case warm-worker sample already retained runtime code; no
retention speed gain is claimed before the matched after measurement.

The complete 108-scope all-features run also passes on the same frozen source.
The genuine ordinary CLI53 checkpoint is recorded above. Historical 5,365-ID
retention, full pinned Test262 and task acceptance remain pending. `CONTINUE.md` and `tasks/` remain open.

## Genuine ordinary CLI subclass checkpoint — 2026-10-14

The normal default release CLI on compiler `d2f4bc51` completes the unchanged
45-case replay with **43 Success, 2 NotImplemented, zero Bug, Crash or
InfrastructureError**. All 29 preceding successes are retained, and all 14
unchanged pinned Function/GeneratorFunction subclass modes now pass. The two
GeneratorFunction runtime fixture modes retain their explicit capability
rejections and the actual replay exit remains 1.

The [portable ordinary CLI checkpoint](docs/rust-rewrite/ordinary-cli-d2-probe45-20261014.json)
contains the exact execution IDs, unchanged fixtures, actual normal Cargo
producers, static runtime bundle and lossless raw replay evidence. The distinct
worker matches the ordinary binary byte-for-byte. The genuine normal Cargo build
took 627.657 seconds. Worker count, one thread and the 60,000 ms deadline retain
their existing policy. This diagnostic is separate from full historical and
pinned Test262 acceptance; CONTINUE.md and tasks remain open.

## Complete all-features workspace checkpoint — 2026-10-14

Compiler source `d2f4bc51` passes the complete 108-scope all-features
workspace: **9,220 passed, zero failed, zero ignored**. Exact retention keeps
all 9,116 eligible preceding passing IDs; four default-only oracle-refusal
controls pass in the same-source default checkpoint and are accounted for
separately. All 70 required controls pass. The complete 855-test CLI inventory
and actual differential worker, campaign and corpus checks are included.

Storage preflights deferred 32 scopes in the original process. Its exit 1 and
original states remain preserved. Two exact same-source continuations pass all
32 complete scopes, producing the accepted logical union; this does not claim
that the original process passed. Source, native test bodies, deadlines and
cache limits are unchanged.

The [portable all-features checkpoint](docs/rust-rewrite/all-features-workspace-d2-native-checkpoint-20261014.json)
contains exact native IDs, Cargo/runtime provenance, raw transcripts and the
same-source default proof. The ordinary CLI checkpoint is recorded above.
Historical 5,365-ID retention, full pinned Test262 and task acceptance remain open.

## Preserve verification transcripts across runs — 2026-10-14

Native partition logs now include the complete resolved run directory identity
and partition index. Default and all-features runs with identical target
basenames therefore keep separate transcripts. The old label regression
reproduces its overwrite; all nine Python partition controls pass with the fix.
Native test membership, bodies, scheduling, CPUs, threads, deadlines and cache
limits retain their existing policy. The completed D2 runs preserve their frozen-source evidence; the next batch
uses the corrected labels.

## Complete default workspace checkpoint — 2026-10-14

Compiler source `d2f4bc51` passes all 107 default workspace scopes:
**9,127 passed, zero failed, zero ignored**. Exact retention keeps all
**9,120/9,120** preceding passes and all 6,399 older passes; all 70
required controls pass. The complete 848-test CLI inventory is included. The
single actual native run exits zero in 3616.979 seconds, with unchanged
source, bodies, deadlines and cache caps. All 538 current Python tooling tests
also pass.

The [portable default checkpoint](docs/rust-rewrite/default-workspace-d2-native-checkpoint-20261014.json)
contains exact IDs, actual Cargo/runtime provenance and lossless raw evidence.
Raw streams retain their original path and exact SHA so an earlier watcher
filename reused by a later run cannot erase the older evidence. The original
failed reader and earlier RED native attempts remain separate from this passing
run. Retired executable bytes are recorded without claiming they remain present.

| Complete native scope | Current four-partition wall time | Previous serial native time |
| --- | ---: | ---: |
| Engine, 889 tests | 355.132 s | 611.96 s |
| Async, 248 tests | 265.105 s | 414.11 s |
| Intl, 293 tests | 302.276 s | 1,222.69 s |

These are consecutive observations with different source/cache state, rather
than a controlled benchmark. Python tooling briefly overlapped async. Each native
process retains one CPU and one test thread; the cloud quota remains four CPUs
and Cargo stays serial.

The complete all-features and ordinary CLI checkpoints are recorded above.
Historical 5,365-ID retention, full pinned Test262 and task closure remain
required. CONTINUE.md and tasks remain until those gates pass.

## Earlier checkpoints

## Verified finite Function subclasses — 2026-10-14

2026-10-14: The source-bound finite Function-subclass focused checkpoint at `d2f4bc51` passed **40 native controls** (33 IR + 7 Engine), including all seven new controls and the unchanged Function/GeneratorFunction subclass fixtures in both modes (14 embedded assertions). Raw native times were 39.50 s and 15.56 s; the closed outer run took 574.67 s including builds. The portable checkpoint preserves the earlier compile-101 and cc35 5-PASS/2-FAIL attempts as separate RED evidence. Full default/all-features workspace verification and ordinary CLI subclass replay remain pending.

See [the portable focused proof](docs/rust-rewrite/focused-finite-subclass-d2f4bc51-20261014.json) for exact IDs, source,
producer hashes and lossless raw evidence. The current complete default run is
active with whole-inventory partitions for Engine, async, Intl and CLI. The
all-features 108-scope plan passed its inventory checks. Full 5,365-ID retention,
pinned Test262 and task closure remain open; CONTINUE.md and tasks are retained.

## Finite synthetic Function subclasses — 2026-10-14

The next source batch prepares bounded, proven Function and GeneratorFunction
bodies through classes with implicit constructors, including class expressions
and synthetic subclass chains. Actual callee lookup, arguments, conversions,
super constructors, realms and new-target prototypes remain live at runtime.
Explicit constructors and unknown dynamic sources retain the existing capability
boundary. Three IR and four engine controls cover these semantics and the
14 unchanged pinned subclass modes. Rust compilation and native verification
remain pending; the CLI checkpoint below records their actual earlier NI outcomes.

## Ordinary CLI repair checkpoint — 2026-10-14

The genuine default release CLI on compiler `f5e3f706` completed the
unchanged 45-case diagnostic replay: **29 Success, 16 NotImplemented, zero Bug,
Crash or InfrastructureError**. The actual replay exit remains 1. The original
21-case projection has 19 successes and 2 explicit dynamic-source gaps.

All 10 RegExp source modes, 2 hashbang Function modes and 2 eval-import modes
pass; the latter groups overlap the original projection. All 14 subclass modes
remain NotImplemented and are the next repair batch. Fixtures, modes, one worker,
one thread and the 60,000 ms deadline are unchanged. The worker is a distinct,
byte-identical copy of the ordinary Cargo binary and uses its static runtime
bundle. The normal build and identity checks completed in 287.397 seconds.

See [the exact CLI checkpoint](docs/rust-rewrite/ordinary-cli-f5-probe45-20261014.json) for every ID, prior transition,
reason, selected Cargo producers and embedded raw replay evidence. Full historical
and pinned Test262 retention, all-features verification and task closure remain
open. CONTINUE.md and tasks are retained.

## Complete default workspace — 2026-10-14

Compiler source `f5e3f706` passes all 107 default workspace scopes: **9,120
passed, zero failed, zero ignored**. Exact comparison retains **6,399/6,399**
earlier passing IDs and all 63 required controls pass. All 848 CLI tests pass.
The 13 storage-deferred frontend scopes passed in their source-bound continuation.

Temporary disk exhaustion caused the original run to fail 30 tests in Engine
and async. Their complete same-source reruns pass all 1,137 tests, including every
original failed ID and earlier pass. The checkpoint retains the original RED
process and its verdicts separately from the successful logical union; it does
not claim that original process passed. Retired executable bytes are recorded
by their observed hashes without claiming that they remain present or archived.

See [the portable native checkpoint](docs/rust-rewrite/default-workspace-f5-native-checkpoint-20261014.json)
for exact IDs, raw inventories/transcripts, actual Cargo and runtime-image
provenance, full passing retention and original failure evidence. The normal
CLI build and 45-case diagnostic replay are closed in the checkpoint above. All-features, full
historical and pinned Test262 verification, and individual task closure remain
pending. CONTINUE.md and tasks are retained.


## Verification storage and native partitions — 2026-10-14

Workspace verification now checks the physical filesystems used by temporary
files, compilation caches and outputs, including symlinked cache tiers. It keeps
the two-GiB build reserve and requires 256 MiB of scratch headroom per active
and scheduled native process. Insufficient space records a deferred scope.

Use `--partition-native-target lila-engine:aot_intl` to run that complete
inventory in disjoint serial workers. Partitions remain idle-exclusive, retain
every qualified ID and use the existing CPU, thread, cache and watcher limits.
The original 293-test Intl scope passed in 1,222.69 seconds of native execution;
four-way execution has not been measured. All 28 Python tooling controls pass.
Refresh them with:

```sh
python3 -B -m unittest discover -s scripts/tests -p 'test_workspace_test_*.py' -v
```

The compiler-source f5e3f706 default run and its frontend continuation cover
all 107 scopes: 9,090 native tests passed and 30 failed after temporary storage
filled. All 848 CLI tests passed. Complete Engine and async reruns are active;
whole-workspace acceptance, all-features verification, ordinary CLI replay and
task closure remain pending. CONTINUE.md and tasks are retained.


## Reduce production rebuilds after test edits — 2026-10-14

The Engine and runtime-image compiler fingerprints now exclude explicit test
trees and six modules compiled only under `cfg(test)`. The same input policy
controls Cargo rebuild watches. Production resource directories still detect
added and removed files; source declarations, manifests, build scripts and
vendor inputs remain covered. Native compiler identity and cache limits retain
their existing policy. See [input coverage](docs/rust-rewrite/aot-cache-fingerprint.md).

All 12 standalone fingerprint controls pass, including the five new lifecycle
and production-invalidation controls. This addresses observed production rebuilds
after two IR test edits. All Intl outputs reproduce unchanged and the 83
associated generator checks pass. Current-source Cargo producers and the full
suites remain pending; no elapsed-time improvement is claimed yet.


## Verified focused repairs — 2026-10-14

The nine failures from the complete default workspace run now pass, including
the previously passing `with` lookup regression. All 23 new controls pass. The
focused logical union contains 2,294 passing tests and no failures or ignores;
this is not a complete workspace result. The original failed receipts remain
recorded alongside the successful correction runs.

Whole-workspace release type checking passed on compiler source `2f10f5a07`.
Native verification then corrected three test expectations without changing
product source or pinned fixtures: two controls now assert the actual spec GetV
nodes, and the eval-import fixture asserts its final Promise object completion.
The corrected checks retain live lookup, arguments, async output, source modes
and original deadlines. See [exact focused evidence](docs/rust-rewrite/focused-workspace-repairs-20261014.json).

The complete default and all-features suites, ordinary CLI replay, full Test262
passing retention and task acceptance remain pending. CONTINUE.md and tasks
remain until those acceptance requirements are met.


## Workspace corrections and finite RegExp source — 2026-10-14

The complete default workspace inventory on source `8429f1805` finished with
9,083 passing and nine failing native tests across 107 targets. Exact retention
found one previously passing IR test regressed by source discovery through a
`with` environment. The seven storage-deferred targets subsequently passed;
the original failed run and its continuation remain separate evidence.

The next source batch repairs that reference lookup and prepared Script import
dispatchers that captured a private helper. It also adds finite RegExp `.source`
discovery using the runtime's UTF-16 escaping rules, keeps dynamic source gaps
explicit, and fixes inspection of expressions in default parameters. Inspection
now reports typed environment identifier reads and calls, including live named
bindings. Structural controls follow the typed replacement-piece collector.

All 15 Intl generator APIs reproduce their 44 outputs without changes; the
83 affected Python checks, formatting and module boundaries pass. These
corrections have not yet passed Rust compilation or native verification. The nine original failures remain open until their exact
IDs pass on this source. Full workspace, ordinary CLI, Test262 retention and
task acceptance remain required; CONTINUE.md and tasks are retained.


## Open workspace failures at source 8429 — 2026-10-14

Compiled source `8429f18059dce2277c229e44c61b55c0a62d178e`; documentation parent `2766245aeba5942cbeeb58661732ac2e7e60700a`. The full **107-target workspace run remains active and incomplete**. Four closed RED native scopes own **2,784 passed / 7 failed / 0 ignored** outcomes:

| Closed scope | Passed | Failed |
| --- | ---: | ---: |
| lila_ir | 1,512 | 1 |
| lila_ir_integration | 706 | 1 |
| structure_builtins | 164 | 2 |
| aot_realm_modules | 402 | 3 |

One previously passing 0a18 IR integration ID now fails: `ir_cases::prepared_dynamic_function::global_source_target_discovery_does_not_cross_shadowed_or_dynamic_references`. The other owned failures are one new negative control with a wrong contract, two stale AOT structure assertions and three new direct-eval import null traps, as classified in Root's review. Private repair packets remain unapplied and uncompiled; the module repair is still in preparation. No fix is verified.

This records failure ownership, not workspace acceptance or complete passing retention. Whole workspace, full pinned Test262, ordinary CLI, current-source historical 5,365 replay and task closure remain open. Earlier partial passes, historical replay counts and all prior owners remain unchanged.

Exact IDs/verdicts, byte-identical packed raw inventories/transcripts, actual Cargo artifacts and source guards are in the [portable failure checkpoint](docs/rust-rewrite/workspace8429-open-failures-20261014.json). The moving workspace owner is not sealed as closed.

### Previous checkpoints and measurements

## Partial native checkpoint at source 8429 — 2026-10-14

Compiled source and documentation parent: `8429f18059dce2277c229e44c61b55c0a62d178e`. The two completed default-profile libraries passed **1,314 exact native IDs**: Test262 library 425 passed (54.30s) and Engine library 889 passed (598.36s), with zero failures or ignored outcomes. All **1,309/1,309 prior passing IDs** were retained, with 0 regressions and 5 new controls. The 13 focused catalogue tests are included in this full two-library count.

The sibling workspace Cargo cohort remains **RED**: its AOT libtest link ended with signal 7 (Bus error) during Root-observed storage exhaustion; Cargo recorded `build-finished success:false`, inner build exit 101, outer exit 1 and 1239.532s wall time. Running its already completed sibling artifacts establishes this partial checkpoint, not whole-workspace acceptance.

Current default type checking passed in 362.998s; the original `63c3c67b94669fc60e76cc1b143bfb4f84bbd2a7` type error (exit 101) is preserved. The actual Python tool result reported 514 passed; it has no portable raw transcript. Genuine Intl verification retained 1,398 source inputs, 15 API steps and 44 byte-identical outputs, with 83 affected Python tests passed. These are separate checks.

Whole workspace, full pinned Test262, current-source historical 5,365 replay, ordinary CLI and task closure remain open. The older 0a18 historical replay and all prior counts/history remain unchanged. The next private RegExp patch is unapplied and unverified.

Exact verdicts, added IDs, retention, source/default-profile Cargo artifacts, binaries, inventories, raw transcripts, failed-cohort and type/Intl provenance are in the [portable partial checkpoint](docs/rust-rewrite/joined-source-8429-partial-native-checkpoint-20261014.json).

### Previous checkpoints and measurements

## Integrated repair batch — 2026-10-14

The strict callable-header, finite Function-constructor, prepared direct-eval
import and numeric-call emission repairs are integrated. Direct-eval imports
retain their Script referrer and use the existing module jobs; unproved dynamic
source remains an explicit capability gap. Numeric call rows preserve live
lookup, argument evaluation and completion handling. Release workspace testing
now uses the selected shared Cargo target and runtime image contract.

All generated Intl outputs reproduce without changes from the combined source,
and the associated generator checks pass. Integrated Rust compilation, native
regressions, the complete workspace and Test262 acceptance are still pending.
The previous closed checkpoint and every remaining failure follow below;
CONTINUE.md and tasks remain until their acceptance requirements are met.

## Closed historical replay and private V3 diagnostics — 2026-10-14

Compiled source `0a18df332ca26b193b8e13bf09c104ff08fdb70c`; documentation publication parent `a86a8b2ba451145491157891a30c68a8af8173d6`. The historical replay has closed. Coverage and exact passing retention are verified; conformance remains RED.

Compiled revision `0a18df332ca26b193b8e13bf09c104ff08fdb70c` completed the unchanged **5,365-ID historical replay**: 5,142 Success, 217 NotImplemented, 0 Crash, 4 Bug, 2 InfrastructureError.

Coverage is complete; historical conformance is RED. Actual replay exit is **2**; the native batch exit is **1**. Both absent DurationFormat modes remain InfrastructureError, with their literal native diagnostics, exits and transcript hashes retained.

`aee5365` passing retention: **4,849/4,849**; 0 regressions, 293 repaired IDs, 0 Bug-to-Crash transitions. Every exact ID/transition is retained.

`3bc254` passing retention: **229/229**; 0 regressions, 23 repaired IDs, 0 Bug-to-Crash transitions. Every exact ID/transition is retained.

`115105` passing retention: **61/61**; 0 regressions, 42 repaired IDs, 0 Bug-to-Crash transitions. Every exact ID/transition is retained.

This checkpoint proves the recorded historical input and exact passing retention. It does not establish full pinned Test262, ordinary CLI, whole workspace, independent-host, oracle, task closure or idle performance acceptance. Native execution remains the recorded default Wasm-AOT worker graph with its runtime requirements, one worker and unchanged 60,000 ms deadline.

The original AE replay's observed exit 0 despite red outcomes is preserved as historical evidence; it is not reused as the current replay exit.

The distinct prior-success union retains 5,078/5,078 IDs, with 0 regressions. Overlapping baseline successes are counted once in this union.

Refresh from the same closed evidence with a fresh export directory (no native execution):

```sh
historical_export_dir=$(mktemp -d /workspace/porffor/target/historical5365-export-refresh.XXXXXX)
python3 -B /workspace/porffor/target/future-verification-drivers-20261012/export-current-historical5365-checkpoint-20261014.py --owner /workspace/porffor/target/historical-native-replay-0a18df332-20261014-retention/receipt.json --owner-sha256 427a50b760853ce07906b69cf2fbcde8045689097c1eb1d7306001203e436ca3 --head 0a18df332ca26b193b8e13bf09c104ff08fdb70c --output "$historical_export_dir/checkpoint.json" --status "$historical_export_dir/status.md"
```

Against AE, 244 Bug → Success, 8 Crash → Success and 41 NotImplemented → Success transitions produced 293 additional successes. The 2 NotImplemented → Bug transitions remain red, and all 4 current Bug IDs remain explicit. Missing modes remain InfrastructureError.

Private numeric-call V3 remains **unapplied**. Its literal-source diagnostic passed **8 controls** and **1 standalone original-case profile**; registered-scope/full-source acceptance remains pending. V2's two REDs are preserved. V3 adds typed EnvironmentReference call admission and explicitly changes the rich-program pre-execution size gate from 65,536 to 131,072 bytes; the stricter AOT body/growth controls are byte-identical.

The original-case V3 wrapper wall was 54.6408s: compilation 27.841727s (preparation 26.940313s) and runtime API call 26.621184s (module 0.534387s; execution/completion 26.076455s). P remained 373,800 bytes, largest body 51,952; unchanged R was 40,315,118 bytes, completion `Normal(Undefined)`. The literal lowering trace records prepass 15.746768s and final body 10.959529s within script lowering 26.922378s (analysis 0.001574s). These identify measured dominant phases; they establish no proposed algorithm, general speedup, idle performance or Test262 acceptance.

The separate accepted native checkpoint remains **6,399 passed**. Ordinary CLI, full workspace, the full pinned suite and current task closure remain open. All prior owners and status are preserved below.

The [portable historical checkpoint](docs/rust-rewrite/historical5365-source-0a18-checkpoint-20261014.json) exposes every exact current ID/verdict, all transitions and remaining failures. Its packed evidence decodes to the **byte-identical original exporter JSON** with all source, worker, raw transcript, snapshot, metadata and historical provenance; its checksum and lossless decoding contract are included.

### Previous checkpoints and measurements

## Private numeric-call Unicode phase comparison — 2026-10-14

Compiled source `0a18df332ca26b193b8e13bf09c104ff08fdb70c` remains frozen; documentation publication parent is `d2716d1a3ed2230e9bd5abd9efe155ccba9de8ec`. The four-file private AOT candidate is unapplied and has separate patch, library and executable identities. Baseline revision metadata identifies R compatibility, not acceptance of this compiler candidate.

The unchanged original Unicode ignoreCase case (299,825 original bytes; 305,744 materialized bytes) recorded **59.03s native libtest elapsed** (59.108s wrapper wall), **31.301s compilation** and **27.628s runtime**. Compilation preparation was 30.352s, emission 0.940s and validation 0.004466s. Its literal trace records parsing 0.023183s and lowering 30.328s inside preparation; the API lowering field (0.001414s) is a separate stage. Runtime module work was 0.517346s (program 0.402977s; R 0.114333s), with execution/completion 27.104s.

| Captured diagnostic | Baseline | Private candidate |
| --- | ---: | ---: |
| Compilation | 26.512s | 31.301s |
| Runtime module work | 123.730s | 0.517s |
| Execution/completion | 45.497s | 27.104s |
| Program bytes / largest body | 1,962,259 / 1,711,700 | 373,800 / 51,952 |
| Sampled maximum process RSS | 2,852,282,368 bytes | 456,359,936 bytes |

Both profiles used the identical 40,315,118-byte R payload and completed `Normal(Undefined)` with module-cache `Miss`. The wrapper-wall ratio is **3.32×** for this one captured case; separate runs and shared load do not establish a general benchmark speedup or idle performance. RSS values are samples, not true peaks. Normal completion is not Test262 acceptance.

The independent controls remain **6 passed / 2 FAILED**: bodies of 67,317 and 820,896 bytes exceeded the unchanged 65,536-byte gates before JavaScript execution. Their incomplete aggregate owner and original failed static-DSO build remain preserved; the corrected shared-DSO build and standalone profile closed successfully. No semantic failure, merge or full-gate acceptance is inferred. The published native checkpoint remains 6,399 passed; the full historical 5,365 replay is still active and retention remains pending.

Exact owner, raw-event/log, command, source, private artifact, R and archive hashes are in the [portable comparison](docs/rust-rewrite/private-numeric-call-profile-20261014.json); earlier measurements and status follow unchanged.

## Measured verification and private Front foundation — 2026-10-14

Compiled source `0a18df332ca26b193b8e13bf09c104ff08fdb70c` remains frozen; documentation publication parent is `8911e9b9598f7969fad2919b8a7b76e74176384d`.

The original sloppy-script Unicode ignoreCase public-API diagnostic recorded **26.512s compilation** (preparation 25.567s, lowering 0.001031s, emission 0.924s, validation 0.017s) and **169.233s runtime** (module 123.730s, execution/completion 45.497s). Program bytes: 1,962,259; largest body: 1,711,700; 35 defined functions, 1089 function imports. Actual cache outcome: `Miss`; completion: `Normal(Undefined)`. This is private-cache cold evidence under captured shared load, with no warm comparison, speedup, idle-performance, replay-phase or Test262 acceptance claim.

The unapplied private joined AST → Parser → Front candidate passed **11 proposed-behavior compatibility controls**, rejected all **13 original invalid inputs**, accepted **5 valid controls** and the outer fixture. Its recorded command wall sum is 18.904s. The focused strict-header defect repair is confirmed; Both Annex candidates remain held/unapplied under the current pin/host contract; their controls do not prove normative correctness. Product integration, full IR and native verification remain pending.

The existing affected-native checkpoint remains 6,399 passed, 0 failed, 0 ignored across 16 targets. Owner wall: 5010.002s; recorded builds 1729.204s, partition controllers 3010.811s, types 65.656s, unattributed remainder 204.330s. Warm worker preparation recorded 1.0056s Cargo-library work and 2.0075s direct leaf linking. The enclosing supervisor-command walls sum 3039.004s; native child-process walls sum 2911.161s. These are distinct, nested observations, not additive phases or per-crate/idle measurements.

The historical 5,365-ID retention replay is ACTIVE and its complete outcome/retention remain pending. Ordinary CLI, full workspace, current pinned Test262 and task closure remain open.

See [measured phases, resources and provenance](docs/rust-rewrite/verification-phase-measurements-20261014.json); the existing [complete affected-native evidence](docs/rust-rewrite/historical-source-0a18-native-checkpoint-20261014.json) remains unchanged.

Reprepare from these immutable owners while the recorded compiler HEADs and publication parent remain frozen, using a fresh output name (no native execution):

```sh
python3 -B /workspace/porffor/target/future-verification-drivers-20261012/prepare-verification-phase-measurements-publication-20261014-v2.py --output-dir /workspace/porffor/target/verification-phase-measurements-publication-REFRESH-UNIQUE
```

The user authorizes continuous pushes to main and faster tooling. Root writes
code; Luna agents do read-only chores/exploration. Finish actual task acceptance
and the full suite before deleting this file or tasks. Retain every failure,
unsupported/missing-input ID and previous passing ID.

## Complete affected native checkpoint — 2026-10-14

`0a18df332ca26b193b8e13bf09c104ff08fdb70c` affected native checkpoint: **GREEN**, complete across 16 targets and 6399 IDs: 6399 passed, 0 failed, 0 ignored.

All 6396 closed c1 and 6390 historical 750 IDs remain. Passing retention: c1 6394/6394; historical 750 6380/6380. Prior REDs passing exact focus and complete union: c1 2/2; historical 750 10/10. Added controls: 3.

Native scope uses validated exact focus and disjoint remaining processes. Actual partition streams own the complete union; process initialization, cache warming and order may differ from one unfiltered process.

Fresh default and standalone Intl type cohorts passed. Genuine Intl generation: 1397 inputs, 15 APIs, 44 outputs, 0 changed identity files, 83 associated Python checks.

Full Python owner: 514 discovered IDs, {'Success': 514}.

Scope remains affected native, recorded source generation and supplied Python tooling. Ordinary CLI, full workspace, current pinned Test262, replay, original 5,365 retention and idle performance require separate evidence.

Build seconds below are recorded command wall times. Captured Cargo HTML files are classified as collector observations; duplicate neighbor payloads remain explicit and their units are not attributed to the capturing invocation.

| Target | Passed | Failed | Ignored | Build s | Native processes |
| --- | ---: | ---: | ---: | ---: | ---: |
| `lila_front` | 177 | 0 | 0 | 24.041 | 2 |
| `dynamic_function_preparation` | 6 | 0 | 0 | 7.017 | 2 |
| `regexp_legacy_astral_ranges` | 6 | 0 | 0 | 16.028 | 2 |
| `module_source_ranges` | 6 | 0 | 0 | 16.020 | 2 |
| `lila_ir_integration` | 703 | 0 | 0 | 189.314 | 2 |
| `lila_aot_wasm` | 340 | 0 | 0 | 284.757 | 2 |
| `runtime_link` | 29 | 0 | 0 | 147.963 | 2 |
| `aot_realm_modules` | 397 | 0 | 0 | 498.500 | 2 |
| `aot_language` | 295 | 0 | 0 | 7.043 | 2 |
| `aot_gc_entries` | 164 | 0 | 0 | 4.048 | 2 |
| `aot_builtins` | 523 | 0 | 0 | 9.083 | 2 |
| `lila_engine` | 888 | 0 | 0 | 72.584 | 2 |
| `lila_test262` | 421 | 0 | 0 | 117.507 | 2 |
| `aot_intl` | 293 | 0 | 0 | 9.096 | 2 |
| `lila_ir` | 1511 | 0 | 0 | 211.482 | 2 |
| `lila_intl` | 640 | 0 | 0 | 114.721 | 2 |

Refresh this source-bound export after its owner closes:

```bash
export_dir="$(mktemp -d /workspace/porffor/target/native-successor-status-0a18df332-20261014-XXXXXX)"
python3 -B /workspace/porffor/target/future-verification-drivers-20261012/export-partitioned-native-successor-checkpoint-20261014.py --owner /workspace/porffor/target/historical-next-partitioned-successor-0a18df332-20261014/receipt.json --owner-sha256 619420748d42c16965288d8e30cfe27b655369daa7ef78afd6b0bbd676e55501 --head 0a18df332ca26b193b8e13bf09c104ff08fdb70c --python-owner /workspace/porffor/target/historical-source-python-0a18df332-20261014/receipt.json --python-owner-sha256 1ae3ae196cf03452b5df95d79ba5244012c487e7a2f895c32765f55cb28560a0 --output "$export_dir/checkpoint.json" --status "$export_dir/status.md"
```

See [every source-bound verdict, retained ID, archive and measured observation](docs/rust-rewrite/historical-source-0a18-native-checkpoint-20261014.json).

The compiled revision is `0a18df332ca26b193b8e13bf09c104ff08fdb70c`. A later docs-only publication commit records this evidence; it is not the compiled revision. Next, Original and Canonical remain at their recorded compiler HEADs. The same-head CLI worker preparation is complete; the historical 5,365-ID retention owner is RUNNING, and its complete outcome and passing retention remain pending. Ordinary CLI, historical 5,365 retention, full workspace, current pinned Test262 and task closure remain open. CONTINUE and task files remain.

## Previous c1 native checkpoint and remaining repairs — 2026-10-14

`c1fecbed30757715e199e94b3f42e68e4ab38e84` affected native checkpoint: **RED**, complete across 16 targets and 6396 IDs: 6394 passed, 2 failed, 0 ignored.

All 6390 prior `750c4d74c` IDs remain present, with 6 added controls (6 passing in focus and complete runs). The complete prior owner's passing set retains 6379/6380; 9/10 old REDs pass both runs. Historical union retention is 6382/6383.

The continuation adopts 10 literally unchanged closed scopes from the interrupted same-source owner, including 2 recorded failures, and executes its 6 storage-deferred scopes. The original owner remains incomplete; metadata and both type cohorts retain their original execution provenance.

Python discovery: 514 IDs, {'Success': 514}. Generated metadata: 15 APIs, 83 associated checks, 1397 bound source inputs, 30 changed identity files and 10 byte-identical semantic outputs. The generator owner records the repaired precommit 750 tree; its outputs are SHA-bound to this commit.

The two remaining failures have a prepared successor repair. Script preparation
uses converted source views while eager Realm module requests retain their
primary-source boundary; primary source hints keep all 256 existing slots.
The recipe control retains its original small input and adds a positive input
above the unchanged admission floor. The original GC fixture is unchanged.
Three new IR controls cover conversion routes, host-loading authority and the
full primary-source budget. These changes require fresh native verification.

See [every verdict, retained ID and measured phase](docs/rust-rewrite/historical-source-c1-native-checkpoint-20261014.json).
Refresh from the frozen compiler and actual owners with
`python3 -B target/future-verification-drivers-20261012/export-c1fecbed3-native-successor-checkpoint-20261014.py --output target/c1fecbed3-native-checkpoint-export-20261014/checkpoint.json --status target/c1fecbed3-native-checkpoint-export-20261014/status.md`.
The exporter verifies evidence; it does not rerun tests. Fresh successor
verification uses the sealed partitioned driver, retaining all 6,396 current IDs
and adding the three IR controls. Exact focused IDs and the disjoint remainder
run once each. Ordinary CLI, full workspace, all 5,365 historical executions,
current pinned Test262 and task acceptance remain open.

## Previous native checkpoint and completion repair — 2026-10-14

Actual compiler `750c4d74c` completes sixteen affected targets with
**6,380 pass, 10 fail and zero ignored**. Every prior inventory
ID is retained; 6,333 of 6,336 prior passes remain passing.
The three prior-pass regressions and all ten failures remain repair inputs.
The same-source Realm retry passes all395 tests; its original110 disk-exhaustion
failures remain in their separate incomplete owner. This checkpoint does not
establish full workspace or current-pin acceptance.

The prepared repair shares one checked source-activation completion exit and
keeps primary record values ahead of supplemental conversion views. It retains
source-depth rejection, caller/arguments restoration, proper-tail retirement and
saved generator state, with six new native controls. Corrected fixture inputs
preserve return ASI, exact UTF16 source text, actual Realm capabilities and
ECMA-402's accepted reversed Date ranges. All six corrected fixture observations
pass against unchanged750 product libraries in both modes. An isolated partial
AOT diagnostic validates all three emitted programs and reduces incremental
With body growth from12069 to7155 bytes under the unchanged9000 gate.
These focused observations do not replace complete successor verification.

See [every source-bound native verdict and timing](docs/rust-rewrite/historical-source-750-native-checkpoint-20261014.json).
The joined successor and genuinely regenerated Intl source identities require
fresh types, exact focused controls, all6390 native IDs, the historical5365
replay and remaining workspace/pin/task gates before acceptance. CONTINUE and
all task files remain until their actual closure.

## Previous native checkpoint and iteration measurements — 2026-10-14

Actual source `32c197968` completes ten affected targets: **5,216 pass,
16 fail, zero ignored**, retaining every one of 5,194 prior test IDs.
Of 5,193 prior passes, 5,182 remain passing and 11 regress; all are repair
inputs. Both previous Realm failures now pass. This is a complete red checkpoint,
not full workspace or pinned Test262 acceptance. The next batch repairs the
confirmed source, state and runtime problems before repeating exact retention.

Source-bound diagnostics measure the original DateTimeFormat case at about
1.2 seconds compiling and 58.2 seconds executing in sloppy mode; strict mode
hits its unchanged 60-second epoch deadline. Both large original String mapping
bodies complete normally in about 15.7 seconds with largest emitted bodies
68,736 and 72,581 bytes. These observations require supervised original-ID replay
before being counted as repairs. The diagnostic leaf links in 2.4–3.0 seconds
from actual optimized libraries, with zero Cargo commands. The native sweep was
active concurrently, so these measurements do not establish idle performance.

See [every verdict, regression, timing and compiler identity](docs/rust-rewrite/native-batch-and-profile-20261014.json).
Refresh the complete affected-target workflow with a fresh owner and exact
frozen source; preserve all 5,232 IDs and the sixteen red IDs. Full workspace,
ordinary CLI, current-pin replay, task stress and publication gates remain open.

## Complete historical comparison and prepared repair batch

Source-bound3bc completes all254 original Bug/Crash IDs: 229Success/23Bug/2Crash,
exit1, no missing/extra IDs, all61 previous repairs retained, zeroBug-to-Crash.
OriginalAE complete coverage is5365: 4849Success/246Bug/8Crash/260NotImplemented/
2InfrastructureError. Both DurationFormat missing modes and the adopted
runner exit-status mismatch remain explicit. See the generated
[complete comparison](docs/rust-rewrite/historical-complete-comparison-20261014.json).

Current3bc native red owner is complete2490:2488pass/2fail, allIDsretained.
One new recursive Function source observation is missing; one old Script module
fixture expectsSyntaxError for host-load instead of the correctTypeError.
Both are repaired in the prepared source batch, with original red owners kept.
The batch also adds semantic UTF16RegExp pattern lowering, checked passive String
root descriptors and compact pure fixed Array recipes. Every exposed Array is
fresh; runtime limits, conversion and source bytes remain unchanged. The appended
program helper ABI changesRtype-section bytes/cache keys; measure the actualmiss.
Both orphanText/dynamic-import IR modules now enter the explicit Cargo inventory.
No successor native pass is inferred. Complete types, focused controls, every
complete affected target and exact original-ID retention are required next.
Closed current native artifacts and actual34a producer bytes are reversibly
archived; closed old focus cache retired, preserving all red results and images.

## Latest verified historical repairs

Actual product `115279841f5e9ab858dbb9fa2d8a2b9c4f06df4d` repairs 61 of
105 observed historical Bug/Crash IDs, including six crashes. Complete exact
replay retains every input: 61 Success, 41 Bug, one Crash, two NotImplemented,
native exit 1, zero Bug-to-Crash. Unsupported is not counted as fixed.
All four complete affected Engine targets pass 2053/0/0 and retain all2037
prior passing IDs, with16 additions. Three targets use the product revision;
881 library tests use test-only successor
`d297a8dc5557f94234f65d607d7d878bbaae744e`. Its sole delta replaces a racy
registry try_lock assertion with a bounded handshake holding the registry
through receipt completion. Product broadcast is unchanged. The original
2052/1 red owner and its archived library bytes remain preserved.

All177 frontend,514 Python and26 AOT runtime-link controls pass on their
recorded revisions; AOT/IR complete test graph checks. RegExp candidate bodies
measure426/427 bytes for16/512 rows. See
[the portable checkpoint](docs/rust-rewrite/historical-repairs-checkpoint-20261014.json).
Do not infer latest whole-workspace, ordinary CLI, oracle or full-pin closure.

Source next owns the verified successor; canonicalAE and original4e remain
frozen for preserved original historical evidence. The next reviewed source
batch now includes Text modules, literal runtime import attributes, host-load
rejection, legacy Date space syntax, computed field names, with-write facts,
stable Math.atanh and finite source propagation keyed by analyzed environment
and storage identity. All eight private patches applied cleanly; formatting
and module guards precede actual type/native verification. No native verdict
for this successor is inferred. Keep runtime conversion and tuple admission
live; no runtime parser, shape exceptions or fake names.
Run unchanged repaired-ID retention plus every remaining historical ID.
Preserve images/DSOs/owners; retire only closed owned caches with manifests
before building. Closed cache retirement reclaimed2494963712allocated bytes;
reversible SHA-checked native archives reclaimed1630003200workspace bytes.
All workers, DSOs, results and unresolved IDs remain. The original runtime
producer's complete bytes are archived before owned generic-path overwrite.

## Actual accepted exact replay checkpoint

Compiler `aee6bc109927c580393d59c0cf6e532e30efd4b3` passes all2108 affected optimized native
tests and all508 canonical Python checks. Every prior native ID/pass is retained;
allthree earlier native failures are fixed. Actual36-ID per-ID/batch/real
interrupt/resume keeps34Success and both missing modes explicitly red. All
raw owners and actual compiler/image identities are sealed in
[the portable checkpoint](docs/rust-rewrite/exact-replay-checkpoint-20261013.json). Canonical remains frozen at this compiler
for the all5365 historical driver. Publication uses this source-only checkout;
do not change canonical HEAD while its replay is active. The new public staging
helper is source-only tooling, independently verified against the actual captured
image; it does not relabel the compiled source as the publication commit.

## Latest accepted compiler

afba1a07a8c653f1d5fba4b4dfc8f26f380a40cc completes private rooted UTF-16 String
construction and producer timing/Cargo-artifact tooling. All fifteen complete
optimized native scopes pass 5,455 / 0 fail / 0 ignored, retain all 5,449 previous
IDs and add six String controls. All 29 focus controls and 38 tooling methods
pass. Both actual old/current CLI workers pass the same 34 fromCharCode and 22
fromCodePoint IDs. Both original Syriac modes pass at 14.638/14.829s versus the
earlier 31.912/34.974s, with unchanged bytes/pins/60-second bounds. See
[the String checkpoint](docs/rust-rewrite/string-character-buffer-checkpoint-20261013.json).

Actual build 930.408s, three-scope addon 39.027s, native schedule 1,322.204s.
Producer phases: fingerprints6.521s, fresh emission/admission24.990s,
native237.061s, manifest4.308s, packaging1.230s. Changed R correctly misses module
reuse; stencil2,155 hits/two misses. Actual custom-build profile is opt-level0.
The complete producer output is preserved by the export owner before another
build. These concurrent observations are not idle timings. Raw owners:
target/string-buffer-{native-build,extra-build,focus,native,native-schedule,
worker,cli,checkpoint-export}-afba1a07a-20261013. Those earlier String commits are integrated and accepted. The existing next
checkout now owns the separate prepared historical repair batch.

781feba5254ae7a4084300534facd9a7d958df58 verifies bounded native R reuse and
native dependency invalidation. All eleven complete optimized native scopes
pass 4,825, retain all 4,821 earlier IDs and add four Engine controls. Full structure
passes 27, retains 25 and adds two fingerprint controls: total 4,852 / 0 fail / 0 ignored,
all 4,846 prior IDs retained, six added. All 23 focus controls pass. Actual staged
CLI retains Promise.try 30, four earlier repaired IDs and both Syriac modes.
See [the cache checkpoint](docs/rust-rewrite/native-runtime-cache-checkpoint-20261013.json).

Two sealed Cargo-producer replays preserve all three bundle files byte-for-byte.
Native phase falls from 676.217s to 1.986/2.032s; total warm producers 41.410/42.037s.
No-run build 1,284.782s; full eleven-scope schedule 1,317.767s. These are concurrent
cloud observations, not idle/general build timing. Producer Cargo profile is 0;
do not infer optimization from the release target. Raw owners are
target/native-cache-{native-build-source,native,native-schedule,structure,
focus-owned,worker,cli}-781feba52-20261013 and
target/native-core-warm-artifact-781feba52-20261013. Setup failures and their
successful fresh successors remain separate.

The preceding accepted semantic compiler,
604a6f9aafb92df7c8bb1ec04002aa8f29397039, completes all 4,821 affected optimized
native tests, retaining all 4,817 previous passes and adding four paired
Promise.try controls. Both corrected older controls retain every input.

Promise.try 30 has 30 Success: four Bugs eliminated, all 26 original passes
retained. Iterator1308 keeps all 1,308 Success. Function893 keeps 891 Success
and the same two NotImplemented records/native exit 1. All four earlier
RegExp/ArrayBuffer Bug IDs and both original Promise CLI realm witnesses pass.
There are zero prior-pass regressions or missing IDs. See the
[complete checkpoint](docs/rust-rewrite/promise-try-checkpoint-20261013.json)
and [normative contract](docs/rust-rewrite/contracts/promise-try-order-identity.md).

Complete raw owners are under target/promise-try-{native,replay,native-build,
worker,focus}-604a6f9aa-20261013 and
 target/promise-try-acceptance-space-604a6f9aa-20261013/receipt.json.
The acceptance queue uses eleven complete native scopes and three complete
families on CPUs 2/3 in 2,702.669s. Build takes 1,407.545s; R precompile takes 630.741s with
1,507 hits/650 misses, excluding earlier fresh R emission/admission. These are
concurrent-load observations, not idle speedup measurements. Failed disk
preflight and wrong-host CLI invocation owners remain alongside their successful
successors. Full pinned conformance and task closure remain pending.

The preceding [constructor/Iterator checkpoint](docs/rust-rewrite/constructor-iterator-checkpoint-20261013.json)
eliminates 112 Iterator Bugs and 22 original Function limitations. Its red
prediction and separate actual comparison remain. The exact 256-body reproducer
passes there; this latest worker retains all eight related native controls.
Two apply-T4 and four BMP dynamic-eval modes remain explicitly unsupported.
Do not invent unused names, source-shape exceptions, a bundled parser or VM.

## Preserve interrupted full owners

Original checkout remains clean at 4e01400fc91a2752fa362bf8882ad136e653089e:

- CPU0: target/full-pin-optimized-4e01400fc-20261013/receipt.json; actual 743c
  worker SHA 8e3d8018e803f0c154eb326af80094d146189adeccba9cc450866b19919a4ebc.
- CPU1: target/full-pin-repaired-b486503d2-20261013/receipt.json; actual b486
  worker SHA 285b61675f1eaddab95c89cfe64e08b2d79d825cf8c9e483b28dbd40bfdf5dd5.

Both started two fresh unfiltered current-pin families, jobs/threads1,
60000ms, reuse64 and fresh Engine/Realm/Store. Neither is complete. Collect the
second family even when first is red. Their older failures are their own source
results, not regressions of the latest compiler. Inspect node failures and
completed IDs, excluding aggregate snapshots. Keep original source/HEAD,
all active workers and their DSOs immutable.

The earlier accepted afba worker started its own two fresh full matrices on
CPU2: target/full-pin-string-afba1a07a-20261013/receipt.json. Worker SHA
660d75fd99b6fb80e3f2a6f28c6d2d52d5ca29ad007e42ff5dd6d766b13c8262.
The original checkout owns unchanged suite/scripts; the actual compiler is the
separately source-bound afba worker. Preserve its DSO and original source/HEAD.
The later import-order correction is formatting only; all native/CLI evidence
above remains bound to the actual afba compiler, without inventing a new build.

## Next work

The exact-list acceptance owner is complete: all2108 affected native tests and
all508 canonical Python controls pass. Actual36-ID per-ID/batch/interruption/
resume preserves34Success and both missing modes. These results belong to
actual compiler aee6bc109, not a later publication or prepared-source commit.
Main58e357322 publishes the sealed checkpoint and staging tool. Historical
all5365 replay was interrupted after saving 1490 native executions, including
1151 Success,96 Bug,236 NotImplemented,7 Crash and no timeout. Its original
incomplete owner remains preserved. Same-image/input native checkpoint resume
is running under
`target/historical-native-replay-aee6bc109-20261013-resume-owner/receipt.json`;
coverage and literal conformance remain unverified until its actual completion.
Keep canonicalAE and original4e sources, HEADs, workers and DSOs immutable.

The three older full owners above are stopped, incomplete and unpassed; their
recorded native controllers are zombies and their old execution sessions are
unavailable. Preserve their raw snapshots, logs, images and source identity.
Do not count partial families as completed closure. Their generated inactive
caches may be retired with owned manifests to make room for new verification.

Root has verified the historical repair batch in the repair checkout:

- Sloppy plain/escaped `let:` follows ordinary label grammar; strict labels still
  reject. The three original let-label Bug executions are retained.
- Logical assignment name inference uses original identifier syntax, preserving
  empty names for grouped targets. Optional constructor syntax rejects `new
  object?.Constructor()` while preserving parenthesized targets and optional
  access after a completed construction.
- Source arrows with potential direct eval retain lexical this/new.target/home
  object/derived initialization cells even when only eval text observes them.
  Paired original Bug/Crash modes and escaped/nested/shadowed controls remain.
- Class heritage/computed-name references and created callable metadata carry
  scoped lexical strictness and restore the enclosing mode, including nested
  classes and direct eval.
- Annex B arguments copying separates parameter collisions from implicit
  arguments. Named variable-environment assignment occurs at declaration
  evaluation; parameter-expression and body closures retain their own chains.
  Existing independent declarations remain static. The old IR control keeps
  both inputs and its ID with the corrected normative expectation.
- Source arguments reads/writes/destructuring and captured closures share the
  existing live named environment. A simple source var arguments starts with
  the initialized ArgumentsObject; formal/body environment ordering remains.
- The finite RegExp candidate helper emits one row loop over the complete static
  table with pooled GC-string ordinals. Descriptor publication copies the exact
  validated active-memory span. The actual old compiler's isolated Intl probe
  measures this helper at27330651bytes against7654321; no conformance result is
  replaced by the diagnostic. Encoded-size and native semantics controls cover
  table growth, every closed row outcome, Unicode, flags and computed misses.
- RegExp substitution collects completed pieces, checks accumulated i64 length
  against the existing string extent, then copies once and publishes. Named
  capture Gets/conversions/reentry remain ordered. Oversized replacement must
  become catchable RangeError rather than GC trap; general remaining-memory
  allocation failures are not claimed solved.

Contracts and paired frontend/native controls accompany the batch. Actual
prepared8cb35ae74 passes all177 frontend controls, retaining all172 old IDs; its
complete IR test/type graph checks after three tracked sparse fixtures were
restored. These are frontend/type verdicts, not JavaScript/Wasm acceptance.

Strict callable metadata, live arguments bindings, compact RegExp emission and
private runtime-bundle publication now have the source-bound native evidence
above. Actual focus passes16, complete affected Engine inventories2053,
runtime-link26 and full Python514. Private CLI capture preserves ordinary
output and produces the actual115279 worker; its105-ID replay is complete red
with61 repaired. Whole-workspace no-run hit ENOSPC before native execution;
keep that failed owner separate. Use serial selected-target builds for the next
batch; archive closed Cargo leaves before generic output overwrite. Keep
canonicalAE/original4e and frozen workers/DSOs unchanged until owners close.

Remaining historical resource/caller findings stay explicit: catchable recursion
needs an actual stack-usage bound, the huge generated Unicode table needs generic
compact emission/body splitting, Unicode-surrogate RegExp matching needs repair,
and optional sloppy caller/arguments extension policy needs coherent controls.
Do not silently drop pending staging IDs or dynamic-source limitations.

Closed execution/cache retirement preserved all images, DSOs, transcripts and
nonpass IDs: debug unfingerprinted1254572032bytes; closed1bb execution478126080;
closedAE native/parity1086394368. Active full/history caches remain untouched.

## Remaining acceptance

ECMA 5345883164f463e87f8b40aca4956157ecba8783;
Test262 2e0a56762801e275a9fdf96dc49d90ba0cddcf63,
tree 6a4268a9354a545d41c3b62efebc478ed8c521f4,
metadata SHA ab47a6da190c07bc54ac2cc752580345f89fce9c31b739e7987be78b5ffea480.
Full census 102,994 executions/53,616 files is not a pass claim.

T00/T27/T28/T29 are complete; T01–T25 in progress; T26 blocked. Complete their
actual criteria, latest-source full pinned families, compiler-bound publisher/
backlog, full-manifest shard/thread and interruption/resume equivalence,
supported-host execution, calibrated subsystem policies and semantic/resource
gates. Default/all-feature native, optimized704/1216 campaigns and original idle
performance have accepted predecessors linked from README. The current product
worker has no oracle and cannot inherit a new T25 campaign verdict. Supported
host retrieval returned403; authored CI is not execution. No idle measurement
during concurrent full runs.

Recovered historical input has all 5,365 IDs at
 target/historical-backlog-replay-input-20261012/executions,
SHA 95d46445f4c3fda986c7cdf390dccdddfb8ab370b8a831e3f77ea7f387a1a43e.
Original September aggregate was local/unpushed; do not ask again or invent its
outcomes. Current replay retains two absent DurationFormat files as owned
infrastructure failures. Negative Apia's collapsed window still lacks normative
fallback authority; retain T22's uncatchable semantic gap. Unsupported dynamic
source stays in the full denominator and prevents literal all-zero closure.
Historical details remain in the previous handoff Git revision and linked
source-bound checkpoints; this file now contains only current actionable state.
