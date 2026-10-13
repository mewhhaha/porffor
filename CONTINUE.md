## Constructor and Iterator checkpoint; candidate-retention repair next

Actual `9ddc57a2c` completes all 4,489 affected native IDs, preserving all 4,485
previous passes. Complete original Iterator1308 replay has 1,308 Success,
eliminates 112 Bugs and retains all 1,196 previous passes. Function893 has
879 Success/14 NotImplemented: ten limitations removed, all 869 prior passes
retained. Raw complete owners:
`target/constructor-iterator-native-9ddc57a2c-20261013/receipt.json` and
`target/constructor-iterator-replay-9ddc57a2c-20261013/receipt.json`.

A separate actual-worker probe catches a 256-body regression that blocks
publishing that batch. Preserve its failed owner at
`target/source-candidate-retention-regression-9ddc57a2c-20261013/receipt.json`.
The next coherent patch preserves all primary candidates/tuples ahead of
supplemental hints, and adds bounded body-IdentifierReference hints for unknown
object-to-parameter conversions. Four IR/four native controls are written.
Compile once, run those controls and original constructor/Iterator controls,
compare complete original families plus all prior pass IDs, retain ArrayBuffer/
RegExp fixes, finish affected native scopes/publication guards, then push.
Expected new removals are ten more constructor IDs; four unused-formal T4 IDs
and four BMP eval IDs remain explicitly unsupported. No new runtime verdict
is claimed until those actual owners finish. See the
[discovery and admission contract](docs/rust-rewrite/prepared-constructor-discovery-20261013.md).

The older checkpoint below retains its own source and pending status.

`c457c82a5` passes the complete optimized IR unit/integration scopes: 2,178 PASS,
zero FAIL/IGNORE, every 2,176 prior passing ID retained and two new controls.
Raw owner: `/workspace/porffor/target/constructor-ir-c457c82a5-20261013/receipt.json`.
[The portable checkpoint](docs/rust-rewrite/prepared-constructor-ir-checkpoint-20261013.json)
retains exact IDs, artifact hashes, commands, logs and baseline comparison.
Forward undefined source hints are installed during hoisting, including nested
functions. Declared function constructor properties retain their source grammar.
The ten original Function/AsyncFunction limitations need actual worker replay;
fourteen stateful source-conversion and four BMP eval limitations remain owned.

The unchanged original prefix also finds 112 Bugs from two absent methods:
Iterator.prototype.includes and join. Their complete registration/native bodies
and strict/sloppy effects/close controls are authored in this batch; no runtime
verdict is claimed yet. See [the normative owners and rules](docs/rust-rewrite/contracts/iterator-includes-join.md).
Build the coherent batch once, run focused controls, compare complete original
Function/Iterator cohorts and prior passing IDs, then complete affected native
scopes and publication checks before pushing.

Disk cache pressure is a real iteration constraint. Inactive debug Cargo
fingerprints are reversibly parked at
`/workspace/porffor/target/completed-debug-fingerprint-parking-20261013/receipt.json`:
717,393,920 allocated bytes become a verified 6,344,788-byte archive in 4.78s.
Every archived file is SHA-verified; active workers/DSOs, release graph and raw
verification owners are unchanged. Restore exact bytes if that debug graph is
needed; completed generated caches remain disposable after their owners close.

## Two fresh repaired full-suite families running — 2026-10-13

The verified ArrayBuffer checkpoint is pushed to main as `b7297ec58`. Its actual
`b486503d2` compiler now runs two fresh, unfiltered complete current-pin families:
`/workspace/porffor/target/full-pin-repaired-b486503d2-20261013/receipt.json`.
The frozen original baseline continues independently at
`/workspace/porffor/target/full-pin-optimized-4e01400fc-20261013/receipt.json`.
Neither owner is complete yet; preserve all nonpassing outcomes and the full
denominator, and collect the second family even when the first is red.

The baseline uses CPU 0 and the repaired worker CPU 1, with one job/worker,
60000ms deadlines and fresh Engine/Realm/Store state. The repaired run uses
256/256/64 MiB function/module/program disk caches and the unchanged two-entry,
256 MiB retained-module cache. Its driver verifies immutable worker/runtime
hashes, actual compiler identity, pins, and completed native/replay owners.
The original `4e01400fc` checkout owns unchanged runner/suite inputs; it is not
the repaired compiler revision. Keep that checkout and both staged workers/DSOs
frozen. The repair checkout may continue independently on CPUs 2–3.

Inspect new failures from per-node `failures` records, excluding aggregate-matrix
snapshots when counting. Newly encountered Function/AsyncFunction constructor
limitations need classification against T13's finite prepared-source contract;
do not assume every unsupported result requires a bundled parser.

## Verified ArrayBuffer batch and retained RegExp fixes — 2026-10-13

Clean `b486503d2` passes all 442 original ArrayBuffer IDs at unchanged 60000ms
limits: 442 Success, zero Bug/Crash/NotImplemented/timeout. Both old maximum-capacity
Bugs are eliminated; all 440 old Success IDs remain, with zero missing IDs or
regressions. Both Unicode RegExp IDs fixed by the previous batch remain Success
on this actual worker. The two batches therefore eliminate all four Bugs exposed
so far by the unchanged baseline in their exact replay cohorts.

All five complete affected optimized native scopes pass: 1,789 PASS, zero FAIL/
IGNORE. Counts are 334/874/496/4/81 for AOT/Engine/builtins/runtime-cache/CLI.
All 1,788 previous passing IDs remain and one strict/sloppy constructor-ordering
control is added. Existing real allocation-failure/resize/transfer controls pass.
See [the complete checkpoint](docs/rust-rewrite/arraybuffer-checkpoint-20261013.json).
Native build and execution are separate owners; actual selected artifact hashes
are verified before execution, and no build-only receipt claims test verdicts.

The CLI build takes 530.204 seconds under concurrent baseline load; R precompile
215.357 seconds with 2135 function-cache hits/18 misses. Private link takes
2.026 seconds and restores ordinary bytes exactly. These are observations, not
idle speedup claims. Sparse checkout initially removed the ignored Cargo config;
its preflight caught that before Cargo ran. `.cargo` is now included explicitly,
and exact original config survives sparse reapply. A native build-space preflight
also rejected before Cargo; reclaiming the completed replay's generated cache
provides space. All failed preflight logs remain and actual verdicts are unchanged.

The RegExp/cache and ArrayBuffer batches are pushed as `3aae33ef6` and `b7297ec58`
with their verified evidence and publication checks. The original checkout, full
worker and shared runtime remain frozen at `4e01400fc`. Continue both complete
families, remaining failures, historical5365 replay, actual shard/resume, supported
hosts, idle measurements and task-specific acceptance. Four BMP dynamic-eval
executions remain explicit unsupported: each generates roughly 65,000 runtime source
strings outside the finite prepared registry. Keep their denominator and verdicts;
no path/source-shape exception or bundled parser/VM is permitted.

Preserve raw owners and worker/runtime images. Completed task-generated caches
are disposable after receipt closure. Keep CONTINUE.md/tasks until actual completion.

## Verified repair batch and next failures — 2026-10-13

Clean `a4b69645d` completes all five affected optimized native scopes: 1,434 PASS,
zero FAIL/IGNORE. Exact comparison retains all 1,433 prior passes and adds one
computed-brace control, with zero missing IDs or regressions. Counts are
334/873/142/4/81 for AOT/Engine/RegExp/runtime-cache/CLI. Full 124-ID Annex B RegExp
replay retains all 118 old Success IDs, eliminates both old Bugs, and preserves
four explicit dynamic-eval NotImplemented results and their details. Native
verdict remains red for those limitations; the comparison itself passes. Original
60000ms bounds, input bytes and pins remain. See
[the portable checkpoint](docs/rust-rewrite/regexp-runtime-checkpoint-20261013.json).

The first native comparison preflight failed after a successful 259-second Cargo
build: libtest displayed ` - should panic` annotations, and old shared-link records
used different schemas. No native test started in that attempt. The corrected
reader normalizes the annotation, selects links by actual executable SHA and
reuses the successful build; the original logs and receipt remain unchanged.
All 1,434 actual native tests then complete. Source and baseline images stay fixed.

The original checkout remains frozen at `4e01400fc`; its full two-family owner is
`target/full-pin-optimized-4e01400fc-20261013/receipt.json`. Keep its source, private
worker and shared runtime unchanged until completion. This repair is verified
in the isolated checkout. Compiler stage labels now reflect Spec IR construction
in preparation. Runtime digest/cache correctness passes; new idle performance
measurements remain pending. The build took 895 seconds under concurrent baseline
load; R precompilation took 207.473 seconds with 2152 function-cache hits/one miss.
These are observations, not an idle speedup comparison.

The next failures found at that checkpoint were both modes of
`built-ins/ArrayBuffer/options-maxbytelength-allocation-limit.js`: 442 original
IDs, 440 Success and two Bugs. The `b486503d2` batch above closes that entire
family and preserves the RegExp fixes. Its maximum-capacity admission follows
pinned `AllocateArrayBuffer` ordering and the existing backing representation.
The full source remains pinned to ECMA262
`5345883164f463e87f8b40aca4956157ecba8783`.

Continue full families, historical5365 replay, actual shard/resume, supported-host
and task-specific acceptance. Preserve raw owners and current workers/DSOs;
completed task caches are disposable after receipt closure. Keep CONTINUE.md/tasks
until actual completion and push verified batches to main.

## Default optimized acceptance and next gates — 2026-10-13

Clean `743c810c7` passes the actual default release build, private shared worker,
all 932 affected CLI/library/ignored-performance tests and both original deep
WeakMap execution IDs at 60,000 ms. Exact comparison retains all 930 prior
native passes, adds two passing host controls, and retains both stress passes:
zero missing IDs or regressions. Native scope counts are 81/848/3.

Fresh one-CPU idle reports complete all three gates, all twenty original compile
and runtime fixtures, and seven measured samples per fixture. Median warm exact/
chunk/cold exact are 28.463 ms/692.374 ms/1.367675 s, all original budgets pass.
Compiler prepare owns 90.3% of stage time and already constructs Spec IR; the
frozen build's emitted scope string and SDK comments incorrectly put that
construction in lower. `a4b69645d` corrects the labels while preserving raw
metadata; do not interpret this historical span as parser-only cost. All 140 measured runtime module-cache
outcomes are hits. GC counters unavailable from Wasmtime remain unavailable.

The worker is 101,359,416 bytes versus ordinary 424,668,048. Private link takes
1.794 s, ordinary restore 0.364 s, and CLI-only release rebuild 124 s. First
default graph cost 1,176 s, runtime precompile 199.953 s with 2,153 hits/zero
misses. Different build work is not a controlled speedup. See
[the portable owners and exact comparisons](docs/rust-rewrite/default-optimized-performance-20261013.json).
The first failed host-mismatch reports and the source guard's failed absolute
checkout-path observation remain intact; portable display paths are normalized
without modifying any raw receipt. A scratch stress preflight initially assumed
a nonexistent `passed` field in the staging schema; no stress test ran there.
The corrected driver validates actual stage exits and runs both fresh modes.

Next run current-pin complete families, historical5365 replay and real shard/
resume ownership; publish only verified complete results. Keep every raw owner,
reversible cache archive, runtime OUT/DSO and current worker. Minimal R is
precompiled; Conformance R, differential session reuse and useful crate cuts
remain proposals. Full task acceptance, T22's negative Apia authority and
supported-host receipts remain unresolved. Delete this file/tasks only after
real completion and push verified batches continuously to main.

## Optimized sustained and private-worker acceptance — 2026-10-13

Frozen clean `b165ef789` passes optimized nightly 704/704 and robustness
1,216/1,216. Exact comparison retains all prior passing IDs and request bytes,
original 5,000/300,000 ms bounds, all eleven grammars/nineteen targets, with zero
missing IDs or regressions. Three fresh full corpus rounds each for ordinary
and shared/stripped release images repeat all aggregate/input/report bytes and
historical semantic observations. Medians are 32.208/21.201 seconds; these
consecutive runs are observations, not a controlled speedup claim. The worker
is 131,047,560 bytes versus ordinary 499,663,192; private link 1.821 seconds,
warm ordinary build 0.502 seconds, ordinary output restored byte-for-byte.
Nightly/robustness take 1,219.103/467.494 seconds. The first optimized graph
takes 1,967 seconds; runtime precompilation alone takes 552.237 seconds with
1,013 function-cache hits/1,140 misses. The
[portable checkpoint](docs/rust-rewrite/optimized-worker-checkpoint-20261013.json)
retains exact raw owners and comparisons. The live progress monitor briefly
counted unfinished robustness slots; completed-only counting fixes that
auxiliary observation. No native/campaign failure or raw verdict was changed.

Next switch once to the default optimized CLI, retaining the completed oracle
worker and every raw receipt. Run idle seven-sample whole-CLI/compiler/runtime
profiles, actual optimized stress, historical/current-pin checks, real
shard/resume, two fresh unfiltered families and verified publication. Bound
worker reuse must preserve fresh Engine/Realm/Store/GC state and whole-session
retirement on timeout; differential reuse is only a proposal. Minimal R is
precompiled; Conformance R and crate dependency cuts remain proposals. Resolve
all task-specific evidence owners, including T22's under-specified negative
Apia window and missing supported-host receipts. Keep this file/tasks until
genuine completion and push verified batches continuously to main. The older
sections below describe their frozen historical sources.

## Complete default native and queue checkpoint — 2026-10-13

Frozen clean `9908850ce` completes all 105 default-feature native/ignored/docs
scopes: 8,881 PASS, zero FAIL, zero IGNORE. This includes all 848 default CLI
IDs, default oracle-refusal controls, full fake publication and original timing
gates. The four immutable phase receipts seal one complete inventory; no failed
outcome is replaced. The older thirteen-scope default Engine subset retains
3,181 passing IDs, adds 69 passing IDs and has zero regressions. Three old IDs
remain visibly absent with source-proven replacements: required weak support,
real weak constructors, and contract ownership independent of temporary tasks.
See [the portable checkpoint](docs/rust-rewrite/native-default-checkpoint-20261013.json).

The Python-only scheduler now starts independent scopes before idle barriers,
retaining original case objects/indices, stable inventory order, exact flags,
continuation membership and all existing drain/isolation rules. Four focused
controls and all 495 Python methods/fifteen formatting/source checks pass. The
[tooling receipt](docs/rust-rewrite/native-default-tooling-20261013.json) records
the frozen checks; publication guards precede the next push. No Rust source or
profile changed and no native scheduling speedup is claimed. Eight
SHA-verified cache archives park 7,006,674,944 allocated debug-library bytes in
1,264,826,265 archive bytes, leaving about 8 GB for a distinct release graph.
Keep every raw receipt, runtime OUT/DSO, product/private/staged executable and
Cargo fingerprint; absent library outputs can rebuild normally. Restore only
exact source/environment/cache generation and never overwrite newer output.

Next build one feature-enabled release graph for CLI/test262 bins, then execute
three fresh complete corpus byte-determinism rounds, nightly 704 and robustness
1,216 with each driver's own cloud flag, optimized private-worker acceptance and
idle performance. Switch once to default release for current-pin stress, actual
shard/resume, two complete unfiltered families and verified publication. Resolve
every task's exact acceptance owner before deletion. T22's Apia contextual-window
semantic gap remains real; supported Linux/macOS artifact retrieval lacks actual
receipts after 403. Historical aa552 backlog input is not the current pin
`2e0a56762801e275a9fdf96dc49d90ba0cddcf63`. Retain this handoff/tasks until actual
completion and push verified batches continuously to main. Older dated sections
below are historical source-bound checkpoints, superseded where later evidence
explicitly resolves their failures.

## Complete all-feature native and measured worker checkpoint — 2026-10-12

Frozen clean `f54ef2ae9` completes all 106 all-feature native/documentation
scopes, including all ignored controls: 8,974 PASS, zero FAIL, zero IGNORE.
Nine storage-deferred scopes complete in separate identical-source continuations;
the composite preserves every original outcome and exact inventory/transcript
hash. Against the complete 032 baseline, all 8,959 prior passes remain, the
remaining mutation failure is eliminated, fourteen new IDs pass and no ID is
missing or regresses. Separately, all 4,525 passes from the earlier red partial
remain and its structural regression is eliminated. Keep both original red owners.
See [the portable checkpoint](docs/rust-rewrite/native-worker-checkpoint-20261012.json).

Seven balanced compiler-identity samples per image and three balanced complete
corpus rounds per image run on this idle cloud machine, with fresh caches and
unchanged fourteen inputs/order/deadlines. Original, stripped and shared/stripped
medians are 40.149, 34.655 and 32.300 seconds; shared/stripped saves 19.6%.
Each image's repeated aggregate, inputs and reports are byte-identical. Between
images, every semantic report field agrees while actual worker image hashes
remain distinct. No broader semantic-equivalence or full-pin claim follows.

The new opt-in `scripts/stage-verification-worker.py` stages a private worker
without changing Rust sources, profiles or ordinary Cargo products. Exact shared
arrays, allocated ELF sections, Cargo-selected artifacts and loaded-image
identities are checked. Fresh/cached debug links and normal-controller/private-
worker complete replay pass. The first auxiliary staging attempt incorrectly
expected a string source revision; preserve its red receipt alongside the
separately corrected structured-identity check. Release mode is implemented but
not executed. Minimal R is precompiled; Conformance R is not. Wasmtime's compiled
module cache already shares across executable identities; raw R cache does not.

Next run the complete independent default-feature native/ignored/docs suite,
then optimized corpus repeated-byte determinism, nightly 704 and robustness
1,216 campaigns, idle performance, historical/current-pin stress, real shard/
resume, two complete optimized families, verified publication and all task gates.
The old T25 `snapshot_determinism` selector matches zero tests; use the complete
repeated corpus byte comparison and retain the actual `snapshot_use_structure`
controls separately. Supported-host artifact retrieval currently returns 403.
Keep fixtures, pins, inventories, original deadlines and passing IDs. Push
verified batches continuously to main. Delete this file and tasks only after
genuine completion. Crate splits and Conformance-R precompilation remain proposals.

## Weak collection checkpoint and next coherent verification — 2026-10-12

The original two 99,999-link WeakMap modes now pass at their original 60,000 ms,
in 18.196/18.160 seconds. All 704 nightly and 1,216 robustness IDs retain their
original requests/order/deadlines and pass with zero regressions. Corpus14/14,
25/25 focused GC controls, one original vendor cancellation test (thirteen cut points),
all-feature/all-target types, 479 Python methods and all thirteen guards pass
on the recorded frozen source. The immutable portable record is
[weak-index-repair-20261012.json](docs/rust-rewrite/weak-index-repair-20261012.json).

The full native attempt and continuation are terminal but incomplete: 44 of 106
scopes execute 4,526 IDs: 4,525 PASS, one FAIL, zero IGNORE. Four new controls pass,
4,521 prior passes are retained, one prior-passing structural check regresses,
and no observed ID is missing. 62 scopes are unstarted or storage-deferred.
Raw receipts are preserved under
target/weak-index-full-native-all-20261012 and
target/weak-index-full-native-parallel-env-20261012; the exact partial composite
is target/weak-index-full-native-partial-20261012. Do not retry the red ID on the
old source or call these partial runs a complete workspace pass.

The newly found catch-all panic is from 672's private import-prelude registration,
not collector logic. Root now spells every Declaration variant and retains the
original structural guard. The runner's next source batch honors isolation in
docs, records each doc inventory hash, reserves disk before fallback relinks,
throttles storage logs and expands a serial Cargo build to the inherited CPU
budget once every native wrapper exits. Five CPU ownership and eighteen exact
cohort controls pass. The frozen batch now passes the original three structural
controls, all 491 Python tests, thirteen guard commands, nine Intl freshness
checks and every all-feature/all-target type. The live child/thread/grandchild
CPU expansion passes within the four-CPU budget. The type check takes 458.508
seconds after this IR edit; the prior warm check took 69 seconds. These are
different runs, not a controlled scheduling speedup. See
[verification-runner-20261012.json](docs/rust-rewrite/verification-runner-20261012.json).
Fresh complete native and optimized acceptance remain required.

1. Push the verified collector, exhaustive-match and tooling batch. Run complete native
   all-feature/default/ignored/docs and compare exact prior IDs, preserving the
   old structural failure and every unexecuted scope explicitly.
2. Measure the actual build CPU events and separate process startup/image hashing
   from compiler work. The [cost record](docs/rust-rewrite/verification-costs-20261012.md)
   proposes an IR contract/lowering dependency cut plus moving runtime construction
   out of AOT, and bounded differential worker reuse with fresh per-case JS state.
   These are proposals, not implemented crate splits or measured speedups.
3. Finish optimized corpus/determinism/sustained campaigns, idle performance,
   historical/current-pin stress, real shard/resume, two complete optimized
   families, verified publication and every remaining task gate. Push verified
   batches continuously to main. Delete this handoff and tasks only when complete.

The inactive global memo cache was copied and SHA-checked before relocation to
/tmp/lila-shared-compile-cache-20261012; its original public path is a symlink.
No current CLI, native runtime image, Cargo fingerprint, fixture or result was
changed. The relocation receipt is target/weak-index-cache-relocation-20261012;
current workspace free space is about 3 GB and /tmp about 1.7 GB. Retain current
runtime-image OUTs 829f43c256a8d0f7 and 3b1acde22ee9c14f, all used test DSOs and
the actual CLI.

## Accepted generator/oracle batch and next measured repair — 2026-10-12

The frozen compiler/oracle repair passes all 704 debug nightly campaign cases:
13 prior failures eliminated, all 691 prior passes retained, no missing IDs or
regressions, with byte-identical requests and the original 5,000 ms deadlines.
All 1,216 robustness IDs retain their original inputs, order and 300,000 ms
bounds and remain passing. All fourteen compiled corpus cases pass; the
original twelve bytes/order remain and two actual minimized witnesses are
appended. Ten selected native scopes pass 1,958 controls: every 1,954 prior
selected ID remains passing and four new controls pass. Eight scopes are
complete and two Engine scopes are filtered. The composite native baseline
retains its separate source states; this is not a complete workspace pass.
See [the source-bound record](docs/rust-rewrite/generator-oracle-repair-20261012.json).

Generator If lowering now balances its region depth and reserves nested
constant-branch phases. Boa's oracle labelled break closes every skipped inner
iterator while preserving target-loop close ownership. Strict/sloppy controls
cover complete values, yielded finally, return/throw, close order and abrupt
identity. Two emitted GC controls prove bounded cyclic churn and escaped
foreign-cycle survival/release. Capacity observations are reserved capacity,
not allocation/live-byte/pause/peak metrics. The first auxiliary replay driver
mistakenly rejected two deliberate syntax negatives; its red receipt and raw
both_failed reports remain unchanged beside the checked continuation.

The next measured repair is WeakMap lookup and collector marking. Both exact
original 99,999-link pinned modes time out at 60,000 ms on the retained clean
672 compiler and this repair compiler: two pre-existing failures, no regression.
The host Vec lookup is quadratic; boxed StoredValue cells also make repeated
whole-registry ephemeron passes quadratic. Both repairs are now written in
maintained source, retaining actual owner/key weakness and all old controls.
The index is non-rooting and Store/collection-epoch scoped; the collector queue
is activated by first-copy notifications for both owner and key. All reservations
precede heap movement. Fresh types, native controls, original stress cases and
retained passing-ID checks remain required; source review is not acceptance.

Actual Cargo fingerprints also prove that Intl's test-only compiled_data feature
widens the workspace library-test graph compared with the CLI build graph.
The opt-in test-package-isolation tooling keeps exact native inventories while
separating those roots; helper checks alone establish no build-speed gain.
Do not force Cargo freshness or inherit passing verdicts. Protect retained
passing library hardlinks before any build can overwrite their canonical paths.

Next finish coherent GC/tooling checks, fresh complete all-feature/default
native/ignored/docs with exact prior-ID comparison, optimized corpus/determinism
and sustained campaigns, performance reports, historical/current-pin stress,
real shard/resume and two full release families, verified publication and every
remaining task gate. Preserve all fixture bodies, counts, deadlines and source
pins. Delete this handoff and tasks only after actual completion.

## Accepted async loop/module batch and next gates — 2026-10-12

The frozen semantic repair passes all seven selected scopes: 1,943 native
passes, zero failures or ignores. All 1,939 prior selected IDs remain; one old
failure is eliminated, no passing ID regresses and all four new controls pass.
Five scopes are complete; async loop bindings and module imports are explicitly
filtered. The four storage-deferred scopes continue on the exact same source
with matching inventories and no failed-test retry. All original timing gates
pass. Types include every all-feature workspace target; eleven source guards
pass. The actual CLI then matches both original mutation seeds and all 22
standard PR-fast seeds on byte-identical requests with the original 5,000 ms.
Both profile-only failures are eliminated; against the preceding GC CLI, four
distinct seed failures are eliminated and no passing case regresses. All twelve
compiled corpus cases and fourteen bounded native robustness cases pass. The
[exact evidence](docs/rust-rewrite/async-label-shared-import-20261012.json)
preserves source fences, observed compiler identities, native IDs, historical
failures and separate native/campaign comparisons; do not double-count them.

Direct loop labels now forward a child's checked owner while labelled non-loop
continuations remain rejected. Compiled import dispatchers share a synchronous
compiler-private options validator while keeping intrinsic/order/referrer/job
authority. New strict/sloppy controls cover zero-trip and entered loops,
captures, awaited finally/overrides and attribute Proxy traps/abrupt identity
across two importers. Developer optimization level two keeps assertions and
overflow checks. AOT/IR test ELFs are retained and verified against actual
passing receipts; restore exact bytes only at canonical Cargo output paths,
then let normal Cargo decide freshness and run every full inventory anew.
Match the native wrapper/no-incremental environment and workspace feature graph
for later CLI builds to avoid repeating normal-library/build-script work.

Next: fresh sustained debug/optimized differential and robustness tiers,
complete all-feature/default native/ignored/docs, optimized corpus/determinism,
historical/current-pin release, stress/shard/resume/publication and all remaining
task gates. The current pinned census is not a pass. Preserve all original
deadlines, bodies, traversal bounds and denominator IDs; keep unsupported or
upstream-unresolved cases visible. Delete this handoff and tasks only after
actual task completion, then verify and push the final tree.

## Complete native verification and heap repair — 2026-10-12

The clean 032d2f5c7 checkpoint completes all 106 all-feature scopes with ignored
controls enabled: 8,959 pass, one fails, none are ignored. All 8,950 baseline IDs
remain; ten earlier native failures are eliminated and no previous passing native
ID regresses. All ten added IDs pass. Full CLI 855, all documentation, complete
fake publication and all three original performance controls pass. Exact mutation
seed nine still regresses from a selected-dimension match to a 5,000 ms timeout
on identical request bytes; seed ten is pending. The
[source-bound comparison](docs/rust-rewrite/native-workspace-032d2f5c7-20261012.json)
preserves both levels, the full source map and current Linux/macOS CI metadata.
Artifact archives were not inspected locally. Default/release/current-pin,
sustained stress, canonical publication and task closure remain required.

After this frozen run is terminal, root profiles both unchanged canonical mutation
seeds with fresh private caches and the original internal 5,000 ms. The diagnostic
parent permits late completion; sampling establishes no campaign pass. Copying
GC consumes 19.28% and 19.79% of sampled self CPU, native compiler symbols 24.20%
and 24.79%, and SHA-256 9.83% and 9.80%. Engine's admitted Minimal Intl reuse is
already cheap, so it is not changed. Root raises the shared configuration's
initial GC heap from zero to 16 MiB within the unchanged 1 GiB Store cap, bumps
the native configuration schema to three, and adds deterministic controls for
128 rounds of dead cycles under a fixed 64 KiB heap and pending Promise reaction
roots/release. All-feature/all-target types and formatting pass. The frozen affected run now
passes all eight complete scopes and 1,578 native controls, including full Engine
874, builtins 493, GC entries 162, weak facility 11, both original mutation seeds
and all three original performance gates. Exact comparison retains every 1,576
prior selected ID: one failure eliminated, none remaining, no prior passing ID
regression, two new controls pass. The actual fresh-cache CLI separately restores
seed nine but still times out at seed ten under the original 5,000 ms. Cold CLI
diagnostics measure execution at 177/189 ms and whole children at 5.201/5.159
seconds; late completion is not campaign acceptance. Fix the remaining measured
startup cost, then finish fresh broad/default/release/pinned/task acceptance
before deleting this handoff and tasks.

## Parsed cache and simple own-Get repair — 2026-10-12

The frozen parsed-cache/simple-Get focus completes 3,424 native controls:
3,422 pass, two fail, none ignored. All 3,419 previous selected IDs remain;
one original failure is eliminated and one remains. All five added IDs pass.
The twenty-case gate and both exact timing gates pass unchanged. Mutation seed
nine still times out at 5,000 ms, with seed ten unexecuted. A new Date trace
assertion introduces the other failure by counting the fixture's two budget
compilations alongside its two cold executions. Root corrects that assertion
to check each actual cache attempt: cold executions lower once and hits never
lower, while retaining the fresh-Realm results and exact private cache budget.
The separate Date-only native follow-up passes; its all-feature/all-target
types and formatting pass. This is a test-only source correction after the
frozen focus, not a single-source full-suite pass. The
[exact record](docs/rust-rewrite/native-parsed-cache-simple-get-20261012.json)
preserves the first failure, both source maps, the comparisons and late cold
diagnostics (seed nine 5.489 seconds, seed ten 6.051 seconds). Diagnostics do not
pass the original campaign. Full 874 Engine, 2,175 IR, 334 AOT, 16 worker and
six Reflect/Get controls pass. Full/default/current-pin/task acceptance remains.

The Array argument/range follow-up completes all ten selected scopes on frozen
source: 2,552 pass, two fail, none are ignored. All 2,553 previous selected native
IDs remain without a failure on a prior passing native ID; the added control
passes. The campaign separately regresses seed nine to its original 5,000 ms
timeout and does not execute seed ten. The twenty-case gate measures 5.111949005
seconds against five. No failure is eliminated by this attempt. The
[exact record](docs/rust-rewrite/native-array-argument-context-20261012.json)
retains both comparison levels and explicitly diagnostic fresh-cache phase
measurements: seed nine 5.384 seconds and seed ten 5.690 seconds, with unchanged
request bodies and internal deadlines. Late diagnostic completion is not native
acceptance. Full 2,175 IR and 334 AOT controls, affected Reflect/Realm/resizable
controls and all 16 worker controls pass.

Root promotes one coherent follow-up after both preceding runs are terminal.
Ambient compilation now retains an actual admitted loader through parsing,
graph discovery and finite-source catalog closure. A parsed Script can try the
exact graph-free cache key before lowering: only prior fully prepared graph-free
artifacts can match; discovered dependencies own a final graph digest or reject.
Cache misses lower the retained source once; graph-backed and embedded authority
paths preserve complete preparation. Source/options/role/Intl/compiler bindings,
artifact validation, invalid-artifact eviction and fresh execution Realms remain.
A cache control covers dependency edits/rejections and another covers parse and
prelude rejection; the real cached-Date Realm control now requires cold-only
lowering. Simple own-Get avoids primitive String length boxing and concrete
OrdinaryObject data-descriptor snapshots. Coerced keys, fresh descriptors, whole
values, accessors, inheritance, receivers, proxies and exotics retain their
existing owners. Added strict/sloppy controls cover these boundaries and GC.
All-feature/all-target types, formatting, the module-boundary guard and nine
other source guards pass. Two guards initially fail only because the shell lacks
the Rust toolchain PATH; their activated retries pass and retain the first logs.
Fresh native acceptance above eliminates the twenty-case failure; the mutation
deadline remains red and complete broad acceptance is still pending.
Preserve deadlines, corpus bodies/order/bounds, exact native IDs and
both mutation seeds; finish full/default/current-pin/task acceptance before
removing this handoff or tasks.

## Full native successor and measured iteration batch — 2026-10-12

The c235e0f4f complete frozen successor executes all 106 scopes and all ignored
controls: 8,952 pass, two fail, zero ignored. All 8,950 prior native IDs remain;
nine original failures are eliminated, two remain, and no prior passing native
ID regresses. All four added IDs pass. The failed mutation campaign separately
regresses seed nine from a verified selected-dimension match to a 5,000 ms worker
failure; seed ten is unexecuted. The original twenty-case performance workload
measures 5.096707977 seconds against five seconds. The complete source-bound
[manifest](docs/rust-rewrite/native-workspace-successor-20261012.json) preserves
both native-ID and campaign-case evidence. Fresh full native unit transcripts
also satisfy the re-verification requirement from the earlier label collision.
Linux/macOS Intl capture and byte/identity/pin/native-count comparison pass in
[CI](https://github.com/mewhhaha/porffor/actions/runs/38202805367); public job and
artifact metadata are retained, without claiming local receipt ZIP inspection.

After sealing that full run, root promoted a coherent measured iteration batch:
immutable source-order declaration recipes shared across Analysis clones,
borrowed exact-signature selection and per-parameter fact cloning, concrete
Array own-data/length Get inside the existing shared kernel, and no redundant
barrier for completed documentation-cohort rows. Facts are reevaluated in their
current flow; previous independent IR algorithms remain cfg(test) equality
oracles. The new Array control covers key effects, receivers, mutation,
inherited abrupt completion and Proxy order. All-feature/all-target Rust type
checks pass. The focused native run completes 2,564 IDs: 2,562 pass, two fail,
none are ignored. All 2,563 previous selected IDs remain without regression on
a prior passing ID; the added control passes. All 2,175 IR controls pass with
previous-algorithm comparisons. Seed nine again matches the oracle, eliminating
its case-level regression; seed ten still times out. The original twenty-case
workload measures 5.330369186 seconds. The
[focused manifest](docs/rust-rewrite/native-measured-iteration-20261012.json)
retains both native and campaign-case comparisons. Cold seed-ten phase diagnosis
is not acceptance: 5.787 seconds total, 540 ms lowering, 892 ms raw R admission
and 1.328 seconds execution, down from the earlier diagnostic's 2.570 seconds.
The tracked September backlog supplies all 5,365 old execution IDs despite the
absent raw aggregate. An exact replay input retains every ID; two reference a
file absent from the current pin and must be reported explicitly. No current
membership or passing verdict follows from extracting this historical listing.
Final formatting, ten source guards and all 475 tooling controls pass. The
module-size guard requires a pure extraction into private signature_lookup.rs;
its all-target IR type check passes after sibling visibility is made explicit.
The native focus predates that extraction; complete fresh native acceptance is
still required. Exact recorded rustdoc checkout-path IDs retain their spelling
under a narrowly tested documentary identity allowance, with product drift
still rejected.
Original deadlines, bodies, order and traversal bounds remain intact. Require
both unchanged mutation seeds, the original timing controls and full/default/
pinned/task acceptance before deleting this handoff or the task folder.

## Complete native baseline and current repair batch — 2026-10-12

The frozen cd8aa4a32 baseline completes all 106 workspace scopes with all features
and ignored controls: 8,939 passes, 11 failures and zero ignores. Exact verdicts
and source/inventory/transcript hashes are preserved in the
[baseline and repair record](docs/rust-rewrite/native-workspace-repairs-20261012.md).
Seven original failures are eliminated across the same 354 controls, one remains
and no previously passing original control fails. The prior 3,574-control gate
also has no regression on a passing ID; wider failures are recorded separately.

A coherent successor repairs the missing immutable ArrayBuffer getter, primitive
WeakMap/WeakSet diagnostic distinction, stale Error-stack expectation, five
source/contract assertions, generated provider example and missing performance
fixture host authority. Static keys, JSON punctuation, immutable Minimal data
identity reuse and producer-only R register allocation address repeated work.
Fresh native acceptance is required; no repaired/full-suite pass is claimed yet.
The first successor focus completes 55 native controls: 54 pass, one fails,
zero ignores. Exact comparison across 53 existing IDs eliminates seven failures,
retains one and introduces no regression; both added IDs pass. Bytes succeeds
on Wasm-AOT and still fails the optional oracle's immutable indexed write.
The coherent oracle repair adds proposal-correct descriptors, Boolean Set
rejection, typed read/write admission for methods, species, codecs and Atomics.
Existing paired strict/sloppy immutable controls now cover both enabled backends;
The first paired oracle follow-up passes ten of eleven controls: Bytes passes
both backends, and the added write-order control passes. The descriptor control
still fails newly enabled oracle freeze coverage; its gate-level regression is
retained explicitly. The next repair fixes the mutable-only freeze shortcut,
immutable-slice post-coercion bounds and ordinary-slice post-species copy bounds,
with an added paired shrink/detachment/order/empty-range control. Native
verification of that final follow-up passes all 12 controls, including both
enabled backends. It eliminates the Bytes baseline failure and the newly exposed
freeze regression, with no failure on any prior passing selected ID. All three
added controls pass. All 473 Python tooling controls, all-feature/all-target
types and ten source guards pass after the final repair.
The Python cancellation fixture's PID publication race is repaired atomically.
The first focus's two raw library transcripts were accidentally replaced by a
diagnostic-label collision; receipts/inventories/verdicts and the full 8950-ID
baseline remain. The manifest records this limit, new labels refuse collisions,
and full fresh native unit verification remains mandatory.
The current timing/documentation checkpoint passes 17 of 19 controls with zero
ignores. The provider doctest now passes; mutation seed nine still times out at
the unchanged 5,000 ms deadline, and the correctly completed twenty-case workload
measures 5.304175098 seconds against five seconds. Neither timing failure is
claimed repaired. Exact phase diagnostics are retained under
`target/native-repair-phase-diagnostics-20261012`; they identify cold admission,
P compilation and execution costs but are explicitly not acceptance.
Use the complete source-fenced runner and exact before/after inventories. Do not
delete this handoff or tasks before all requested acceptance is actually met.

## Bytes module and remaining acceptance — 2026-10-11

The clean `77d7d3e46` complete inventory attempt finishes with 2,897 passes and
four failures across 2,901 native controls; 83 targets are storage-deferred and
one library build fails with actual ENOSPC before native admission. The separate
source-bound AOT library executes all 333 controls: 332 pass and one With size
guard fails at 10,548 bytes (4,289 to 14,837; unchanged ceiling 9,000).
All 447 Test262 library and four corpus controls pass. The three campaign
failures are actual original-five-second AOT timeouts, with no phase attribution
in their canonical reports. Sixteen original-budget worker controls pass.
Every original report is preserved unchanged. The IR conditional assertion must
look through its legitimate lexical prefix; complete branch/Try ownership remains
required. A root-written coherent Bytes/raw-loader/graph/wire/native/oracle batch,
shared selected With setup and idempotent Intl writes are now being verified.
Do not claim full acceptance or delete this ledger before final required checks
and durable ownership migration.

# CONTINUE — handoff state (2026-10-11)
## 2026-10-11 typed-root and static With repair

The `17492b7a3` retry completes 362 source-fenced native controls: 355 pass and
seven fail, with no ignores. All 16 worker controls pass; six campaign scopes
still hit the original five-second AOT budget. The static With-read size guard
still fails at 22,401 added bytes. Its actual static lowering path now uses
shared selected GetValue dispatch; known GC roots also carry concrete Wasm
local types to remove redundant heap checks. Final workspace types, 469 tooling
controls and 27 prechecks pass; native timings and full-suite acceptance remain
pending. A separate Test262 library transcript has
447 passes but no collected watcher status. See the [repair record](docs/rust-rewrite/typed-gc-read-repairs-20261011.md).


## 2026-10-11 complete-read follow-up

The `c51cd5cd5` native attempt has 801 passes and eight failures across 809
admitted controls. The ordinary-generator campaign now matches; seven generated
campaigns still hit the original five-second AOT budget. A repeated With-read
body-size control fails at 22,401 added bytes. Ordinary reads now keep resolve
and get together in R, and libtest compilation can omit disposable incremental
scratch while normal library caches remain reusable. The
[implementation and actual acceptance boundary](docs/rust-rewrite/combined-read-repairs-20261011.md)
retains every failure and unstarted scope. Full suites, pinned acceptance and
ownership migration remain open.


## Shared binding and generator repair — 2026-10-11

The latest complete-inventory attempt on `1ad5e2be0` runs 478 native tests:
472 pass, six campaigns fail and none are ignored. The original five-second
worker target passes all sixteen controls, and the Test262 library passes all
447 controls. Two builds exhaust disk before native admission; 243 targets
remain unstarted. Five failing campaign scopes contain actual AOT timeouts;
one also proves a generator continuation invariant failure. Original reports
and outcomes are preserved. The next repair shares complete environment
resolution/read dispatch, accounts for nonyielding generator try states in
conditional branches, and keeps test selectors on one Cargo feature graph.
Final workspace types, all 465 tooling tests, eleven repository guards and
eight Intl reproduction checks pass. Fresh native acceptance is pending. See
the [repair record](docs/rust-rewrite/shared-binding-repairs-20261011.md).

## Cold compiler verification repair — 2026-10-11

The unchanged `05f1a740a` focused attempts finish with 494 native passes,
twelve failures and zero ignores across 506 tests. The complete corpus and
CLI policy/deadline checks pass. Two TypedArray metadata assertions now use
literal upstream feature vectors; a signed-zero worker and eight campaigns
proved original five-second AOT timeouts. The ninth campaign failure lacks
enough report detail to classify. Dev/test Intl and Wasm emission now use Rust
optimization level 1; the optional Boa oracle drops its unused `cdylib`.
Raw runtime residency is bounded to two profiles/256 MiB; mode and parsed
Intl-kernel retention is bounded to two profiles. Active owners remain valid.
Final workspace types, all 458 tooling tests, 47 publication-driver controls,
eight Intl reproduction checks and repository guards pass. Fresh complete
native verification is pending. Original evidence is preserved; no
deadline, case or runtime admission is changed. See the
[repair record](docs/rust-rewrite/cold-compiler-repairs-20261011.md).

## Verification iteration repair — 2026-10-11

The coherent batch is applied: shared relational/TDZ helper emission, bounded
immutable Intl selection reuse, guarded serial in-process CLI captures,
publication-session worker-reuse identity, exact current pinned harness/cohort
inventories and differential fixture/diagnostic repairs. Cross-host Intl CI
captures and compares actual Linux/macOS evidence; no cross-host result exists
yet. All 458 tooling tests, 47 publication-driver controls, eight regenerated
Intl checks and final all-feature/all-target types pass. Thirteen initial
native controls pass before the source-bound Intl refresh; the Test262 link
then exhausted disk, so its native result is incomplete. Fresh complete
acceptance is pending. The clean-source retry passed eleven native controls,
then exhausted disk during the Realm build. Obsolete workspace library copies
and inactive caches were reclaimed; the public batch receipt explicitly records
two overwritten old transcripts and separately sealed fresh retry evidence.
The obsolete
serial CLI attempt was deliberately interrupted before its complete result;
none of its partial tests count as completed acceptance. The four engine tail
targets passed 370 tests, including 141 RegExp and 199 Temporal controls. The
corrected private fixture replay passed all 191 fake IDs. Preserve the raw V5
intrinsic bootstrap diagnostic and all original five-second campaign failures.
The repaired acceptance fixture explicitly isolates the complete selected
three-node graph while retaining the actual caught TypeError Realm anchor.
See the [batch record](docs/rust-rewrite/verification-iteration-repairs-20261011.md).

## Workspace inventory repair and smaller tests — 2026-10-11

Four stale weak/Error stack inventories are repaired. All 301 focused native
checks, joined all-feature/all-target types, eight Intl reproduction checks and
twelve guards pass. `LILA_TEST_STRIP_DEBUG=1` halves the Engine test executable
through the shared-image linker, with allocated sections checked before its
digest is sealed. Complete repaired-source workspace/default/ignored, release,
campaign and current pinned acceptance remain open. The preceding sweep keeps
4,229 passes and four failures across 169 completed scopes, plus two interrupted
scopes; it was deliberately stopped for this repair. The original five-second
campaign now fails first on CrossRealm seed 1270, so Temporal 3215 is unreached.
See the [checkpoint](docs/rust-rewrite/workspace-inventory-repairs-20261011.md).

## Bounded worker reuse and native iteration — 2026-10-11

The opt-in `--reuse-case-workers` route passes all 21 focused native controls,
including same-process recovery after a semantic failure and fresh Realm/Store
state. Every process is retired after 64 cases or an infrastructure failure;
final retirement errors reject success. A sixteen-case synthetic comparison
passes the same IDs in 65.367 seconds one-shot and 5.119 seconds reused, with
unchanged budgets. The verified Linux test linker shares immutable runtime-image
data; product CLI links remain static. Six IR doctests, all 443 tooling tests,
eight regenerated Intl reproduction checks and twelve guards pass. The obsolete
AOT API snippet is plain text, so that documentation target has zero executable
examples. Full workspace and current full pinned acceptance are next. See the
[checkpoint](docs/rust-rewrite/case-worker-reuse-20261011.md).

## JavaScript weak integration — 2026-10-11 focused checkpoint

The sixteen routes, actual GC brands, kept-root job boundaries and finalization
jobs pass nineteen focused native controls with zero failures or ignores.
Joined all-feature/all-target types pass. All 677 weak-feature IDs and all
70 Error stack proposal IDs pass with zero failures or timeouts. The selected
run reuses 240 exact validated checkpoints from the interrupted WeakMap attempt;
full-suite and task acceptance remains open. This supersedes older unavailable-product paragraphs
below. See the
[integration checkpoint](docs/rust-rewrite/weak-product-integration-20261011.md).

## Weak collector foundation — 2026-10-11 verified checkpoint

The pinned copying collector now has real weak targets and owner/key-gated
ephemerons. Ten forced-GC controls, the thirteen-boundary cancellation control
and eleven image controls pass: 22 native passes, zero failures or ignores.
Joined workspace types pass. The image leaf archive is 10,718 bytes; three
producers have matching digests. This foundation checkpoint is superseded by
the verified JavaScript integration above; complete family acceptance remains
open. See the
[foundation checkpoint](docs/rust-rewrite/weak-collector-foundation-20261011.md).

## Iterator chunking proposal — 2026-10-11

The chunks/windows batch passes 24 focused native controls with zero failures
or ignores and the complete official proposal selection on both backends:
156/156 AOT and 156/156 oracle, with original deadlines and exact execution-ID
censuses. On-demand GC Lists preserve whole values and borrowed Realms. Joined
types and all twelve repository guards pass. Full-suite acceptance and task
closure remain open. See the
[checkpoint](docs/rust-rewrite/iterator-chunking-20261011.md).

Work directly on `main` and push to `origin/main`; no feature branches or
force-push. Start with `AGENTS.md` and `tasks/README.md`. The detailed cloud
receipt is [docs/rust-rewrite/cloud-continuation-20261009.md](docs/rust-rewrite/cloud-continuation-20261009.md).
Use Luna agents for read-only chores/exploration; root writes code.

## Error stack accessor proposal — 2026-10-11

Error.prototype.stack is implemented in Wasm AOT and the optional oracle.
All seventeen focused native controls, joined all-feature/all-target types and
twelve guards pass. The complete official proposal selection passes 68/70 AOT
and 70/70 oracle at this earlier checkpoint. The later weak integration above
passes all 70/70 AOT IDs, including both weak-receiver modes. Full-suite
acceptance and task closure remain open. See the
[checkpoint](docs/rust-rewrite/error-stack-accessors-20261011.md).

## Runtime image metadata boundary — 2026-10-11

The verified batch moves immutable image data into a linked data archive through
a small leaf crate, preserving the checked producer, shared source fingerprint
and Engine's native admission. Joined all-feature/all-target types, twelve guards
and all eleven focused native controls pass, including the default graph.
Engine metadata is 1.3–1.8 MB versus about 288 MB; image metadata is 3.8 KB.
Git-only reuse passes: Engine revision refreshes while all three producer outputs
keep their timestamps and digests. Complete workspace/pinned acceptance and task
closure remain open.
See the
[metadata checkpoint](docs/rust-rewrite/runtime-image-metadata-boundary-20261011.md).

## Complete upstream source pin update — 2026-10-11

The whole official Test262 tree and embedded assertion/resource/property
helpers are updated together. Exact ECMA-262 and upstream/content-tree identities
are embedded. All fourteen focused source/host/harness checks, 488 distinct tooling
tests, joined all-feature/all-target types and twelve guards pass. The complete
current census contains 102,994 execution IDs across 53,616 test files. See the [source update](docs/rust-rewrite/conformance-source-pins-20261011.md).
Source-only old-image discovery is not execution acceptance. New Iterator
chunks/windows and Error.prototype.stack tests expose missing product features;
implement their actual algorithms. Keep historical publication counts and the
unfinished task ledger until fresh complete acceptance.

## Async disposal and thenable-job Realm repair — 2026-10-11

The complete affected module passes all ten native controls with zero failures
or ignores, including both backend/script-mode combinations. Resolving functions
use their constructor or running thenable-job Realm; Proxy helpers carry that
execution Realm explicitly for errors and argument Arrays. The disposal oracle
preserves shared List identity, required Await timing and intrinsic suppression.
Joined all-feature/all-target types and all eleven repository guards pass.
The preceding nine-pass/one-failure diagnosis and low-disk link failures remain
in the [repair receipt](docs/rust-rewrite/async-disposal-and-thenable-realms-20261011.md).
Fresh complete workspace and pinned-suite acceptance, task completion and
handoff deletion remain pending. The complete official Test262 and exact
ECMA-262 source pin update is now verified above; implement the newly exposed
Iterator chunking and Error stack accessor proposal features next.

## Native agent message IDs — 2026-10-11

The coherent ID batch preserves arbitrary primitive BigInt IDs independently of
Int32 coercion. Each command owns its primitive data; no GC root crosses Stores.
Nullable canonical decimal byte arrays select the BigInt branch in the GC host
ABI. The sender checks its SharedArrayBuffer before coercion. The receiver roots
both the shared native resource and its own canonical byte array before retrieval
acknowledgement, then constructs a real local BigInt through the existing parser.
Internal broadcast function arity now reflects both arguments. Native and ABI
controls cover exact positive/negative/zero IDs beyond 64 bits, two workers,
separate wrappers, shared backing, observable coercion and malformed admission.

All 22 focused agent controls pass with zero failures or ignores. Joined
all-feature/all-target types and all eleven repository guards pass. The complete
current pinned Atomics selection and broad verification remain pending.
See the [ID checkpoint](docs/rust-rewrite/agent-broadcast-ids-20261011.md).

## Native agent retrieval and parent deadline — 2026-10-11

Broadcast queues every recipient before waiting for retrieval acknowledgements.
Worker retrieval validates the shared backing and roots its local ExternRef
before acknowledging; the worker registry is unlocked during the wait.
Disconnected deliveries retain their join handles and failure owners.
The parent arms one absolute execution budget. Broadcast receipts, worker
readiness and command receives use its remaining time; worker epochs inherit
that budget. Cleanup wakes native sleeps and existing Atomics waiters before
joining workers. Host compilation is checked against the budget before spawning;
its synchronous compiler work is still subject to the outer case supervisor.

All 18 focused agent checks pass, including real shared-memory delivery,
waitAsync notification/expiry, failure preservation, two-worker retrieval and
a ten-minute sleep under a ten-second parent budget. Joined all-feature/all-target
types and all eleven repository guards pass. BigInt message IDs are covered by
the later focused ID checkpoint; fresh complete agent-tree/pinned
acceptance remain open. See the
[retrieval receipt](docs/rust-rewrite/agent-broadcast-retrieval-20261011.md).

## Verification repair batch — 2026-10-11

Complete active Intl payload reproduction passes, including all 1,082
NumberFormat locales. Collator's current emitter identities and dependent
locale-collations binding are refreshed normally. All sixteen current Intl
source/image checks pass. CI checks the consumed timezone native image and adds
the missing Collator check; historical timezone rows/report remain unchanged.
The watcher now wakes immediately on process exit, with all eight focused
cleanup/status controls passing. All 438 tooling tests and joined all-feature
types pass; both repaired IR targets pass all six tests. The controls retain
the parser-range rewrite and typed with/global Reference guarantees. See the
[repair checkpoint](docs/rust-rewrite/verification-repairs-20261011.md).

The all-feature workspace probe was interrupted after 168 complete scopes:
3,359 passes, two IR failures and two documentation ignores. All 217 frontend
and 969 AOT checks pass. Its incomplete engine scope saw 25 invalid-topology
failure markers: auto affinity initialized two compilation workers before later
controls explicitly required one. The same executable proves the conflict and
passes both affected lifecycle controls under one test CPU. Future workspace
drivers retain auto affinity for Cargo compilation, but nest the ordinary
single-CPU cloud launcher around every test. Environment-only worker settings
do not override the library's affinity-derived first default pool. Preserve all
original receipts and archived transcripts. Fresh complete repaired-source
verification, task completion and deletion remain pending.

## Intl source recovery — 2026-10-11

The memoization checkpoint passes 432 tooling tests with zero failures/errors,
including four new NumberFormat memoization controls. Bounded producer caches
preserve exact data, provenance and coverage bytes for eight varied locales;
the isolated extraction measures 15.145 seconds versus 20.705 initially.
Complete pinned NumberFormat reproduction now passes; see the verification
repair receipt above and the
[generator receipt](docs/rust-rewrite/intl-generator-memoization-20261011.md).
The preceding source-recovery checkpoint passes 428 tests.
Restore-and-pin repairs track the exact Unicode17 `UnicodeData.txt` and CLDR47
`scriptMetadata.txt` previously omitted by `*.txt`; byte counts and SHA-256 pins
are unchanged. Intl identity generators bind current production and Cargo.lock,
without changing profile payloads. The original 366-test red checkpoint remains
recorded. See the [receipt](docs/rust-rewrite/intl-tooling-repair-20261011.md).
Complete Rust workspace/runtime, differential and pinned acceptance remain open.

## Worker source reuse and cloud drivers — 2026-10-11

That worker/driver checkpoint passes all 437 tests, including five new
cloud-driver controls. Differential/robustness/closure wrappers accept explicit
`--cloud` and validate the inherited finite machine cap; local 4096 MiB policy,
counts, deadlines and actual native acceptance remain unchanged. See the
[driver contract](docs/rust-rewrite/cloud-verification-drivers-20261011.md).
The supervised worker now executes its discovered one-case manifest without
rediscovery. Both focused worker controls and all four exact case-set structural
controls pass; the persisted source and exact execution IDs are independently
checked. The regenerated shortcut inventory changes three line numbers only.
See the [worker receipt](docs/rust-rewrite/test262-worker-source-reuse-20261011.md).
Complete workspace/ignored/default, differential and pinned acceptance remain
pending. Active Intl payload reproduction is now complete; see the repair
checkpoint above.

## Cargo runtime reuse — 2026-10-11

The Cargo producer shares the bounded Cranelift function cache while regenerating
and admitting the current raw R and publishing the full current compiler-bound
package. Embedded package/native/manifest bytes now each have one immutable
static allocation. Three focused embedded-runtime unit controls and the existing
fresh-program native-R reuse control pass. The complete bundle is byte-identical
across every probe. Actual core precompile measures 120.724 seconds initially
and 37.776 seconds fully warm, with 2,098 hits and zero misses. The earlier
256 MiB function-cache budget evicts the approximately 313 MiB core working set
and supplies no meaningful reuse; the cloud activation now defaults that disk
tier to 512 MiB, preserving stricter explicit settings and the inherited machine
memory/CPU limits. Local defaults remain unchanged.
The [receipt](docs/rust-rewrite/build-runtime-reuse-20261011.md) retains the
low-budget diagnostic, timings, metadata experiment and exact native evidence.
Complete workspace/default-only/ignored-runtime, differential and pinned
acceptance remain pending. Task states and canonical conformance counts remain
unchanged. Do not delete this handoff or the unfinished task ledger yet.

## Cloud checkpoint

The cloud machine has a finite 32 GiB inherited cgroup cap. Heavy work uses
`python3 -B scripts/limited_verification.py --cloud -- <command>`, one CPU/worker,
and a two-entry retained-module cache up to 256 MiB, reduced further to at most
one eighth of the inherited cap. Stricter explicit limits survive. Earlier
checkpoints used one entry/64 MiB. Local machines keep the existing
4 GiB/no-swap/grouped-OOM systemd policy. Rust/Cargo, rustfmt, Clippy and bundled
LLD live in `/workspace/.lila-tools`; source `activate.sh` in each Cargo shell.
Locked/offline fetch/build and default CLI inspect/run/build smoke checks pass.
No Node runtime or system service is a product dependency.

Cloud CPU use remains serial by default. The verified opt-in
`--cloud-cpus auto` mode exposes up to the inherited affinity and visible
cgroup CPU quota (four CPUs here), honoring stricter inherited native-worker
limits. Cargo and libtest remain serial. Direct CLI `--jobs N` selects its
compiler pool; Test262 `--threads` independently selects case concurrency.
Explicit test counts and deadlines stay unchanged. Three real Wasm fixtures
produce identical output with one and four compiler workers and reuse the
embedded native core in both modes; all 28 production launcher controls pass.

The final frozen engine Full10 checkpoint completes all thirteen default-feature
scopes at **3,184 passes, zero failures and zero ignores**, with unchanged source
and original deadlines. Complete CLI Full3 is green at **943 passes, zero failures
and four existing ignores**. The fresh identity-checked product fake run passes **191/191 exact IDs over
190 files**, including all 187 Wasm-safe members, with zero failures/timeouts.
Four isolated cases with one compiler worker each finish in **190.096 watched
seconds** under the inherited four-CPU quota, preserving 60,000-ms case limits.
Final format, architecture, shortcut-accounting and ledger guards also pass.
The exact [fake receipt](docs/rust-rewrite/cloud-continuation-20261009.fake-final.json)
and [guard receipt](docs/rust-rewrite/cloud-continuation-20261009.final-guards.json)
preserve identities, IDs, commands and hashes.

The frozen engine Full4 checkpoint records 3,157 passes, 15 failures and zero
ignores. After the repairs, Full5 completes all thirteen scopes at 3,176 passes,
zero failures and zero ignores, with unchanged source and original deadlines.
Both exact receipts are preserved. Full5 precedes the additional CLI-discovered
BinaryData repairs below; it does not certify those newer changes. Earlier
partial and serial-invalid receipts remain diagnostic history.

The earlier source batch repairs retained binding write policies, suspended
array-pattern scopes, generator `var` pattern hoisting, ordinary descriptor
snapshots, non-simple parameter named-self policies, early `with` global
fallback References and folded RegExp descriptor parity. The serial IR
checkpoint passes all 1,496 tests; the loop/epoch, Float16, descriptor and
ordinary assignment controls pass. Three missing Segmenter corpora are now
tracked with exact committed-archive bytes; eight corpus checks and thirteen
launcher checks pass.

All 15 complete-checkpoint failures now have authored repairs or valid fixture
corrections, with original semantic intent and execution limits preserved.
The batch adds discarded generator Updates, restricted scalar global errors,
canonical shared RegExp alias accessors, lossless UTF-16 literal keys, loaded
computed imports, declared host adapters, a correct structure boundary and CLI
runtime-sidecar cleanup. Arguments configurable snapshots, required RegExp
empty captures and Hebrew YearMonth carrier limits get distinct positive and
negative controls. The first joined all-feature/all-target type check passes in
297 Cargo / 300 watched seconds. The frozen focused run records 1,540 passes and
two failures: a newly added R-free comparison and the original restricted-global
exception constructor diagnostic. The final joined type check passes in
230 Cargo / 240 watched seconds. Both corrections pass their affected controls:
one artifact unit and all seven restricted-global native tests. All fifteen
original failures also pass the complete Full5 checkpoint.

CLI Full1 finishes its driver on unchanged source, but its main target is
incomplete: the wrapper incorrectly assigned 900 seconds to the Test262 subset
whose documented outer stall budget is 3,600 seconds. That target stops with
exit124 after 289 observed passes and seven observed failures. The other sixteen
targets complete at 95 passes, two stale structure-guard failures and three
pre-existing performance ignores, including the full fake publication test.
These counts are separate from a complete CLI verdict.

The next coherent repair batch fixes immutable transfer length coercion,
immutable slice RangeError and pre-allocation source revalidation, and native
waitAsync expiry before notification. Fixture corrections compare isLockFree's
boolean conversion with size1 and use an actual unsupported dynamic-source
case. The verdict structure guard explicitly includes the existing internal
case-worker producer. New native controls preserve error precedence, exact
bounds, clock domains and mixed expired/live FIFO behavior. Joined types pass
in 260 Cargo / 270 watched seconds. All 75 focused checks pass across nine
scopes with unchanged source and zero failures/ignores. Complete final CLI,
engine and identity-checked product fake acceptance were pending at that checkpoint.

CLI Full2 completes all seventeen default-feature scopes on unchanged source:
928 passes, 15 failures and four pre-existing ignores, with no incomplete native
verdict scope. The main target records 832 passes, 14 failures and one ignore;
publication reaches at least 180/191 cases before its original 900-second
deadline. Cloud limits and explicit worker/cache counts were inherited correctly;
available logs do not establish the throughput cause. The timeout remains red.

Root's next batch repairs `in` operand retention/order, runtime private-name `#`
descriptions, strong collection receiver-error classification and exact Intl
identity output. It corrects stale private-extensibility and RegExp-admission
fixtures, bind/global inspection counters and the RegExp source owner guard,
and registers all six missing integration targets in the closed hygiene domain.
Private installation controls retain duplicates, mutation and ordinary
non-extensible/sealed/frozen receivers; new strict/sloppy `in` controls retain
abrupt, coercion, Proxy and suspended-operand order. Initial joined types pass
in 325 Cargo / 330 watched seconds. The first focused attempt passes all 1,499
IR checks, then is deliberately stopped during native compilation when review
finds a missed prefixed private-name string-pool entry. The collector is now
repaired and the existing private-callable fixture adds field-only, uninitialized
static and escaped-name controls. Fresh types, complete focused checks and
final CLI/engine/fake acceptance were pending at that checkpoint.
No deadline or ignore-ledger changes were made.

The private-name correction passes joined types in 258 Cargo / 270 watched
seconds. A separate publication lifecycle review proves two case workers wrote
passing snapshots after the publisher was killed; no cross-scope overlap or
throughput cause is established. The Linux worker spawn now arms a parent-death
signal and rejects a lost-parent race before exec. A subprocess regression kills
a disposable supervisor and verifies its separately grouped worker terminates.
This joined source passes all-feature/all-target types in 7.10 Cargo / 15
watched seconds. The frozen focused9 run completes all 23 scopes: 1,525 passes,
one failure and zero ignores. All 1,524 non-publication checks pass, including
the original fourteen main CLI failures and Linux worker cleanup. Publication
still times out at 900.43 seconds after 190/191 cases. Its deadline remains
unchanged and this checkpoint remains red.

Source tracing finds that each supervised case worker independently hashes its
loaded executable for evidence and again for cache identity. A startup probe
measures one image hash at 0.47 seconds for the 600 MB debug CLI; this does not
prove the publication timeout's cause. The cache now derives a versioned key
from the verified source and loaded-image digests already owned by
`CompilerIdentity`. Unavailable identity disables program/runtime/graph cache
access while preserving compilation. A test-helper lifetime error is corrected;
fresh all-feature/all-target types pass in 161 Cargo / 165 watched seconds.
All 24 focused cache, identity, graph and publication checks pass across eight
unchanged-source scopes. Publication completes in 888.52 seconds under its
original 900-second deadline. The exact receipts retain the earlier red runs.
Complete CLI/engine and the explicit identity-checked product fake run were
pending at that checkpoint. Startup probes are diagnostic only and establish no general speedup.

CLI Full3 completes all seventeen default-feature scopes on unchanged source:
943 passes, zero failures and four pre-existing ignores. Its main scope records
846 passes/one ignore; the 187 exact raw Wasm-safe IDs all pass. Full fake
publication completes in 898.43 seconds under its original 900-second deadline.
The compact receipt names all four ignored stress/performance controls.

The owner's iteration-time request prompts a controlled shared-core probe.
The 41,195,754-byte R Wasm uses a 251,691,352-byte native bundle. Both 1/64-MiB
and 2/64-MiB configurations invoke its native factory six times; 2/256 MiB
invokes it once across the same six executions. Warm R loads measure about
24 ms instead of 130 ms. All three existing native controls pass in each
configuration; total wall timings include persistent-P warming and are not a
general performance benchmark. The cloud launcher now retains R and P within
the measured bounded ceiling; all fifteen launcher controls pass. Rust compiler
and fixture source remains identical to green CLI Full3. Local limits remain
one entry/64 MiB. Future watched commands use `--poll 1` to avoid the default
15-second completion lag. Engine Full10 and explicit product fake acceptance are now green.

## Accepted batch and integration

All thirteen engine scopes, seventeen CLI scopes, joined types, final focused
controls, 28 launcher controls and fresh 191-ID product fake acceptance pass.
Native Rust/fixture hashes are unchanged across final engine and CLI evidence.
CLI has four pre-existing stress/performance ignores; report them explicitly.
Keep the documented 3,600-second outer stall budget for the full main CLI scope
and preserve tests' individual deadlines and explicit worker/cache counts.
Final guards pass; run the actual `check-readme-status-artifacts.sh origin/main`
against the resulting commit before a normal push. The exact cloud install
script and startup instructions are tested and saved through the environment
configuration draft workflow; Review/Publish is its activation step. No task
states or publisher conformance totals are changed by hand.

## Property storage checkpoint — 2026-10-10

The next verified batch gives ordinary named-property tables an explicit logical
insertion extent and geometric capacity. Typed Wasm GC `array.copy` preserves
the rooted live prefix; deletion holes and String/Symbol insertion order stay
intact. Arguments indexed descriptors also grow geometrically with null spare
slots kept absent and the parameter map independently bounded. Rooted completion
snapshots read the logical extent. Lookup remains linear.

All 969 unique backend controls and 44 focused native controls pass, including
GC during a getter, descriptor attributes, deletion/re-addition, sparse Arguments,
Proxy/global-reference behavior and snapshots. The first backend sweep exposed
a stale source guard for already-correct loop binding modes; the corrected guard
and all remaining targets pass. A fresh identity-checked product fake run passes
all 191 exact IDs over 190 files, including all 187 Wasm-safe IDs, in 209.105
watched seconds with the original 60,000-ms deadlines and four isolated cases.
The fixed three-round 4,096-key ordinary/Arguments probe passes before and after;
execution measures 110.275 and 105.292 seconds respectively. These single samples
do not establish a general speedup. Full engine/CLI sweeps above describe the
prior checkpoint, not a fresh complete sweep of this storage change.
See the [storage receipt](docs/rust-rewrite/property-storage-20261010.md).

## Blocked historical replay and remaining work

The requested September 30 aggregate with 5,365 exact failing execution IDs is
absent; the owner believes it was local and never pushed. Do not ask again,
substitute another snapshot or claim the historical replay completed.
Its former path was `target/publication-freeze-20260930-*/test262/snapshots/`.

Indexed ordinary-property lookup now has collision/tombstone handling, shared
UTF-16/Symbol hashing and one entry per key. Its frozen verification passes all
969 backend controls and 193 native controls (including the complete 150-test
GC-entry target), with zero failures/ignores. Fresh identity-checked product fake
acceptance passes all 191 exact IDs, including all 187 raw Wasm-safe IDs, with
zero failures/timeouts. The fixed probe measures 88.424 seconds of execution;
this single sample is not a general benchmark. See the
[indexed-lookup receipt](docs/rust-rewrite/property-index-20261010.md).
Folded HasProperty/Get remains further work.
Geometric ordinary/Arguments table growth is committed as `9c0862ddd`.
Indexed lookup is committed and pushed as `7fedbd9cd`.
The next implemented source batch removes quadratic ordinary/exotic own-key
construction. Unique, disjoint physical key owners append directly; canonical
numeric indices use the existing unsigned heap sort before String/Symbol
insertion-order passes. A new native control covers reverse numeric insertion,
high unsigned indices, deletion/re-addition, accessors, empty and single-key
objects. Proxy duplicate/invariant checks and namespace order remain separate.
The frozen source passes all 969 backend controls, 52 focused native controls,
all-feature/all-target types, CLI build and final guards. Fresh identity-checked
product fake acceptance passes all 191 exact IDs, including all 187 raw
Wasm-safe IDs, with zero failures/timeouts. The unchanged probe measures
2.026 seconds of execution versus 88.424 seconds before this enumeration
change; these are single diagnostic samples. No fresh complete engine/CLI sweep
is claimed. See the [own-key receipt](docs/rust-rewrite/own-keys-20261010.md).
Previously recorded compiler
gaps include the vendored parser's parenthesized member assignment target, Bytes
module kind, logical/compound `with` RHS representation and AST-based module
syntax stripping. These are separate follow-up scopes, not results of this batch.
All thirty task states remain four complete, 25 in progress and T26 blocked.

Raw watched logs and private drivers are ignored under
`target/continuation-cloud-20261009/`; the checked-in compact receipts preserve
verdicts, commands, hashes and diagnostics for fresh checkouts. Check disk before
broad scopes and clear only completed incremental state between serial targets.

## Parser/module range batch — 2026-10-10

The verified batch admits parenthesized simple destructuring assignment
and rest targets through the existing assignment-target authority. Module
syntax now comes from the original parser's item ranges, stored on source-text
records; the second lexer and default-declaration reparse are removed. Controls
cover negative targets, receiver/order semantics, UTF-16/ASI/import attributes,
regex ambiguity and anonymous declaration boundaries. Dynamic call heads also
come from the original parser and survive earlier source insertions; Script
discovery uses its retained AST. The fallback scanner is confined to conservative
parse-failure probes. The 2026-10-11 frozen checkpoint passes 217 frontend
checks, 1,500 IR unit checks, 11 IR integration checks, all-feature/all-target
types, 38 focused native checks and all 191 exact product fixture IDs over 190
files, including 187 Wasm-safe IDs. There are zero failures or timeouts. Original
case limits and the inherited cloud resource policy remain unchanged. See
[the receipt](docs/rust-rewrite/parser-module-ranges-20261010.md). Complete
workspace/all-ignored, differential and current pinned-suite acceptance remain
pending.

The latest task audit still finds substantial open acceptance requirements:
fresh default/all-feature/all-ignored workspace verification, current agent-tree
and stress acceptance, current pinned full-suite repeatable publication,
sustained stress/differential runs and the architecture/integrity audit.
Weak reachability/ephemerons and BigInt message IDs have verified focused
successors. Intl cross-host capture/compare CI exists, but actual supported-host
receipts remain required. Do not remove
this handoff or tasks until the requested actual task completion is established.
