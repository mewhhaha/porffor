# T25 — Differential testing, fuzzing and performance discipline

## Broad verification and remaining performance proof — 2026-10-08

The complete frozen tooling sweep passes all 38 commands, 405 Python methods
and both shell regressions without skips. The 810-target all-feature workspace
sweep was deliberately stopped while red and remains incomplete. Its failed native, corpus, worker and source
controls have prepared repairs; the complete repaired-source sweep remains pending.

The runtime/tail-call and environment batch changes the linked R/P ABI and needs
fresh artifact/cache/native proof. The earlier ABI-2 helper/startup receipts below
retain their original scope. The unchanged five-second campaign still needs a
complete pass, followed by ignored cold/warm controls, the full deterministic
corpus in debug and optimized builds, sustained differential and robustness
tiers, and calibrated subsystem budgets. Existing generators, reducers, reporters
and policy checkers are implemented; execution and measured acceptance remain.
Unavailable allocation/live-GC/pause metrics must stay explicit.

**Status:** Source batch in progress — selected-worker replay, durable campaigns, arithmetic/object/module/control-flow, negative-source and stateful/metamorphic grammars with preserving reducers are authored; compilation/execution, broader fuzz boundaries, subsystem budgets and sustained campaigns remain open

**Parallel group:** Validation lane  
**Depends on:** T01, T02, T03, T04  
**Blocks:** Confidence and performance gates in T26

## Runtime persistence and cache acceptance — 2026-10-08

Validated raw runtime R now persists through the existing cache; Chinese/Dangi
catalogs are built once. Native cache identity follows the packaged Wasmtime
source rather than an unrelated ancestor Git repository. Four identity controls
and two process controls verify real R reuse across differing executable mtimes,
with fresh P compiled and executed. Full coercive addition now uses one R helper
with explicit caller execution Realm and Environment and R/P ABI version two.

`production-addition-helper3` passes the workspace type check in 53.041 seconds
and all 31 focused helper/artifact/codec/native controls. Existing repository
audits pass; the whole watched checkpoint takes 330 seconds. The unchanged
five-second campaign nevertheless remains red: its cold attempt stops at native
R loading; its warm original CrossRealm baseline completes and matches the oracle,
but the transformed fresh P still times out in native compilation. That request
emits in 2.858 seconds, including 1.197 seconds of Intl admission and 0.503 seconds
of raw-R loading, followed by 0.370 seconds of native-R loading. Temporal is unrun.
A successful baseline does not establish the complete campaign.

The subsequent `production-startup-tables1` passes the workspace type check and
40 focused Unicode, timezone and native controls. Both CrossRealm baseline and
transformed programs now complete under the unchanged deadline with native R
cached and match SpecExec. The transformed case emits in 2.432 seconds, then
loads native R in 0.339 seconds, compiles P in 1.452 seconds and executes in
0.024 seconds. Temporal seed 3215 is reached: lowering takes 0.312 seconds,
emission 2.523 seconds and native R loading 0.341 seconds before fresh P
compilation times out. Its transformed case remains unrun. Cold native R
compilation also remains over budget; these cache-qualified observations do
not close the campaign or cold-start gates.

The tooling repair rerun passes all 84 affected Python methods and the restored
historical artifact pair passes the guard in an isolated proposed commit view.
The later complete tooling sweep passes. Complete repaired all-feature workspace
coverage, ignored default-product timing controls, sustained debug/optimized
campaigns and subsystem budgets remain required. The launcher retains one CPU, 4096 MiB,
zero swap and one-entry/64-MiB native retention. No deadline, task status or pinned
conformance count changes. See the [current checkpoint](README.md#runtime-persistence-and-verification--2026-10-08)
and [runtime artifact contract](../docs/rust-rewrite/contracts/runtime-artifact-persistence.md).

## Original campaign timeout diagnosis — 2026-10-08

The final rebuilt worker confirms the same 5,000-ms failure in
`task-closure-final-20261008`: parse 1.83 ms, lower 129.95 ms, then timeout before
emission completes. SpecExec still completes eight actions; transformed
CrossRealm and Temporal remain unrun. Final type/format checks and both adjacent
Realm fixture corrections pass, but the checkpoint remains red. Final evidence:
`target/verification-tmp/lila-generated-campaign-302393-1791478300025590477-0`.

The subsequent bounded profile measures 6.452 seconds of cold emission and a
41.2-MB runtime image. Samples identify Unicode inventory construction, named-zone
certification and Chinese/Dangi calendar generation; native compilation is still
active at 12 seconds. Normalization now avoids identity-scalar allocations and
case folding enumerates the pinned mappings directly. Both exhaustive inventory
comparisons pass, along with the focused native Unicode/folding controls.

`task-closure-focused-20261008` nevertheless retains the original seed 1270
failure at 5,000 ms during emission (parse 1.80 ms; lower 122.31 ms). Native tests
preceded this fresh worker, exercising the existing native disk cache, while
the raw runtime artifact is still rebuilt in each process. These repairs do not
close cold-start or generated-campaign acceptance. Evidence is retained at
`target/verification-tmp/lila-generated-campaign-227144-1791477058089273934-0`;
see the [current checkpoint](README.md#closure-audit-and-verification--2026-10-08).

The fresh `fast-iteration-campaign-20261008` replay rebuilds the selected worker
with the later Collator precheck and retains seed 1270 and its 5,000-ms deadline.
It still times out during Wasm emission: parsing takes 2.22 ms, lowering
148.87 ms, and native compilation is not reached. Worker stderr falls to
118 bytes with no missing-metadata flood. SpecExec completes all eight actions;
transformed CrossRealm and Temporal remain unrun. The exact request, pair and
reports are retained at
`target/verification-tmp/lila-generated-campaign-4136252-1791474858413422907-0`.
The launcher confirms 4096 MiB, zero swap, one CPU and the one-entry/64-MiB module
cache. This is a fresh performance failure, not semantic-equivalence evidence.
Profile the remaining emission cost before the next focused repair.

`tasks-original-campaign-trace1` keeps the original seed and 5,000 ms worker
budget. CrossRealm seed 1270 still fails: parsing takes 5.39 ms and lowering
219.02 ms, but emission has not completed when the worker expires. Native
compilation and JavaScript execution have not started. SpecExec completes all
eight actions with normal Number 263; the transformed and Temporal cases remain
unrun. The retained case/report/pair and worker stderr preserve this failure.

The joined Collator metadata precheck rejects only locale/type pairs absent
from the selected immutable inventory under its own ICU fallback chain. Actual
profile admission and projection recording remain unchanged. Both source
identities are regenerated and check cleanly. Fallback and projection-domain
controls are authored; runtime parity and timing improvement remain unverified.
This source change does not close the five-second performance failure or alter
its deadline.

## Closed native targets and serial robustness tiers — 2026-10-07 dry source

The five remaining no-payload robustness targets now consume their complete
native map before admission. Foreign fields reject just as they do for the IR,
prelude and filesystem targets; valid wire bytes and fingerprint domains are
unchanged. Focused controls cover exact round trips and foreign domain fields.

The existing differential CI now owns a serial robustness tier through the real
CLI/worker boundary: seven native targets with two cases in PR runs, all nineteen
targets with sixty-four cases in nightly/debug/optimized runs. Shared seed bytes
and ranges are identical across tiers. Fresh synchronized tier records retain
pending work, exact inputs, native reports and logs; missing/incomplete/foreign
evidence or command failure keeps the tier red. Independent targets continue,
without retrying failed work or changing its original deadline. The established
300,000 ms native robustness budget does not change generated differential
grammars' five-second budget. That generated-campaign performance failure is
still open and cannot be declared fixed by this tooling.

The common verification launcher carries the established one-entry/64-MiB module
image retention cap, including stricter positive caller limits, alongside the
kernel 4096-MiB/no-swap/one-CPU boundary. Invalid inherited cache values refuse
before launching rather than falling back to larger product defaults.
No builds, tests, benchmarks or campaigns ran during this source update. Driver,
wire and resource controls are authored; relevant focused checks, sustained
campaigns, measured/calibrated subsystem budgets and full task acceptance remain
required. See the [robustness](../docs/rust-rewrite/contracts/differential-robustness.md)
and [resource](../docs/rust-rewrite/contracts/verification-memory-budget.md) contracts.

## Independent IR admission and invocation memory — 2026-10-06 source

The `ir-admission` robustness target adds bounded native candidates for existing
object-property, optional-delete and synchronous-body constructors. An actual
empty Script retains the product compilation context; only the admitted body is
replaced before ordinary emission and validation. Native inputs cannot supply
source/function identities, environment bindings or continuation certificates.
The original byte mutation, stage journal, replay and reducer retain exact input
bytes. This exercises those post-lowering admission domains; the ordinary Script
and Module targets still exercise folding and flow analysis in the real lowerer.
It does not establish arbitrary IR coverage or a separate optimizer pass.

Opt-in runtime profiles now have an invocation-owned Linux RSS sampler with
10 ms intervals, entry/final readings, bounded reads and thread shutdown on every
exit. Version-two reports bind the scope, interval, count, elapsed span and RSS
observations to the original invocation evidence. A sampled-RSS budget is
available only when all measured invocations supplied that metric. Other process
threads and retained caches contribute; this is not exact peak or live-GC memory.
Unavailable GC and complete linear-memory counters remain explicit. The combined
workspace type check passes. Five IR-admission controls, 19 robustness library
controls, two actual worker/CLI native-input replay controls, two compiler
inspection controls, three Engine RSS sampler controls and all 24 CLI performance
controls pass. The wire target uses an empty struct variant so foreign fields
reject during decoding. Native runtime-profile controls, broader boundary
coverage, calibrated budgets and sustained campaigns remain pending. See the
[robustness contract](../docs/rust-rewrite/contracts/differential-robustness.md) and
[runtime profile contract](../docs/rust-rewrite/contracts/runtime-performance-profile.md).

## Bounded URI, prelude and filesystem targets — 2026-10-07 source

Four explicit URI targets retain native UTF-16 units, including lone surrogates,
and invoke the original builtins through product compilation and execution.
The `prelude` target selects an original frontmatter execution mode and consumes
the original harness loader/materializer inside an owned fixture. Only the
genuine embedded profile grants host ownership; candidate overrides invalidate
it through the original store. The `filesystem-resolver` target exercises actual
resolution and direct-load confinement with fixed owned files and optional Unix
symlinks. Missing platform support, fixture IO and cleanup failures stay red.

All six targets use the existing exact-byte wire, stage journal, selected worker,
bounded mutation campaign and crash reducer. Native domains have explicit small
payload limits; neither prelude nor filesystem acceptance claims program execution
or conformance. Native, selected-worker and CLI controls are authored and unrun.
Sustained campaigns and broader semantic observations remain open. See the
[robustness contract](../docs/rust-rewrite/contracts/differential-robustness.md).

## Native rooted completion snapshots — 2026-10-07 source

Schema v7 adds genuine post-job Script/Module completion graphs through the
Engine and Boa heap adapters, with unchanged source, explicit Test262 host
authority and eight fingerprinted limits. The shared driver retains canonical
aliases/cycles, original UTF-16 and Symbol origins, ordinary Object/Array/Function
property graphs, descriptors, prototypes and actual retained Realm anchors.
It does not invoke getters, proxy traps or reflection wrappers. Unsupported
exotics and exhausted budgets remain red even when both backends match.

Native replay, report, comparison and terminal admission consume the checked
graph owner and require its exact requested budget. Old protocols retain their
semantics. Pure admission/comparison controls and two actual selected-worker
programs are authored and unrun; the new native inputs explicitly allow
120,000 ms for cold compilation, leaving existing deadlines unchanged. Function
behavior, other intrinsic identities/internal slots and sustained campaigns
remain open. See the [rooted snapshot contract](../docs/rust-rewrite/contracts/differential-rooted-completion-snapshot.md).

## Conformance and data evidence — 2026-10-06 source

`performance conformance` now aggregates original verified matrix snapshots,
exact timeout/mode identities and retained slow cases. Newly completed nodes
mint a measured invocation sidecar bound to the exact terminal snapshot bytes;
resume timing and missing historical timing remain explicit. `performance data`
admits and retains existing Intl exports, measuring their actual component,
framing and identity bytes with Unicode/tzdb provenance. Neither command launches
a suite or generates data. SDK, CLI and meaningful admission controls are source
work, uncompiled and unrun; performance budgets and sustained acceptance remain
open. See the [evidence aggregation contract](../docs/rust-rewrite/contracts/performance-evidence-aggregation.md).

`performance check` now consumes the original admitted compiler/runtime reports
and an explicit bounded budget policy. Exact fixture/metric/statistic rules can
limit medians, p95 values and deterministic byte/count footprints, using absolute
limits and/or integral relative allowances. Incompatible measurements and absent
metrics refuse; an exceeded budget retains all rule decisions and exits red.
Meaningful source controls are authored. Calibrated tracked thresholds and actual
measurements still require later acceptance. See the [budget contract](../docs/rust-rewrite/contracts/performance-budget-checks.md).

## Negative source and stateful scenarios — 2026-10-06 source

`negative-source-v1` sends closed malformed and early-error Script/Module
sources through the actual selected entry frontends. Native oracle syntax
rejection and product Parse/Early phases remain distinct from runtime failures
and capability gaps. Phase-preserving reduction retains both actual reports;
the expected negative assertion does not rewrite `both_failed` into a green
executable-corpus result.

`builtin-stateful-v1` adds finite Array/buffer, iterator close, Map/Set,
Proxy/coercion, subclass/species and Promise job schedules.
`metamorphic-stateful-v1` compares the actual baseline and transformed programs
after checked binding renaming, neutral blocks or equivalent finite loops.
Paired replay retains both sources and both backend observations. The SDK,
CLI and eleven-family serial campaign consume these producers. Controls are
authored and unrun; arbitrary boundary fuzzing, subsystem budgets and sustained
acceptance remain open. See the
[negative-source](../docs/rust-rewrite/contracts/differential-generated-negative-source.md)
and [scenario](../docs/rust-rewrite/contracts/differential-generated-scenarios.md)
contracts.

The 2026-10-07 source successor adds `builtin-stateful-v2` and
`metamorphic-stateful-v2` with checked cross-Realm and Temporal families.
Eight actions per family retain actual primitive values, prototype/marker
comparisons and ordered conversion hooks through the same exact paired wire,
three transformations and preserving reducer. Realm source explicitly selects
schema v6 and the existing Test262 host profile; Temporal remains product v3
with ISO dates and fixed offsets. Old v1 pairs retain their original seed mapping
and exact source. The two serial PR cases now select these new families, and
focused real-worker controls cover all sixteen actions in two bounded programs.
The native wire, admission and reduction controls are authored; validation is
pending the joined checkpoint. Arbitrary Realm graphs, clock/calendar generation
and a general Temporal object observer remain open.

## Generated control flow — 2026-10-06 source

`control-flow-v1` adds checked bounded statement trees for ordinary, Generator,
Async and AsyncGenerator functions. Branches, loops, captured per-iteration and
block cells, Try/Catch/Finally, labelled control and suspended Return/Throw requests
produce actual schema-v3 completion/print observations. The reducer remints the
same source proof after every edit, preserving protocol/strictness and the actual
mismatch dimensions or backend failure phase. The SDK, CLI and eighth serial
campaign family consume this source. Meaningful parser, damaged-owner, reduction,
worker-failure and real campaign controls are written and unrun; broader generation
and sustained execution remain open. See the
[contract](../docs/rust-rewrite/contracts/differential-generated-control-flow.md).

## Deterministic generated campaigns — 2026-10-06 source

The actual library/CLI `differential campaign` route now runs a checked sequence
of 1–128 nonwrapping seeds through the existing V1/V2/V3 generator, selected
workers and grammar-preserving reducer. Its new directory begins incomplete;
every initial/reduced request and actual observation is retained before the
case decision. Only verified or reduced mismatches enter the separate replayable
corpus directory. Shared/rejected outcomes stay red; worker failure stops with
remaining entries pending. Cooperative embedder cancellation is checked before
each replay and cannot publish a partial green result.

Meaningful failure, cancellation, replay identity and output-refusal controls
are authored but unrun. The same real campaign now also consumes `object-probe-v1`:
checked Object/Array graphs, full descriptors, Symbol/string keys, prototypes,
sharing/cycles, extensibility and realm anchors rendered through the existing v5
capture. Its reducer prunes/reindexes the graph through the same constructor and
preserves graph/print difference dimensions or the original backend failure phase.
It retains each changing source fingerprint rather than conflating reduction with
the original signature. `object-mutations-v2` adds checked ordered Get/Set,
descriptor/deletion/prototype/integrity and Array length operations. The same
v5 observer retains their actual results and final graph; one alias traversal
preserves targets, receivers, assigned values and Symbols during reduction.
SDK/CLI controls and actual-worker replay are authored and unrun. See the
[mutation contract](../docs/rust-rewrite/contracts/differential-object-mutations.md).
`module-graph-v1` adds 1–16 complete Module sources, exact
named/namespace import and reexport edges, live cells, static cycles and metadata
traces. Its reducer removes source edges and prunes/reindexes dependency sources
through the actual v4 graph validator, retaining completion/print difference
dimensions or the original backend phase. `module-graph-v2` now adds generated
dynamic imports, exact attribute rows and top-level Await. One private checked
source graph requires a forward dependency order for all V2 edges, preventing
self/ancestor evaluation waits. The same renderer and reducer preserve V1's
static cycles and its original deterministic draw/source order. Module, CLI and
real selected-worker campaign controls are authored and unrun. Broader AST/mutation schedules, subsystem
performance and full T25 acceptance remain open.
See the [campaign contract](../docs/rust-rewrite/contracts/differential-generated-campaign.md).

## Selected workers — 2026-10-04 dry source

All initial and reduced replays require a `DifferentialWorkerRunner`. Native
corpus/graph decoding remains in the parent; legacy JavaScript closure admission
and actual backend execution occur in fresh workers after a bound header.
The supervisor binds source/revision and selected-image provenance, enforces a
pre-spawn attempt deadline, bounds request/journal/frame IO, retains committed
incomplete print prefixes, and retires the group/direct child/staging before
completed observations can publish. Feature-off and unsupported platforms
refuse execution explicitly.

WorkerFailure is a distinct red report domain with no mismatch signature. Any
candidate worker failure interrupts reduction and returns the actual rejected
case/report; the CLI cannot persist it. Three foundation, seven graph and three
generated/probe controls retain their fixture bytes and expected semantics while
selecting Cargo's named real worker. Additional controls cover deadlines,
provenance, source admission, process/staging cleanup and nonpersistence. CLI
controls join actual hidden dispatch and explicit embedder worker selection.
All current compilation, tests/guards, runtime and pinned acceptance remain
unverified. Complete all task source before the serial 4096 MiB aggregate
checkpoint. The kernel cap bootstrap itself remains unverified. See the
[worker lifecycle](../docs/rust-rewrite/contracts/differential-worker-lifecycle.md).

## Current repository state

The 2026-10-05 source batch adds whole-corpus library/CLI replay and a dedicated
feature-enabled CI workflow. Its compiled inventory contains all twelve current
JSON entries across schemas 1 through 5. Every input remains represented;
malformed entries, shared failures, unsupported observations and worker/admission
failures stay red, with per-case evidence and an initially incomplete aggregate.
The CLI refuses output reuse and has no subset switches. CI uses the confirmed
4 GiB, one-CPU launcher on a delegated Linux host for every build/test/replay;
no uncapped fallback is available. The source was freshly recovered after
external staging loss and remains unexecuted. See the
[whole-corpus contract](../docs/rust-rewrite/contracts/differential-whole-corpus.md).

The same batch adds a separate opt-in `lila performance report` command for the
existing twenty fixtures and three timing gates. It records raw samples, robust
summaries, compatible-baseline deltas and compiler/toolchain/platform/workload
identities on an explicitly identified idle machine. This is source reporting
infrastructure, not timing acceptance or subsystem-wide metrics. See the
[performance-reporting contract](../docs/rust-rewrite/contracts/performance-reporting.md).
Broader generators, reducers, fuzz campaigns and metrics remain open; neither
source addition completes T25.

The compiler profile successor adds `lila performance compile` through the
actual SDK preparation/lowering/emission and product Wasmtime validation stages.
It uses the same twenty-source corpus, one warmup per source, 3–100 raw samples,
retained deterministic artifacts and compatible-baseline stage comparisons.
Actual module/function/code/data/custom-section footprints accompany timings;
baseline admission rederives the byte inventory from retained Wasm. SDK and
report controls are authored and unrun. The runtime successor measures the
actual product execution phases and before/after GC reserved capacity on the
same twenty sources, retaining raw samples and compatible baseline identities.
Allocation, live-size, collection-count, pause and peak metrics remain explicitly
unavailable where Wasmtime exposes no counter. New subsystem budgets and
sustained measurements remain open. See the
[compiler profile contract](../docs/rust-rewrite/contracts/compiler-performance-profile.md)
and [runtime profile contract](../docs/rust-rewrite/contracts/runtime-performance-profile.md).

The MAIN134 source successor adds schema 5 selected object probes through the
existing CLI/worker replay. A probe FunctionBody selects named roots and actual
identity anchors; the same captured-primordial reflection harness runs through
both real backends. One validated bounded graph compares cycles/sharing, all
own-key order, complete descriptors, prototype edges, extensibility, primitive
payloads and Symbol identities. Error Realm comparison uses executed prototype
identity against a selected anchor, rather than a name or host address. Exact
print events include deliberate Proxy reflection traps. Capture failures remain
red. The new graph controls and three selected-worker controls are authored,
unexecuted source; this is not T25 acceptance. Arbitrary post-execution object
observation, unselected Realms/internal slots and the broader task debt remain
open. See the [selected object probe contract](../docs/rust-rewrite/contracts/differential-selected-object-probes.md).

T28 removed the retired JavaScript fuzz, benchmark and execution surfaces. The
durable starting points are ignored Rust-owned Wasm-AOT performance probes,
snapshot determinism tests and an explicitly feature-gated spec-exec oracle.
The 2026-10-04 source batch makes the existing twenty-case timing workload
portable through one private compiled corpus. Its fixed array requires exactly
twenty ordered fixture names, and each name comes from the same literal that
requires its actual fixture with include_str!. A missing or renamed fixture
therefore fails all-target Rust compilation even though timing probes stay
opt-in. The exact original local workload order, fixture bytes, three probe
names, warmup/timing spans and 1s/5s/5s limits remain. The ledger, README and
workflow describe the current compiled owner. The ref105 combined all-target
Rust type and hygiene checks passed. Actual idle-machine timing remains pending;
this is not performance or full T25 acceptance. See the
[portable corpus contract](../docs/rust-rewrite/contracts/portable-performance-corpus.md).

The `lila differential replay` command
consumes a closed versioned Rust-owned corpus entry,
always runs Wasm-AOT first, and can run the off-by-default spec-exec oracle only
when both the cargo feature and explicit `--oracle spec-exec` flag are present.
Its JSON report has a versioned stable case fingerprint and mismatch signature.
Replay compiles both backends with the ordinary product host-surface policy;
probes do not silently gain Test262-only globals.

Schemas v1/v2/v3 carry one entry source and no dependency graph, so their corpus
decoder admits only Scripts whose outer AST contains no dynamic import. Module
goals, true outer Script dynamic imports and Script sources whose outer closure
cannot be proved are rejected before execution. Because eval, Function
constructors, created realms and agents can create an import absent from that
AST, replay also fixes both backends to the shared `RejectAll` module-loading
policy. AOT graph discovery, including agent worker compilation and its cache
retry, rejects requests before filesystem access; spec-exec root, created-realm
and agent contexts use Boa's rejecting loader. Valid existing Script wire bytes
and fingerprint inputs stay unchanged. The additive v4 source path below embeds
and fingerprints the complete declared graph. The
program-Wasm cache key carries the policy in a versioned fixed-discriminator,
presence/length-framed tuple, so variable filename/source bytes cannot alias a
different policy.

The 2026-10-03 implementation-first batch adds schema v4 through one immutable
`Arc<EmbeddedModuleGraph>`. The runtime constructor validates the complete
entry, Module sources and exact resolution table. Entry source, goal and locator
are projections of that owner; the separate strict v4 wire shape has no competing
outer fields. Script, Module and Unlocated referrer roles stay distinct, including
a Script importing a separate Module at the same locator. Attribute keys use
UTF-16 code-unit order. Metadata URLs are separately declared observations and
never resolution bases. The graph fingerprint includes every source, URL,
request and target, including unused rows; the full SHA-256 is retained in v4
case and mismatch identities and consumed by the Embedded AOT cache key.

The real Engine adapter admits and parses the entry once, carries a required
complete catalog in its private prepared-module variant, and lowers only the
entry closure plus declared dynamic candidates. Computed operands and runtime
attributes select exact compiled variants. Unused malformed source does not
poison entry compilation, dynamic-only syntax failures use the existing import
rejection jobs, and Source/Defer/Evaluation phases retain their own semantics.
Agent compilation carries the same graph and consumes explicit Unlocated rows.
The spec adapter uses the same authority without filesystem fallback. Embedded
created Realms share their actual context/interner while retaining separate
globals and per-Realm Module instances; agents retain their separate contexts.
Lossless request conversion denies lone-surrogate strings rather than aliasing
literal backslash spellings. Filesystem and RejectAll retain their prior routes.

V4 reuses the v3 primitive-completion and ordered-print comparator and its
observation gaps. It preserves the `not_established` semantic-equivalence result;
Object/Symbol completions and unavailable output remain contract violations,
and two engine failures remain red. Seven authored paired cases cover cycles,
independent metadata URLs, Script self-import, computed exact attributes,
undeclared/unused rows, dynamic parse failures, defer activation and source-phase
rejection. Adapter controls cover Realm/cache/interner ownership and explicit
Unlocated worker policy. Source review, formatting and reversible patch checks
provide no executable acceptance: dependency resolution, compilation, emitted
Wasm, both backend runs and broad verification remain pending. See
[`differential-embedded-module-graph.md`](../docs/rust-rewrite/contracts/differential-embedded-module-graph.md).

The Rust-owned `integer-arithmetic-v1` campaign is the first non-decorative
consumer of that replay path. `lila differential generate-arithmetic` uses a
stable SplitMix64-v1 stream to build one or more self-checking Script probes
from a closed Add/Sub grammar. Leaves are integers in `-32..=32`; the public
plan types admit only 1–32 checks and depths 1–4, and expected results are
accepted only while they remain exact safe integers. The reducer admits a
typed non-zero budget of 1–512 replays and only constructs candidates whose
`(check count, node count, sum of absolute literals)` complexity decreases
strictly. It removes contiguous check ranges, replaces binary expressions by
children, and shrinks literals toward zero. A candidate is retained only when
it preserves both the mismatch direction and the failing backend phase; it
does not compare source-dependent mismatch fingerprints.

The generator command is protected by the same two independent oracle gates
as replay. It writes a schema-v1 case that the existing replay command consumes
only after both backends complete, or after a mismatch has been reduced; shared
failures and observation-contract failures are reported but are not persisted
as corpus entries. The committed seed-1/checks-4/depth-2 case pins the PRNG,
grammar rendering and schema encoding.

The 2026-10-03 dry batch adds the closed `integer-bitwise-v2` grammar to the
same actual generation/replay/reduction path. A generation plan requires a
grammar, and the CLI defaults to V2; explicit
`--grammar integer-arithmetic-v1` preserves the original V1 draw order, rendering
and committed corpus identity. V2 adds unary `~` and binary `&`, `|`, `^`,
`<<`, `>>`, `>>>` to Add/Sub. Validated exact integer operands convert modulo
2^32, and shift counts use ToUint32 modulo 32. Unary and binary AST constructors
preserve arity through generation, source emission and reduction; reduced
programs keep their grammar, and a mismatched corpus plan is rejected.
V2 checks use `Object.is` to distinguish positive zero. The grammar does not
generate negative zero, NaN, fractions, Infinity or BigInt. Multiply and
arithmetic unary minus remain outside V1/V2; the shared exact evaluator now
represents both zero signs for the separate V3 grammar below.

Conversion/shift/reducer/CLI regression sources and one hand-authored two-backend
probe accompany that extension. No generator, oracle, compilation or tests ran
for this batch. Current V2 runtime behavior, deterministic generated corpus
capture and broader T25 acceptance remain pending. The consumed invariant and
verification scope are recorded in
[`differential-integer-bitwise-grammar.md`](../docs/rust-rewrite/contracts/differential-integer-bitwise-grammar.md).

The same dry batch adds explicitly selected `integer-product-v3`: Add,
Subtract, Multiply and Negate with leaves in `-8..=8`. The existing depths
1–4 give a maximum intermediate magnitude of 8^16 = 2^48, so generation needs
no retry or approximate arithmetic. A closed exact result owns positive zero,
negative zero or a validated safe nonzero integer. Exhaustive evaluation and
rendering preserve signed zero; parenthesized unary operands prevent negative
literals from becoming decrement tokens. The actual grammar constructor,
generator, reducer, schema-v1 replay and CLI consume the extension. V1/V2
draw order, source identity and the V2 default remain unchanged by source
inspection. Finite controls cover all admitted depths, signed-zero laws,
overflow, grammar identity, unary syntax and reduction. Rustfmt parsing and
ordinary reversible patch checks passed; compilation, generation, oracle and
runtime execution, deterministic corpus capture and broader acceptance remain
pending. See the
[`product grammar contract`](../docs/rust-rewrite/contracts/differential-integer-product-grammar.md).

The engine exposes backend identity, a typed normal-or-throw completion, and an
execution-scoped `print` event channel for both backends. Corpus protocol is a
closed, version-bearing type rather than an independently mutable integer and
contract string:

- schema v1 remains `self_checking_no_output` byte-for-byte. It deliberately
  projects the richer engine result back to normal-versus-error disposition
  and verifies that both transcripts are empty.
- schema v2 is `primitive_completion_no_output`. It retains completion kind
  and compares only owned primitive values: `undefined`, `null`, Boolean,
  canonical Number bits, UTF-16 String units and decimal BigInt. Symbol and
  Object are outside this contract and make the report red rather than being
  guessed, coerced or treated as matching types.
- schema v3 is `primitive_completion_print_transcript`. It compares the same
  primitive completion plus the exact ordered root `PrintLine` transcript.
  Both transcripts must be captured; unavailable output is red, event
  boundaries and order are significant, and a match has its own distinct green
  verdict. V3 mismatch signatures hash separately length-delimited typed fields
  and print events rather than embedding raw output.
- schema v4 uses that same observation contract with a complete embedded graph.
  Its entry may be Script or Module, and its versioned case and mismatch identities
  retain the full graph digest. Earlier protocols keep their original wire and
  observation behavior.

All report versions say semantic equivalence is `not_established`; matching
the dimensions declared by one bounded protocol is not whole-program semantic
equivalence. A shared engine failure or no-output contract violation is red,
not a match. Unavailable v3/v4 output is likewise red. The committed v1/v2/v3
cases and feature-gated end-to-end contract tests make this slice durable, but
they do not satisfy object/identity
comparison, full-corpus replay, layered generation, a general AST reducer,
fuzz, performance, or CI requirements.

The protocol's private `OutputComparisonPolicy` now has no incidental debug,
clone, copy, equality or default capability. Its exhaustive projection still
selects captured-empty output for v1/v2 and captured print transcripts for v3/v4;
its exhaustive consumer still checks Wasm before spec-exec and runs before
backend projection. A future protocol row must select a policy, while a future
policy row must define its comparison. The recursive structure guard pins all
seven source mentions, both producer mappings, both consumer rows and that
ordering. The
focused `either_backend_output_makes_a_no_output_case_red` and
`v3_matches_primitive_completion_and_exact_ordered_print_transcript` witnesses
cover both policy rows. At the earlier checkpoint, the structure target passed
4/4, both exact owner witnesses passed 1/1 and the package formatting check was
green. That derive-only
closure changes no wire bytes, fingerprints, mismatch semantics or execution
order. Its contract is
`docs/rust-rewrite/contracts/differential-output-comparison-policy.md`.
Independent review corrected one documentation-only exhaustiveness overclaim;
the executable invariant was clean. The shared workspace compile and repository
gates passed for that source. The v4 projection and producer/projection guard
updates are authored and await the current combined checkpoint.

The replay source domain and closed schema-v3 comparison/signature rules are
normative in
`docs/rust-rewrite/contracts/differential-source-closure.md` and
`docs/rust-rewrite/contracts/differential-primitive-print-transcript.md`.

The private backend execution envelope and its closed result payload are now
Debug-only owned authorities rather than cloneable or equality-comparable
facts. Their seven production mentions and 12 production result mentions form
one lifecycle: the sole producer constructs each complete envelope, comparison
retains Wasm-before-spec-exec borrow-before-consume order, and the five-arm
consuming projection moves every payload into the public observation. The
Rust-lexical guard fingerprints each relevant body and prevents clone/equality
bypasses or alternate projection routes. This capability-only closure changes
no wire bytes, fingerprints, mismatch semantics, verdicts or execution order;
the structure target passed `6/6`, the neighboring output-policy target passed
`4/4`, and all four exact semantic witnesses passed `1/1` at that earlier
checkpoint. The updated v4 execution/projection guards and feature-gated
two-backend replay remain deferred to the broader T25 checkpoint. Its focused
contract is
`docs/rust-rewrite/contracts/differential-backend-execution-ownership.md`.

The in-process Test262 worker now carries one journal admission through the
non-cloneable `RunPhase -> QueuedCase -> CaseAdmission -> AdmittedCase`
ownership chain. Each transition consumes the previous authority, and
`run_case_entry` now consumes the admitted proof instead of borrowing it, so a
single durable admission cannot authorize a second runner entry. The worker
retains only the path string needed by a possible retirement diagnostic; that
string cannot enter the runner. The Rust-lexical closure and its scoped
non-claims are recorded in
[`test262-admitted-case-ownership.md`](../docs/rust-rewrite/contracts/test262-admitted-case-ownership.md).
The focused structure target passes `4/4`; this source-equivalent ownership
checkpoint changes no journal bytes, selected case, execution result, snapshot
or published conformance count.

## Objective

Build automated methods that discover semantic divergences, crashes, hangs and pathological code generation before they become one-off Test262 investigations. Maintain performance without adding test-specific or observably incorrect fast paths.

## Bounded observed-execution contract

The first structured boundary is additive. `Engine::observe_script` and
`Engine::observe_module` return an owned observation whose completion is
either `Normal(value)` or `Throw(value)`. Compiler diagnostics, module-loader
failures, host failures, Wasmtime traps, timeouts and invalid backend ABI data
remain `EngineError`. Existing `run_*` entry points preserve their public API
and abrupt-completion shape: Wasm legacy execution turns a JavaScript throw
into an `EngineError`, while spec-exec keeps its separate legacy execution path.
Throw classification and spec-exec job/print side effects therefore do not
change as an incidental consequence of differential work.

One Wasm string edge case is an intentional correctness change rather than a
compatibility claim. The observed core decodes the runtime's bounded UTF-8/WTF-8
payload directly to UTF-16, so a normal or thrown String containing a lone
surrogate is observable. Legacy `run_*` execution now succeeds for a normal
lone-surrogate completion with a generic non-scalar String note where its old
strict Rust UTF-8 renderer returned `EngineError`; ordinary scalar diagnostics
retain their legacy rendering.

Spec-exec compatibility is behavioral, not merely a matching return type. Its
legacy path still returns immediately after a top-level Script throw,
does not drain that Script's queued jobs, and uses its historical stdout host
printer. The observed path has its own host checkpoint: it drains queued jobs
after capturing the primary completion, while never replacing a primary throw
with a later job failure. Module observation stages parse, load, link and
evaluate separately: parse/load/link failures are engine failures, whereas a
rejected evaluation promise is a JavaScript throw.

The shared value domain is deliberately smaller than a serializer:

- `undefined`, `null`, Boolean, Number, String and BigInt retain owned value
  data. Numbers canonicalize NaN while preserving signed zero, and Strings
  retain UTF-16 code units.
- Symbol is type-only in this batch. Description, registry membership and
  identity remain explicit observation gaps.
- Object is type-only. The observer must not call user coercion hooks, inspect
  properties, publish a backend heap handle or guess an object class.

Each observation also owns the ordered `print` lines emitted by its root
execution context. Those lines are the result of the program's actual host
`print` operation, including its ordinary argument coercions; collecting an
event must not add a second coercion. The event sink is scoped to one execution
and shared with the job checkpoint and module evaluation performed by that
execution. It is not a process-global transcript. Legacy execution selects a
closed delegate-only mode and streams to its existing host hook without
retaining a duplicate transcript.

Diagnostic text stays inside the same boundary. An opaque Symbol or Object
throw receives only a type label; Boa debug rendering and Wasm heap handles are
kept out of public observed outcomes (including their `Debug` output).
Structured Wasm execution never reads the throw-diagnostic globals or invokes
the legacy renderer. A separate legacy mode retains its bounded historical
human diagnostic, using a generic placeholder for valid non-scalar UTF-16.

Corpus and report schema v1 remain the self-checking, no-output protocol with
their original serialization, fingerprint and mismatch-signature vocabulary.
Schema v2 is additive and compares the typed primitive completion directly. A
version/contract cross-pair cannot inhabit the in-memory corpus type. Schema v3
is additive again: a closed output policy requires both root transcripts to be
captured and compares their ordered `PrintLine` strings alongside the v2
primitive domain. No protocol establishes whole-program semantic equivalence.

The v1/v2/v3 corpus protocols couple goal and source into one
outer-source-closed Script program type. The wire still spells both fields, but
a Module or actual/conservatively possible outer Script dynamic import cannot
inhabit that legacy program variant because those schemas carry no graph.
V4 instead owns its Script or Module entry in the complete EmbeddedGraph variant.
Admitted dynamic source does not weaken legacy dependency sealing: imports it
creates reach only the fixed rejecting loader.

This seam does not yet carry a partial transcript inside `EngineError`,
identify Symbols or Objects, expose Symbol descriptions, classify Error
objects/realms/prototypes, or isolate a backend panic/host crash. Agent-produced
output is excluded from the common typed-transcript contract in this batch:
Wasm worker stores run delegate-only and the root typed outcome does not capture
their lines, while spec-exec's shared observed session can currently surface
agent lines. Their presence and ordering are therefore not backend-comparable
and consumers must not rely on them. Differential replay shadows the existing
realm output hook so all protocol versions can still report output emitted
before an `EngineError`; the typed outcome is authoritative for completed
normal and throw executions.

The private replay/comparison implementation now has an explicit compile-time
capability boundary: it exists only under unit tests or the off-by-default
`spec-exec-oracle` feature, while public schemas and the typed feature-off
`OracleNotLinked` result remain in default product builds. Test-only snapshot
mutation and feature-only module-loader fixtures have matching narrow gates,
and the uncalled template-source scanner is deleted. The focused
[`differential oracle compile boundary contract`](../docs/rust-rewrite/contracts/differential-oracle-compile-boundary.md)
records the original SHA-256 witnesses
`8ed6a8721c8d157ea263418918138258a2e68a26670059923570f814b293b69e`
and
`bcecce80a7145d8c00525efc0bbfe0ec3b3a7110a6b7f8aa1590706231d21a89`.
At the Batch BX checkpoint, default and `spec-exec-oracle` package checks are
green without `lila-test262` warnings; the new boundary target passes `3/3`,
the retained output-policy and backend-ownership targets pass `10/10`, the two
focused comparison units pass `2/2`, and the feature-enabled committed
two-backend replay passes `1/1`.

## Differential execution framework

Create a runner that can execute the same generated or minimized program through:

- Lila Wasm-AOT (the product under test);
- Lila `spec-exec` (the internal Boa-based oracle — this differential role is the only permitted use of an interpreter in the project);
- one or more optional external standards-oriented engines configured
  explicitly for developer testing.

The retired JavaScript implementation is neither an oracle nor a runnable
backend. Git history may explain an old behavior, but differential tooling must
not restore or execute that product surface.

The framework must compare structured observations rather than only process exit status:

- printed/output event sequences;
- normal result value and type, including NaN/signed zero/BigInt/String/Symbol-safe rendering;
- completion kind and arbitrary thrown value;
- error constructor/prototype realm where observable;
- property descriptors, own-key order and prototype identities for selected probes;
- side-effect logs for coercion/property/trap/evaluation order;
- timeout, panic, Wasm validation failure and host crash.

The reference backend is an oracle candidate, not infallible truth. Store disagreements with all observations so maintainers can determine which engine is wrong.

## Input generation

Implement layered generators:

1. **Grammar-aware programs:** derive valid syntax from supported AST constructors and feature flags.
2. **Negative syntax/early-error inputs:** mutate bindings, contexts and grammar constraints while tracking expected phase.
3. **Operation sequences:** create values/objects/proxies and invoke shared abstract operations with side-effecting hooks.
4. **Builtin scenarios:** generate receiver/argument combinations, descriptors, subclasses, species constructors and cross-realm objects.
5. **Stateful API programs:** arrays, typed buffers, iterators, promises, generators, modules, collections and Temporal sequences.
6. **Metamorphic transformations:** rename bindings, insert semantically neutral blocks, compare equivalent loops, clone realms and reorder unobservable declarations.

Seed generation from T01's failure metadata and newly green Test262 cases, but do not copy expected answers into compiler code.

## Reduction and replay

- Build an AST-aware reducer that preserves the observed mismatch/crash/timeout.
- Reduce statements, expressions, bindings, object properties, pattern features, numeric/string values and harness setup.
- Preserve parse goal, strictness, feature flags and async/module execution mode.
- Store minimized cases in a versioned regression corpus with the seed, engine versions, compiler commit and mismatch signature.
- Provide one command to replay the entire corpus and one command to reproduce a single case.

Every fixed crash or semantic mismatch should add the minimized case to a focused crate/CLI test when practical.

## Robustness fuzzing

Fuzz the following boundaries independently:

- JavaScript parser and early-error classifier;
- AST-to-IR lowering;
- IR validation and optimization passes;
- Wasm emitter and module validation;
- value serialization/debug reporting;
- Test262 frontmatter/prelude/snapshot parsers;
- RegExp, JSON, URI, numeric and Temporal parsers;
- module resolver/loader path handling.

No arbitrary input may panic the Rust process, invoke undefined behavior, allocate without limits, or emit invalid Wasm without returning a structured error.

## Performance measurement

Create reproducible benchmarks and tracked budgets for:

- parse, lower, emit and Wasm validation time;
- generated Wasm byte size, function count, helper duplication and static data size;
- runtime throughput/latency for representative language and builtin workloads;
- peak linear memory, allocation rate and GC pause/retained size;
- full Test262 node duration, timeout count and slowest cases;
- Intl/Unicode/time-zone data footprint;
- debug vs optimized compiler builds.

Record hardware/toolchain metadata and compare medians or robust percentiles. Do not treat a noisy single run as a regression.

## Optimization rules

- Correctness comes first; every optimization needs a semantic guard and slow/fallback path.
- Static shapes, constant folding, builtin direct calls and allocation elision must deopt/fallback when prototypes, accessors, proxies, realms, species or coercions make them observable.
- Performance fixes may not branch on Test262 paths/source text.
- A faster timeout is not a pass; timeouts must become completed correct executions.
- Add differential tests specifically for every optimization guard.

## CI tiers

Define three practical tiers:

- **PR-fast:** deterministic unit tests, regression corpus, small fuzz seed set, fake suite and focused changed-family tests.
- **Nightly:** longer differential/fuzz campaigns, sanitizer-like Wasm validation, stress GC/agents and performance smoke comparisons.
- **Full conformance/release:** complete pinned matrix, full corpus, repeated determinism and benchmark report.

CI artifacts must retain minimized failures, random seeds and comparison reports so failures are reproducible locally.

## Acceptance criteria

- Differential runs produce stable machine-readable mismatch signatures.
- The reducer turns seeded complex mismatches into meaningfully smaller reproductions.
- The regression corpus is deterministic and green on both debug and optimized builds where applicable.
- Parser/lowering/emitter fuzz targets run for a sustained campaign with zero unhandled panics or invalid-memory behavior.
- Performance dashboards identify compile/runtime/size regressions by subsystem and pin.
- Every enabled optimization has a test that forces both fast and fallback paths.
- Full-suite timeout counts trend as explicit performance debt and reach zero before T26 closes.

## Required tests

```sh
cargo test --workspace --quiet
cargo test -p lila-ir fuzz_regressions --quiet
cargo test -p lila-aot-wasm fuzz_regressions --quiet
cargo test -p lila-test262 snapshot_determinism --quiet
cargo test -p lila-test262 --features spec-exec-oracle differential::generated_arithmetic::tests::committed_generated_arithmetic_case_replays_through_both_backends -- --exact
cargo run -p lila-cli --features spec-exec-oracle -- differential generate-arithmetic /tmp/lila-t25-arithmetic.json --grammar integer-arithmetic-v1 --seed 1 --checks 4 --depth 2 --max-replays 64 --oracle spec-exec
cargo run -p lila-cli --features spec-exec-oracle -- differential generate-arithmetic /tmp/lila-t25-bitwise.json --grammar integer-bitwise-v2 --seed 1 --checks 4 --depth 2 --max-replays 64 --oracle spec-exec
cargo run -p lila-cli --features spec-exec-oracle -- differential replay /tmp/lila-t25-arithmetic.json --oracle spec-exec
cargo run -p lila-cli --features spec-exec-oracle -- differential replay crates/lila-test262/tests/differential/v2/t25-foundation-primitive-number.json --oracle spec-exec
cargo run -p lila-cli --features spec-exec-oracle -- differential replay crates/lila-test262/tests/differential/v3/t25-foundation-primitive-number-and-print.json --oracle spec-exec
```

The exact CLI may differ, but equivalent seed/replay/reduce commands, CI jobs and artifact retention are required before closing this task.
