## Current acceptance inventory — 2026-10-20

[Current capabilities, builds, tests and remaining acceptance](current-inventory.md).
Current source has 7,560 native passes and one timeout in 49/113 closed AF
scopes; discovery stopped and old cloud coordinators are absent. The 624
tooling checks and all 12 ordinary libraries (3,865 tests) pass. Later gates
remain unstarted. Dated checkpoint notes below are historical, not live status.

All-feature discovery is complete at compiler source `867ad6145`:
**9,649 native passes, one timeout and 11 documentation passes across 113 scopes**.
Every failure and its executable are preserved. One subsequent idle diagnostic
passes the same five-second request, but establishes no repair and does not
replace the suite failure. Default, release, conformance and cleanup remain open.

The verifier now admits the exact whole fake-suite publication target to its
opt-in cloud CPU budget, keeping its idle barrier, original jobs/worker count
and deadline. All620 Python tooling controls pass, including real separate-device
storage checks. Native publication speedup remains unmeasured. An Intl startup
comparison linked the existing optimized library in1.99s and retained all28
samples; small, variable median differences did not justify changing production
scheduling. Keep CONTINUE/tasks until every acceptance criterion passes.
[Exact discovery, diagnostic, measurements and tooling controls](complete-af-discovery-cpu-tooling-20261019.json).

Verification checkpoint — 2026-10-19: twelve more whole scopes pass
1,185/0/0, bringing the frozen all-feature discovery to57 closed scopes,
7,966 passes and the same unresolved seed9 timeout. Regexp152, Temporal202,
Realm/module/language checks778 and the remaining small scopes are green.
CLI855 is still running. Full AF113, default111, release, conformance and
cleanup remain pending; no original failure has been replaced by a retry.

Closed private diagnostic bytes were archived with full mode/size/SHA readback
before retirement, recovering514MB of workspace space and1.03GB of tmpfs RAM.
Normal Cargo/Intl/D6/shared caches, sources, fixtures and literal outcomes
remain. Passive CLI timing samples about three CPU cores of the four-core
quota; a fourth partition is a future scheduling opportunity, with no speedup
claim or change to the current run. Keep CONTINUE/tasks until acceptance.
[Exact native scopes, preserved storage and passive measurements](discovery-storage-checkpoint-20261019.json).

Intl verification checkpoint — 2026-10-19: the whole Intl target passes all
299 IDs, zero failures or ignored tests, in 2,288.55 seconds. Four further
whole scopes pass 1,171/0/0; the frozen discovery snapshot now records 45
closed scopes with 6,781 passes and the original seed 9 timeout. Full discovery
continues; default, release, conformance and cleanup remain pending.

All 55 Intl fixture setup calls explicitly selected one compiler thread,
despite the runner allocating four CPUs. A separate test-only candidate honors
positive inherited SCRIPTW_JOBS, capped by available parallelism, and stays serial
when unset. All 299 IDs, assertions, fixtures and deadlines remain. Product
sources and runtime fingerprint inputs are unchanged. The candidate is saved
locally, uncompiled and unpushed; its speed benefit is not measured.

One original-budget idle replay is prepared after discovery and full failure
preservation. It uses the exact archived binaries and request, with read-only
observation of existing trace/journal files. No original case has been rerun.
Keep CONTINUE/tasks until every acceptance criterion passes.

[Exact native outcomes, prepared candidate and diagnostic controls](intl-native-budget-candidate-20261019.json).

Matched native checkpoint — 2026-10-19: the same 41 scopes and 5,611 test
IDs now record 5,610 passes and one failure, compared with 5,599 passes and
12 failures at the previous source. All 12 pool setup failures now pass.
One previously passing mutation case, seed 9, timed out at its original
5,000 ms deadline. Its exact request and worker bytes match the successful
cold check; that earlier pass did not establish reliable timing margin.
Every original ID and literal result is retained.

Full all-feature discovery is still running at source 867ad6145. Fresh default,
release, conformance, performance and task cleanup remain pending. Default
verification starts only after green all-feature results. Keep CONTINUE/tasks.

A separate, uncompiled candidate replaces per-element GC String extraction
with one checked packed-i16 read and records extraction timing. It preserves
the original six native selectors, adds assertions for every UTF-16 value,
empty strings, collection, array types, lengths and expired roots, and passes
all 14 source audits. Its correctness and timing still require ordinary native
verification. Neither a timeout repair nor a performance gain is established.
Follow-up measurement: the old host extraction loop takes a median 26 ms
for 65,536 code units in a standalone seven-sample diagnostic. All samples
and the slow outlier are retained; no original worker case was rerun. This
makes bulk copying a secondary optimization and does not justify restarting
broad verification by itself. Finish the current discovery, preserve its
failures, then measure the original request once while idle before choosing
a substantial repair. The original failed worker image is fully archived
and verified, with its live file retained.

[Exact matched outcomes, original timeout and prepared candidate](matched-native-progress-20261019.json).

Verified repair checkpoint — 2026-10-19: source867ad6145 is pushed to main.
All14 source audits and all12 affected whole AF libraries pass3864/0/0.
Every prior library ID remains; all12 previous pool-setup failures now pass,
including complete Test262476 and CLI82. Both repairs use normal pool setup
and retain the original four-CPU budget and assertions.

All four original cold requests also pass with fresh caches, four jobs and
the unchanged5000ms deadlines. Earlier red outcomes remain preserved. Host,
cache and other phase timings vary, so no causal whole-product speedup is
claimed. Full AF113 revalidates these12 scopes once and runs the remaining101;
fresh default111 follows green AF, with no AF outcome carry. Release230,
conformance, performance and cleanup remain pending. Keep CONTINUE/tasks.
[Exact audits, native IDs, cold inputs/outcomes and tooling](bounded-digests-pool-checkpoint-20261019.json).

All-feature discovery checkpoint — 2026-10-19: the frozen71 source has
completed12 whole library scopes with3852PASS/12FAIL/0IGN. Engine973, IR1607,
AOT442 and Front177 pass. Test262475/1 and CLI71/11 expose two test setup
conflicts with the process-wide compiler pool. Both small test-only repairs
are prepared; neither has been compiled or accepted. Keep the original four-CPU
budget, every native ID and every assertion for their verification.

The verifier now checks four exact default-only tests against their explicit
oracle-off source attributes. The continuation revalidates all12 completed
scopes without repeating native IDs, then runs the remaining101 scopes.
Failed executables, their shared image/producer and literal logs are archived
with full mode/size/SHA readback. The two archived failed test executables were
retired to recover link space. Passed IR/Intl Cargo output paths now stay warm
within256MiB; successor results still require ordinary Cargo and fresh tests.
Full AF/default, the original cold timeout, conformance and cleanup remain open.
[Exact native outcomes, cfg sources, prepared fixes and continuation](all-feature-library-discovery-20261019.json).

# Remaining acceptance after compiler-fact verification — 2026-10-19

Verified repair checkpoint — 2026-10-19: source71bced27c is pushed to main.
All14 source audits and eight whole ordinary scopes pass3148/0/0. Both original
baseline failures now pass: the timeout control within Test262438/0/0 and the
original CLI ID through one-shot and reused workers (one filtered control;
whole CLI remains pending). The matched thousand-method sample passes its
original60s budget with49,400,430→20,429,399 allocations, a58.6% reduction.
This is a fixture measurement; broad performance acceptance remains pending.
Intl646/0/0 took335.60s in one process; the old644 used three partitions in
109.506s, so those wall times do not establish a regression. The broad schedule
retains partitions. Original fresh5000ms/four-job cold4 remains3PASS/1TIMEOUT.
Full AF113 is running at frozen71, with fresh default111 after it. Keep all
literal reds and CONTINUE/tasks until every acceptance criterion passes.

The separate next candidate bounds both unchanged embedded-image SHA256 checks
to at most two threads within the explicit CPU/affinity budget, adds timing
traces, and retains all source/profile/checksum/runtime/Wasmtime admission.
It is uncompiled and unaccepted; its benefit is unmeasured. The primary remains
frozen while broad discovery runs. A separate artifact-only watcher tries to
retain at most256MiB of passed IR/Intl libtests before their owner retires its
paths; it never supplies native outcomes or modifies Cargo fingerprints.
[Exact checkpoint, raw evidence and prepared candidate](focused-repair-and-startup-candidate-20261019.json).

Test-helper visibility correction — 2026-10-19: the isolated identity
scaffold had exposed a private temporary-path helper and therefore missed the
real sibling-module access error. The existing cfg(test) helper is now
crate-visible; its body and product behavior are unchanged. The corrected
isolated probe compiles the exact helper and passes3/0/0. The af10 IR scope
finishes1607/0/0 with full Cargo/resource proof; its warm artifact and outcomes
are preserved. Nine waiting successors stopped before native admission.
The corrected source still requires ordinary focused validation before main.
[Exact correction and preserved IR evidence](suite-output-visibility-correction-20261019.json).

Prepared suite-output identity repair — 2026-10-19: the original CLI scope
records847PASS/1FAIL. A successful child recomputed a journal-contaminated suite
pin and wrote a different snapshot filename. The candidate carries one explicit
owned output namespace through both worker protocols; all other source inputs
remain hashed. Tracked suite inputs, harness overlap and contaminating scratch
destinations reject. Three isolated std-only controls pass; full product
validation remains pending. The existing CLI regression retains nested snapshots
and exercises one-shot and reused workers. Baseline continuation validates59
completed scopes (8742PASS/2FAIL), then runs only52 unexecuted scopes. Both
literal failures and original images remain preserved; no native IDs repeat.
[Exact failure, source and isolated controls](suite-output-identity-candidate-20261019.json).

Prepared-source provenance correction — 2026-10-19: the read-only3c check
rejected stale generated DateTime identities. All13 recipes were genuinely
refreshed in producer order and independently rechecked; ten generated
identity/provenance files changed. The active1c native discovery continues.
Only ten unstarted successors were superseded, with zero native outcomes lost.
The corrected descendant still needs14 source audits, all eight ordinary
focused scopes and the original allocation sample before main publication.
[Exact rejected preflight and generated correction](token-provenance-correction-20261019.json).

The completed prepared batch now passes all14 original source audits and
all23 identity controls. Receiver projection and its old-expression comparison
share one consumed helper; the original lowering parent stays below its810-line
limit. Closed proof admissions bind exact bytes, and shortcut inventory/status
are genuinely regenerated after the timeout helper moved source locations.
Ordinary focused validation and allocation measurement remain pending.

Prepared lowering repair — 2026-10-19: historical allocation stacks identify
2,110 early receiver and 2,118 late parameter signature-copy samples. The
projection source is byte-identical at that sample and current frozen source.
The candidate borrows only needed receiver facts and skips the late lookup for
parameterless functions. Test builds compare receiver facts against the old
expression; stored facts and nonempty parameter handling remain unchanged.
Current allocation savings and ordinary native correctness are unmeasured.
Run complete IR and the original method fixture before accepting or pushing.
[Exact attribution and candidate](signature-projection-candidate-20261019.json).

The prepared timeout/DateTime batch retains the original50ms deadline; isolated
schema3 passes, whole raw-profile values match, and parse medians are146.994ms
before/81.907ms after (phase only). Ordinary Cargo acceptance remains pending.
[Exact code, tests, measurements and superseded queue](timeout-token-candidate-20261019.json).

Current frozen source `1c302df9f` passes all 3120 affected default IDs across
six complete scopes: AOT 442/modules74/IR 1607/Engine972/templates 21/cache4.
Zero failed or ignored; all3115 prior IDs plus five controls remain. Both
original60s template failures and original raw-Unicode staging pass. Exact
bounded compiler CPU, serial test/build, Cargo/ELF and recursive graph proofs
are sealed before retiring passed closed harnesses. Full111 default discovery
carries these six and executes the other105, exact9549native+11docs. Fresh113
all-feature discovery will run the prepared repair source after current default
discovery and ordinary repair validation. The unstarted old-source waiter is
superseded with zero outcomes lost. Retain exact IDs and measure actual inventory. All broad gates
remain unaccepted until their own receipts complete.

Original cold4 stays3PASS/1original5000msTIMEOUT; whole35 stays33PASS/2FAIL,
with every earlier ID and each literal red retained. These variable timings
and allocation 49,390,823→49,400,430 establish no causal speedup or allocation
reduction. All 14 source audits and the ordinary CLI/worker build pass965.861s.
Keep CONTINUE/tasks until every acceptance requirement passes.
[Complete3120 proof](iteration-default-scopes-20261019.json).
[Exact earlier timeouts](iteration-native-checkpoint-20261019.json).

Previous native-mode source `be26faf2e` passes997 Engine IDs but records
30PASS/5FAIL across whole35 and3PASS/1TIMEOUT across cold4. These historical
reds remain unchanged. [Prior Engine/worker proof](large-program-mode-engine-workers-20261019.json).

Prior CAB compiler/worker source is `cab9f9f2ec6ec138a67123f7135bb0be8b244c02`.
Complete affected2146PASS/0FAIL/0IGNORED includes four whole scopes/2144IDs and
filteredIntl2. All2139 prior whole IDs/2137 passes remain; five added controls
pass. Both original60s template failures pass in the complete ordinary scope.
[Exact current outcomes and diagnostics](shared-facts-native-cold-checkpoint-20261019.json).

Cold4 is separately complete-red:2PASS/2original5000msTIMEOUT, four actual jobs
and fresh normal caches. Object-mutation seeds9/10 fail; seed10 previously
passed, so its regression remains open. Temporal/module-graph passes remain.
Complete whole35 now passes with every prior35 ID/pass retained. Recursive
worker graph and broad gates remain unrun. The first
handoff rejected before cases; its corrected immutable-receipt continuation
reused the actual successful Cargo build and completed same-source14/616/2146,
retaining every rejected receipt. Actual products and sharedR are archived and
byte-verified. No green sealer or prepared broad successor was launched.

Matched allocation calls49,387,675→49,390,823 establish no reduction. Separate
warm/fresh diagnostics pass but replace no cold failure. Finish the measured
cold repair and all four original requests, then rerun affected whole scopes
and the recursive graph against that actual successor.
The original failed receipts/images must survive every future overwrite.
All616 Python controls and14 source audits pass for this compiler tree; they
supply no missing broader acceptance.
[Exact35 and bounded optimization experiments](cold-start-hills-and-workers-20261019.json).
The successor separates program128KiB and runtime1MiB native mode limits.
Its smaller private program image is measured; actual cold acceptance remains red.

Prior252 complete default discovery records9542PASS/2FAIL and11docPASS across
all111 scopes. The last complete green default at `2d719` records9538 native/
11docPASS, with ordinary release/original-four evidence in the
[ordinary checkpoint](runtime-type-suffix-ordinary-checkpoint-20261019.json).
Prior-source results supply no current-source acceptance.

## Native suites and retention

Complete all111 current-source default scopes even while the separate cold
performance gate remains red.
The current inventory contains9549 expected native and11 documentation IDs; a successor
must retain every prior ID and account for added controls explicitly. Carry
only proven complete same-source/profile scopes with exact Cargo/graph and raw
outcome evidence. All four CAB affected whole scopes pass at their prior source;
filteredIntl2 and unrun worker targets never enter the default denominator. A disk
deferral has no native outcome; retain its receipt without inventing outcomes.

Then run all 113 all-feature scopes with freshly validated integration Cargo
graphs, followed by ordinary default and all-feature all-target checks. Run
the complete original 230-ID replay once, with one worker, the original
60-second execution/300-second compilation budgets, and fresh four 64 MiB
cache tiers. Compare each of the prior 219 successes by exact execution ID.
Record all outcomes, including literal fixture/spec conflicts, without
selectively replacing failures with separate passes.

Use the finite inherited cloud cgroup budget. Cargo and libtest stay serial;
bounded native partitions and code generation use only the verified cloud CPU
opt-ins. Keep the shared runtime, normal dependency graph and admitted exact
artifacts. Retire a passed test executable only after its proof is sealed.

## Mechanics and idle measurements

After those owners drain, verify the same real `harness` inventory serially,
in two disjoint shards, and through an interrupted/resumed family. Compare
exact IDs, outcomes and normalized details. Preserve the genuine partial
checkpoint before resuming; signaling is limited to that owner's process
group. Existing fake-suite/native controls do not replace this real check.

Use the ordinary default release executable for idle `performance report`,
`compile` and `runtime`, seven samples each, with the original 1s/5s/5s gates
and all 20 compile/runtime fixtures. Preserve red measurements. A candidate
measurement is not a calibrated policy; policy acceptance still needs admitted
compatible baseline/candidate reports and actual calibration.

## Debug and optimized oracle campaigns

After fresh worker verification, capture the actual non-test, `spec-exec-oracle` debug CLI in
`target/shared-facts-oracle-debug-admission-20261019-run1`. Its successful
all-feature integration Cargo artifact event and verified executable bytes bind
the copy. This avoids an extra debug Cargo build. Image
capture alone does not prove the oracle works.

For each profile, use the first complete 14-case `differential replay-corpus
--oracle spec-exec` round as the explicit oracle gate. Complete three fresh
rounds and compare every retained relative path and byte within that image.
Then run the unchanged nightly drivers: 11 grammars times 64 seeds (704), and
19 robustness targets times 64 seeds (1,216), preserving the original
5,000/300,000 ms requests and every literal red result. Debug and release
identities are separate byte-comparison baselines.

After every ordinary-image owner and the debug campaigns finish, preserve the
ordinary release ELF, all its Cargo hardlink aliases and the frozen230-replay
image. The completed replay may share a verified inode with the ordinary
image. Before an oracle build, either detach a byte-verified replay copy or
preserve and unlink every Cargo alias under the Cargo lock, leaving the frozen
ordinary inode unreachable by Cargo. Recheck every remaining alias and byte;
an ordinary replay path alone does not establish independence. Build only the
oracle feature in the already warmed
`target/release-cli` directory:

```sh
python3 scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  cargo build --release --target-dir target/release-cli --locked --offline \
  -p scriptw-cli --bin scriptw --features scriptw-cli/spec-exec-oracle --jobs 1 \
  --message-format=json-render-diagnostics --timings
```

This reuses ordinary release dependencies; oracle-only Boa dependencies still
need compilation. Admit the actual `scriptw` bin artifact with
`profile.test == false`, `profile.opt_level == "3"`, the oracle feature and
unchanged source. Independently copy/hash that executable before the optimized
campaigns. Never edit Cargo fingerprints or reuse debug artifacts as release
dependencies. Keep the frozen ordinary image for later product conformance.

## Two current-pin families, with one also supplying T01

T26 owns two independently fresh, unfiltered full families. Once the source
is ready for this gate, use the actual ordinary product image:

```sh
SCRIPTW_BIN="$PRODUCT" CLOSURE_EVIDENCE_DIR="$OUT/closure" \
  bash scripts/check-test262-closure.sh --cloud
```

The default is one worker; the existing explicit cloud `--threads 4` route
uses at most the machine quota with one compilation job per worker. Keep
60,000ms per execution, without resume or outcome carry. The explicit
`--reuse-case-workers` option is supported by the current closure contract;
each request receives fresh Realm, Store and GC state, and each worker
handles at most 64 requests. Both full families still execute afresh. The current source-pin census contains 102,994 mode-aware executions.
Both complete result maps must agree and contain zero nonpasses for T26 to
pass. Explicit unsupported dynamic-source cases remain nonpasses.

One of these same completed families can supply T01's baseline, deterministic
backlog and publication. This avoids a third full execution. The source owner
`conformance_closure::run_fresh_family` produces the ordinary verified aggregate;
`load_publishable_aggregate_summary` and `generate_backlog` consume that generic
complete current-image evidence. Neither loader imposes a separate run-owner
label. A complete red family can satisfy T01's baseline requirement, while the
closure verdict remains rejected. This does not reduce T26's two fresh families.

Use the exact `first.aggregate_json` and `first.family` from `closure.json`
(normally `closure-first`), or the corresponding second family. Preserve/copy
the whole `first/` directory, with all relative filenames, bytes and modes
unchanged; there is no extra `snapshots/` subdirectory. Verify the complete
source/copy inventories before consumption. Do not rename or relabel evidence.

Run `test262 generate-backlog` twice against that same image and family in an
owned lexical whole-suite view, as described in the older
[backlog recipe](remaining-acceptance-eb1ca63c9-20261015.md#small-remaining-runnable-batches),
then compare the JSON/text pair byte for byte. For publication, stage the exact
family in the publication worktree's `test262/snapshots` and use the actual
`test262 publish-status --snapshot-dir ... --snapshot-name closure-first
--readme-path ...` command. It writes the canonical status JSON/text pair and
generates the README status block. Preserve the referenced aggregate family;
do not hand-edit counts. Older EB-source outcomes do not carry into this source.

## Historical, supported-host and final gates

The immutable 5,365-ID replay remains separate, against historical Test262
tree `aa55200d1310384c5cf69ea95b2a2ecba457007b`. Preserve its genuinely absent
import-bytes fixture and resulting literal outcome. Do not substitute the
current vendored tree or prior EB-source outcomes.

The current hosted Intl workflow completed its jobs, but artifact retrieval is
blocked by the active cloud network policy. A saved draft does not change that
policy. Retry the ordinary download only after the required artifact host is
actually allowed; validate both actual host payloads with the cross-host checker.
Job metadata is not cross-host acceptance.

Keep `CONTINUE.md` and `tasks/` until every task criterion, literal conformance,
calibrated performance and supported-host gate passes. Publish verified
milestones to main as they close; no queued gate is a completed result.
