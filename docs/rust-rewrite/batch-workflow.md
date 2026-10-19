# Batch workflow: write concurrently, verify once, triage afterwards

Engine `aot_intl` test setup honors the runner's bounded `LILA_JOBS`, capped
to actual available CPUs. An absent setting selects one job; invalid settings
fail. This repairs55 hardcoded serial setup calls while keeping the whole299
inventory and all fixture/assertion/deadline checks. Native validation of this
setup repair is pending.

## Opt-in native compiler CPUs for complete targets — 2026-10-19

The workspace runner normally pins each native executable to one CPU. An outer
`--cloud-cpus auto` does not change that child policy. To let an exact complete
target use the inherited quota for Wasmtime and code generation, add
`--native-target-cpus auto --native-cpu-target lila-engine:lila_engine` to
`scripts/run_workspace_tests.py`, launched under
`scripts/limited_verification.py --cloud --cloud-cpus auto`. Repeat
`--native-cpu-target PACKAGE:TARGET` for additional complete scopes and include
`--include-ignored`. The default remains one CPU per native executable.

These targets run exclusively after other native work drains. Cargo and
libtest remain serial, with the original executable command, inventories,
deadlines, cache limits and watcher allowance. The child resource receipt binds
actual affinity, native/Rayon jobs, Cargo jobs and libtest threads. The opt-in
rejects filtered targets, documentation, timing/campaign barriers and
partitioned targets; the separate differential CPU option retains its policy.
The exact whole `lila-cli:fake_suite_publication` test is also eligible: add
`--native-target-cpus auto --native-cpu-target lila-cli:fake_suite_publication`.
It remains one idle-exclusive libtest scope. Its two existing case workers
share the inherited CPU budget; explicit CLI `--jobs 1`, worker count and
900-second deadline stay unchanged. This opt-in has focused scheduler coverage;
a native speedup has not yet been measured.
Continuations must retain the exact selected targets and CPU policy. This
changes scheduling only; native results and elapsed-time gains need actual
verification at the selected source.

## Pending literal-eval and compiler-profile batch — 2026-10-19

The candidate reuses a stateless real Wasm factory only after a typed proof of
one inert scalar expression from normally parsed/lowered direct-eval IR. Each
call supplies its own scalar tag/bits; the context retains no source value or
declaration state. The original11,000 distinct-source and1,000-method tests keep
their60s limits. Four new native controls cover proof rejection, kernel shape
and per-call values. Compilation and complete affected scopes are still pending.

Five exact third-party package overrides optimize wasmtime, its Cranelift
backend, cranelift-codegen, cranelift-frontend and regalloc2 in dev/test. Cargo
graph inspection confirms opt3/checks-off for their ten compiled units, while
compiled workspace crates and wasmparser retain debug and overflow checks.
Normal Wasm validation, features, quotas, deadlines and cache capacities remain.
All13 Intl identity recipes were refreshed and rechecked. This narrower change
must be measured separately from the earlier whole-graph profile comparison;
new Cargo fingerprints must fill a new native-cache generation. Do not relabel
retained artifacts or infer a speedup from the earlier1.77x ratio.

Finish static controls and freeze the coherent candidate before its first
Cargo build. Drain the current full-default discovery and preserve failed
images before advancing its source. Use the verified opt-in idle-build mode,
then run complete affected native scopes and the original cold requests before
fresh default/all-feature acceptance. Old-source outcomes establish inventory
retention only. Keep CONTINUE and tasks until all literal acceptance gates pass.

## Diagnose native source compilation — 2026-10-18

`LILA_NATIVE_SOURCE_TRACE=1` enables the existing execution-local timing trace,
bounded to 256 callbacks. Each traced module lookup now also reports the
in-memory module-cache result and Cranelift function-cache hit/miss deltas. The default path does
not obtain the cache or read those counters. A missing function cache is reported
as unavailable rather than as zero activity.

The counters are process-wide: other concurrent compilations can contribute.
Use a single isolated case worker for attribution and preserve the original
case deadlines and bounded cache settings. An in-memory module-cache miss can
still hit Wasmtime's disk module cache or reuse compiled functions, so it does
not by itself measure native code generation.
The phase includes validation, cache reads and compilation; combine its duration
with the counter deltas before choosing an optimization. Instance reuse can
bypass module lookup entirely. Diagnostic output does not establish acceptance.

## Opt-in Intl incremental scratch — 2026-10-18

Set `CARGO_INCREMENTAL=1 LILA_TEST_KEEP_INTL_INCREMENTAL=1` alongside
`--no-test-incremental` to retain Cargo's existing incremental argument for the
normal `lila_intl` library. The cloud activation script defaults Cargo incremental
compilation to zero, so both environment variables are required there.
The wrapper still removes incremental scratch from every libtest and other
normal crate. It adds no incremental argument to release builds or queries.
The runner records the exception; existing low-disk pruning and the two-GiB
build reserve still apply. Budget roughly900MiB per retained Intl unit after
several edits; different Cargo dependency graphs can produce separate units.

A private opt2 compile-only experiment used the exact ordinary Intl dependency
graph on two admitted cloud CPUs. Ordinary A/B builds took98.53/127.38s;
initial incremental A took126.95s and an identity-only edit B took11.54s.
The opt-in wrapper then undid/reapplied the edit in13.41/13.65s. Four linked
consumers confirmed equal real provider identities for ordinary/incremental A
and ordinary/incremental B, and different identities between A and B. All90
workspace-tooling controls pass, including14 wrapper controls.

These measurements include concurrent native/source verification; they do not
establish a full Cargo, native or CLI speedup. Rustc retains its normal implicit
codegen-unit choice for each incremental mode. Private changed provenance was
never admitted to product execution. The complete source/data copy was verified
against clean source112a765c1 after restoring the private edit. See the
[lossless experiment](intl-incremental-identity-20261018.json).

## Opt-in bounded Rustc code generation — 2026-10-18

Set `LILA_TEST_CODEGEN_JOBS=auto` when launching the cloud verification runner
with its shared runtime linker. The existing compiler wrapper then gives each
serial Cargo compilation a private LLVM jobserver: one implicit slot plus one
pipe token for each additional admitted CPU. It validates the inherited finite
memory cap and CPU quota, preserves inherited affinity limits, requires
`CARGO_BUILD_JOBS=1`, and preserves all compiler arguments. Queries and the
default mode retain the original environment. Native tests keep their original
worker counts, deadlines and cache limits.

For the bounded workspace command below, prefix its invocation with
`LILA_TEST_CODEGEN_JOBS=auto`; keep `--cloud-worker-cpus auto` so the coordinator
can assign the available CPUs. This is an explicit scheduling option and adds
no optimization flags or Cargo artifact profile.

All 11 controls pass. Two fresh 1,024-function Cargo fixture builds match an
independent Python checksum. Switching the option in either direction reuses the
same Cargo artifacts with unchanged output bytes, inodes and timestamps. The
fixture ran alongside product compilation; its timings do not establish a Lila
rebuild speedup. See the [lossless fixture](bounded-rustc-codegen-20261018.json).

### Choose LLVM capacity before compilation — 2026-10-19

For expensive uncached compilation, add `--idle-builds` to
`scripts/run_workspace_tests.py`, together with `--no-test-incremental` and
`LILA_TEST_CODEGEN_JOBS=auto`. The runner drains its owned native tests before
an uncached Cargo build, then uses the existing bounded idle compilation
command. Already built scopes may still execute concurrently. Receipts record
the option and each drain; continuation requires the same policy.

Widening affinity after Rustc starts does not ensure more LLVM workers. The
16-unit probe started on one CPU and widened after0.4s: old-wrapper walls were
13.33/10.78s at about one average CPU. A private dynamic-token prototype stayed
near one CPU at10.89/10.41s and was discarded. Starting the original wrapper
with four CPUs took5.58/8.81s at about two average CPUs. An initial single-unit
fixture is retained as a negative control. All99 workspace and37 cloud-budget
controls pass, including actual scheduling-flow checks for uncached opt-in,
cached opt-in and the default overlap. A real plan-only invocation retains its
selected scope. This is a small compiler probe, not a measured full Lila build
speedup. [Exact source, commands, outcomes and controls](idle-codegen-startup-capacity-20261019.json).

Current native-cost observations — 2026-10-19: the complete four-scope cohort
passes1,171 IDs, including all299 Intl tests. A passive observer records250
conservative stdout visibility spans from the completed Intl scope; several
custom projection tests span roughly190–337s under the actual concurrent run.
These are scheduling observations with censored/coalesced output, not exact
internal phase timings. Do not change active partitions based on partial data.

Eight fresh single-CPU `Module::new` probes use the exact current557,768-byte
mutation-seed9 program and existing normal Cargo dependency artifacts. The
dev/test median is2.5093s and retained release-dependency median1.4195s, with
identical features and product Fast settings. Dev/test is opt2 with assertions
and overflow checks; release is opt3 without them, so individual causes remain
confounded. The older release artifacts supply diagnostic dependency provenance,
not current release acceptance. Other-CPU native work continued and only the
debug Cargo target was locked. No generated module was executed, no product
profile changed and the original5s cold timeout remains red.
[Lossless cohort and cost evidence](native-cost-hills-20261019.json).


## Preserve current provenance timestamps — 2026-10-18

`python3 -B scripts/intl_source_identities.py --write` checks each producer before
regenerating stale output, then rechecks all 13 recipes. A current producer keeps
its output bytes and timestamps, including when a downstream identity needs a
refresh. This avoids unnecessary Cargo invalidation from unconditional writers.
Five focused controls and an actual 13-recipe no-op refresh pass; all 5,858
checked-out tracked files retain bytes, modes, inodes and modification times.
This does not measure a Cargo build-time saving. See the
[lossless check](intl-idempotent-refresh-20261018.json).


## Bounded full-workspace linking — 2026-10-18

The default metadata inventory currently contains 98 native targets (13
libraries, 84 integrations and one binary) plus 13 documentation targets.
Discover these from Cargo metadata on each run; the all-feature inventory adds
its required-feature binary. A selected core run does not cover this inventory.

On machines that cannot retain all integration ELFs, run:

```sh
. /workspace/.lila-tools/activate.sh
python3 -B scripts/limited_verification.py --cloud --cloud-worker-cpus auto -- \
  python3 -B scripts/run_workspace_cohorts.py \
  --output target/workspace-default-fresh --cohort-size 8 --test-processes 4 \
  --isolate-test-package lila-intl \
  --partition-native-target lila-engine:lila_engine \
  --partition-native-target lila-engine:aot_realm_modules
```

The outer runner seals the complete inventory before executing any cohort.
Normal libraries run first, then bounded groups of integration/binary selectors,
then documentation. Same-selector targets from different packages stay together.
Intl roots run separately to retain the existing test-feature isolation.
Each child uses the existing source-bound runner, original native inventories,
deadlines, barrier scheduling, bounded caches and shared immutable runtime data.

Every normal native build intentionally keeps the global Cargo `--lib` roots.
Their ELFs remain available for Cargo freshness checks across batches. A proven
zero-test library supplies that selector in integration batches without
repeating meaningful library tests. Auxiliary executions must still inventory
and pass with zero tests and never add to suite counts. If that library gains a
test, the outer runner rejects rather than duplicating it. Cargo remains the
authority for artifact reuse; a stable root set alone is not a speed result.

After a child has exited successfully, the outer runner seals a separate
retirement record, checks exact package/target/kind identity, inventory and
transcript hashes, and executable bytes/profile path, then removes only that
cohort's completed integration/binary ELFs under the Cargo lock. Library ELFs,
raw logs, inventories, source bindings and the child receipt remain intact.
The child honestly records its original preservation; the outer receipt records
later retirement without rewriting the earlier evidence.

A failed or deferred cohort leaves acceptance false and all missing targets
open. Complete acceptance requires every original qualified target exactly
once, with the separately identified zero-test auxiliary omitted from totals.
Planning and fixture controls never count as Lila native acceptance. The outer
coordinator does not yet offer interruption resume; its child receipts retain
their existing exact-source continuation contract.

## Retain test binaries when disk space permits — 2026-10-17

Add `--preserve-test-artifacts` to the workspace runner to keep completed native
test executables and unselected Cargo siblings. Their exact paths, sizes and
hashes remain in the receipt; unselected outputs have no execution verdict.
Ordinary Cargo decides whether a later invocation can reuse an output. The
default still removes these files, and incremental-cache policy is unchanged.
The six retention controls cover the default, explicit preservation, unchanged
failed verdicts, changed executable rejection and profile-path boundaries.
No measured build-time speedup is claimed from these controls alone.

```sh
python3 -B -m unittest scripts.tests.test_workspace_artifact_retention
```

## Test-only compiler identity inputs — 2026-10-15

The production fingerprint and build-script rerun inventory exclude only
explicit project test paths. Native Function controls and their test observer
are out-of-line `cfg(test)` modules; editing them must not invalidate emitted
program caches. Their production parent module, decoder, factory and other
compiler/runtime/data inputs remain covered. Cargo continues to track and
rebuild the tests themselves through ordinary test-target dependencies.

All twelve existing fingerprint controls pass after extending both the test-only
lifecycle matrix and the production-input invalidation matrix. The standalone
compile took 0.438 seconds and native execution 0.177 seconds under concurrent
suite load. This proves invalidation behavior; the avoided runtime rebuild cost
has not been measured as a controlled before/after speedup. See the
[captured check](native-test-fingerprint-boundary-20261015.json).


## Reuse a captured native CLI — 2026-10-13

For a producer that explicitly captures a private non-test CLI in its owned
native build, staging can reuse that image without invoking another Cargo graph:

```sh
python3 -B scripts/limited_verification.py --cloud -- \
  python3 -B scripts/stage-verification-worker.py --profile release \
  --share-runtime-images --from-native-build-owner target/BUILD_OWNER/receipt.json \
  --output target/verification-worker-fresh
```

The complete build owner must contain `native_cli_worker` with its actual Cargo
artifact, executable/compiler identity, graph and shared-image identity. Ordinary
no-run receipts without that capture reject. The clean project source must still
match the exact owner commit; changing HEAD requires a new build owner. The mode
honors an explicitly shared `CARGO_TARGET_DIR`, uses a fresh output directory,
and rejects changed executable/runtime bytes or mismatched profile/oracle features.
It copies and executes the captured CLI's identity command and seals the actual
worker receipt. It does not rebuild or replace the ordinary product, execute
native tests, or make a default-feature claim about a widened native test graph.
Verify the ordinary product on its own graph. The six admission controls use
fixture bytes; actual native staging has a separate executable-bound owner.

## Complete native workspace runner — 2026-10-12

Run from the repository root after writing the whole batch:

```sh
. /workspace/.lila-tools/activate.sh
python3 -B scripts/limited_verification.py --cloud --cloud-worker-cpus auto -- \
  python3 -B scripts/run_workspace_tests.py --output target/workspace-fresh \
  --all-features --include-ignored --test-processes 4 --partition-cli \
  --shared-runtime-linker scripts/test-shared-runtime-images/cc \
  --strip-test-debug --no-test-incremental --stop-on-build-failure
```

Use the available Rust toolchain activation path on other hosts. Omit
`--all-features` for the independent default-feature suite and its oracle-refusal
controls. By default, every Cargo selector uses the selected workspace package
graph. `--isolate-test-package lila-intl` explicitly gives that package its own
test cohort and excludes its test root from the other native cohorts. This
prevents its test-only ICU `compiled_data` feature from widening those cohorts;
normal transitive dependencies remain present. It changes neither selected
targets nor native IDs. Retry receipts must retain the same isolation setting;
historical broad library carry cannot cross these separate graphs. Documentation
uses a complete normal cohort plus each isolated package's own cohort, sealing
every selected package inventory and retaining completed siblings separately.
Build-time savings require actual Cargo measurements; helper checks alone do
not establish a gain. For a bounded selection, `--build-selected-together`
builds the union of its normal native selectors in one Cargo invocation. For
example, the five-target core selection combines `--lib`, `--test aot_realm_modules`
and `--test lila_ir_integration` on the same workspace roots. This avoids switching
between separate library and integration dependency graphs. Isolated packages
and documentation keep their own cohorts. Same-selector unselected siblings
remain explicitly unexecuted; each selected target retains its original
inventory, execution and verdict. Retry requires the same original exact target
inventory and selector union; dropping a same-selector package sibling rejects.
Continuation inputs must seal that original inventory before narrowing pending work, and
historical library carry cannot cross this build policy. The option can retain
more binaries at once, so use the existing storage preflight and measure actual
build reuse before claiming a saving.

Every qualified native ID is inventoried; the CLI partitioner checks
four disjoint exact native lists before any body runs and retains each original
transcript. Each native process has one CPU and one test thread. Cargo stays
serial. When every active native wrapper exits during a build, the build process
tree can use the quota-bound parent CPU set; only compilation masks expand,
and Cargo-lock-dependent native cleanup waits until the build finishes.
Original-budget campaigns, workers, corpora, performance, publication
and documentation drain other native work first. Documentation collects every
package outcome with `--no-fail-fast`; a failing package remains failed.

The receipt binds source state, actual Cargo artifacts, shared data, inventories
and native outcomes. Completed test ELFs can be removed to keep the run within
available disk after their identity and verdict are sealed. Build deferrals are
incomplete coverage, never passes. A continuation may select only previously
unexecuted scopes from the identical source; preserved pass/fail outcomes cannot
be replaced. Compare exact IDs before and after repairs, recording failures on
new IDs separately from regressions on previously passing tests.

See [measured costs and proposed crate boundaries](verification-costs-20261012.md)
for the current timing frame and the dependency cuts that could reduce rebuilds.

## Fast source-structure preflight — 2026-10-15

The Annex B collection structure module uses only the Rust standard library.
Its existing three assertions can run before any workspace build:

```sh
mkdir -p target/structure-preflight
CARGO_MANIFEST_DIR="$PWD/crates/lila-ir" \
  python3 -B scripts/limited_verification.py --cloud -- \
  rustc --edition=2021 --test \
  crates/lila-ir/tests/annex_b_direct_function_collection_structure.rs \
  --crate-name annex_source_structure_preflight -C debuginfo=0 \
  -o target/structure-preflight/annex-source-structure
python3 -B scripts/limited_verification.py --cloud -- \
  ./target/structure-preflight/annex-source-structure --test-threads=1
python3 -B -m unittest discover -s scripts/tests -p test_limited_verification.py -v
python3 -B -m unittest discover -s scripts/tests -p test_workspace_test_partitions.py -v
```

On this cloud host the standalone structural compile took 0.149 seconds and
all three assertions took 0.017 seconds. Together with 45 focused Python controls,
the owned quick run took 0.453 seconds. It checks the same structural assertions
as the complete IR integration target without linking its dependency graph.
Complete integration and workspace runs remain required for acceptance.

The complete workspace coordinator uses `--cloud-worker-cpus auto`: it needs
all explicitly selected native worker CPUs even when `LILA_JOBS=1` and
`RAYON_NUM_THREADS=1`. Its nested partition coordinator uses the same mode.
Using `--cloud-cpus auto` there would shrink the worker set to the native
compilation job limit. Cargo builds keep their separate compilation policy.

## Explicit cloud conformance workers — 2026-10-15

For complete idle-exclusive differential native targets, opt into bounded
compiler/admission CPUs without increasing case or backend concurrency:

```sh
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  python3 -B scripts/run_workspace_tests.py --all-features --include-ignored \
  --output target/differential-native --test-processes 1 \
  --differential-native-cpus auto \
  --only-targets lila-test262:differential_generated_campaign
```

Existing `LILA_JOBS` and `RAYON_NUM_THREADS` limits still apply. The coordinator
drains independent scopes first and seals actual affinity, native jobs and the
unchanged serial libtest setting. Performance, publication, documentation and
other scopes keep their existing CPU policy. Continuations must preserve this
opt-in setting. External package and disk admission checks every function body
and every existing Intl component; their independent work can be concurrent.
Only the immutable Cargo package can reuse full producer body validation before
native compilation. That private Engine boundary retains runtime identity,
checksum, section, data and layout admission; see the
[safety contract](contracts/immutable-runtime-build-package.md). Without an
explicit job setting, runtime admission work remains serial.

Development/test CLI and Test262 package overrides strip debug information at
the final link. The selected worker still hashes every byte of its own loaded
executable, checks file metadata around the read and retains exact executable
identity in manifests. Stripping removes unused debug sections without changing
allocated code or data; dependency optimization and release line tables remain.
Private-copy measurements reduced the496 MB worker to274 MB. Fresh ordinary
Cargo products at252cbb179 now pass all35 focused native IDs, including all15
campaign tests, plus39 filtered admission controls. Their separate four-request
fresh-cache gate remains three passes/one original5,000ms timeout. Preserve each
complete result and all preceding red observations; broader gates remain open.
[Lossless checkpoint](runtime-admission-checkpoint-20261019.json).

The CLI accepts `LILA_JOBS` as the native compilation default; an explicit
`--jobs N` wins. Differential runners also accept that explicit environment
setting or `new_with_compilation_jobs(path, N)`, otherwise defaulting to one. The selected
worker receives the positive job count before constructing its Engine. This
setting does not increase case/backend concurrency or extend deadlines.
`--cloud-cpus auto` supplies a job count bounded by inherited affinity and cgroup
CPU quota; Cargo and libtest retain their independent serial limits.

For a conformance command that already specifies its worker count, give the
workers the inherited CPU set bounded by the tightest visible cgroup CPU quota:

```sh
LILA_JOBS=1 RAYON_NUM_THREADS=1 \
  python3 -B scripts/limited_verification.py --cloud --cloud-worker-cpus auto -- \
  ./target/release/lila --jobs 1 test262 run --execution-backend wasm-aot --threads 3
```

Keep the command's existing worker count, deadlines and other arguments. This
option selects CPU affinity independently of native compilation job limits;
Cargo, libtest and native compilation defaults remain one. It preserves explicit
CLI arguments and feature flags. It requires `--cloud` and cannot be combined
with `--cloud-cpus auto`, which continues to bound both native compilation jobs
and CPU affinity by explicit `LILA_JOBS` / `RAYON_NUM_THREADS` limits. Neither
option expands inherited affinity or the machine's visible CPU quota.

## Opt-in iteration tooling — 2026-10-11

The Linux x86_64 test linker can share the exact immutable runtime image data
between native test executables. Prefix PATH with
`$PWD/scripts/test-shared-runtime-images` when building tests in
`target/debug/deps`. Product links and doctests use the static archive. Cargo
can reuse an old static test binary because PATH does not change its unit
fingerprint; remove only that completed test executable and rebuild if it defines
the runtime-image symbols and `readelf -d` shows no `liblila_runtime_images_*`
dependency. A test that uses no runtime image needs no shared dependency. The cache key binds the
archive, linker, builder and target; symbol bytes are verified before admission.
Each successful test link records its executable digest and shared-image identity
in `target/shared-test-runtime-images/test-links.jsonl`. Keep the cache while
any such test executable is in use.

Set `LILA_TEST_STRIP_DEBUG=1` with that linker to remove DWARF sections from
those native test executables after linking and before the executable digest is
sealed. Every allocated ELF section must retain its exact bytes, flags, address
and alignment. Global symbols and compiled panic locations remain; Rust
backtraces lose DWARF source lines. This explicit option reduces disk pressure
for broad verification. It does not alter product or doctest links. As with PATH,
the environment option does not change Cargo's unit fingerprint: relink a
completed older test executable before claiming stripping. Never rewrite a
running executable. The linker event records the original/transformed digests,
size, tool identity and allocated-section census.

`lila test262 run --reuse-case-workers` opts into at most 64 sequential requests
per supervised process, sharing compiler/runtime caches. Every case still
creates a fresh Engine, Realm, Store, GC state and native agent ownership.
The parent retains its original absolute attempt budget and exact snapshot,
compiler, source and execution-ID validation. Timeout, crash, protocol error
or infrastructure failure retires the whole process group before the next
request; completed semantic failures remain failures and may reuse the process.
Case worker count, compiler jobs and JavaScript deadlines are unchanged.
The one-shot path remains the default. Full workspace and pinned acceptance
are required independently of these iteration options.

The [verified checkpoint](case-worker-reuse-20261011.md) records 21 native
controls, exact artifact identities and a sixteen-case timing comparison.

## Closure audit and verification — 2026-10-08

The integrated ownership and Unicode batch passes the workspace type check,
2,166 IR/doctest checks (one existing documentation example ignored), 24 focused
AOT controls, two exhaustive Unicode inventory comparisons and twelve native
Unicode/RegExp/URI functions. Repository audits and formatting pass. The native
Realm fixtures now install their missing ordinary Test262 wrapper; assertions
are unchanged. The original five-second campaign still times out during emission.

The task plan remains open: real weak-reference support, unresolved Temporal
rounding, full pinned conformance and canonical publication remain required.
See the [current checkpoint](conformance-ownership.md)
for exact evidence scopes, commands and retained failures.

## Focused iteration checkpoint — 2026-10-08

The fresh all-feature/all-target check and corrected ShadowRealm embedded-import
control pass under the existing 4096-MiB/no-swap/one-CPU launcher. Refreshed source
audit, generated shortcut accounting and task-plan checks pass. Replaying the
original five-second CrossRealm campaign with a rebuilt worker remains red
during Wasm emission; preserve its deadline and evidence for profiling.
The retained native journal is ahead of the older unfinished-queue notes below:
45 of 46 functions passed there, and the remaining fixture now passes its exact
rerun. These runs retain separate source and resource scopes. See the
[ownership and acceptance requirements](conformance-ownership.md)
for results and the next focused step; no fresh full-suite result is claimed.

## Implementation and verification — 2026-10-08

The current source batch joins live prototype/property facts, ordinary global
Reference ordering, removal of source-name and operand-discarding shortcuts,
implicit module await from `await using`, RegExp character domains, Temporal
partial-date formatting, Intl sign placement and selected locale ownership.
ShadowRealm now has native construction, finite evaluation, wrapped callables
and per-Realm module imports. Optional forwarding and finite computed keys feed
the existing source-candidate owner while actual Get/Call execution stays live.
Function-cache hits now update the recency used for bounded pruning.

`tasks-dry-closure-foundation2` passes the all-feature/all-target workspace check
in 67.45 seconds, all 2,135 IR tests across 144 targets, 16 cache checks and both
prepared-source catalog checks. No test failed or was ignored. The complete
checkpoint took 390 seconds under one CPU, 4096 MiB aggregate RAM and zero swap;
retained native modules remain capped at one entry and 64 MiB. Module ownership
checks pass within the existing parent budgets. The generated shortcut inventory
passes with 26 classified observations: 23 harness adaptations, three diagnostic
entries and zero semantic shortcuts. Pinned conformance counts are unchanged.

`tasks-shadow-native2` passed four native gate functions in 1,050 seconds:
construction/subclass branding, finite-evaluation lifetimes, wrapped callable
identity/execution and isolated module caches. The first three cover strict and
sloppy modes. `tasks-dry-closure-native1` then passed the optional-alias cohort in
both modes in 364.18 seconds. It was deliberately stopped during its second
case after 466 seconds (exit 143); seven cases were incomplete or unstarted.

`tasks-realm-bootstrap1` passes in 210 seconds: source guards, the all-feature/
all-target workspace check (60.36 seconds; Cargo: 59.02) and all 16 focused AOT
controls (six helper unit tests, ten integration tests), with none failed or
ignored. The validated optional artifact has main at 534,595 bytes, the shared
initializer at 662,558 and ShadowRealm at 24,632. Every body stays below 1 MiB,
and both bootstrap callers use the same helper. The earlier IR/cache/catalog
and native passes retain their preceding-source scope.

`tasks-dry-closure-native2` received an unexpected SIGTERM after 188 seconds
(exit 143), during the first function's strict-mode compilation. Sloppy namespace
execution passed, but no test function completed. Service accounting recorded a
3.8 GiB peak and no OOM termination entry.

`tasks-dry-closure-native3` ended on 2026-10-08 with 30 native functions
passing, four failing and 19 incomplete or unstarted. It received SIGTERM after
5,153 seconds under one CPU, 4096 MiB and zero swap. Service accounting reported
4 GiB peak without an OOM termination entry; the termination cause is unconfirmed.
The durable result journal preserves every completed outcome. The failures are
public-global binding shadowing, strict global-var assignment after RHS deletion,
and both class-initializer grammar controls exposing duplicate function IDs.

The joined source repair separates source `globalThis` from proven global-object
identity and retains Global Environment References through reads, deletion,
plain/logical/eager assignment and numeric updates. Class-owning functions keep
canonical bodies when specialization cannot remap their complete identity graph;
Wasm planning rejects duplicate function IDs. Source review also adds related
getter, Proxy and late-lexical controls to two existing native cohorts. Obsolete
raw-global mutation IR paths are removed. Arithmetic compound assignment now
consumes its initialization witness before lowering the RHS, preserving TDZ
ordering while initialized const writes still follow operand coercion.

`tasks-global-class-repair2` passed the workspace check and all 48 planning
controls before an unexplained SIGTERM at 477 seconds interrupted full IR.
The joined TDZ/assertion successor, `tasks-global-class-repair3`, finishes in
525 seconds. Its all-feature/all-target workspace check passes in 80.68 seconds;
all 19 affected AOT controls and the CLI inspection control pass, including both
original class artifacts. Full IR completes all 144 targets: 2,141 controls pass,
six fail and none are ignored. `tasks-function-conversion-repair1` resolves all
six stale receiver, effect and dynamic-source assertions: its four exact unit
controls and all five number-constructor controls pass. The workspace check
passes in 81.52 seconds; the whole checkpoint finishes in 120 seconds.

The Function-family repair converts arguments whose count is absent from the
prepared table before reporting the typed source boundary. Its expanded native
cohort passes for all four constructor kinds, including ordered conversions,
original exceptions and exactly-once effects. `tasks-dry-closure-native4` then
received SIGTERM after 460 seconds during the public-global fixture's strict
native compilation; sloppy execution completed, but that function has no result.
The durable journal retains one completed pass and 44 unfinished functions.
Successful unchanged controls retain their preceding-source scope.

`tasks-original-campaign-trace1` reproduces the original five-second failure
without changing its seed or deadline. CrossRealm seed 1270 times out during
Wasm emission, after 5.39 ms parsing and 219.02 ms lowering, before native
compilation or JavaScript execution. SpecExec completes all eight actions;
the transformed case and Temporal seed remain unrun. Retained worker stderr
identifies repeated absent Collator metadata probes. This is performance failure
evidence, not a semantic equivalence result.

The reviewed Collator precheck now rejects impossible locale/type pairs using
that provider's exact metadata inventory and ICU fallback chain. Complete
profile admission and projection recording remain unchanged. Both source
identities are refreshed; all ten focused Collator image/projection controls pass.

The latest source join removes the obsolete rejection of suspending Switch in
an ordinary generator that captures an enclosing With environment. Existing
captured cells, CaseBlock records and retained assignment References own the
composition; the native Switch cohort now covers it in sloppy mode. String-pool
slots also retain checked insertion indices, so later literals cannot renumber
already collected builtin strings, and literal lookup uses direct map access.
The all-feature/all-target workspace check passes in 110.81 seconds, along with
seven IR, ten Collator and three string-pool unit controls. The first process
ended with SIGTERM during artifact checks. Its successor runs only those four
unfinished checks: all pass, including identical shared coercion-helper bytes
across changed source literals. No control fails or is ignored in the completed
24-test scope. Native execution and measured cache-speed gains remain pending.

`tasks-dry-closure-native5` completes the expanded captured-With scenario's
sloppy execution: emission takes 16.30 seconds, native compilation 333.11 seconds
and JavaScript execution 183.77 ms. Its 1,638,171-byte main body selects the
existing size-optimized compiler; strict main is also oversized at 1,201,087.
The run is deliberately stopped after 511 seconds during strict compilation to
finish a shared global-read owner. No native test function completes. Scoped
and persistent-ancestor logs record no OOM event; the 4096 MiB cap is retained.
The 45-function native queue remains open.

The joined successor outlines complete global ResolveBinding/GetBindingValue
through four typed helpers for strict/sloppy reads and typeof. The caller passes
its selected Global Environment and actual execution Realm; the existing
Reference, delegate refresh, TDZ and whole-exception operations remain the
semantic owner. Retained mutation References stay in their original caller.
All 14 affected Intl identity generators and final checks pass without changing
payload bytes. The enlarged original Switch artifact now checks real helper
calls and the unchanged 1 MiB body limit; the existing global-read native cohort
adds foreign-Realm and TDZ controls. Joined verification remains pending.

Known acceptance gaps remain. Genuine weak reachability needs a runtime facility;
arbitrary unavailable dynamic source and Temporal's unresolved collapsed-rounding
policy remain explicit limitations. The original five-second campaign failure,
sustained performance evidence, remaining native cohorts, full pinned conformance
and independent-host acceptance remain open. Historical evidence below keeps its
original scope; this checkpoint does not close task acceptance.

See the [property-fact contract](contracts/primitive-property-read-effects.md),
[source-identity contract](contracts/compiler-source-identity.md) and
[ShadowRealm contract](contracts/shadowrealm-implementation-sequence.md).

## Earlier implementation and verification — 2026-10-07

The `converters1` checkpoint passed the whole-workspace, all-feature, all-target
type check (69 seconds) and all 28 focused AOT controls. Its scheduling fixture
has 15,279,867 code bytes and no body above 1 MiB. Duration.round is 888,220 bytes,
total is 556,512, compare is 343,640 and bootstrap is 863,565. The whole Zoned
converter is 366,181 bytes and the Plain converter is 59,634. Both actual Wasm
artifacts validate. The original lifecycle fixture then completed native
compilation in 80.6 seconds under 4096 MiB, but failed instantiation at the
`agent_can_suspend` import. The complete checkpoint exited 101 after 330 seconds;
it produced no successful native execution result and no memory-limit failure.

The successor separates host signatures with no concrete GC references into
singleton type groups, retaining the one recursive runtime graph and deriving
membership/order checks from the declared operands. It also retains the actual
allocated resource finalizer through source-to-IR ownership and propagates
property-append emission errors. Real JSON records, bounded URI/prelude/loader
robustness targets, and generated cross-Realm/Temporal scenarios are the next
coherent dry batch. Its final source join adds a shared bounded rooted snapshot
schema/traversal, actual Wasm and Boa adapters and explicit differential v7 wire.
Those sources, controls and contracts are joined. The `tasks-json-snapshot5`
checkpoint passed the all-feature, all-target workspace type check in 49.53
seconds and completed 103 passing focused tests, seven failures and none ignored
in 705 seconds. Six failures were outdated source assertions. The seventh exposed
Boa dropping source-import attributes before the embedded loader; its source
repair preserves request evaluation order and exact referrer-Realm errors while
refusing unavailable static and dynamic source objects. Native execution was
deferred. All repairs are joined for the affected checkpoint.

The independent `tasks-ir-complete4` run completed 141 targets with 2,093 passes,
one failure and none ignored in 270 seconds. All 43 failures from `focused-ir3`
now pass. Its sole failed fixture labelled a SourceText record as JSON; the
written repair retains distinct request identities with legitimate SourceText
attributes and checks their canonical order. `tasks-snapshot-repair2` passes the
current type check and all 22 affected controls. Its original native fixture
compiled in 82.57 seconds and instantiated successfully, clearing the canonical
host import blocker. Execution then trapped at module offset `0xc562b`.
The retained artifact's GC-aware parse identifies `Environment.DEFINING_REALM`
followed by `RefAsNonNull`: main's generic environment had no Realm-bound global
parent. Its source repair joins initial Realm/global-environment construction
before the existing activation/captured-cell allocation. The checkpoint exited
101 after 225 seconds without a memory-limit failure.

`tasks-main-root1` passes the workspace type check and the native control for
entry-Realm prototypes, escaping captured cells and global function publication.
The original lifecycle fixture now executes without the null trap, but its
empty output fails the unchanged assertions. One bounded diagnostic traced this
to plain Await returning target zero after registering its reaction: Promise
settlement prematurely marks the original activation completed. The written
shared suspension return reads the actual activation's committed resume point
for ordinary Await, module handoff, async disposal and both awaited iterator
implementations. Async generators retain their separate body-status protocol.
Its first checkpoint stopped at the existing module-size guard before compiling;
the helper now has its own small private module without raising an existing
budget. `tasks-async-suspension2` passes the full workspace type check in 49.21
seconds and completes all five native controls in 465 seconds: two pass, three
fail and none are ignored. Immediate async return and async finalizer behavior
pass. The remaining controls expose an implicit Atomics progress result missing
from the GC checkpoint and known Symbols represented by String expressions.
The joined repair gives Symbols a closed IR variant, updates its actual
lowering/analysis/emission consumers, and returns explicit timeout progress while
allowing settled Promise reactions to notify other waiters. Two bounded native
diagnostics retain the failure evidence without modifying the original controls.
`tasks-symbol-progress1` passes the all-feature, all-target workspace type check
in 63 seconds and all 20 selected Symbol IR controls. Its five native controls
complete in a 765-second checkpoint: three pass, two fail and none are ignored.
Ordinary Await now validates and preserves lexical state/rejection; awaited
iterator closing passes; the unchanged original mixed array lifecycle passes in
both strict and sloppy modes. The dedicated Symbol fixture incorrectly called
an unloaded `$262` harness; its repair uses the actual `__lilaCreateRealm` hook
and checks the full symbol catalog in one loop to avoid duplicating native code.
The Atomics progress control fails earlier at shared typed-array admission; a
bounded diagnostic retains the actual view and buffer identities. Affected
verification remains pending at that checkpoint. The same one-CPU, 4096 MiB,
zero-swap cap remains.

The waiter diagnostic identifies `new Int32Array(new SharedArrayBuffer(8))` as
Float64 storage with length one and an Int32 prototype. Planning had aliased all
twelve concrete constructors to the first kind's specialized body. The repair
removes that alias and emission filter, so indices, bodies, counts and names
consume the same compiled-builtin list. `tasks-constructor-progress1` refreshes
and checks all fourteen affected Intl identities, passes both source guards and
the workspace type check in 46.58 seconds, then passes all three native controls
in a 435-second checkpoint with none ignored: waiter reaction/notification,
all fifteen Symbol identities across dynamic and created-Realm access, and all
twelve TypedArray storage kinds across scalar/shared construction and a different
newTarget prototype. The storage control also checks actual element conversion.
Native compilations take 76.31, 89.76 and 114.98 seconds under the same cap.

The next batch runs the complete IR suite and native JSON, rooted graph,
selected-worker, generated campaign and URI/prelude/filesystem controls in order.
Upcoming standalone graph and async-switch fixtures now call the exposed `gc`
and `__lilaCreateRealm` hooks instead of an unloaded `$262` harness. Existing
worker deadlines and generated conformance counts remain unchanged.
`tasks-ir-native-records1` passes all 2,094 IR unit/integration controls across
141 targets, with none failed or ignored. Its documentation scope separately
passes five compile-fail controls and retains one pre-existing illustrative
ignored snippet. Native JSON compilation then rejects a loop's source-name
lookup for a private invocation cell. Native graph controls pass created-Realm
anchors and explicit unsupported-exotic/budget results; the post-job throw graph
is rejected. The complete 1,110-second checkpoint finishes with five passing and
five failing native/campaign test functions across three failed commands, none
ignored. URI execution and worker/CLI prelude/filesystem controls pass. The worker
Module fixture has an invalid `file://` URL, its Script exceeds the original
120-second cold-compilation deadline, and the generated campaign fails at its
five-second budget.

The retained graph diagnostic executes the exact original source: the Throw
root and job output survive, but admission reports a duplicate own property key.
Function allocation publishes `name`, then inferred naming appended a second
entry. The joined source repair uses ordinary descriptor definition, addresses
the checked private loop cell directly through its invocation frame, and fixes
the fixture URL to `lila://`. Existing naming controls also check own-key
uniqueness. The whole repair precedes affected compilation and native controls;
worker/campaign deadlines and graph admission remain unchanged.

`tasks-loop-name1` passes formatting, both source guards and the whole-workspace,
all-feature, all-target type check in 49.72 seconds. Its 615-second checkpoint
finishes with four native passes, three failures and none ignored. The original
rooted Throw graph and both inferred-name controls pass, as does ordinary
generator phase ordering. The other generator controls reveal initializer and
selector bindings discarded with temporary operand scopes. The written repair
keeps those declarations in their enclosing scope while preserving runtime cells
and phase guards. JSON now executes, but its default remains undefined: bounded
diagnostics locate a private module consumer copying a completion branch target
over the generator's committed resume point. It now consumes actual activation
status, matching ordinary generator resume. These related source repairs precede
one affected compile/native checkpoint; prior green controls are not repeated
without an affected path.

`tasks-scope-module1` passes both source guards and the full workspace type
check in 45.55 seconds. The original mixed-loop artifact control then rejects
`caught` at its legacy preflight: recursive catch/finalizer admission forgot the
enclosing checked loop's branch targets. The checkpoint exits before native
tests. The joined repair carries explicit iteration/CaseBlock targets and labels
through admission, preserving rejection of unowned Break/Continue. The visitor
now has its own private module within the existing parent size budget. Original
strict/sloppy artifact and native controls remain required.

`tasks-control-scope2` passes the full workspace type check in 51.22 seconds.
The joined admission/source-fixture repairs pass all eight dispatcher controls,
all ten resource-scope source controls, and the artifact control validating all
four original strict/sloppy mixed-loop sources. Existing size budgets remain.
`tasks-scope-native5` completes in 1,245 seconds with six native test functions
passing, two failing and none ignored. Both ordinary-generator controls, both
private-module live-binding/error controls and both v7 rooted worker controls
pass. The workers retain their original 120-second deadlines. The mixed phase
fixture passes in both modes; its completion fixture fails at the first awaited
iterator value after a checked loop, before its strict variant runs. JSON now
reaches its own invariant assertion. Separate bounded diagnostics retain both
failures before the next source repair. The generated-campaign performance
failure remains open; this checkpoint changes no pinned conformance counts.

Bounded diagnostics locate both remaining failures before the next compile.
ForOf's array and direct iterator return the expected value, but its private
incoming cell had no compiler alias, so initialization read a global property.
The joined repair attaches ForOf/ForIn input aliases and rejects unresolved owned
bindings. JSON passes its first nine data/namespace assertions, then rejects
malformed input with an Error whose prototype came from a fresh intrinsic
constructor. Trusted constructor references now load installed Realm slots with
explicit builtin demand. Original fixtures and intrinsic-error controls remain
the acceptance gate; this source repair is not yet verified.

`tasks-iterator-intrinsics2` passes the all-feature/all-target workspace check in
42.42 seconds and all four mixed-loop artifacts. The ForIn completion artifact
passes, while the enumeration artifact exposes a lost uncaptured head alias as
an emission error. Iterator initialization now retains its declarations in the
iteration scope, using the same checked operand operation as classic For heads.
The failed artifact and affected native cohort remain required; no native tests
ran in this checkpoint.

`tasks-iterator-intrinsics3` passes the workspace check in 37.60 seconds and the
previously failing ForIn enumeration artifact. Its 1,770-second checkpoint has
eight native test functions passing, two failed and none ignored. All four
mixed-loop fixtures pass, as do JSON, all three ForOf family controls in both
modes, Annex B ForIn and both source-import intrinsic/error-category controls.
Original assertions and deadlines are unchanged. The two remaining ForIn
failures are a labelled plain-async admission guard that omitted complete iterator
owners, and an ordinary-generator runtime assertion. The label guard now consumes
the existing checked iterator-state projection; an actual artifact control joins
the existing negative missing-owner control. A diagnostic copy retains the
original ordinary-generator fixture and adds assertion/TDZ output before the
next source repair. It fails at the first head TDZ assertion. The lowered capture
uses hop zero, but runtime unconditionally inserts the named function's self
record. Analysis had only inserted that record when direct eval was present.
Named expressions now always plan that physical record, and both original and
generated capture-hop consumers count it. New IR controls cover all four callable
protocols and nested names; full IR, labelled admission and the affected native
controls remain required at that source checkpoint.

`tasks-for-in-capture1` passes the workspace check in 53.98 seconds. Its full IR
sweep passes 2,094 controls and fails two old environment-order expectations that
omit the physical named-function record. Those expectations now include that
record and retain their original outer-scope checks. `tasks-for-in-capture2`
passes all 2,096 unit/integration controls across 141 targets, with none failed or
ignored, plus five compile-fail documentation controls and one pre-existing
ignored illustrative snippet. Both labelled Wasm admission controls and all five
affected native tests pass. Original Generator and Async ForIn fixtures pass
both modes; named-function assignment and both direct-eval self-binding controls
also pass. The checkpoint finishes in 1,035 seconds with no failed commands.
It retains one CPU, 4096 MiB aggregate RAM and zero swap. Remaining prepared
native acceptance, broad/pinned conformance, sustained campaigns and external
performance evidence remain open.

The serial `tasks-remaining-native1` acceptance run stops with exit 137 under
the same cap. Its shared PlainDate/Duration created-Realm control fails during
emission because `disambiguation` is absent from the pool. The calendar gate
emits both complete converter helpers before public-builtin discovery reaches
its fixpoint; the literal gate omitted that helper ownership. The applied repair
makes both converter literal sets consume the calendar gate and adds a real
PlainDate-only Wasm emission/validation control. Constructor newTarget defaults and
object-valued prototypes pass both original modes. The family-publication
fixture has a 1,520,883-byte body and selects the existing size-optimized native
compiler; the earlier sub-1-MiB measurements apply to their named fixtures.
Publication passes its non-strict execution, but the strict compile ends the
scope in an OOM kill, confirmed by the systemd user journal. No complete
publication-test pass is claimed. The successor runs the affected and remaining
Temporal test functions in fresh processes with one retained native module and
a 64 MiB image-cache limit, under the unchanged aggregate cap. Original paired
modes and deadlines remain; unaffected completed controls are not repeated.

`tasks-temporal-literals1` passes both source guards, the all-feature/all-target
workspace check in 45.97 seconds, and the actual PlainDate-only Wasm regression
in 25.61 seconds. All five affected/remaining created-Realm controls pass both
original modes, with no failures or ignored tests, in a 1,815-second checkpoint.
Each native test has a fresh process, one retained module and a 64 MiB image
cache, inside the unchanged one-CPU/4096-MiB/no-swap scope. This includes both
publication modes after the earlier OOM. Read-only inspection of the actual
cached publication artifact attributes its 1,520,883-byte largest body to
`lila::main`; the prior scheduling-fixture body limits are unchanged. The two
constructor controls retain their earlier successful paired-mode results.
Remaining native acceptance builds its selected targets together, resolves
actual libtest names, then runs every selected original test in a fresh process
with its Cargo package working directory. Per-test results retain cgroup memory
measurements. Broad/pinned conformance and the other open obligations remain.

`tasks-remaining-isolated1` completes all 50 selected test functions in 7,950
seconds: 39 pass and 11 fail, with none ignored and no OOM. All failures are
ordinary libtest status 101; the resource cap remains unchanged. The four
Duration controls, object construction, the paired mixed
array lifecycle and both paired Proxy Realm controls pass. The generated-error
function-coercion control traps in its prepared `realm.evalScript` entry before
the source body runs. Inspection of the actual cached Wasm locates a null
direct-eval context being required and stored in the Script Environment.
Prepared entry planning discarded the Script kind, so global and indirect
entries exposed their deliberately null ABI parameter as an owned direct-eval
context. The joined repair retains the original kind and materializes that role
only for DirectEval; global Script roles remain absent. It was applied after
this stable-source run. No native pass is claimed for the failed control or the
repair.

The same collection run passes all six BigInt controls and the paired East
Asian field-order control. East Asian arithmetic, Duration and projection
controls return an unexpected Throw; unchanged-fixture rooted-graph diagnostics
are prepared to identify the actual error. Limits and partial-date controls
fail earlier because shared converter emission lacks its checked Intl host
import. The joined companion repair derives host import planning from the same
calendar-helper gate as body emission and literal collection, with an actual
partial-date constructor emission/validation regression. Umm al-Qura arithmetic,
limits, projection and relative Duration pass both modes; its fields fixture
returns an unexpected Throw, and its partial-date fixture has the same missing
helper import. The joined repairs are still unverified natively.

Collection also exposes three test-setup errors. The oracle entry test's
`lila-test:` URL is rejected by the embedded graph constructor, which requires
`lila://`. The resource fixture puts `using` directly in switch clauses, where
the existing typed early error is correct. Its correction adds explicit
Blocks and asserts disposal before fallthrough and labelled exit. A bounded
search finds the same issue in both mixed resource fixture files and the native
switch lifecycle control; their corrections join this batch. Shared-capability
controls in legal lexical scopes and loop heads remain intact. The runtime
profile test incorrectly requires positive capacity before main runs; Wasmtime
returns zero for an uninitialized GC Store, as the profile contract already
documents. The assertion permits that state while retaining allocation,
output and completion checks. The profiling throw/backend-error control passes.
The paired async-switch control, all three selected Intl controls, array storage,
both RegExp modes and CLI manifest integration also pass. After applying the
joined source patch, all nine corrected resource parsing/lowering checks pass
with no diagnostics using the existing build. The repair checkpoint will compile
once, rerun failed controls and the directly affected prepared-script/resource
controls, under the same cap.

The unchanged East Asian graph diagnostics capture actual Error messages:
arithmetic returns day 12 instead of 1, the leap-year Duration span is 12 months
instead of 13, and advancing by the calendar's year-month count misses the next
year. The common arithmetic predicate omitted East Asian and Hebrew domains,
so both used Gregorian arithmetic on the retained ISO fields. Its source repair
now exhaustively matches every arithmetic domain from the calendar inventory;
adding a domain requires classifying that dispatch at compile time. The same
predicate also owns Hebrew MonthDay regulation. Existing Hebrew arithmetic,
partial-date and relative-Duration controls join the affected native checkpoint.
These diagnostic observations precede the repair build and are not passing
acceptance evidence.

The fourth unchanged graph diagnostic isolates the Umm al-Qura field failure:
the fixture's `make` wrapper uses Object.assign, which invokes the year getter
before Temporal receives the bag. Its syntax-order observation now passes the
accessor bag directly, retaining the original zero-later-read assertion. The
same correction is made in the tabular Islamic and thirteen-month field
fixtures. Product month-code validation already occurs before the year read.
`tasks-calendar-diagnostics1` completes in 570 seconds under the unchanged cap;
it contains nine passing parsing/lowering checks and four captured failing
calendar completions. It does not validate the new arithmetic dispatch. The
complete joined repair now proceeds to one workspace check, its converter
artifact regression and 22 failed/affected native test functions in fresh
processes. No broad or pinned result is inferred from that scope.

`tasks-collected-repairs1` passes formatting, both source guards and the
all-feature/all-target workspace check in 49.14 seconds. The converter artifact
regression passes emission and Wasm validation in 25.70 seconds. The first of 22
affected native functions passes: the original generated-error coercion/Realm
control completes without the prepared-entry null trap in 137.29 seconds. The
remaining native controls are running; earlier diagnostic throws are not counted
as passes.

The joined run also passes both modes of East Asian limits (326.05 seconds),
partial dates (269.04 seconds) and full projection (264.29 seconds), resolving
their earlier import/projection failures. East Asian arithmetic and Duration
still return unexpected Throw completions after this repair; their follow-up
graph diagnostics will use the newly built source and only actual failing
selections. The rest of the 22-function checkpoint is still running. These two
failures keep the calendar batch open despite the passing projection control.

The same joined run passes both Umm al-Qura field modes (248.27 seconds), both
partial-date modes (225.53 seconds), oracle entry rejection (0.03 seconds) and
the corrected runtime profile control (104.27 seconds). The legal ordinary
resource fixture exposes an emission error for its first private lexical name.
Resource initialization finds the owned activation cell but bypasses the binding
allocator that publishes its compiler-scope alias. A staged repair removes that
raw-storage shortcut; the existing allocator still selects the exact owned cell
and binds its name before any initializer/body expression is emitted. It will
join the remaining collected repairs after the current run; no native pass is
claimed for this resource correction.

The collection was deliberately stopped through the watcher's process-group
cleanup after 3,219 seconds to return to source repairs. Seventeen functions
completed: twelve pass and five fail, with none ignored; five remain unfinished.
The additional passes include both tabular Islamic field modes (250.20 seconds),
both thirteen-month field modes (288.22 seconds), Hebrew partial dates (209.83
seconds) and Hebrew relative Duration (251.95 seconds). Hebrew arithmetic also
throws, and mixed resource registration exposes the same missing binding-alias
path (`third`). The switch-resource lifecycle was interrupted during its first
native compilation; it and all four prepared-script/context controls remain in
the explicit successor queue. Status 143 is this intentional cancellation,
not an OOM or a test result. All ten failed/unfinished selections are retained.
The resource alias patch is applied but uncompiled. The three remaining calendar
failures were diagnosed against the existing build before the next rebuild.

`tasks-calendar-diagnostics2` captures all three remaining throws in 345 seconds.
The East Asian and Hebrew arithmetic fixtures incorrectly equate `right.since(left)`
with `left.until(right)` despite their own direction-dependent leap-month pairs.
They now assert the specified same-receiver negation, keeping every pinned
year/month component, serial difference and reconstruction check. The East Asian
Duration fixture incorrectly expects add/subtract to use a `relativeTo` option.
It now asserts the required RangeError and zero reads of the extra argument;
plain/zoned compare, total and round observations remain. See
[DifferenceTemporalPlainDate](https://tc39.es/proposal-temporal/#sec-temporal-differencetemporalplaindate),
[DifferenceTemporalPlainYearMonth](https://tc39.es/proposal-temporal/#sec-temporal-differencetemporalplainyearmonth)
and [AddDurations](https://tc39.es/proposal-temporal/#sec-temporal-adddurations).
These fixture corrections and the resource alias repair are complete in source.
The successor retains all ten failed/unfinished selections, original execution
deadlines and modes, one CPU and the 4096 MiB cap. No new pass is claimed before
that combined checkpoint finishes.

`tasks-calendars-resource-repairs1` finishes in 2,415 seconds: the all-feature/
all-target workspace check passes in 63 seconds; eight native functions pass
and two fail, with none ignored. The three corrected calendar fixtures pass
both modes. Ordinary-generator/plain-async resources pass all four observations,
and switch resources pass both modes. Mixed resource registration passes both
modes, then its separate completion fixture exposes an observation-order error:
an already queued next can advance disposal before the caller's promise reaction.
The fixture now checks the live scope inside finally and retains the queued
whole-Return/disposal assertions, consistent with
[AsyncGeneratorYield](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-asyncgeneratoryield).
Its existing registration and completion observations are split into separately
selectable tests so the corrected fixture can be retried independently.

Indirect eval and fresh global Script cells pass both modes; the nested
direct-eval context control also passes. The Realm Script conversion control
fails admission on its Symbol argument. A private proof now admits that known
conversion failure through the real host call, where ToString produces the
captured Realm's TypeError before dispatch. Unknown-source rejection is retained.
An IR regression covers direct, optional and captured calls without a prepared
source. Both repairs are applied after the native cohort finishes. Their joined
checkpoint includes the full IR suite, the original Realm conversion control,
the corrected resource completion fixture and the remaining non-String eval
identity control under the same resource limits.

`tasks-resource-realm-followup1` finishes in 690 seconds. The workspace check
passes in 60 seconds, followed by all 2,097 IR unit/integration controls across
141 targets with none failed or ignored. Five compile-fail documentation tests
pass; one pre-existing illustrative snippet remains ignored. Both the new Symbol
admission regression and existing unknown-source rejection pass. The corrected
resource completion fixture passes in 128.82 seconds, and non-String eval identity
passes both modes in 224.90 seconds. Realm conversion still fails admission.

The 15-second `tasks-realm-admission1` IR-only diagnostic isolates that failure
to the Symbol call after earlier effectful calls; all preceding conversion
observations admit successfully. Mutable global Symbol no longer has a proven
constructor identity there. Capturing the Symbol in a const before those effects
makes the complete original observation sequence admit with no diagnostics.
The fixture now retains that input, and the IR regression also covers a captured
Symbol after explicitly replacing global Symbol. Product code is unchanged from
the passing full IR run. The affected IR controls and paired native Realm
fixture remain queued for the final focused check.

`tasks-realm-symbol-fixture1` finishes successfully in 270 seconds. All three
affected IR admission controls pass, including the captured Symbol after global
replacement and the retained unknown-source rejection. The complete native
Realm conversion fixture passes strict and sloppy modes in 246.96 seconds.
This closes the collected resource/calendar/prepared-Script repair batch.
The original failures and scoped results remain recorded; no full pinned,
campaign, performance or independent-host acceptance is inferred. Every native
payload used the original execution deadline, one CPU, fresh test processes,
the 64 MiB retained-module cache limit and the kernel 4096 MiB/no-swap cap.

The bounded task audit found no further concrete source blocker before this
checkpoint. Remaining acceptance includes full IR/native/pinned regressions,
sustained campaigns, calibrated performance and independent host evidence.
T05/T21 still record the unavailable weak-reachability backend facility. A
positive proposal module source representation still needs a concrete loader
contract; JSON continues to reject source phase. These are open obligations,
not inferred passes from the dry source audit.

### Preceding size-reduction checkpoints

The combined calendar/scheduling, pooled-string, shared-coercion and T25 batch
passed the whole-workspace, all-feature, all-target type check. The Oct 6
`coercion-t25-2` checkpoint passed all 13 selected AOT controls, including actual
Wasm validation and retained body-size bounds. The source async run is 265,811
bytes; Duration.round is 2,360,810 bytes. T25's IR admission, RSS parsing/report
admission, performance and actual native-input worker/replay controls pass after
the Oct 7 `wire-native1` repair made `ir_admission` reject foreign wire fields.
That checkpoint repeated the workspace type check successfully, then passed all
19 robustness library and two worker/CLI integration controls.

The object-operation, function-metadata and pure calendar successor passed the
workspace type check in 65 seconds and all 17 selected AOT controls in
`object-calendar3`. Source guards pass after moving the operation facades beside
their private-kernel compilers; the module budgets remain intact. The scheduling
fixture has 22,519,753 code bytes; main is 2,240,660, async run is 257,898 and
Array.fromAsync's fulfilled continuation is 335,546. Temporal.round is 1,861,354,
total is 1,509,457 and compare is 1,311,181. All nine calendar helpers validate
with their real callers. The original native fixture still hit the 4096 MiB
kernel cap in Cranelift before a result; its 47,094,354-byte artifact has a
2,183,108-byte largest body. No owned payload remains running.

The common 57-branch GC projection, ordinary allocation and explicit-Realm
ToObject successor passed all source guards and the workspace type check in
`projection1` (66 seconds). Its focused cohort passes 17 controls and fails the
new whole-fixture size gate; native execution was not started. The scheduling
fixture has 20,322,029 code bytes. Main is 863,564 bytes, async run is 208,269,
and Array.fromAsync's fulfilled continuation is 333,152. The remaining bodies
above 1 MiB are Duration compare (1,311,800), round (1,861,681) and total
(1,513,907). The exact date-conversion multiplicity is retained source evidence:
Duration relativeTo expands two Zoned and three Plain conversions.

The written, unverified successor shares those complete conversions with the original
callable FunctionContext and caller Environment as distinct declared operands.
No synthetic callable entry or current-Realm inference is allowed. Preserve
option presence, ordering, intrinsic prototypes and whole abrupt completions.
The PlainDate success carrier uses the existing GC PlainDate record and is
projected/released only after a normal result; this adds one temporary allocation
without changing the abrupt ABI. Keep the 1 MiB fixture gate and stronger
512 KiB helper/source bounds. Do not increase the resource cap or run native
checks per edit.
Independent source review covers the actual contexts, option presence, root
lifetime, planning stubs and whole completion transport. After the final source
joins, format/refresh metadata once, compile once,
run affected controls, then resume native and broad verification sequentially.
Final complete IR, native profile, broad and pinned acceptance remain pending.

## Earlier calendar/scheduling checkpoint — 2026-10-06

The dry source batch and its first repair batch passed `cargo xc --keep-going
--locked --offline --all-features` across the workspace and all test targets on
2026-10-06. Watched checkpoint8 records the initial type pass; focused-ir3 records
the repaired type pass and 140 IR targets with 2,042 passes, 43 failures and none
ignored, improving from focused-ir1's 1,864 passes and 221 failures. Both complete
IR checkpoints took 330 seconds including their build steps. They used the
confirmed one-CPU, 4096 MiB aggregate, zero-swap cap. Task-plan checks passed;
focused-ir3's sole module-boundary failure was a one-line class-lowering size
overrun. The affected 20-target focused-ir4 checkpoint passed 1,559 tests with four
assertion failures in 180 seconds; focused-runtime1's rerun passed all 20 selected
tests after those assertion fixes. Both source guards now pass. All 13 focused
Intl service-selection/export tests, seven oracle controls and the native
observation control also pass. The first mixed array lifecycle test hit the
4096 MiB kernel cap during native compilation and produced no result. A separate
already-built diagnostic stopped at its earlier 3 GiB bound: the 126,572,682-byte
artifact contains 102,303,826 code bytes, led by Temporal.Duration.round
(6,955,567 bytes), main (6,298,230) and the source run function (4,794,629).
The shared calendar/scheduling and passive UTF-16 data successor completed
`shared-helpers4` in 210 seconds: eight controls passed and two size controls
failed. Pooled strings and the complete Temporal artifact pass Wasm validation.
Adding 15 KiB of literal text adds one bootstrap byte. Lifecycle main is now
2,835,768 bytes, but async run is 3,698,752 bytes; lifecycle code totals
79,693,040 bytes. Temporal.round is 3,517,305 bytes and compare is 2,515,451 bytes,
both over their retained limits. Existing typed coercion helpers still have
inline public callers; connecting those callers is the next common source repair.
T25 independent IR admission inputs and invocation memory observations are written
with their controls and contracts for the next combined type checkpoint. The preceding
workspace check reported only a private-type reference in the new string control;
its corrected control compiled and passed in the focused run. This does not claim
a current complete workspace pass. Native, final broad and pinned conformance
remain pending. The kernel resource budget is unchanged.

The successor now also joins mixed
Reference/expression/per-key/catch owners and ordinary generator counterparts,
linked RegExp snapshots and exact required runs, paired currency projection and
admitted Intl export/import, dynamic module graph campaigns, compiler-stage
profiling and serial CI campaign tiers. Plain Async Reference/classic phases,
common iterator/resource families, suspended lexical SuperCall and Custom
calendar/numbering/named-zone/service joins are authored and typechecked. Their
runtime verification remains pending. They do not reopen per-edit receipt,
format, build or test loops; the mandatory verification ladder follows source closure.
The same pass now includes actual product runtime profiling and checked
control-flow generation/reduction across all four function protocols. Their
controls are part of that one deferred checkpoint.
Negative-source and stateful/metamorphic campaigns now belong to the same
source checkpoint, including their actual phase and paired-replay owners.
Conformance node timing and existing Intl bundle footprint reporting now join
that checkpoint; neither reporting command starts execution or data generation.
Budget checks over retained compiler/runtime distributions also join the source
checkpoint; they reuse original evidence admission and do not rerun benchmarks.
Recursive RegExp playback, pure-empty syntax, and proved independent repetitions
now have source/control owners. Bounded robustness campaigns, crash reduction and
original Test262 seed replay join the same unverified source checkpoint.

Ordinary-generator and plain async array/With/ForIn, mixed async-generator
classic-loop/If/value/With/Switch/ForIn, original scoped environment anchoring and required
empty RegExp replay have independent source reviews. Eight Intl filters through
Collator and Segmenter plus SDK/cache v13/worker/CLI consumers are reviewed.
Source review and isolated formatting establish no current compile/runtime
acceptance. The successor now includes mixed patterns, complete iterator heads,
lexical/iterator/classic/Switch resource lifetimes, the complete Conformance
profile and real generated differential campaigns. These remain uncompiled and
unexecuted. General RegExp continuation compression remains source work; Custom
service selection now has the checked physical dependency closure, sparse export
and Wasm admission. Cross-host reproducibility remains an evidence requirement.

The current user instruction prioritizes rapid completion of the dry source pass.
Keep independent source lanes moving; defer repeated receipt/seal/peer bookkeeping,
formatting and verification until the combined source checkpoint. Record material
decisions and unresolved gaps, without producing a new archive for every edit.

Finish remaining iterator/resource, Intl and general RegExp owners before
expensive compilation. Implement independent chunks concurrently and coordinate
shared parents; use bounded source inspection while writing.
Update affected owner guards once, refresh actual metadata once after sources
stabilize, seal the joined source, then compile once and run focused followed by
broad/pinned checks sequentially with reused artifacts. One CPU, 4096 MiB
aggregate RAM, zero swap and serial workers apply to every payload. No earlier
type pass or source peer substitutes for current runtime acceptance.

## Active implementation pass — 2026-10-06

The reviewed source successor includes ordinary-generator Switch and complete
Throw regions, object-owned pattern suspensions, actual computed cover conversion,
class-name parser provenance, and combined List/RelativeTime projections through
SDK/cache/CLI consumers. Earlier exact RegExp counters, complete optional regions
and terminal Delete remain in the same unverified batch. Meaningful controls,
contracts and actual private-owner guard joins are authored with the code.

Array-owned suspensions, broader ForIn/With and iterator/resource/mixed async
owners, remaining Intl component filtering and general huge nullable RegExp
acceleration remain source work. Implement those coherent chunks concurrently
after freezing their real ownership seams. Keep shared parent edits coordinated.
Do not compile, run individual suites or refresh metadata during this source pass.

After sources stabilize, refresh metadata once, seal the joined source, compile
once and run affected focused then broad/pinned suites sequentially with reused
artifacts. All payloads use the existing one-CPU, 4096 MiB aggregate, zero-swap
launcher and serial workers; no uncapped fallback exists. Earlier MAIN type/runtime
results do not verify current source and no task closes on source review alone.

## Earlier exact/optional/List implementation pass — 2026-10-06

The reviewed uncompiled successor now includes exact RegExp bound/counter storage,
the corrected capture-only replay proof, complete optional-chain regions/terminal
Delete, eager assignment-pattern/scoped For joins and an actual List image
projection with selected SDK/CLI/cache consumers. Retained controls and current
contracts are part of each source packet. Remaining Switch/iterator/pattern-owned,
mixed async/resource and general Intl projection work precedes expensive checks.

Finish source joins and affected owner guards, then refresh metadata once after
source stabilization. Compilation/tests remain deferred until the requested whole
source pass ends. Use the existing one-CPU, 4096 MiB aggregate, zero-swap launcher
and serial workers for every later verification payload. No old MAIN type/runtime
result applies to this successor.

This describes how to run a batch of compiler work across several agents without
serialising on compilation and without agents conflicting in the shared backend
files. It is the operational form of the workflow sketched in `AGENTS.md`
("batch implementation before expensive verification").

## Complete source checkpoint — 2026-10-05

The main compiler/GC source batch is written: production paths, consumed
types, meaningful compiler and semantic controls, fixture registration and
current documentation have finished independent source review. The concrete
differential-corpus, performance-reporting and release-gate source is also
written; full task acceptance remains open. The complete
GC representation switch is integrated once from saved MAIN preimages; individual
source lanes do not run separate build checkpoints. The selected runtime still
lacks a weak-reference/ephemeron facility, which remains explicit capability
debt without a substitute object model or passing skip.

The complete source and integration repairs passed the whole-workspace,
all-feature, all-target Rust type check on 2026-10-05. The watched attempt29
checked MAIN128 with `cargo xc --keep-going --locked --offline --all-features`
and exited 0 after 30 seconds (Cargo 24.49 seconds). The launcher confirmed the
4096 MiB aggregate kernel cap, zero swap, grouped OOM and one CPU. Authored Rust
controls typecheck without being executed. The cheap checkpoint through MAIN132
is complete:
22 resource/process controls and all 13 commands pass after the affected
formatting, module assertion and 56-entry shortcut inventory checks. Emitted Wasm validation,
runtime and pinned conformance remain unverified. Run the existing ladder serially through
`scripts/limited_verification.py`: one combined compilation, focused regressions,
then the required broad suites with reused artifacts. The launcher must confirm
the aggregate 4096 MiB kernel cap, swap zero and grouped OOM before any payload;
there is no uncapped fallback or budget increase. A source checkpoint does not
close task acceptance or the release gate.

The first default-product focused compiler/GC checkpoint attempted 86 test
functions: 60 passed, 21 failed and five have no completed results because the
sparse Array target aborted with a stack overflow. No tests were ignored.
The shared emitter, planning and Realm failures are being repaired as one source
batch before affected regressions resume. Broad suites and pinned conformance
remain unverified; these focused results do not close task acceptance.

The 58-path successor through MAIN135 passed the all-feature, all-target
workspace type check (attempt31) and all four affected source checks. Its complete
focused compiler/GC cohort contained 88 test functions: 45 passed, three failed
emitted-Wasm validation, 40 remained incomplete and none were ignored. The dense
test completed without the earlier native stack overflow; it and two reached
module controls rejected a missing i64 operand. Verification was cancelled with
exit 143 before repeating the shared invalid emission, and owned processes were
retired. Source inspection found four single-constant, double-store initializations
in Temporal and RegExp literals. The complete source successor repairs them and
adds whole-corpus differential reporting, a fresh-run conformance closure gate
and measured performance reports. Its code, controls, CLI routes and contracts
are written; its type check passed and current runtime acceptance remains pending.
Broader generation/reduction tooling, runtime verification, current pinned
conformance, idle-machine timing and task acceptance remain open. Generated
full-suite counts are unchanged. Every verification payload retains the confirmed
4096 MiB aggregate cap, zero swap and one CPU; scratch uses disk-backed
`target/verification-tmp` because shared `/tmp` is nearly full.

MAIN136 passed the all-feature, all-target workspace type check (attempt32)
and all four repository checks. Its focused compiler/GC attempt completed
45 passing controls and two failures, with 41 incomplete and none ignored.
The dense control reached a later Wasm validation error: two TypedArray
constructor alignment branches consumed an i64 remainder as an i32 condition.
The owned runner was cancelled with exit 143 and its reached processes were
confirmed retired. The next source repair normalizes both nonzero conditions
and strengthens the existing construction cohort before verification resumes.
Current runtime acceptance and broad/pinned verification remain open.

The MAIN137 alignment source repair passed all four repository checks and
compiled in the focused default-product run. That 92-control run stopped after
45 passes and one emitted-Wasm validation failure; 46 controls remained
incomplete and none were ignored. Validation reached a later RegExp finite-atom
branch that loaded a checked I64 Boolean word as an I32 condition. MAIN138 stores
that checked predicate in I32Local and requires explicit zero-extension for
numeric width arithmetic. It also connects property-definition and assignment
facades to their existing typed runtime helpers; their compilers retain private
physical bodies. This removes repeated source emission while preserving whole
completion, receiver, strictness and caller-environment ownership. The complete
code, semantic controls and contracts passed the all-feature, all-target type
check (attempt33) and all four repository checks. The 102-control focused run
stopped after 45 passes and one code-growth failure, leaving 56 incomplete and
none ignored. The dense-literal module now passes Wasm validation; its producer
added 259,840 bytes for 896 elements, exceeding the 229,376-byte bound. No later
target ran after this failure. MAIN139 replaces per-element numeric-key and
generic property-definition dispatch with complete standard descriptors
published directly into the existing Array indexed storage. Its fresh-literal
owner preserves holes, active-Realm prototypes and expression/abrupt order.
This source repair awaits focused verification; broad/pinned acceptance remains
open and published full-suite counts are unchanged.

The next T23 source batch carries real immutable ICU Locale, ListFormat and
Collator component images. Their operation consumers load admitted data; the
artifact carries matching payloads and the Engine checks them before cached or
fresh native execution. Source review also moved the 605 existing IR controls
out of the public facade without changing their names or source literals, and
replaced the retired no-collector CLI assertion with live closure/catch roots.
MAIN142 passed the whole-workspace, all-feature, all-target Rust type checkpoint
(attempt36, 96.545 seconds) under the confirmed memory/CPU limits after the exact
exporter build dependency and T02 iterator-family owner repairs. This establishes
the shared image SDK foundation, without runtime or conformance proof. The next
complete source batch adds the real native Number/Plural profile image and the
seven-marker ICU Segmenter image, followed by actual native DisplayNames,
RelativeTimeFormat and DurationFormat template images. Duration retains the
selected Number and List owners; RelativeTime retains selected Number/Plural
ownership, and DisplayNames retains selected Locale canonicalization data.
Artifact admission now covers twelve component images. The final source lanes
add the complete IANA2026a transition/country authority, DateTime profiles and
four actual calendar payloads, CLDR timezone names and four native Locale
information tables. Locale retains its actual keyword-alias payload, and
Collator retains its own complete locale inventory. DateTime plans bind the
selected DateTime, Locale and IANA content identities. The provider rejects
foreign dependent owners even when their content digests match.

All current operation consumers now use retained selected image data. The
finite pinned Minimal and named Custom provider therefore declares Embedded
placement. Unused normal baked-data dependencies are removed; build exporters
and independent development oracles remain explicit. This complete successor
is source-only: compilation, native execution, artifact controls and reproducible
byte verification remain pending. Conformance and general filtered custom
images remain open; current Custom names group exact pinned components, and
named-zone settings retain their complete provider-identity check. Published
full-suite counts are unchanged.

The ordinary generator source now retains complete yielding selectors and
conditional/logical arms, eager operator operands and template substitution
order. Compound assignments retain GetValue before the RHS; plain assignments
capture a WriteOnly Reference and defer PutValue errors until after the RHS.
The existing iterator linear assignment route is preserved. Source private
GetValue applies getter effects before later property/call evaluation.

Eager binding patterns consume complete suspended initializers through the
existing single-Get and binding-iterator owners. Staged object literals allocate
once and define each actual property with the same ordinary/native semantic
body: computed key conversion precedes the value, prepared class naming
precedes class initialization, and methods keep the retained object as HomeObject.
Returned and discarded literals use that same owner. Independent source reviews
and authored GC/abrupt-order controls are complete for these scopes; compilation
and execution remain pending. Eager assignment-pattern and scoped classic For
head joins are now source-reviewed. Suspensions inside patterns, broader iterator/control,
mixed async-generator and resource continuations remain explicit compiler work.
See the [pattern initializer contract](./contracts/generator-pattern-initializers.md)
and [statement consumer contract](./contracts/generator-statement-consumers.md).

Recent staging directories and local receipts under `target` disappeared during
the interrupted turn. The integrated source and three watched logs survive.
The source successor is retained in the working tree with small recovery
notes outside `target`; missing receipts are not recreated as new proof.

## Historical implementation-first pass

The 2026-10-03 work on `tasks/` follows the user's request to implement the
remaining task batches before testing individual pieces. Authors finish the
production paths, types, meaningful regression sources and documentation
concurrently. Use constructors, closed enums, exhaustive matches and consuming
plans where they make a plausible mistake a compile error. Do not add unused
types or abstractions merely to describe a future implementation.

The user reiterated on 2026-10-04 that this is a full-task dry-coding pass and
reported excessive RAM use and a crashed process. Finish all remaining task
source, types, meaningful controls and documentation before any further Cargo,
JS compilation, tests, runtime or conformance execution. Source inspection,
isolated formatting and syntax parsing remain available. Earlier combined Rust
proofs below retain their exact historical scope; they do not authorize another
checkpoint before the complete source pass ends.

Later verification requires `python3 scripts/limited_verification.py -- ...`.
Its default kernel cgroup budget is 4096 MiB for the command and all descendants,
with no swap and grouped OOM termination. It reads back the actual limits before
executing the payload, narrows inherited affinity to one CPU and exports serial
Cargo, libtest and Rayon defaults. If the manager or actual limits are absent,
it refuses to run. Its memory option can lower the limit and rejects values
above 4096 MiB before scope creation. Do not raise the budget after a failure.
Cargo and the CPU wrapper also default to one job. Use CLI `--jobs 1 --threads 1`
and run verification stages sequentially. Resource-launcher controls are authored
but unexecuted; kernel bootstrap remains distinct from compiler/task acceptance.
See the [memory contract](contracts/verification-memory-budget.md).

Integrate reviewed source with owned preimages backed up. The interrupted broad
run retains its authentic partial results.

Temporal field and difference authors now have distinct existing owners:
`temporal_plain_date_methods.rs` and `temporal_plain_year_month_methods.rs`
retain fields/.with, while their consumed private `difference.rs` children own
rounding and until/since. The shared calendar parent and private Hebrew leaf
coordinate the consumed month proof, projection and year/month policies.
All these changes belong to one complete calendar batch, with one final
compilation checkpoint; the source extraction is not a byte-equivalence claim.
The Hebrew/shared MonthDay batch passed the ref87 combined type checkpoint.
Complete Chinese/Dangi, plain-async switch block-disposal and String symbol-method
batches are now integrated through ref90 with their meaningful controls and
contracts. The source-suspension visitor is extracted into a private child before
the combined build, preserving the parent module budget and every admission
policy. Those batches and the complete Array copy-method Realm batch passed one
whole-workspace/all-target type checkpoint at ref93, after the explicit
Chinese/Dangi no-era arm correction. Formatting, module inventory and task-plan
checks passed. No lane runs a numeric oracle or runtime suite during source
authoring. Focused and broad executable verification still remain pending.

The subsequent AggregateError, Iterator.from and splice batches finished code,
invariants, meaningful controls and documentation before one combined ref97
whole-workspace/all-target Rust type check. The watched command
`./scripts/run-watched.sh --label invariant-first-combined-types-20261004-attempt15 --stall 900 -- cargo xc --locked --offline`
passed with exit 0 in 30 seconds of watched wall time (Cargo 17.03 seconds).
Formatting, module inventory and task-plan checks passed with exit 0. The
authentic log and lock hashes are recorded in the
[closure task](conformance-ownership.md#t26).
This checkpoint includes current Concat species migration and authored Rust test
targets; it does not execute tests, validate emitted Wasm or establish conformance.
Runtime regressions and broad verification remain deferred.

The complete generator ordinary-property, indexed-collection invocation, source/
final-backend invocation retirement and portable performance-corpus batches
subsequently passed one ref105 whole-workspace/all-target Rust type checkpoint.
The watched attempt18 finished with exit 0 in 30 seconds (Cargo 17.07 seconds).
Formatting, module boundaries and task-plan checks also passed. The closure task
records its exact log and unchanged lock hashes. This proves Rust types and
consts, including authored Rust test targets; it does not compile JavaScript
fixtures, execute semantic controls or time the performance corpus.

Implementation progress and verified acceptance remain separate. At the final
checkpoint, compile the combined source once, run focused regressions, then the
required broad checks sequentially. Task completion and conformance claims still
require that evidence; dry code alone does not satisfy the release gate.

Everything here is measured on the 16-logical-CPU / 93 GiB development machine
unless marked as an estimate.

## Why this shape

Three facts drive the design, and each was measured rather than assumed:

1. **rustc is not the bottleneck.** An incremental engine/CLI rebuild is
   `1.04 s`; a comment-only rebuild in the 48,608-line `builtins/standard.rs` is
   `4.42 s`. A *cold single Test262 case* is `13.73 s`, of which `11.73 s` is
   native Cranelift compilation. The cost is in compiling emitted Wasm, times
   53,131 cases — not in compiling Rust.
2. **The real-suite sweep is a ~15 hour job.** Calibrated at `1.0 s` per case
   with `--threads 8 --jobs 8`. It cannot sit in any inner loop.
3. **The shared backend files are the concurrency limit, not the machine.**
   Adding one builtin touches ~14 places across 6 files. Until the T02 split
   lands, two builtin lanes conflict no matter how they are isolated.

So: agents write in parallel and do not compile; one integrator compiles once;
verification climbs a ladder whose expensive rungs are entered deliberately.

## The ladder

Pick the cheapest rung that can answer the question in front of you.

| Rung | Command | Cost | Catches | Does not catch |
|---|---|---|---|---|
| 0 | `cargo check -p <crate>` / `cargo xc` | 1–5 s / 15–40 s | types, borrows, missing match arms | anything semantic |
| 1 | `cargo test -p lila-ir`, focused `-p lila-engine <filter>` | 30 s–3 min | lowering/IR/engine semantics | real-harness shapes (`$262`, `propertyHelper`, async `$DONE`) |
| 1b | one CLI area module, e.g. `--test cli array::` | 1–3 min | that area's end-to-end CLI behaviour | every other area |
| 1c | the whole CLI suite, run as resumable area chunks by `scripts/rung1c-chunks.sh` (recount the current partition and tests; see below) | **~26 min** at `--test-threads=8` on 16 CPUs; ~2.5 h at `--test-threads=3` on 4 CPUs | end-to-end CLI behaviour | conformance beyond the fixture corpus |
| G | golden capture + `diff -r` (see below) | ~10 min each side | **any** change in emitted bytes | nothing, for refactors — this is the refactor gate |
| 2 | fake fixture suite (191 executions from 190 physical files; current debug rerun passes 191/191 on 2026-09-30) | 10–60 s warm; current cold rerun 540 s | the runner itself | conformance beyond the fixture corpus |
| 3 | `shard 1/25` on the real suite | est. 15 min–3 h | broad cross-subtree regressions | families smaller than ~25 cases |
| 4 | the lane's own ownership-map prefix | 2–40 min | the lane's own family | anything outside the lane |
| 5 | `report-all --resume`, 498 nodes | ~15 h | everything | nothing; too slow to iterate on |

Rungs 0–2, 1b/1c and G are measured. Rungs 3–5 derive from the `1.0 s`/case
calibration; rung 3 in particular has never been run end to end.

Rung 1c is an integration checkpoint, not an inner-loop command. A lane should
run rung 1b for its own area — the per-test cost varies by more than 1.7× across
modules (`heap` is `1.5 s`/test, the whole-suite mix is `2.6 s`/test), so do not
extrapolate one module's cost to the suite.

### Engine integration tests are grouped by area

Each `crates/lila-engine/tests/<area>/main.rs` is one Cargo test target; the
former per-file targets are modules of it
(`tests/<area>/<module>.rs`, test path `<module>::<test>`). Every target links
Wasmtime/Cranelift/ICU, so one binary per area keeps both link time and disk use
bounded while still isolating areas in separate processes. Select a file with a
module filter, and a single test with `--exact`:

```sh
cargo test --locked -p lila-engine --test aot_intl -- aot_intl_collator:: --test-threads=1
cargo test --locked -p lila-engine --test aot_intl -- aot_intl_collator::<test> --exact
```

Areas: `aot_async`, `aot_builtins`, `aot_gc_entries`, `aot_generators`,
`aot_intl`, `aot_language`, `aot_realm_modules`, `aot_regexp`, `aot_temporal`,
`structure` (source/structure guards, compiler identity, rooted observation) and
`runtime_cache` (tests that re-execute their own binary with a private cache).
New integration tests are new modules of the matching area; keep a test that
needs a pristine process in `runtime_cache`. Any test that compiles must call
`lila_engine::configure_compilation_jobs(1)` first, because modules of one area
share a process and its one compilation pool.

### Wasm backend integration tests are grouped by area

`crates/lila-aot-wasm/tests/<area>/main.rs` is one Cargo test target per area; each
former per-file target is a module of it (`tests/<area>/<module>.rs`, test path
`<module>::<test>`), for the same link-time and disk reasons as the engine
targets. Select one former file with a module filter:

```sh
cargo test --locked -p lila-aot-wasm --test intl_temporal -- temporal_instant_epoch_proof_structure::
```

Areas: `emission` (artifact, body, size and emission tests, `product_artifact`,
`emit_golden`), `runtime_link` (runtime/program split, snapshots, host imports),
`modules_realms` (module units, created realms, ShadowRealm bootstrap, per-realm
authority), `intl_temporal`, and the source/structure guards `structure_async`,
`structure_builtins` and `structure_language`. Shared fixtures live in
`tests/fixtures/` and are declared once per area in its `main.rs` (`linked_bodies`,
`product_programs`); `tests/unit/` is compiled into the library's unit tests.
New tests are new modules of the matching area. Source-text guards read the crate
through `CARGO_MANIFEST_DIR` or `include_str!("../../src/...")` (one directory
deeper than the old per-file layout).

### Rung 1c terminates, and checks its own expectations

On a machine that can hold the whole suite in one process lifetime, run it
exactly like this. No `--skip`:

```sh
./scripts/run-watched.sh --label b3-cli --stall 900 -- \
  python3 scripts/limited_verification.py -- \
    cargo test -p lila-cli --test cli -- --test-threads=1
```

On a container that restarts hourly, that invocation cannot finish, and the
supported form is `python3 scripts/limited_verification.py -- ./scripts/rung1c-chunks.sh`
— its Cargo payloads also require the confirmed cap and one test thread. It runs
the same suite as resumable
per-module chunks, banked one verdict at a time. The partition grew when batch 7
split `tests/cli/language.rs` three ways: 105 tests in one libtest process
OOM-SIGKILLed three times, and the chunk set cannot be partitioned by test-name
filter (the hygiene test asserts each chunk's filter is exactly `<module>::`), so fewer
tests per **process**, i.e. more modules, was the lever batch 7 reached for.
It was not the *only* lever, and the note that said so had checked only
environment knobs. The predecessor's in-process Wasmtime module LRU was a
retention candidate bounded only by 64 entries. The 2026-10-03 dry implementation
now uses a private owner with both that entry cap and a 512 MiB compilation-image
budget. `LILA_MODULE_MEMORY_CACHE_ENTRIES` and
`LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES` override the positive limits. Oversized
modules execute without retention. This bounds cache-held code/data/debug images,
not active instances, compilation or RSS; the historical OOM cause and a fresh
completed run remain unverified. See the
[image budget contract](contracts/module-memory-image-budget.md). Recount the chunk
count the same way you recount the test count — it moves. It is tracked precisely because
every batch used to re-derive it from a lane note. Its own header carries the
four properties that must not be "simplified", and
`known_failures::rung_1c_chunks_cover_every_cli_area_module` fails if its chunk
set stops partitioning the suite.

Verification retains `--test-threads=1`. Libtest runs each test on `main`, so
`known_failures::execution_path` cannot observe its per-test name. A valid ledger
with no CLI hang permits the existing bounded in-process capture path; any CLI
hang sends every unnamed serial invocation through the guarded child path.
Missing names and invalid ledgers also use guarded children. Every in-process
capture still constructs a fresh Engine and JavaScript Realm. Commands that
explicitly exercise subprocess behavior retain their actual subprocesses.
Use `-- --exact <name>` to select one test; keep the thread setting at one.

Keep `--stall` at 900 regardless: on a 4-CPU box with a sweep holding two of
them, a single cold Wasm-AOT compile can exceed 300 s of log silence, and the
300 s default then kills a perfectly healthy run with exit code 124. As always,
judge a long run by whether its **log is still growing**, never by elapsed time
against an estimate.

**Do not compare the result against a document.** The expected non-green
outcomes are tracked in `crates/lila-cli/tests/known-failures.tsv` and the
suite enforces them itself, so a green rung 1c means "exactly the declared
outcomes, for the declared reasons" and a red one means something moved. Seven
kinds of drift are failures rather than notes someone has to remember:

| Drift | How it fails |
|---|---|
| new failure | ordinary red test |
| declared failure starts passing | libtest: `test did not panic as expected` |
| declared failure fails for a different reason | `should_panic` message mismatch |
| declared test renamed or deleted | `cargo xc`: E0425/E0603 on a `const _` line |
| ledger row with no test, or test with no row | `known_failures::*` hygiene tests |
| `#[ignore]` added with no owner | `known_failures::every_ignored_test_is_declared` |
| **hang in an undeclared test** | `lila run exceeded ... in process` after the hang timeout |

That last row is the one the table used to be missing, and its absence was not
cosmetic. `execution_path` routes only *declared* hangs to the guarded
subprocess; every undeclared test takes the in-process path, so a new hang could
never produce the guarded path's "this is a NEW hang" message under the
documented `--test-threads=2` invocation. The in-process path is now bounded by
the same timeout, on a worker thread that is leaked rather than killed — so the
suite terminates whichever path the hang appears on. A `fail`-state row whose
test later starts hanging is covered by the same bound.

Neither bound can distinguish "blocked" from "pathologically slow": the declared
hang's fixture prints nothing before it blocks. The timeout is calibrated (900 s,
the same headroom as `--stall 900`) so that a cold Wasm-AOT compile on a loaded
4-CPU box finishes well inside it; treat a timeout as "hung *or* very slow" and
investigate before adding a row.

`binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture` used to
hang the suite forever near the end of the run, which is why the old invocation carried
`--skip atomics_wait_core` and why rung 1c was never actually a gate. It was then
declared a hang (owner T17) and run as a guarded child process, and **that is how
its fix was detected**: batch 6 measured `test did not panic as expected`, twice,
which is the second row of the drift table doing its job. The row, the attribute
and its `const _` were deleted together and it is an ordinary passing test as of
batch 6.

The ledger declares no hang at this head. Under the historical named-worker
invocation, the guarded subprocess path in `tests/cli/main.rs` had a call site
without a declared-hang traveller. The current serial invocation uses that
path for every CLI test because its thread is named `main`. Both bounds and
ledger routing remain live and keep the next hang bounded and reportable.

Two naming traps, both of which have cost time:

- **libtest names carry no target prefix.** `cli` is the cargo *target*, so the
  name is `binary_data::run_...`. `cli::binary_data::run_...`, which this
  document used to print, matches nothing as a filter.
- **`--skip` and filters are substring matches**, not exact names. Prefer
  `-- --exact <name>` when you mean one test.

#### Adding a row

1. Get the names: `sed -n '/^failures:$/,/^test result:/p' target/watched/b3-cli.log`.
2. Get the message per failure: `grep -n '^---- .* stdout ----' -A6 target/watched/b3-cli.log`.
3. Add a ledger row (target, test, state, owner from
   `test262/backlog/ownership-map.tsv`, reason, evidence).
4. Add `#[should_panic(expected = "<stable substring of that message>")]` and
   `pub(crate)` to the test, and a `const _: fn() = crate::<module>::<name>;`
   line in `tests/cli/known_failures.rs`.

   **The attribute must stay on one physical line and use the exact
   `expected = "..."` spelling.** `known_failures::scan_source` asserts that any
   line starting with `#[` also ends with `]`, and the `should_panic` parser
   accepts only that spelling. A wrapped or `\`-continued attribute fails the
   hygiene tests with a pointed message — and the `\`-continued form is already
   idiomatic in this tree for long `#[ignore = "..."]` reasons
   (`crates/lila-aot-wasm/src/planning.rs`), so a contributor following local
   convention will trip it. Shorten the substring instead of wrapping the line.
5. Re-run until green.

A bare `#[should_panic]`, or an empty `expected`, is rejected by the hygiene
tests: it passes on any panic and would turn the next genuine defect in that
test green.

### Always run long commands under the stall guard

Two failure modes here are silent, and both have cost hours:

- **Work that hangs.** Product Test262 cases now have a supervisor deadline
  starting before the compiler process is spawned. Wasm-AOT retains its finite
  300000 ms cold-compilation allowance plus `--timeout-ms`; timeout and crashes
  remain failures. Direct library compilation and broad Cargo tests still need
  an outer watched deadline. A hung run looks exactly like a slow one.
- **Buffered output.** Piping a long run into `tail`/`head` hides all progress
  until exit, so "no output" becomes indistinguishable from "still working".

`scripts/run-watched.sh` closes both: output always lands in `target/watched/`,
the log is polled for growth, a healthy run emits a heartbeat, and a run whose
log goes quiet for `--stall` seconds is killed and reported with exit code 124.

```sh
./scripts/run-watched.sh --label sweep --stall 900 -- \
  python3 scripts/limited_verification.py -- \
    ./target/release/lila test262 report-all --resume --threads 1 --jobs 1
```

Judge a long run by whether its **log is still growing**, never by elapsed time
against an estimate.

### Before adding any tracked data file, run `git check-ignore -v`

`.gitignore` line 3 is a bare `*.txt`. It is not scoped to a directory, so it
swallows any `.txt` anywhere in the tree, `git add -A` reports nothing, and the
file simply never exists for anyone else.

```sh
git check-ignore -v <path>   # exit 1 = tracked. exit 0 prints the rule eating it.
```

This has already cost this repository two files. The historical
`benchmarks/wasm-aot-20.txt` workload was machine-local for exactly this reason.
The 2026-10-04 source successor consumes its unchanged twenty-case order through
the private compiled `crates/lila-cli/tests/perf/chunk_cases.rs` corpus: each name
requires its actual fixture at compile time, and the fixed array enforces twenty.
The probes retain their exact timing spans, limits and opt-in policy; Rust type/
hygiene checks passed at ref105; actual timing acceptance remains pending. See the
[portable corpus contract](contracts/portable-performance-corpus.md). An earlier
`crates/lila-cli/tests/known-failures.txt` was silently dropped while this
document and `README.md` went on citing it for three batches — nobody noticed,
because the suite that would have used it could not be run.

Use `.tsv` for hand-maintained tables; that is already the convention
(`test262/backlog/ownership-map.tsv`, `test262/backlog/shortcut-allowlist.tsv`,
`crates/lila-cli/tests/known-failures.tsv`). Do **not** "fix" this with a `!`
negation in `.gitignore`: `*.txt` has real users, and the next such file walks
into the same trap. Better still, give the file a consumer that fails without it
— `known-failures.tsv` is an `include_str!`, so its absence is a compile error.

### Rung G — the refactor gate

`crates/lila-aot-wasm/tests/emission/emit_golden.rs` runs the real
`parse -> lower -> emit` pipeline over every `.js` file in the current CLI
fixture corpus and records emitted byte length, a content hash, and the backend
`debug_dump` per fixture. It is inert unless `LILA_GOLDEN_OUT` is set.

```sh
git stash
LILA_GOLDEN_OUT=$PWD/target/golden/before python3 scripts/limited_verification.py -- cargo test -p lila-aot-wasm --test emission -- emit_golden:: --test-threads=1
git stash pop
LILA_GOLDEN_OUT=$PWD/target/golden/after python3 scripts/limited_verification.py -- cargo test -p lila-aot-wasm --test emission -- emit_golden:: --test-threads=1
diff -r target/golden/before target/golden/after
```

Keep captures under `target/` (gitignored), never `/tmp`. Each side costs ten
minutes and is useless alone; a `/tmp` cleaner reaping the baseline part-way
through a refactor means paying for it twice.

When the refactor is spread across new files, `git stash` alone will not park it
— untracked files are not stashed. Move them aside explicitly, capture the
baseline, then restore:

```sh
mv crates/lila-aot-wasm/src/intrinsics target/golden/intrinsics-parked
git stash push -- crates/lila-aot-wasm/src/builtins/bootstrap.rs crates/lila-aot-wasm/src/lib.rs
# ... capture baseline ...
git stash pop && mv target/golden/intrinsics-parked crates/lila-aot-wasm/src/intrinsics
```

Empty diff means byte identity. This exists because the ordinary suites assert
on program *output*, so a refactor that perturbs emission order, function index
assignment, or property installation order can leave every CLI test green
while changing the emitted module. Two independent runs were verified
byte-identical, so a non-empty diff is signal, not noise.

Use it for **every** pure refactor of `lila-ir` or `lila-aot-wasm`. Do not
use it for feature work — a feature is *supposed* to change the bytes.

## Running a batch

### 1. Prepare (integrator)

Identify the complete feature batch, the independent chunks, their file owners,
and the focused regressions before implementation. Record the current source
identity, suite pin, and latest completed verification receipts so later results
can be compared with the actual baseline. For a pure refactor, capture the
pre-batch emitted-byte golden as described above; feature work does not require
a byte-identical golden.

```sh
git rev-parse --short HEAD
git rev-parse HEAD:test262/vendor/test262
```

### 2. Write (N lanes, no builds)

Use `test262/backlog/ownership-map.tsv` to assign failure ownership, then split
the feature batch into independent implementation chunks. Several chunks may
belong to the same task. Assign disjoint files to lanes, and give shared
registries, protocols and integration files one owner. If the work is coupled,
first establish the smallest concrete seam needed by its actual callers.
Lanes may share a checkout or stage reviewed patches separately while the
integrator's source is frozen. **Lanes run no cargo commands at all.**

Because lanes get no compile feedback, two rules are load-bearing:

- **Follow the established shape.** A lane adding a builtin should copy the
  structure of an existing one end to end rather than inventing a variation. The
  first lane of any new kind is a pilot run by an agent *with* build access; the
  rest copy the pilot.
- **Review each lane's patch and source identity before integration.** Use
  read-only inspection, patch checks, formatting and other cheap checks during
  implementation. The integrator may run an early focused compile or test when
  it resolves a specific foundation risk or unblocks later work; record that
  reason instead of rebuilding after each edit.

### 3. Integrate and verify (integrator, complete batch)

Apply the reviewed patches in their dependency order and finish all code,
types, tests and documentation before expensive verification. Compile the
complete batch once, run its focused regressions, then run the broad suites
sequentially so they reuse build artifacts. For example, replace the focused
Temporal target below with the actual target inventory chosen for the batch:

```sh
python3 scripts/limited_verification.py -- cargo xc --locked
python3 scripts/limited_verification.py -- \
  cargo test --locked -p lila-engine --test aot_temporal -- aot_temporal_zone_authority:: --test-threads=1
./scripts/run-watched.sh --label batch-workspace --stall 900 -- \
  python3 scripts/limited_verification.py -- \
    cargo test --locked --workspace --no-fail-fast --quiet -- --test-threads=1
python3 scripts/limited_verification.py -- cargo build --locked -p lila-cli --bin lila
./scripts/run-watched.sh --label batch-fake --stall 900 -- \
  python3 scripts/limited_verification.py -- ./target/debug/lila --jobs 1 test262 run \
    --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 \
    --execution-backend wasm-aot --threads 1 \
    --snapshot-dir target/test262-scratch/batch-fake
```

Use fresh labels and snapshot identities for each checkpoint. Fix every
discovered failure, rerun the affected focused tests, and finish with one broad
verification checkpoint. Once checks pass, repeat or broaden them only for new
changes, failures or unresolved concerns. Preserve failed logs and report
exactly what ran and what remains unverified. Fake-suite success is harness
evidence; full pinned conformance still requires its own completed run.

### 4. Gate (integrator)

```sh
python3 scripts/limited_verification.py -- ./target/release/lila test262 shard 1/25 \
  --snapshot-name gate-$(git rev-parse --short HEAD)-post \
  --snapshot-dir target/test262-scratch/gate \
  --threads 1 --jobs 1 --resume
```

Three sharp edges, all verified in the source:

- **Shard indices are 1-based.** `shard 0/25` silently means shard 1.
- **Shard state is selection-specific.** The exact selected execution ids and
  shard tag are hashed into the manifest, so two shards under one
  `--snapshot-name` still receive different snapshot, checkpoint, and journal
  paths. Distinct names remain useful for operators, but are not a correctness
  requirement.
- **A shard is process-green only when it selected at least one case and every
  selected case passed.** Failed shards still write their diagnostic snapshot,
  then exit non-zero; a zero-case selection is `NoEvidence`, not `0/0` green.
- **`compare-snapshots` cannot diff shard runs.** It requires a complete
  498-node aggregate on both sides. Until a `compare-run-snapshots` equivalent
  exists, rung 3 gives a pass count with nothing to compare it against; treat it
  as a smoke test, not a regression gate.

Keep every non-authoritative snapshot under `target/test262-scratch/` (gitignored).
`test262/snapshots/` is already 423 MB across 82,717 files of lane debris.

### 5. Triage and fan out

`generate-backlog` and `publish-status` require a complete aggregate. Until one
exists, use the commands that tolerate a partial sweep:

```sh
./target/release/lila test262 triage-status   --snapshot-name baseline-wasm-aot-aa55200
./target/release/lila test262 failure-details 'built-ins/Array/fromAsync' --snapshot-name baseline-wasm-aot-aa55200
```

`failure-details` is already the right shape for handing to an agent: it groups
by `(detail_hash, outcome, kind, origin)` with representative tests, i.e. one
*failure family*, which is what a fix lane should own. Prioritise
`Crash > Bug > NotImplemented` — but note that **timeouts are currently folded
into `Crash`**, so a crash-ranked list promotes slow-but-correct cases above
genuine defects until that is separated.

Each fix lane verifies with its own prefix at rung 4. `lila test262 run <prefix>`
exits non-zero unless `passed == total`, which is exactly the "start from a
failing filter, end with it green" rule.

## The baseline sweep

```sh
setsid env \
  LILA_FUNCTION_CACHE_LIMIT_BYTES=268435456 \
  LILA_MODULE_CACHE_LIMIT_BYTES=134217728 \
  LILA_PROGRAM_CACHE_LIMIT_BYTES=134217728 \
  python3 scripts/limited_verification.py -- ./target/release/lila test262 report-all \
    --snapshot-name baseline-wasm-aot-$(git rev-parse --short HEAD:test262/vendor/test262) \
    --threads 1 --jobs 1 --resume \
  > target/test262-scratch/baseline.log 2>&1 < /dev/null &
disown
```

- **Use `setsid` and `disown`.** A plain `nohup ... &` launched from an agent
  session was killed at session teardown after 18 of 498 nodes.
- The aggregate is rewritten after every node and cases checkpoint every 10, so
  a kill loses at most 9 cases. Re-running the identical command resumes.
- Poll from another shell with
  `lila test262 progress-status --snapshot-name <name>`.
- Watch for silence: each product Test262 compile and execution shares a
  pre-spawn process deadline, and the parent retires the worker group before
  recording its result. A deadline failure is still unresolved performance or
  correctness debt. Keep the outer watched deadline for the sweep supervisor
  and other native preparation work.
- Do **not** use `scripts/publish-real-status-low-ram.sh` here. It runs one case
  at a time with one compiler job and re-walks all 53,399 suite files per node;
  it serves the separate low-RAM publication checkpoint. When that checkpoint
  runs after the source pass, it must also inherit the confirmed kernel cap.

### Cache tuning matters more than it looks

Measured over 300 cases: the program-Wasm tier grows ~`9 MiB` per case and the
Wasmtime module tier ~`17 MiB`, and **both are keyed by source text**. Every
Test262 case is a distinct source, so across a single sweep neither tier ever
serves a hit — they are pure write and prune churn, and holding the full suite
would take on the order of `1.5 TiB`. The Cranelift stencil tier is keyed per
function, so builtin bodies shared by every case are written once and hit
thereafter.

Hence the asymmetric settings above: a large function tier, minimal
program/module tiers. Raising the single `LILA_CACHE_LIMIT_BYTES` knob
instead would be consumed within a few hundred cases.

Re-sweep **per milestone, not per batch** — and mandatorily whenever the
test262 pin moves, since every prior snapshot becomes uncomparable.

## Earlier state of the enabling work

Landed:

- `.cargo/config.toml` unifies build flags between `scripts/dev.sh` and bare
  `cargo`, ending recurring full-workspace rebuilds.
- Per-tier cache budgets (`LILA_{FUNCTION,MODULE,PROGRAM}_CACHE_LIMIT_BYTES`).
- The golden capture (rung G).
- `crates/lila-cli/tests/cli.rs` split into `tests/cli/`, so feature lanes no
  longer all append to one large file. The source, compiled, ignored and chunk
  counts move independently as tests, feature gates and area modules change;
  no count in this document is authoritative. Recount at the integration head.
  Use the **exact-line** form — the same one the hygiene scanner itself uses
  (`known_failures.rs`), not a substring grep — and use libtest's list mode to
  resolve feature gates and ignored tests:

  ```sh
  awk '/^[[:space:]]*#\[test\][[:space:]]*$/{n++} END{print n}' \
    crates/lila-cli/tests/cli/*.rs
  python3 scripts/limited_verification.py -- cargo test -p lila-cli --test cli -- --list | tail -1
  python3 scripts/limited_verification.py -- cargo test -p lila-cli --test cli -- --list --ignored | tail -1
  awk '$1 == "run_chunk" {n++} END{print n+0}' scripts/rung1c-chunks.sh
  ```

  `--list` counts compiled ignored tests; the `--ignored` list isolates those
  that do not execute in the default run. The difference between source and
  compiled counts is the active `cfg` gating. `grep -h '#\[test\]' … | wc -l`
  is a substring match and over-counts prose that names the attribute, so use
  the exact-line `awk` form.

- **`crates/lila-cli/tests/known-failures.tsv`** — the tracked ledger of
  expected non-green outcomes for this crate's three test targets, enforced by
  `tests/cli/known_failures.rs` at compile time (file existence via
  `include_str!`, test existence via `const _`) and by libtest at run time
  (`should_panic` with a required non-empty `expected`). This is what makes
  rung 1c a gate instead of a reading exercise.

- **`intrinsics/<family>.rs`** — the 4,760-line
  `init_builtin_constructor_object` is split into 15 family modules;
  `bootstrap.rs` went 8,080 to 4,117 lines and its dispatch is now one-line
  delegations. Verified byte-identical across all 527 fixtures. Enforced by
  `check-module-boundaries.sh`.

Current builtin ownership (T02):

- **Descriptor registry** — `lila-ir/src/builtins/catalog.rs` already owns one
  macro row per builtin. Independent checked function/global ordinals preserve
  Wasm function indices and global enumeration order; declaration order retains
  the derived `Ord` contract. The former parallel-table extraction is complete
  as a source boundary, so new builtin rows must use this existing registry.
- **Standard dispatch** — `compile_standard_builtin` is now mostly a flat,
  exhaustive match delegating to family owners. Its iterator disposal body has
  moved intact to the private `iterators/symbol_dispose.rs` owner; remaining
  inline family bodies and the generator/Array iterator compiler methods still
  need ownership work. The old 38,309-line inline-body census does not describe
  the current source.

Coordinate edits to the registry and remaining dispatcher/lowering/emitter
hotspots. Existing private family leaves are independent ownership surfaces;
property installation already has its family owners. Source boundaries alone
do not establish emitted-artifact equivalence or full T02 acceptance.

### How to run an extraction like this safely

The `intrinsics/` split is the template for the two remaining ones:

1. **Capture a golden baseline first**, under `target/golden/before`.
2. **Pilot one small arm**, verify byte-identity, and only then fan out. Proving
   the shape on 16 lines turns the 4,700-line run into a mechanical repeat.
3. **Dry-run the extraction** before writing. The dry run here caught 7
   interspersed no-op or-patterns that a naive pass would have mangled.
4. **Move bodies verbatim.** Re-bind shared context by destructuring a struct
   into the original identifier names rather than rewriting call sites — that is
   what makes byte-identity a meaningful claim.
5. **Re-run golden and diff.** Empty diff or it did not land.
6. **Add the new boundary to `check-module-boundaries.sh`**, and confirm the
   check fails when a module is removed rather than trusting that it would.

## Shared build directories across isolated source roots

When switching complete candidate source roots while reusing CARGO_TARGET_DIR,
clear the twelve workspace packages before the all-target checkpoint. Cargo
may reuse a relative dependency-info record if an independently prepared
source tree has older timestamps than the last build from another root. The
2026-10-01 Segmenter checkpoint observed this as five missing exports from a
stale lila-intl artifact while the candidate declarations were present.

Use the candidate’s actual workspace package names with cargo clean --offline
--locked -p for each package, keeping CARGO_TARGET_DIR on the isolated batch
build. Preserve third-party artifacts, frozen executables and publication
caches. Bind an owned clean receipt to the candidate manifest, source/MAIN/pin
guards, exact package list and build directory. Run the fresh all-target gate
after the clean completes; verify its log identifies every current workspace
package path. Keep the original failed command as historical evidence.

## Source provenance and default Intl isolation — 2026-10-18

Before expensive Rust verification, run
`python3 -B scripts/intl_source_identities.py`. The CI source checks run this
preflight too. When compiler inputs change, regenerate with
`python3 -B scripts/intl_source_identities.py --write`; this follows the
named-zone -> country-membership -> DateTime -> locale-query dependency order
and checks all 13 recipes again after writing. Failed producers stop dependent
writers. Semantic CLDR/IANA rows are still generated by their existing producers.

The workspace runners automatically isolate `lila-intl` when it belongs to the
selected workspace. An omitted caller flag therefore cannot merge its
`compiled_data` dev feature into the normal dependency graph. Explicit isolation
for other packages remains available. The bounded coordinator uses the effective
isolation sealed by the inventory, preserving the complete target denominator,
separate Intl scopes and normal library roots across integration batches.

[The source-bound proof](verification-preflight-and-default-isolation-20261018.json)
retains37 Python controls, all13 actual main-source identity checks and a complete
small Cargo fixture: eight targets, seven tests, six cohorts, six fresh library
checks and five retired completed integration outputs. This verifies tooling;
it does not establish compiler, performance or full conformance acceptance.


## Focused native roots

Use `--selected-feature-roots --build-selected-together` with the workspace
runner's ordinary `--only-targets` selections to avoid linking unrelated
libtest harnesses. Each selected package is built with the empty
`lila-verification-feature-roots` test root. Before a build, the runner uses
Cargo's inspection-only unit graph API to compare each selected root against
the ordinary workspace library graph. Features, profiles, target identities
and every transitive dependency edge must match exactly. Cargo inspection uses
`RUSTC_BOOTSTRAP=1` only in a copied planning environment; compilation retains
the original environment. Unavailable or changed inspection output rejects the
build. The auxiliary is inventoried/executed separately with zero IDs/outcomes
and does not enter selected suite totals. Default features, default package
isolation and a fresh receipt are currently required; unsupported options fail
before compilation. Ordinary full-feature/custom-graph verification remains
available without this opt-in.

To protect the 41 MB immutable runtime from program-module cache eviction,
build callers may set `LILA_BUILD_MODULE_CACHE_DIR` to a dedicated directory.
It uses the same module-cache budget and Wasmtime's exact cache key; it does not
change execution caches, skip fresh runtime emission/admission, or reuse a
previous source's package/manifest. Cross-target builds retain precompilation.
Unset preserves the ordinary cache directory. Initial filling still compiles
normally; subsequent compatible runtime builds can reuse the native image.
