# Lila Rust AOT + Test262 execution plan

## Workspace audit and remaining acceptance — 2026-10-08

All thirty task states remain unchanged: four complete (T00/T27/T28/T29),
25 in progress and T26 blocked. The frozen `f493e2988` tooling sweep is complete:
38/38 commands, 405/405 Python methods, 318 recorded subtests and zero skips,
errors or failures. Both shell regressions pass, including the actual committed
historical-publication guard. The separate 810-target all-feature Rust sweep
was deliberately stopped while red; its retained outcomes are diagnostic baseline
receipts. Its final command states are 391 passed, 127 failed, one interrupted
and 291 pending; completed verdicts record 5,762 passing checks, 355 failures
and three ignored documentation examples. The complete repaired-source sweep
remains pending.

Prepared repairs distinguish stale pre-GC fixtures from lost type invariants and
observed semantic defects. They restore consuming value/reference roles and
correct actual Promise, environment, Realm, Locale, Temporal and tail-call
owners while keeping meaningful original assertions. The joined source and
runtime ABI need fresh type, artifact, native and cache verification before a
new broad checkpoint. Earlier focused passes remain evidence for their recorded
source; they do not validate unapplied repairs.

The next closure evidence is concrete:

| Scope | Evidence still required |
| --- | --- |
| T02/T03 and shared local criteria | Joined types and source guards; all affected native/artifact controls; complete workspace and fake-suite publication; default-only oracle refusal and product dependency checks. T02 also needs representative before/after behavior and comparable build-time/binary-size measurements. |
| T01 | Complete current compiler-bound pinned matrix, two byte-identical backlog generations and actual canonical publisher output. A complete red baseline can satisfy baseline reporting; it cannot satisfy T26. |
| T05/T21 | Genuine weak/ephemeron/finalization facilities, plus actual rooting/cycle stress. Strong copying GC is already present. |
| T07/T08/T10–T12/T14–T24 | Their full pinned semantic families and specific native/resource criteria. Local test success does not replace the required zero-failure subtree evidence. |
| T13 | Complete finite-source/context/Realm controls and honest unsupported accounting. The permitted unavailable-source boundary remains incompatible with literal T26 closure while any pinned case needs it. |
| T22/T23 | Correct remaining Temporal/Intl semantics and reproducibility on the supported host set with identical pinned data. |
| T25 | Unchanged cold/warm timing controls, the complete debug/optimized corpus, sustained differential/robustness campaigns and measured subsystem policies. The original campaign is still red at fresh Temporal P compilation after warm CrossRealm succeeds. |
| T26 | Current all-zero complete aggregate, two fresh identical full runs, real shard/resume equivalence, required stress/product checks and compiler-bound publication. |

Retain the serial resource policy: one CPU, 4096 MiB aggregate RAM, zero swap,
one compilation/test worker and one retained native module up to 64 MiB.
All-features excludes four default-only oracle-refusal controls; run those
separately. Three ignored documentation examples do not cover the four ignored
runtime acceptance controls (one heap stress and three timing tests).

After the complete repair batch, refresh the workspace with the existing capped
driver and a fresh evidence directory:

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- \
  python3 target/verification-tmp/workspace-serial-audit.py --all-features \
  --output target/verification-tmp/workspace-all-features-repair-checkpoint
```

Resume is valid only for identical source, toolchain, environment and feature
scope. Preserve every failed or interrupted receipt. Refresh pinned counts only
through the normal publisher after its complete matrix; the recovered October 2
pair and generated README block remain historical and have no current compiler
binding. No task status or pinned count changes in this checkpoint.

## Runtime persistence and verification — 2026-10-08

Validated runtime artifacts now persist through the existing bounded cache;
Chinese/Dangi catalogs are constructed at Rust build time. The full CoerciveAdd
runtime helper receives the actual caller execution Realm and Environment,
preserving evaluation order, conversion hooks and complete abrupt results.
The R/P ABI is version two. Packaged Wasmtime cache identity no longer inherits
an unrelated parent Git checkout or fragments native R by executable mtime.

`production-addition-helper3` finishes in 330 watched seconds under the unchanged
one-CPU/4096-MiB/no-swap launcher and one-entry/64-MiB native retention limit.
The all-feature/all-target workspace type check passes in 53.041 seconds.
All 31 focused controls pass: seven helper ABI, three artifact, six runtime-codec
and fifteen native controls (two caller-Realm addition, seven numeric/bitwise,
five compound-addition and one saved-left-value). None is ignored. Formatting,
module ownership, identity, host ABI, task-plan, legacy, shortcut/accounting and
whitespace checks pass. The 36-file Python/two-script tooling sweep completed
all 405 discovered Python methods: 387 passed and 18 parent methods failed,
with zero skips. Thirty-three of 38 commands passed. Failures identify three
stale Intl expectations, one stale publication-supervision fixture and missing
canonical historical publication files. The affected rerun passes all 84 Python
methods with no skips. The restored exact historical pair passes the real
artifact guard in an isolated proposed commit view, with original repository
files, index and refs unchanged. The later complete tooling sweep passes all
38 commands and all 405 Python methods, as recorded above.

`production-startup-tables1` passes its all-feature/all-target type check in
64.886 seconds and all 40 focused controls: one exhaustive Unicode mapping,
29 named-zone admission, six image/projection and four native Date/Unicode
tests. All generated identities and repository source audits pass. Exact TZif
payloads share validated data within one constructor; original scalar case
mappings are built once with compiler-version and byte-parity checks. The
checkpoint finishes in 330 watched seconds under the same resource limits.

The campaign gate remains red at its original 5,000-ms deadline. The cold attempt
stops during native R loading. After the native controls, both CrossRealm
baseline and transformed programs complete and match SpecExec, including all
eight actions. Temporal seed 3215 is now reached but its fresh P times out
during native compilation; its transformed program remains unrun. This is
partial campaign progress. The [T25 checkpoint](25-differential-fuzzing-performance.md#runtime-persistence-and-cache-acceptance--2026-10-08)
retains the measured phase costs.

The preceding cache checkpoint passes four identity controls and both process
controls: verified different executable mtimes produce zero hits/two misses,
then one hit/one miss while different P executes correctly. These controls are
now wired into the capped CI persistence job; a new CI result is still required.
The three stale parser/BigInt guards also pass their focused repairs without
weakening the original ownership checks. The earlier default sweep remains
incomplete: 139 targets passed, three failed, one was interrupted and 666 were
pending; completed targets contained 2,297 individual passes and three failures.
Those historical outcomes are not a current complete workspace result.

Refresh the unchanged campaign through the same resource boundary:

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- \
  cargo test --locked --offline -p lila-test262 --features spec-exec-oracle \
  --test differential_generated_campaign \
  realm_and_temporal_campaigns_retain_all_eight_actions_and_actual_paired_observations \
  -- --exact --test-threads=1 --nocapture
```

All thirty task statuses remain unchanged: four complete (T00/T27/T28/T29),
twenty-five in progress and T26 blocked. Remaining non-performance blockers
include real weak/ephemeron support, the Temporal contextual rounding-window
gap, unsupported dynamic-source cases in literal conformance accounting and
independent-host acceptance. Complete current pinned evidence and both canonical
compiler-bound publication artifacts remain pending. The recovered October 2
publisher pair matches the retained aggregate and generated README block byte
for byte; its older schema has no compiler binding. Exact restoration preserves
historical evidence without changing counts or claiming a current refresh.
Full workspace/fake/pinned verification,
ignored product stress/timing controls and sustained debug/optimized campaigns
remain required. No task status or pinned conformance count is promoted.

## Closure audit and verification — 2026-10-08

The full task plan remains open. The current runtime explicitly rejects real
weak reachability/ephemeron operations (T05/T21). Temporal's Apia rounding
behavior still has an [open specification issue](https://github.com/tc39/proposal-temporal/issues/3310)
and an [unmerged draft proposal](https://github.com/tc39/proposal-temporal/pull/3318),
rechecked on 2026-10-08. T26 requires a fresh complete zero-failure aggregate and
canonical publication; neither a source audit nor these focused checks closes
that gate. Historical Test262 counts are unchanged.

The integrated batch retains main inside its module package, exposes globals
only through a borrowed sealed view, and extracts literal roots, RegExp
publication and function metadata into cohesive children. Existing parent
budgets remain unchanged. The public emitter result `WasmArtifact` is nameable
without exposing package constructors. Normalization avoids allocation for
identity scalars, and case folding enumerates the pinned mappings directly.
Exhaustive reference comparisons preserve every table value and its order.
Local recovery snapshots are preserved and excluded from Git staging; tracked
and product identity checks remain strict.

Fresh verification uses the unchanged 4096-MiB aggregate RAM, zero-swap,
one-CPU launcher and one-entry/64-MiB retained-module cache:

- `task-closure-final-20261008`: the final all-feature/all-target workspace
  check passes in 65.07 seconds; final formatting passes. The preceding
  `task-closure-types2-20261008` check also passed.
- `task-closure-unicode-parity-20261008`: both exhaustive inventory controls pass.
- `task-closure-ir-20261008`: all 2,161 unit/integration checks and five doctests
  pass; one existing documentation example is ignored (225 watched seconds).
- `task-closure-focused-20261008`, `task-closure-artifacts3-20261008` and
  `task-closure-structure2-20261008`: all 24 focused GC, linked-runtime,
  artifact and ownership controls pass after restoring the canonical owners
  and stale source markers. Artifact checks validate both linked modules,
  exact canonical import/export types and actual pooled-helper byte independence.
- All seven native backreference-folding controls pass. The final Unicode run
  passes all three strict/sloppy cohorts in 55.87 seconds. Its Realm fixture
  now installs the ordinary `$262` wrapper used by neighboring Engine tests;
  original assertions remain. The same setup correction passes the URI and
  legacy-RegExp Realm cohorts in the final checkpoint (two additional functions).
- `task-closure-audits-20261008`: module budgets, identity, host ABI, task plan,
  legacy retirement, three identity regression tests, formatting and whitespace
  pass. Canonical shortcut reports are regenerated and verified: 26 classified
  observations, including zero semantic shortcuts.

Final index review preserves byte-exact pinned CLDR 47 input files, including
their upstream whitespace. The repository-owned changes pass the whitespace
check with that pinned input directory excluded.

The original CrossRealm seed 1270 still exceeds its unchanged 5,000-ms deadline
in the rebuilt final worker (`task-closure-final-20261008`), during emission after
1.83 ms parsing and 129.95 ms lowering. SpecExec completes all eight actions; transformed CrossRealm
and Temporal remain unrun. The native disk cache had been exercised by the
preceding tests, but the raw runtime module is regenerated in every new worker.
The retained request and reports are under
`target/verification-tmp/lila-generated-campaign-302393-1791478300025590477-0`.
Profiling the preceding source measured 6.452 seconds of cold emission and a
41.2-MB runtime, including Unicode, zone certification and Chinese/Dangi catalog
construction. The Unicode repair preserves semantics but does not close this
performance failure. The final checkpoint is therefore red despite the passing
native, type and formatting steps. Canonical `published-status-wasm-aot.json`
and `.txt` were absent at that earlier checkpoint; their later exact historical
restoration is recorded above. Fresh compiler-bound publication remains pending.
No task status or published conformance count is promoted.

## Focused iteration checkpoint — 2026-10-08

The fresh `fast-iteration-types-20261008` checkpoint passes
`cargo xc --keep-going --locked --offline --all-features`. The shortcut source
inventory and generated accounting are refreshed from their canonical inputs;
both checks and the task-plan validator pass. Accounting remains 26 classified
observations, including zero semantic shortcuts.

The newer retained `target/watched/native-q1.jsonl` records 46 completed native
functions: 45 passes and one ShadowRealm embedded-import failure. This includes
passes for all four earlier global/class failures and the global-read and Switch
cohorts. That journal used concurrent two-CPU lanes; it does not establish the
one-CPU/4096-MiB verification contract. Its passes retain their original source
scope and are not a fresh combined-suite result.

`fast-iteration-shadow-realm-20261008` now passes the sole failed function,
`embedded_realm_requests_keep_aliases_distinct_from_script_edges_and_wait_for_tla_cycles`,
in 102.47 seconds. The existing fixture correction detaches imported wrapped
functions before calling them, so an Array receiver does not cross the Realm
boundary. One test passes, none fail or are ignored; nine are filtered out.
The fresh type and native checkpoints confirm the kernel 4096-MiB/no-swap,
one-CPU limit and one-entry/64-MiB retained-module cache.

`fast-iteration-campaign-20261008` rebuilds the selected differential worker and
replays the original CrossRealm seed 1270 at its unchanged 5,000-ms deadline
under those same limits. It still times out during Wasm emission, after 2.22 ms
parsing and 148.87 ms lowering. Worker stderr is now 118 bytes, with no missing
Collator metadata flood. SpecExec completes all eight actions; transformed
CrossRealm and Temporal remain unrun. The test is red, with semantic equivalence
unestablished. Exact requests and reports remain under
`target/verification-tmp/lila-generated-campaign-4136252-1791474858413422907-0`.

Next, profile the remaining emission cost using this retained request, then
rerun the same bounded campaign after a focused repair. The native queue's
original failed fixture is resolved, but performance acceptance, full pinned
conformance, canonical publication and task-wide acceptance remain open.

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

See the [property-fact contract](../docs/rust-rewrite/contracts/primitive-property-read-effects.md),
[source-identity contract](../docs/rust-rewrite/contracts/compiler-source-identity.md) and
[ShadowRealm contract](../docs/rust-rewrite/contracts/shadowrealm-implementation-sequence.md).

## Earlier verified checkpoints — 2026-10-07

The complete Temporal conversion batch passed `cargo xc --keep-going --locked
--offline --all-features` across the workspace and all test targets, plus all
28 focused AOT controls on 2026-10-07. Every body in that scheduling fixture is below 1 MiB.
The scheduling fixture has 15,279,867 code bytes; Duration.round is 888,220,
bootstrap is 863,565 and the async run is 208,269. Both conversions transport the
actual callable FunctionContext and caller Environment.

Native compilation and instantiation completed under 4096 MiB; compilation took
82.57 seconds. Independent host signatures now have separate canonical type
groups with a compile-time ordering assertion. Main's defining-Realm repair
passes the workspace type check and its native prototype/captured-cell control.
The original lifecycle fixture now executes without the null trap, but its
missing output exposed premature async completion on Await. The written shared
return repair covers ordinary Await, module handoff, async disposal and awaited
iterator steps/closing. The joined repair gives known Symbols a distinct IR
variant and restores explicit progress transport between Atomics waiting and
Promise jobs. `tasks-symbol-progress1` passes the full type check, all 20 Symbol
IR controls and three native controls: ordinary Await, awaited iterator closing
and the original mixed array lifecycle in strict and sloppy modes. Its two
failures have a joined repair: `tasks-constructor-progress1` passes the workspace
type check and all three affected native controls. All fifteen Symbol identities
agree across constant, dynamic and created-Realm reads; waiter reactions can
notify another waiter; all twelve typed-array constructors keep the actual
storage kind separate from newTarget prototype selection. Planning and emission
consume one compiled builtin list. No controls were ignored. The full IR and
first native record/campaign acceptance batch completed under the same resource cap.
The completed source batch includes genuine
JSON module records, bounded URI/prelude/filesystem robustness, generated
cross-Realm/Temporal cases, and retained original resource-finalizer ownership.
The completed source join adds post-job rooted completion snapshots and the
explicit v7 differential protocol. One checked traversal owns canonical graph
identities and finite budgets; both backend adapters read actual retained data
without executing JavaScript hooks. Unsupported exotics and budget exhaustion
remain red. The whole-workspace, all-feature, all-target type check passes.
The first focused pass completed 103 tests successfully and failed seven, with
none ignored. Six source assertions and an oracle source-import attribute/error
issue have a joined repair that passes the type check and all 22 affected controls.
Whole native, broad and pinned acceptance
remain pending.

T25 passes five IR-admission controls, 19 robustness library controls, two
worker/CLI native-input replay controls, two compiler-inspection controls,
three Engine RSS sampler controls and all 24 CLI performance controls. Native
runtime-profile checks and sustained campaigns remain pending. The latest
complete IR run passes all 2,096 unit/integration controls across 141 targets with
none failed or ignored. Five compile-fail documentation controls pass; one
pre-existing illustrative snippet is ignored. The native cohort finishes with
five passing and five failing test functions, none ignored. URI execution,
worker/CLI prelude and filesystem checks, created-Realm graph anchors and
explicit graph rejection pass. A joined source repair covers private loop-cell
lookup, duplicate inferred function names and the Module fixture URL. The
worker's earlier cold-compilation timeout remains performance evidence, and the
campaign failure at its five-second budget remains open. Deadlines are unchanged.
`tasks-loop-name1`
passes the workspace type check and four of seven native controls: the original
post-job graph, both inferred-name controls and generator phase ordering. Three
failures have a written source repair for initializer/selector binding lifetimes
and private module resume, which had repeated instantiation and left JSON defaults
undefined. The joined admission repair retains checked loop/CaseBlock targets
through catch/finalizer regions. The workspace type check passes in 51.22
seconds, along with 18 focused controls and all four original mixed-loop Wasm
artifacts. `tasks-scope-native5` finishes with six native test functions passing,
two failing and none ignored. Both ordinary-generator, both module-resume and
both rooted worker controls pass. Mixed phase fixtures pass both modes; the
completion fixture fails at the first awaited iterator value after a checked
loop. `tasks-iterator-intrinsics3` verifies the joined input-alias, head-scope and
installed-constructor repairs: the workspace check passes in 37.60 seconds, and
eight native test functions pass with two failed and none ignored. Both original
failures are resolved. All four mixed-loop fixtures, the JSON fixture, all three
ForOf family controls in both modes, Annex B ForIn and both module intrinsic/error
controls pass. The remaining ForIn failures are a labelled async admission gap
and an ordinary-generator head TDZ assertion. The joined repair admits checked
iterator labels and counts every physical named-function self record in capture
analysis. The workspace check passes in 53.98 seconds. `tasks-for-in-capture2`
passes the full IR suite, both labelled Wasm admission controls and all five
affected native tests. The original ForIn controls pass both modes, along with
named-function assignment and direct-eval self-binding controls. Converter
literals now share the actual helper gate. `tasks-temporal-literals1` passes the
workspace check in 45.97 seconds, the PlainDate-only artifact control and all
five remaining Realm controls in both modes. Fresh test processes and a 64 MiB
retained-module cache pass the publication case that previously hit the 4 GiB
cap. `tasks-remaining-isolated1` completes 50 native test functions with 39 passing,
11 failing, none ignored and no OOM. All selected Intl, array storage, RegExp and
CLI manifest controls pass. The joined source repair retains prepared Script
kind, includes shared-converter host imports and corrects the oracle URL,
resource fixture grammar and lazy-capacity assertion. All nine corrected
resource parsing/lowering checks pass. Calendar diagnostics identify omitted
lunisolar arithmetic domains and accessor bags copied before observation. The
source repairs use exhaustive arithmetic dispatch and original accessor bags.
Native verification remains open. Original execution deadlines remain.
The joined repair passes the all-feature/all-target workspace check in 49.14
seconds and the converter artifact regression. Its native collection completed
17 functions: 12 pass and five fail; five remain unfinished after an intentional
stop for repairs. Follow-up diagnostics identify incorrect leap-month `since`
assertions and calendar-unit Duration addition in the fixtures. Those corrections
and the resource binding-alias repair are written. Their successor passes the
full workspace check in 63 seconds and completes all ten functions: eight pass
and two fail, with none ignored. Calendar fixtures, ordinary/async resource
scopes, switch resource lifetime, indirect eval and fresh Script cells pass both
modes; nested direct-eval context also passes. The two remaining repairs move a
resource lifetime observation inside its finalizer and admit known Symbols to
Realm Script's existing ToString error path. Unknown executable source remains
an AOT capability gap. The follow-up passes the workspace check, all 2,097 IR
tests across 141 targets, five compile-fail documentation tests and the resource
completion fixture. One pre-existing illustrative documentation snippet remains
ignored. Non-String eval identity passes both modes. The final Realm fixture
captures its Symbol before calls that can invalidate mutable global facts and
passes both modes in 246.96 seconds. All three affected IR admission controls
pass. The collected resource/calendar/Script repair batch is verified; full
pinned conformance and remaining campaign, performance and host acceptance stay open.
Earlier Intl/oracle
results keep their original scope. Every payload
retains one CPU, 4096 MiB aggregate RAM and zero swap. Current source, native,
broad and pinned acceptance remain open; generated conformance counts are unchanged.

Ordinary-generator and plain async array/With/ForIn owners have independent
source reviews. Mixed async generators now have reviewed complete classic-loop,
If, value, With, Switch and ForIn source owners. The successor now joins mixed
literals, optional/private/Super operands, suspended assignments and per-key
initialization, catch patterns and lawful sloppy head initializers. Ordinary
generators and plain async functions share the same consuming Reference algorithms.
The successor extends complete classic loops to Async and complete ForOf/ForAwait/
ForIn to the three resumable protocols. Common resource capabilities cover lexical
lists, iterator heads, classic heads and nested blocks in switch clauses. Direct
clause-level resource declarations remain early errors. Suspended lexical async
SuperCall retains the original derived new.target/constructor before arguments.
Private source certificates retain exact
Await/Yield kinds, phase ranges and captured Block/Try ancestry. The original
Reference, environment, iterator, completion and request machinery remains
shared. Checked Object Environment and two-cell CaseBlock storage proofs gate
all actual With and complete generator Switch carriers. Selector terminal values
are consumed before their generated operand bindings leave scope.
Meaningful controls are written and typechecked; execution remains pending.

Custom Intl source has eight independently reviewed locale filters: List,
RelativeTime, DisplayNames, Duration, coupled NumberFormat/PluralRules,
DateTimeFormat, Collator and Segmenter. Wasm build/run accepts
`--intl-profile custom:ID` with `--intl-list-locales`,
`--intl-relative-time-locales`, `--intl-displaynames-locales`,
`--intl-duration-locales`, `--intl-number-locales`, `--intl-datetime-locales`,
`--intl-collator-locales` and `--intl-segmenter-locales`. Original global
authorities and actual fallback dependencies stay available. Segmenter retains
all eight global Unicode/LSTM/dictionary rows for arbitrary input scripts while
filtering actual locale-tailored overrides. SDK, cache, workers and CLI
consume the selected owners; this successor has no runtime acceptance.

The source batch adds complete mixed array patterns and synchronous/awaited
iterator phases. One actual resource capability now spans each lexical list,
per-key iterator initialization/body, classic For or Switch CaseBlock. Exact
source registrations and original finalizers are consumed by opaque carriers;
the shared native iterator, environment and disposal algorithms remain physical
owners. Meaningful ordering, TDZ, cancellation and damaged-carrier controls are
written and unrun.

The complete pinned Conformance image profile, strict Custom locale/currency
manifests, currency/calendar/numbering/named-zone projections and admitted data
export/import are authored. Checked service selection now physically omits frames
outside the real dependency closure and separately gates public operations.
Manifest v6, cache v18, sparse export and Wasm consumers share that authority.
RegExp has the linked choice arena, recursive required runs, completed optional
child snapshots and discharged capture-assertion admission. Independent
repetition uses a proved input bound and preserves the exact optional gap;
pure empty syntax is simplified in the original literal/runtime producers.
Real multi-seed arithmetic/object/module campaigns now include attributed
dynamic imports and top-level Await through `module-graph-v2`.
`control-flow-v1` adds checked branches, finite loops, finalizers, captured cells
and all four function protocols through the real completion/print observer.
Negative-source campaigns retain actual frontend phases; stateful builtin and
metamorphic campaigns retain both original source variants and real observations.
Bounded robustness runs and crash/timeout reduction retain exact native inputs
and actual worker stages. Original Test262 failures/newly passing cases now have
a separate checked seed/replay bridge preserving modes, harnesses and ownership.
Compiler and runtime profiling retain the same twenty-source corpus; runtime
reports actual phase timings and GC capacity snapshots, with unavailable
allocation, collection, live-size, pause and peak counters stated explicitly.
Conformance performance reports consume original admitted snapshots plus actual
node invocation timers; data reports measure existing admitted Intl bundle frames
with Unicode/tzdb provenance. Historical timing gaps remain unavailable.
Explicit performance budget checks consume original repeated compiler/runtime
evidence and retain every selected metric's decision; calibrated thresholds and
actual measurements remain pending.
Reproducible cross-host evidence and pinned conformance remain open. Every payload uses one CPU, 4096 MiB aggregate RAM, zero swap and serial
workers. The initial type/source checkpoint and IR failure collection are
complete; a coherent repair batch is under way. Generated suite counts and
earlier result scopes are unchanged.

## Earlier Switch/object/RelativeTime source pass — 2026-10-06

Ordinary generators now have authored Switch regions, complete yielding Throw
operands and object-pattern key/target/default/rest suspension owners. The same
pattern lowering retains original lexical and scoped For cells. A parser-scope
proof separates inferred class display labels from explicit inner class bindings,
preserving outer reads, closures and TDZ. Actual frontend cover conversion and
native destructuring consumers are joined. These changes have independent source
reviews and meaningful strict/sloppy GC and completion controls; they are unrun.

One checked Custom Intl profile now composes independently filtered ListFormat
and RelativeTimeFormat data. List filtering retains Duration dependencies;
RelativeTime filtering retains the actual Number owner. SDK, cache, workers and
CLI share that selection. Wasm build/run accept `--intl-profile custom:ID` with
`--intl-list-locales fr,ja`, `--intl-relative-time-locales fr,pl`, or both flags.
The source includes native frame admission, formatting, cache and CLI controls.
Other component filters and complete Conformance producers remain open.

Array-owned suspensions, broader ForIn/With and suspended iterator/resource/mixed
async owners remain source work. Compilation, native execution, measured size
and pinned conformance follow the complete source pass. Every verification
payload is capped at one CPU, 4096 MiB aggregate RAM, zero swap and serial workers.
Generated counts remain unchanged; historical results retain their earlier scope.
See the [batch workflow](../docs/rust-rewrite/batch-workflow.md).

## Earlier exact/optional/List source pass — 2026-10-06

The uncompiled successor adds exact decimal RegExp bounds and source-sized native
counters, complete optional-chain suspension regions and terminal property Delete,
eager assignment patterns/scoped For heads, and a physically filtered ListFormat
profile shared with DurationFormat. Independent source reviews cover these owners;
the corrected capture-only replay proof remains separate from general huge nullable
acceleration. Existing controls are retained, with obsolete numeric-domain assertions
updated to the admitted exact-bound behavior.

Compilation, native execution, artifact-size measurement and pinned conformance
remain deferred while the remaining source batches are written. Historical checks
below retain their exact earlier scope. Every verification payload uses one CPU,
4096 MiB aggregate RAM, zero swap and serial workers. Generated counts are unchanged.
See the [batch workflow](../docs/rust-rewrite/batch-workflow.md).

## Current atomic source pass — 2026-10-05

The invariant-first implementation now has source chunks for the whole-value GC
compiler path, native entry families, sparse Array authority and disposal
continuations. The complete Temporal provider/calendar/epoch and method
composition is written and sealed. Final heap/provider and passive metadata
retirement, retained compiler controls and source-guard composition have
finished independent source review. The combined source checkpoint includes
types, meaningful finite semantic controls and current contracts.

Before focused verification, the complete source batch and preceding integration
repairs passed the whole-workspace, all-feature, all-target Rust type check on 2026-10-05 under the
confirmed 4 GiB aggregate cap, zero swap, grouped OOM and one CPU. Authored Rust
test targets typecheck. The cheap checkpoint through MAIN132 is complete: 22 resource/process
controls and all 13 commands pass, including the affected formatting, module
assertions and 56-entry shortcut inventory checks. Emitted Wasm validation,
runtime and conformance remain unverified for this batch. Older results below
retain their recorded scope. All unmet acceptance criteria remain open,
including the explicit weak/ephemeron facility
gap and the release gate. Complete coherent source batches precede verification
with the confirmed 4 GiB aggregate kernel cap and serial workers described in the
[batch workflow](../docs/rust-rewrite/batch-workflow.md).

The first default-product focused compiler/GC checkpoint attempted 86 test
functions: 60 passed, 21 failed and five have no completed results because the
sparse Array target aborted with a stack overflow. No tests were ignored.
The shared emitter, planning and Realm failures are being repaired as one source
batch before affected regressions resume. Broad suites and pinned conformance
remain unverified; these focused results do not close task acceptance.

The first repair batch through MAIN134 passes the whole-workspace,
all-feature, all-target type checkpoint and all four affected source checks.
The affected 28-test attempt then aborted its dense-storage target with a
compiler stack overflow. It was cancelled after source inspection proved the
remaining ObjectDelete/ProxyDelete emitter cycle: zero test results completed,
all 28 remain incomplete, and no tests are counted as skipped or passing.
The owned runner and descendants were cleaned up. The confirmed 4 GiB cap,
zero swap and one worker remained enforced. A complete source successor covers
the typed Delete helper, supervised Test262 compilation deadline, selected
object differential probes and Realm-owned RegExp legacy state before another
verification attempt. This successor has no inherited type/runtime/conformance
pass; task acceptance and canonical publication remain open.


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
images remain open; current Custom names group exact pinned components and
produce their selected Custom identity through artifact admission. Named-zone
settings retain their complete provider-identity check. The library now carries
`CompileOptions.intl_profile` through preparation, compiled units, cache identity
and agent workers. The CLI accepts `--intl-profile minimal|custom:ID` for Wasm
builds and Script/Module execution. One admitted bundle supplies both emitted
frames and compiled catalogues; supported-values-only programs carry its full
bound image group. Explicit unsupported choices reject before source execution.
These consumer controls are authored, with compilation and runtime acceptance
still pending. See the [selection contract](../docs/rust-rewrite/contracts/intl-compilation-profile.md).
Published full-suite counts are unchanged.

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
head joins now have independent source peers. Suspensions inside patterns, broader iterator/control,
mixed async-generator and resource continuations remain explicit compiler work.
See the [pattern initializer contract](../docs/rust-rewrite/contracts/generator-pattern-initializers.md)
and [statement consumer contract](../docs/rust-rewrite/contracts/generator-statement-consumers.md).

Recent staging directories and local receipts under `target` disappeared during
the interrupted turn. The integrated source and three watched logs survive.
The source successor is retained in the working tree with small recovery
notes outside `target`; missing receipts are not recreated as new proof.

This directory is the 30-task epic-level implementation backlog and
current-status record for the Rust rewrite. It is designed so multiple contributors can work
concurrently without turning the remaining large IR and Wasm backend modules
into permanent merge-conflict bottlenecks. Individual task status fields and
their dated current-state sections are authoritative only until the next
current-pin aggregate or material repository change; generated Test262
artifacts remain the authority for conformance counts.

The north star is the repository contract in `AGENTS.md`: Lila compiles
JavaScript directly to Wasm, does not ship an interpreter/VM inside the
artifact, and drives the pinned real Test262 suite to zero unowned failures.
Fake-suite results are smoke tests only. An `Unsupported` result is visible
debt, never a passing result, and must not be hidden in a skip list or status
denominator.

Backend policy: `wasm-aot` is the product. It is the execution path every task
targets, the only backend whose results may be published as Lila conformance,
and the only backend the T26 release gate accepts. `spec-exec` (the Boa-based
engine in `crates/lila-spec-exec`) is an internal differential-testing and
debug oracle only, used by T25 and quarantined by T27 — never the CLI default,
never a silent fallback, never part of an emitted artifact, and never a source
of published conformance numbers. Wherever a task mentions running spec-exec,
that run is oracle triage; the Wasm-AOT run is the requirement.

## Current implementation pass — 2026-10-03

The user requested invariant-first dry implementation across the remaining
tasks before testing individual pieces. Reviewed production changes, types,
regression sources and documentation are being integrated together. Source
inspection and formatting continue; compilation, runtime tests and conformance
sweeps wait for the implementation checkpoint described in the
[batch workflow](../docs/rust-rewrite/batch-workflow.md).

The current source batches include complete finite computed RegExp sets,
same-chain optional Call awaits and bounded generator Property/Call chains.
The actual weak builtin boundary now consumes the selected runtime's unavailable
facility, with strong-retaining weak producers retired and capability failures
kept distinct from JavaScript exceptions. These are dry implementations with
authored controls; they carry no inherited compile, runtime or task acceptance.

The previous broad run was interrupted and retains its authentic partial
results. Those results do not verify the combined workspace. All unmet task
acceptance criteria stay open, including semantic GC, weak reachability and the
release gate; published conformance numbers remain unchanged.

## Current status snapshot — 2026-09-29

| State | Tasks | Repository evidence |
|---|---|---|
| Complete | T00, T27-T29 | Repository contracts are enforced, the interpreter is quarantined from the product, the legacy JavaScript product is retired, and the Lila identity cutover is verified |
| In progress | T01-T12, T14-T25 | Substantial implementation exists; T23's deterministic Intl architecture is live, but each task retains unmet acceptance criteria described in its current-state section |
| Finite prepared sources implemented; unmatched runtime source remains open | T13 | Prepared direct/indirect eval, realm Scripts and all four Function-family bodies compile through the real compiler; runtime conversions and exact source matching remain observable, while unprepared source stays explicit typed Wasm-AOT debt |
| Blocked final gate | T26 | The retained historical Wasm-AOT aggregate is not green; a fresh complete compiler aggregate and canonical publication remain outstanding |

The retained version-7 baseline
`current-pin-wasm-aot-20260907-c5115bf03-2x1-12g` records `87,641/102,043`
passes and `14,402` failures, with a checked-in
[generated failure backlog](../test262/backlog/aa55200d1310384c5cf69ea95b2a2ecba457007b/wasm-aot.json)
and a nested status artifact refreshed on `2026-09-14`. These are historical
compiler results under the current suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; focused repairs do not determine
new full-suite totals. T01 still owns the fresh compiler aggregate, backlog
regeneration and canonical publication.

Current shortcut counts and semantic-only removal ownership are derived in
[the generated accounting report](../test262/backlog/current-shortcut-status.md).
Run the source-level audit and generator checks before using that report.
Use that report for current classification totals and removal owners.
Audit green establishes no selector drift; full conformance
still requires complete execution evidence.
Each observation retains a closed classification, reason and concrete
owner/removal task. Do not close a semantic task from focused green leaves
while its full-tree and materialization-removal criteria remain unmet.

## Historical focused retirement checkpoints

The checkpoints below retain their original counts and dates. Intermediate
shortcut inventories and capability gaps describe those checkpoints; the
current summary and generated accounting report above govern present status.

The final twelve T18 semantic observations are gone, leaving T18 with zero
shortcut ownership. Its five physical String cases retain their exact vendored
sources across ten sloppy/strict executions. The spec-exec oracle passes
`10/10`; Wasm-AOT passes `0/10` and classifies all ten as typed `Unsupported`:
four direct-`eval` sources require a caller-environment lowering seam, while the
ordinary-`Function` source requires a target-Realm environment seam. These are
visible T13 dynamic-source gaps, not skipped or passing product cases. Six
adjacent non-dynamic product controls pass all `12/12` sloppy/strict Wasm-AOT
executions.

Reduced assertion selection has been deleted in full. All 17,540 physical
sources and 33,715 executions that formerly selected a reduced body now use the
full LocalMerged `assert.js`. The typed-array literal contract has 319 physical
sources and 622 executions: 296/576 use the full helper and 23/46 explicitly
omit unused assertion code. The SameValue and CompareArray assertion modes,
their prelude constants and their source-shape predicates are gone. The
compact typed-array descriptor probe now accepts only the `TypeError` raised
by a strict write to non-writable `length` or `name`; every other setter
failure propagates unchanged. Exact Wasm-AOT runs for the `copyWithin`,
`findLast` and `findLastIndex` `length.js`/`name.js` cases pass all `12/12`
sloppy/strict executions, and a Proxy-setter regression pins non-`TypeError`
propagation. The exact `%TypedArray%.prototype.at` helper matcher and source
guard are also gone. A 15-source/30-execution invariant pins unchanged bodies
in both Script modes and both prelude profiles: all 13 typed-array-helper
consumers use the complete vendored `testTypedArray.js`, three also use the
complete configured `propertyHelper.js`, and the two resizable-helper cases
retain only T13's separately owned static-subclass substitution. The rebuilt
post-delete leaf passes all `30/30` sloppy/strict Wasm-AOT executions, and three
exact adjacent controls pass `6/6` with every non-success bucket at zero. The
retirement covers the formerly generic
`ArrayBuffer.isView`, typed-array defined-length, `%TypedArray%[@@species]`,
TypedArray sort/`of`, DataView constructor, ProxyCreate, `Error.isError` and
staging `flatMap` cohorts.

The exact `%TypedArray%.prototype.filter` and `map` source matchers and their
compact prelude consumers are now gone. A shared invariant scans all 84
physical sources and 168 sloppy/strict executions in each directory, pins the
18 retired matcher contracts in both prelude stores and permits only complete
`testTypedArray.js` or no typed-array helper. Filter has 81 complete consumers
and three sources without the include; map has 79 and five. Six metadata
sources also use the complete configured `propertyHelper.js`. Sixteen matcher
paths move from the intrinsic fragment to the complete helper, while the two
controls were already complete. This removes two T17 semantic shortcuts and
two diagnostic guards without changing the resizable-buffer admissions. The
rebuilt release CLI passes all 36 exact current-pin executions as of
`2026-08-30` under suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`;
the then-live `slice/invoked-as-func.js` compact route also passed `2/2`
before the separate slice retirement, with every non-success bucket at zero.

The combined exact `%TypedArray%.prototype.every`/`some` source matcher and
fingerprint guard are now gone. A replacement invariant pins 12 physical
sources and 24 sloppy/strict executions in four closed three-source cohorts,
with exact bytes and provenance under both prelude profiles. Only the three
`every` cases that declare `testTypedArray.js` change, moving from the split
dispatcher to the complete 14,921-byte helper. Three other `every` cases remain
without that helper; three `some` consumers remain on the typed-array literal
plan's 12,362-byte split route, and three others remain without it. The six
`resizableArrayBufferUtils.js` consumers retain T13's static-subclass
substitution. This removes one T17 semantic shortcut and one diagnostic guard,
leaves the broad `every`/`some` resizable-buffer admissions unchanged, and
at that checkpoint narrowed the retired iterator/find contract to 41 paths. The rebuilt
release CLI passes all 24 exact executions as of `2026-08-31` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; the surviving
then-surviving `find/callbackfn-resize.js` split route passed `2/2`, with every non-success
bucket at zero.

The `%TypedArray%.prototype.slice` family-prefix selector, exact eight-path
source matcher and fingerprint guard, and slice-specific compact property
selector are now gone. The replacement invariant scans all 91 physical sources
and 182 sloppy/strict executions in both prelude stores and permits only 87
complete 14,921-byte `testTypedArray.js` consumers or four sources without that
helper. Fourteen former compact and eight former intrinsic cases now use the
full helper; 65 cases were already full. The three metadata sources retain
complete `propertyHelper.js`, `not-a-constructor.js` retains complete
`isConstructor.js`, and the four `resizableArrayBufferUtils.js` consumers
retain T13's exact static-subclass substitution. This removes two T17 semantic
shortcuts and one diagnostic guard, leaves the broad slice resizable-buffer
admissions unchanged, and limits shared family-prefix compaction to `includes`,
`indexOf` and `lastIndexOf`. The rebuilt release CLI passes all 44 changed
executions as of `2026-08-31` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; exact surviving-route controls
pass `6/6`, with every non-success bucket at zero. This is a focused
post-retirement replay, not a new complete `182/182` execution claim.

The final TypedArray family-prefix compaction for `includes`, `indexOf` and
`lastIndexOf` is now gone. A combined exact invariant scans all 130 physical
sources and 260 sloppy/strict executions in both prelude stores and permits
only 117 complete `testTypedArray.js` consumers or 13 sources without that
helper. Fifteen former compact and twelve former intrinsic cases now use the
full helper. The 13 no-helper cases remain distinct from the 11 T13
static-resizable-helper consumers. Deleting the shared 5,254-byte helper, its
source parser and the three-prefix selector removes five T17 semantic
shortcuts while preserving broad resizable-buffer admission and the closed
literal/iterator/find authorities. The rebuilt release CLI passes all 54
changed executions as of `2026-08-31` under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; the literal-plan and iterator/find
controls pass `4/4`, with every non-success bucket at zero. This is a focused
post-retirement replay, not a new complete `260/260` execution claim.

The shadowed 41-path TypedArray iterator/find matcher layer is now gone. All 17
iterator and 24 find contracts were already exact members of the closed
319-case literal plan, so the deleted fallback did not change materialized
bytes. A replacement invariant pins 82 sloppy/strict executions and 164
materializations across both prelude stores: 18 physical sources use the split
full-vendored plan, 23 have no `testTypedArray.js`, 21 retain compare-array
provenance and T13's static resizable-helper rewrite, and local/vendored STA
provenance is exactly `28/82`. The deletion removes four semantic and two
diagnostic observations. The rebuilt release CLI passes six representative
sources in both Script modes (`12/12`) under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
zero. This is not a complete `82/82` replay or broad T17 closure.

The split dispatcher no longer scans source text for ten tail-only bindings or
conditionally retains the unused 2,854-byte end of `testTypedArray.js`. The
closed literal-plan invariant proves all 218 FullVendored physical sources,
representing 420 executions, have zero references to those bindings and always
materialize the canonical 12,362-byte split with FNV-1a
`0x92c7_bac7_27f5_772d`; the split appears exactly once and the tail marker is
absent. Drifted cases and helpers still fall back to the full vendored prelude.
Removing the dead source predicate and full-tail branch deletes two T17
semantic observations without changing admitted materialization bytes. Four
representative `some`, `find`, `entries` and `copyWithin` sources pass all
`7/7` applicable executions under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every non-success bucket at
zero. This is not a complete `420/420` product replay.

Fourteen AggregateError and SuppressedError core-property materializations are
now gone. Their 28 exact sloppy/strict executions preserve pinned sources and
full applicable helper provenance; six raw cohorts pass `36/36` including
eight adjacent prototype controls. T24 therefore owns five remaining
observations, all explicit dynamic-source substitutions rather than ordinary
Error semantics.

On 2026-09-28 the remaining per-path rewrite layer was deleted in full: 94
stale path rewrites across the iterator-helper, Array and TypedArray selector
tables plus 4 `$262`-dependent realm rewrites, with their dispatcher arms,
source guards and fingerprint tests, and then the now-empty
`rewrite_wasm_aot_self_contained` dispatcher itself with 59 rewrite-only
fingerprint tests. Two compiler fixes enabled the deletion: the
`every`/`some`/`find`/`reduce` iterator-close sites now use the shared
GetMethod close helper instead of a hand-rolled `return` read, so
non-callable callbacks close the iterator before the `TypeError`; and
builtin-sourced global entries degrade when the script observably writes that
global, so clobbered intrinsics inside closures throw `TypeError` instead of
using stale bindings. All 98 affected iterator-helper paths pass exact
`lila test262 run` replay, all 73 Array filter/flatMap paths pass, and the
final two T24 rewrites (`undefined/S15.1.1.3_A1.js`,
`Boolean/proto-from-ctor-realm.js`) pass `2/2`, all under suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b` with every non-success bucket at
zero. The token-aware inventory now assigns 0 observations to T15 and T24,
and the exact ledger holds 69 entries: 29 legitimate adaptations, 34
diagnostic observations and 6 semantic shortcuts owned by T13 (2,
resizable-helper static-subclass substitution) and T17 (4, typed-array
literal/split-helper selection). Both audits verify green.

On 2026-09-28 the T13 resizable-helper substitution was deleted as well.
The helper's finite `new Function` candidates are covered by prepared
Function sources, so all 188 consumers materialize exact helper bytes with
outcomes identical to the substituted baseline (185 pass `2/2`, same 3
staging `Unsupported`). The ledger now holds 64 entries with 4 semantic
shortcuts, all T17 typed-array literal/split-helper selection. Both audits
verify green.

On 2026-09-28 the T17 typed-array literal/split-helper plan was deleted in
full: the 319-case contract table, the nine-method selector, the split
dispatcher, the intrinsic fragment and the compareArray omission, with two
plan-only invariant tests and the plan branches of every cohort test
collapsed to full-helper expectations. Exact `lila test262 run` replay of
all nine method directories is unchanged versus the plan baseline, and the
same session fixed the only failures those directories had: a backend bug
where `new TypedArray(array)` ran a spurious `ToNumber` over Array
arguments, joining the source on every construction and OOM-trapping past
the 1GiB cap (`copyWithin` goes `122/6-crash` to `128/0`). The ledger now
holds 56 entries with 0 semantic shortcuts: 23 legitimate adaptations and
33 diagnostic observations. Both audits verify green.

Twenty Iterator-helper metadata branches are now gone across `every`, `some`,
`find`, `reduce`, `map`, `filter`, `flatMap` and `take`. The pinned-source
matrix covers both Script modes and exact LocalMerged/vendored helper bytes and
provenance. The focused invariant passes `1/1`, and an isolated raw Wasm-AOT
run of those exact twenty sources passes all `40/40` sloppy/strict executions
with every non-success bucket at zero. The eight enclosing selector tables
retain other rewrites, so that checkpoint remained at 360 total entries, 212
semantic shortcuts and 36 T15-owned observations.

The complete seven-case `Iterator.prototype.forEach` Test262 materializer is
also gone. Its one built-in and six staging sources now retain exact pinned
bytes across both Script modes, with exact LocalMerged/vendored assertion,
`sta.js`, `compareArray.js` and active-realm-host provenance. Removing its
dispatcher and path-selector body drops the current inventory to 186 total
observations, 49 semantic shortcuts and 32 T15-owned observations. The earlier
dated `27/27` built-in and `12/12` staging leaf results were rewrite-backed; the
raw 14-execution replacement replay remains pending.

The earlier T17 cleanup checkpoint left 190 entries after deleting the exact
nine-case DataView accessor-metadata, nine-case accessor wrong-receiver,
four-case `ArrayBuffer.isView` typed-array-argument, its callable-alias case, and
four-case DataView BigInt-get ToIndex rewrites, plus the eight-case numeric
DataView setter conversion
rewrite, the typed-array buffer defined-length expansion and the four-case
`%TypedArray%[@@species]` compact-helper authorization and the fifteen-source
ArrayBuffer metadata compact-helper boundary and the forty-two-source DataView
method metadata rewrite. Their real-source
invariants pin unchanged sources and the complete ordinary helper boundary;
the accessor wrong-receiver matrix passes all
eighteen raw sloppy and strict executions, and the numeric setter matrix passes
all sixteen. The TypedArray sort value matrix, `TypedArray.of` zero case and
eleven borrowed Array callback resize cases now preserve all 13 pinned sources
in both Script modes and both prelude stores. An isolated post-delete Wasm-AOT
run of those exact sources passes all `26/26` sloppy/strict executions with
every non-success bucket at zero. Removing their constructor fan-outs, helper
omission and dispatch paths deletes 29 more semantic observations. That cleanup
checkpoint left T17 with 161 entries, 80 semantic and 81 diagnostic. A separate
direct raw preflight of the eight top-level DataView constructor surface sources passed
all `16/16` sloppy/strict executions through complete vendored assertion and
declared helpers. Each reported `backend_used: WasmAot`, and every non-success
bucket stayed at zero. This was unchanged-source evidence before arm removal,
not a post-delete production-dispatch run. The replacement 8x2 invariant pins
exact LocalMerged and vendored-only materialization. The rebuilt production
dispatcher then passes the same exact `16/16` cohort with every non-success
bucket at zero. Removing those eight arms changes only the surviving selector
fingerprint, so that checkpoint's counts remained unchanged. The remaining T17
materializations stay open.

The two borrowed `Array.prototype.at` resizable-buffer sources now preserve
their exact pinned bodies in sloppy and strict modes. A scoped direct raw
preflight passed all `4/4` executions after combining complete vendored
`sta.js`, `assert.js`, `resizableArrayBufferUtils.js` with only T13's
replacement of the dynamic subclass block with three static classes, and the
exact source. The full unmodified helper still hits the explicit
Function-constructor AOT-unsupported boundary. This is pre-delete source
evidence, not full-helper support or a post-delete production-dispatch run. The
replacement 2x2 invariant pins exact LocalMerged and vendored-only
materialization, including the original suffix and the sole helper replacement.
After deletion, the rebuilt production dispatcher passed that exact `4/4`
cohort with every non-success bucket at zero while retaining T13's helper
substitution.
Deleting the complete rewrite helper, its dispatch and its two path predicates
removes three T16 semantic observations, leaving 73 T16 entries at that
checkpoint. The exact
`Array.prototype.includes/resizable-buffer-special-float-values.js` source then
passed a separate raw `4/4` preflight across both Script modes and both prelude
stores. Every execution reported `backend_used: WasmAot`; the sole helper
change was T13's static-subclass substitution. The unmodified helper still
reaches the explicit Function-constructor AOT-unsupported boundary, so this is
not full-helper or post-delete production-dispatch evidence. Removing only its
terminal materializer preserved the two neighboring Array `includes` rewrites
and shared dispatcher. After deletion, the rebuilt production dispatcher
passed the exact source in both Script modes (`2/2`) with every failure and
non-success bucket at zero. That historical checkpoint had 356 entries,
including 208 semantic shortcuts; T16 owned 72. The two remaining Array
`includes` sources each pass a separate raw `4/4` preflight across both Script
modes and both prelude stores. Every execution reports `backend_used: WasmAot`
after only T13's static-subclass helper substitution. The unmodified helper
still reaches the explicit Function-constructor AOT-unsupported boundary. The
expanded five-source invariant pins exact source, mode, prelude and provenance
bytes and the sole helper substitution for the two retired Array `at` sources
and all three retired Array `includes` sources. Removing the final two-source
rewrite authority deletes three more semantic observations. That historical
checkpoint had 353 entries, including 205 semantic shortcuts; T16 owned 69 and
T17 owned 161. After deletion, the rebuilt production dispatcher passed
the exact final two-source cohort in both Script modes (`4/4`) with every
failure and non-success bucket at zero.

The exact `built-ins/Array/prototype/map/resizable-buffer.js` source then passed
a pre-delete raw `4/4` matrix across both Script modes and both prelude stores
with exact source bytes and only T13's static-subclass helper substitution. The
unmodified helper still stops at the explicit Function-constructor
AOT-unsupported boundary, so this is neither full-helper support nor
post-delete production-dispatch evidence. The expanded six-source invariant
pins the map source, declared comparison and resizable helpers, and exact
LocalMerged and vendored-only bytes and origins in both modes. Deleting only
the map branch from the known-static `for-of` rewrite removes one T17 semantic
observation. The remaining TypedArray accessor authority and shared
resizable-directory substitutions stay intact. That checkpoint's inventory had
352 entries: 35 legitimate
harness adaptations, 113 diagnostic instrumentation sites and 204 semantic
shortcuts. T16 owns 69; T17 owns 160, split between 79 semantic shortcuts and
81 diagnostic guards. After deletion, the rebuilt production dispatcher passed
the exact map source in both Script modes (`2/2`) with every failure and
non-success bucket at zero. The seven pinned Array iteration
`resizable-buffer.js` sources for `find`, `findIndex`, `findLast`,
`findLastIndex`, `every`, `some` and `filter` then passed an exact raw `28/28`
matrix across both Script modes and both prelude stores. A separate `find`
preflight supplied `4/4`; the sibling proof lanes supplied `24/24`. Every run
used Wasm-AOT and preserved the exact source. Only T13's replacement of the
dynamic subclass helper block with three static classes was applied. `filter`
declares the comparison and resizable helpers; the other six declare only the
resizable helper. The unmodified helper still reaches the explicit
Function-constructor AOT-unsupported boundary. The expanded thirteen-source
invariant pins exact modes, sources, includes, prelude bytes and origins,
original suffixes, no-rewrite boundaries and T13 contract membership. Deleting
the complete handwritten iteration rewrite, its sole dispatch and seven path
predicates removes eight T16 semantic observations without changing broad
per-method admission or the neighboring mid-iteration, `toLocaleString` and
search rewrite authorities. After deletion, the rebuilt production dispatcher
passed the exact seven-source cohort in both Script modes (`14/14`) with every
failure and non-success bucket at zero. That historical checkpoint had 344
entries, including 196 semantic shortcuts; T16 owned 61. The six pinned Array
`reduce` and `reduceRight` resizable-buffer sources then passed an exact raw
`24/24` matrix across both Script modes and both prelude stores. Every run used
Wasm-AOT, preserved the pinned source, retained the declared `compareArray.js`,
and applied only T13's static-subclass replacement in
`resizableArrayBufferUtils.js`. A representative unmodified-helper run stopped
at the explicit Function-constructor dynamic-code-generation boundary, so this
is scoped pre-delete evidence rather than full-helper support. The expanded
nineteen-source invariant pins exact modes, source and prelude bytes, origins,
suffixes, no-rewrite boundaries and T13 contract membership. Deleting the
complete reduce rewrite, its sole dispatcher call, both one-caller source
builders and the obsolete synthetic rewrite test removes six T16 semantic
observations without changing broad reduce admission or neighboring resizable
authorities. After deletion, the rebuilt production dispatcher passed the exact
six-source cohort in both Script modes (`12/12`) with every failure and
non-success bucket at zero. That historical checkpoint had 338 entries,
including 190 semantic shortcuts; T16 owned 55. The four pinned Array `indexOf`
and three pinned Array `lastIndexOf` resizable-buffer sources then passed an
exact raw `28/28` matrix across both Script modes and both prelude stores. Every
run used Wasm-AOT with the exact source and declared resizable helper; only
T13's static-subclass replacement changed. The unmodified helper stopped at the
explicit Function-constructor dynamic-code-generation boundary. Dry review
found that the handwritten `lastIndexOf` rewrite had hidden a missing broad
Array `lastIndexOf/` resizable admission. A single closed prefix set now admits
`includes/`, `indexOf/` and `lastIndexOf/`, and its admission witness covers all
three. The expanded twenty-six-source invariant pins exact modes, source and
prelude bytes, includes, origins, suffixes, no-rewrite boundaries and T13
contract membership. Deleting both complete search rewrites, their two
dispatcher calls, seven path predicates, two obsolete synthetic tests and the
two dead shared prelude/constructor builders removes nine T16 semantic
observations; consolidating the two previous search admissions removes one
diagnostic observation. Neighboring mid-iteration and `toLocaleString`
authorities and broad TypedArray search admission remain. After deletion, the
rebuilt production dispatcher passed the exact seven-source cohort in both
Script modes (`14/14`) with every failure and non-success bucket at zero. The
Array-search retirement checkpoint had 328 entries: 35 legitimate harness
adaptations, 112 diagnostic instrumentation sites and 181 semantic shortcuts;
T16 owned 45. The fourteen Array
`every`/`some`/`filter`/`find`/`findIndex`/`findLast`/`findLastIndex`
grow/shrink-mid-iteration sources then passed all `56/56` raw executions across
both Script modes and both prelude stores, split into `24/24` quantifier and
`32/32` find-family cases. Every run reported `backend_used: WasmAot`, kept the
exact source and ordered `compareArray.js` plus
`resizableArrayBufferUtils.js` includes, and used only T13's static-subclass
replacement. The unmodified helper stopped at the explicit
Function-constructor dynamic-code-generation boundary. The pinned-source
invariant now owns all fourteen paths with exact modes, stores, bytes, origins,
suffixes, no-rewrite boundaries and T13 membership. Deleting the complete
shared rewrite, sole dispatcher call, one-caller constructor list and obsolete
synthetic test removes its entrypoint and fifteen direct predicates while the
seven broad Array admissions, T13 helper contract and neighboring Array
values, iterator and `toLocaleString` authorities remain. After deletion, the
rebuilt production dispatcher passed the exact fourteen-source cohort in both
Script modes (`28/28`) with every failure and non-success bucket at zero. That
historical checkpoint had 312 entries and 165 semantic shortcuts; T16 owned
29. The three pinned Array `values` base/grow/shrink resizable-buffer sources
then passed all `12/12` raw Wasm-AOT executions across both Script modes and
both prelude stores with byte-exact sources and ordered `compareArray.js` plus
`resizableArrayBufferUtils.js` includes. Only T13's static-subclass replacement
was applied. Its helper fingerprint is `0x6466_6602_9ee8_9d5d`; the three case
fingerprints are `0x5e5c_6ead_7b7c_0dda`, `0x3d18_7152_c6ff_a624` and
`0x60c2_a9ec_1dff_dd03`. Changed helper, path, include or source bytes keep
`new Function` and reach the explicit Function-constructor dynamic-code-
generation boundary. The exact invariant now owns all three modes, stores,
bytes, origins, suffixes, no-rewrite checks and T13 memberships. Removing both
complete rewrite functions, their two sole dispatcher calls and both obsolete
synthetic tests deletes two entrypoints and three direct predicates. Broad
Array-values admission, Array keys/entries iterator paths, T13's helper
contract and `toLocaleString` remain. That checkpoint's inventory had 307
entries: 35 legitimate harness adaptations, 112 diagnostic instrumentation
sites and 160 semantic shortcuts. T16 owns 24; T17 remains at 160, split between 79
semantic shortcuts and 81 diagnostic guards. After deletion, the rebuilt
production dispatcher passed the exact three-source cohort in both Script modes
(`6/6`) with every failure and non-success bucket at zero.

The three pinned Array `toLocaleString` resizable-buffer sources then passed an
exact raw `12/12` matrix across both Script modes and both prelude stores. Every
execution used Wasm-AOT, preserved the pinned source, declared only
`resizableArrayBufferUtils.js`, and applied only T13's replacement of the
dynamic subclass block with three static classes. The helper fingerprint
`0x6466_6602_9ee8_9d5d` and case fingerprints `0x9da9_18f5_d04d_d764`,
`0xc380_4490_04ea_5b59` and `0x07d1_d14e_3a0b_bb89` admit that one change.
Changed helper, path, include or source bytes retain `new Function`; a
representative unmodified-helper run stopped at the explicit
Function-constructor dynamic-code-generation boundary. The expanded invariant
pins the three exact sources, modes, stores, bytes, origins, suffixes,
no-rewrite checks and T13 memberships. Deleting the complete Array
`toLocaleString` rewrite, its sole dispatch and obsolete synthetic test removes
one entrypoint and three direct predicates. Broad Array `toLocaleString`
resizable admission and its witness, T13's contract, TypedArray
`toLocaleString` behavior and neighboring DataView rewrites remain. The
pre-retirement baseline contained 307 entries and 160 semantic shortcuts. The
regenerated source ledger has 303 entries: 35 legitimate harness adaptations,
112 diagnostic instrumentation sites and 156 semantic shortcuts. T16 owns 24;
T17 remains at 160 and T18 owns 12. After deletion, the rebuilt production
dispatcher passed the exact three-source cohort in both Script modes (`6/6`)
with every failure and non-success bucket at zero.

The seven pinned `%TypedArray%.prototype` accessor resizable-buffer sources
then passed an exact raw `28/28` matrix across both Script modes and both
prelude stores: `byteLength/resizable-buffer-assorted.js`,
`byteLength/resized-out-of-bounds-1.js`,
`byteLength/resized-out-of-bounds-2.js`,
`byteOffset/resized-out-of-bounds.js`,
`length/resizable-buffer-assorted.js`,
`length/resized-out-of-bounds-1.js` and
`length/resized-out-of-bounds-2.js`. Every execution used Wasm-AOT, preserved
the exact source, declared ordered `compareArray.js` and
`resizableArrayBufferUtils.js` includes, and retained the exact
`resizable-arraybuffer` feature with empty flags and no negative metadata. Only
T13's static-subclass replacement changed the helper. The unmodified helper
stopped at the explicit Function-constructor dynamic-code-generation boundary.
The renamed shared Array and TypedArray invariant pins the seven new sources
with exact modes, stores, source and prelude bytes, origins, suffixes and T13
contract membership. Deleting the complete known-static `for-of` wrapper and
TypedArray accessor rewrite, the wrapper's sole materialization call and the
obsolete identity assertions removes all 13 T17 semantic observations;
ordinary materialization now appends the original source directly. The three
broad TypedArray accessor admissions and T13's helper contract remain. The
historical pre-delete ledger contained 303 entries: 35 legitimate harness
adaptations, 112 diagnostic instrumentation sites and 156 semantic shortcuts.
The regenerated ledger contains 290 entries: 35 legitimate, 112 diagnostic and
143 semantic. T16 owns 24; T17 owns 147, split between 66 semantic shortcuts
and 81 diagnostic guards; T18 owns 12. After deletion, the rebuilt production
dispatcher passed the exact seven-source cohort in both Script modes (`14/14`)
with every failure and non-success bucket at zero. This does not claim broad
T17 closure.

The 43 pinned DataView method wrong-receiver sources now keep their original
bytes. The exact set contains `this-is-not-object.js` and
`this-has-no-dataview-internal.js` for the 21 mapped methods present at the
current pin, plus the sole
`getInt32/this-has-no-dataview-internal-sab.js`. Mapped `setBigUint64` has none
of those files, and no other mapped method has the SAB suffix. A pre-delete
direct raw probe covered `getInt8` primitive receivers, the `setFloat16`
wrong-slot case, the `getBigInt64` and `setBigInt64` metadata shapes, and the
`getInt32` SAB case across both Script modes and both prelude stores. All
`20/20` executions reported `backend_used: WasmAot`. This bounded proof did
not run every physical source. The replacement invariant scans all 22 mapped
methods against all three suffixes and pins the exact 43-source census,
contract fingerprints, metadata, mode order, admission, original bytes,
LocalMerged assert-only materialization and vendored `assert.js` then `sta.js`
materialization. Deleting the sole dispatcher call, complete rewrite and
obsolete synthetic test removes exactly six T17 semantic observations. The
verified pre-retirement ledger contained 290 entries, including 143 semantic
shortcuts. The regenerated ledger contains 284 entries: 35 legitimate, 112
diagnostic and 137 semantic. T16 owns 24; T17 owns 141, split between 60
semantic shortcuts and 81 diagnostic guards; T18 owns 12. The shared method
mapper, range and resizable rewrites, method-metadata and accessor invariants,
and broad DataView SAB admission remain. After deletion, the rebuilt production
dispatcher passed all 43 exact sources in both Script modes (`86/86`) with
every failure and non-success bucket at zero. This does not claim broad T17
closure.

The 41 pinned DataView method range sources now keep their original bytes. The
exact cohort has `index-is-out-of-range.js` for all 11 getters and 10 setters,
plus `range-check-after-value-conversion.js` and
`index-check-before-value-conversion.js` for those same 10 setters. The current
pin has none of the three files for `setBigUint64` and no getter
conversion-order files. A pre-delete raw run passed every physical source with
LocalMerged sloppy materialization (`41/41`). The `setUint16` range-after,
`setBigInt64` index-before, `getBigUint64` out-of-range and `setFloat16`
out-of-range representatives also passed both Script modes and both prelude
stores (`16/16`). The first manually assembled conversion-order stream omitted
LocalMerged `sta-preamble.js` and failed because `Test262Error` was unbound.
Restoring the normal prelude made that source pass; no corrected compiler or
runtime cell failed. The replacement invariant pins the closed 41-source
census, absent files, fingerprints, metadata, modes, admission, original bytes
and no-rewrite boundary. LocalMerged materialization uses `assert.js` then
`sta-preamble.js` for the 20 conversion-order sources and only `assert.js` for
the 21 out-of-range sources; vendored-only materialization always uses complete
`assert.js` then `sta.js`. Deleting the sole dispatcher call, complete range
rewrite, `dataview_method_range_info`, `dataview_method_call` and obsolete
synthetic test removes exactly six T17 semantic observations. The verified
post-wrong-receiver baseline,
after its `86/86` production run, contained 284 entries and 137 semantic
shortcuts. The regenerated ledger contains 278 entries: 35 legitimate, 112
diagnostic and 131 semantic. T16 owns 24; T17 owns 135, split between 54
semantic shortcuts and 81 diagnostic guards; T18 owns 12. At that checkpoint
the shared method mapper, complete resizable rewrite and helpers, admissions
and neighboring invariants remained. After the following resizable deletion,
a rebuilt production run passed this exact range cohort (`82/82`) with every
failure and non-success bucket at zero.

The 22 pinned DataView method `resizable-buffer.js` sources now also keep their
original bytes. The exact cohort has one source for each of `getInt8`,
`getUint8`, `getInt16`, `getUint16`, `getInt32`, `getUint32`, `getFloat16`,
`getFloat32`, `getFloat64`, `getBigInt64`, `getBigUint64`, `setInt8`,
`setUint8`, `setInt16`, `setUint16`, `setInt32`, `setUint32`, `setFloat16`,
`setFloat32`, `setFloat64`, `setBigInt64` and `setBigUint64`. A pre-delete raw
run passed all sources through Wasm-AOT with LocalMerged and vendored-only
preludes in both Script modes (`88/88`). The replacement invariant pins all 22
source fingerprints and bytes, exact metadata, both modes, admission,
no-rewrite status and exact prelude order, provenance and bytes. LocalMerged
materialization uses `assert.js` then `sta-preamble.js`; vendored-only
materialization uses `assert.js` then `sta.js`. Deleting the sole dispatcher
call, complete resizable rewrite, its value-literal helpers, the now-dead
shared method mapper, all three mapper-only test assertions and the obsolete
synthetic test removes exactly five T17 semantic observations. The verified
post-range checkpoint contained 278 entries, including 131 semantic shortcuts,
and assigned 135 observations to T17. The regenerated ledger contains 273
entries: 35 legitimate, 112 diagnostic and 126 semantic. T16 owns 24; T17 owns
130, split between 49 semantic shortcuts and 81 diagnostic guards; T18 owns
12. Broad DataView resizable, SAB and immutable admissions, constructor and
accessor authorities, and neighboring source invariants remain. The same
rebuilt production run passed the exact resizable cohort (`44/44`), for
`126/126` combined DataView method executions. This does not claim broad T17
closure.

That verified method run and its 273-entry ledger, including 126 semantic
shortcuts, form the historical constructor pre-retirement baseline. The 43
pinned DataView constructor validation sources now keep their original bytes.
The exact cohort has ordinary and SAB sources for 19 filenames, plus the
ordinary `buffer-not-object-throws.js` source and four ordinary
resize-during-custom-prototype sources. Those five SAB counterparts are absent
at the current pin. A bounded pre-delete raw probe ran eight representative
sources through LocalMerged and vendored-only preludes in both Script modes,
then ran one LocalMerged sloppy execution for each of the other 16 filename
arms. All `48/48` executions reported `backend_used: WasmAot`; no compiler,
runtime or harness cell failed. The replacement invariant pins the
43-present/5-absent census, sorted source-contract fingerprints, exact
metadata, both mode executions, admission, no self-contained rewrite and
original bytes. Its LocalMerged groups are now 32 full-assertion sources, nine
full assertion plus `sta-preamble.js` and two full assertion plus
property-helper sources. Vendored-only materialization uses exact `assert.js`
then `sta.js` bytes, plus `propertyHelper.js` for the two extensibility sources.
Deleting the sole dispatcher call, complete constructor rewrite, its sole
filename selector and the obsolete synthetic test removes exactly seven T17
semantic observations. That T17 retirement checkpoint contained 248 entries:
35 legitimate, 112 diagnostic and 101 semantic. T16 owns 24; T17 owns 105, split
between 24 semantic shortcuts and 81 diagnostic guards; T18 owns 12. Broad
DataView SAB and resizable admissions, the existing eight-source
constructor-surface invariant, method and accessor replacement invariants,
metadata authorities and unselected constructor neighbors remain. After
deletion, the rebuilt production dispatcher passed all 43 exact sources in both
Script modes (`86/86`) with every failure and non-success bucket at zero. This
does not claim broad T17 closure.

The pinned `toReversed/this-value-invalid.js` and
`toSorted/this-value-invalid.js` sources now execute without handwritten
replacements. A pre-delete raw probe passed both sources in sloppy and strict
LocalMerged modes (`4/4`), and six representative change-by-copy programs
passed with the complete upstream `testTypedArray.js`. The replacement
invariants pin both receiver contracts and the exact 21-source
`toReversed`/`toSorted` helper cohort across 42 Script executions, both prelude
stores, unchanged source suffixes and the intact 14,921-byte upstream helper;
neither compact nor split dispatcher materialization is admitted. Vendored-only
coverage at that checkpoint was a materialization/provenance assertion, not an
execution claim. The typed host boundary described below now supplies the
missing materialization contract. Deleting both
receiver rewrite authorities and the two family-specific dispatcher-split
gates removes twelve T17 semantic observations from the 266-entry constructor
checkpoint. The rebuilt production CLI passes the complete `toReversed` and
`toSorted` directories (`18/18` and `24/24`, `42/42` combined) with every
failure and non-success bucket at zero. Shared split-helper machinery remains
for independently owned TypedArray families; this does not claim broad T17
closure.

The `with/` directory no longer selects that shared split-helper path either. A
bounded pre-delete raw probe passed four representative unchanged executions
(`4/4`). The replacement invariant pins all 22 physical sources and 44
sloppy/strict executions, exactly 21 full `testTypedArray.js` consumers, the
one no-helper neighbor, source contracts, metadata, both prelude stores and
unchanged source suffixes. Deleting the sole `with/` selector removes one T17
semantic observation. The rebuilt production CLI passes the complete directory
(`44/44`) with every failure and non-success bucket at zero. Split-helper
ownership remains for other independently tracked TypedArray families; this
does not claim broad T17 closure.

The family-prefix selectors for `toLocaleString`, `slice`, `filter` and `map`
are now all retired. Exact invariants cover `39/78`, `91/182`, `84/168` and
`84/168` physical/execution identities respectively and permit only complete
`testTypedArray.js` or an explicitly absent helper. Earlier complete-leaf
replays passed `78/78`, `182/182`, `168/168` and `168/168` before the slice
retirement. The first three prefix deletions plus a source-text guard removed
four T17 semantic observations; the later slice wave removes two more semantic
observations and one diagnostic guard. Its rebuilt CLI passes all 44 changed
executions plus `6/6` adjacent authority controls, rather than claiming a new
complete `182/182` sweep. The final `includes`, `indexOf` and `lastIndexOf`
prefix compaction is now gone too. A combined invariant covers 130 physical
sources and 260 sloppy/strict executions in both prelude stores: 117 use the
complete helper and 13 omit it. Fifteen former compact and twelve former
intrinsic cases now use the full helper. Removing the shared helper, source
parser and selector deletes five T17 semantic shortcuts. The rebuilt release
CLI passes the 54 changed executions plus `4/4` surviving-authority controls
under suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`; this is not a
complete `260/260` replay or broad T17 closure.

Test262 prelude loading now records private `None`, `EmbeddedSpecExecSta`, or
opaque complete Wasm-AOT host ownership. `EmbeddedWasmAotHostOnly` combines the
Wasm-AOT host with complete vendored named helpers. The embedded-host witness
can be constructed only inside its child module. Named `assert.js` and `sta.js`
must exist before ownership is stored, and replacing either entry revokes it.
Non-raw materialization resolves declared includes before host planning, fails
with the execution id and missing include name, and fixes host-requiring source
order as strict directive, host, `assert.js`, then `sta.js`. The source-and-resolved-helper census contains
797 physical sources and 1,547 executions; ten exact self-contained rewrite
sources account for 20 executions, leaving 787 physical sources and 1,527
executions that emit the host prelude. Agent workers receive that same
host/assertion/`sta.js` prelude through private materialized state rather than
runner-side source inspection. The pinned Atomics notification case also passes
through the product runner with that exact host order.

The four ProxyCreate target-shape sources also pass unchanged in sloppy and
strict modes (`8/8`). Removing their complete source-rewrite authority deletes
five semantic observations. All four now use the full LocalMerged `assert.js`,
including the two sameValue-only cases.

The Proxy apply non-callable-trap Realm source passes unchanged in sloppy and
strict modes (`2/2`) with complete LocalMerged Realm and assertion preludes.
Removing its exact-path rewrite and the later null-handler branch first left 10
observations assigned to T11. Retiring the complete `Proxy.revocable` rewrite
removes four more. Its 17 ordinary physical cases preserve their pinned
sources and declared helpers. `tco-fn-realm.js` preserves raw
`other.evalScript`, which resolves to the typed `RealmEvalScript` AOT
unsupported boundary owned by T13 rather than a manufactured Proxy result. The
Proxy checkpoint's 307-entry inventory assigned six observations to T11;
the remaining apply and construct rewrites stay open.

## Non-negotiable rules

1. Product execution remains `parse -> early errors -> spec IR -> lowering IR -> Wasm codegen`.
2. Do not add source-path, test-name, or assertion-text branches that manufacture Test262 results. Existing focused materializations must be catalogued and retired as general semantics replace them.
3. Every change starts with a reproducible failing real Test262 filter or exact case and ends with the same command green. Add a small CLI/engine regression fixture when it isolates the behavior better than the upstream case.
4. Preserve evaluation order, abrupt completion, realm ownership, property attributes, observable coercions, and proxy traps. Passing the happy path is not enough.
5. Do not hand-edit published conformance totals. Use `lila test262 publish-status` or `scripts/publish-real-status-low-ram.sh` after a complete verified matrix.
6. Keep `unsafe_code = "forbid"`. New dependencies require a reason, license review, deterministic behavior, and a clear Wasm/runtime story.
7. Feature PRs should not combine unrelated refactors. When a prerequisite interface is missing, land the interface first under its foundation task.
8. The interpreter stays quarantined. No CLI or library product path may execute user programs through `spec-exec` by default or as a silent fallback, and emitted Wasm never embeds an interpreter/VM or feeds user source to one. T27 enforces this in code; every other task must not reintroduce it.
9. Backend design targets the experimental Wasmtime lower bound from `AGENTS.md` (Wasm GC, typed function references, reference types, `exnref` exception handling). Do not build second object models, closure representations, or exception mechanisms for runtimes that lack these features; reject such runtimes at the boundary.
10. The legacy JavaScript product exists only in Git history at the recovery commit recorded by T28. Do not restore its compiler, runtime, package, publication, benchmark, or playground surfaces. JavaScript remains only as Rust-owned test/conformance data or vendored source.

## T01 comparison identity — 2026-09-06

Snapshot comparison now rejects a missing requested name instead of silently
substituting another complete run and reporting an empty diff. Explicit
self-comparison, complete-evidence validation and status/backlog discovery remain
intact. The retained `snapshot_comparison_identity` library test family covers
the boundary with the same product-front-end compile-negative fixtures and an
explicit private test worker role. CI inventories and executes all ten
controls; this remains distinct from real-suite conformance evidence. See [T01](01-baseline-and-generated-backlog.md) and the
[comparison contract](../docs/rust-rewrite/test262-snapshot-comparison.md).

Next: finish a fresh guarded current-pin Wasm-AOT matrix with fixed compiler
inputs, publish only its verified canonical artifacts, then regenerate and
curate the historical failure backlog. Compare explicitly named compatible runs; mandatory compiler
provenance is still a separate schema migration. No aggregate status or T26
closure is claimed by this repair.

## How to execute one task

1. Read this file, the selected task file, `AGENTS.md`, and the touched crate manifests.
2. Record the exact baseline commands and counts in the PR description.
3. Implement the smallest general semantic layer that fixes the whole failure family; do not special-case the representative test.
4. Run `cargo fmt --all --check`, targeted crate tests, focused CLI fixtures, and the real Test262 filter listed by the task.
5. Search for regressions in adjacent filters that share the same abstract operation or builtin.
6. In the PR description, report: files changed, semantic invariant added, exact tests/counts, remaining failures, and follow-up task IDs.

## Parallel work graph

### Bootstrap and coordination

T00 is complete. T01-T04 have landed useful infrastructure and module
boundaries, but their remaining acceptance criteria are still active.

| ID | Task | Parallel notes |
|---|---|---|
| [T00](00-operating-contract.md) | Operating contract and contribution protocol | Documentation/CI only; independent |
| [T01](01-baseline-and-generated-backlog.md) | Reproducible real-suite baseline and generated failure inventory | Independent; feeds every lane |
| [T02](02-modularize-ir-and-wasm-backend.md) | Split monolithic IR/backend modules | Coordinate before broad feature work |
| [T03](03-conformance-harness-integrity.md) | Honest Test262 harness and host contract | Independent of most language semantics |
| [T04](04-spec-operations-and-completion-abi.md) | Shared abstract operations and completion ABI | Foundation for most feature lanes |

### Core semantic foundations

These foundations are all in progress. Use the landed T02/T04 interfaces, and
coordinate changes to remaining large shared modules.

| ID | Task | Primary ownership |
|---|---|---|
| [T05](05-values-heap-gc.md) | Value representation, heap, GC, weak reachability | runtime + Wasm heap modules |
| [T06](06-realms-intrinsics-cross-realm.md) | Realms, intrinsics, host hooks, cross-realm identity | runtime + intrinsic bootstrap |
| [T07](07-parser-grammar-early-errors.md) | Parser boundary, grammar coverage, early errors | front + IR parser boundary |
| [T08](08-environments-control-flow.md) | Environments, TDZ, references, control flow and abrupt completion | IR lowering + control-flow emitter |
| [T09](09-functions-classes-private-elements.md) | Call/construct, functions, classes, private elements | function/class lowering and emitter |
| [T10](10-object-model-descriptors-exotics.md) | Ordinary objects, descriptors, integrity, exotic object protocol | object/descriptor modules |

### Feature lanes

These lanes have partial implementations at different depths. Their dependency
lists still identify semantic ownership; a dependency marked in progress does
not forbid focused work when its required interface already exists.

| ID | Task | Depends on |
|---|---|---|
| [T11](11-proxy-reflect-metaobject.md) | Proxy and Reflect meta-object protocol | T04, T05, T06, T09, T10 |
| [T12](12-modules-linking-loading.md) | Modules, linking, namespace objects, TLA host flow | T06, T07, T08, T09, T10 |
| [T13](13-dynamic-source-evaluation.md) | `eval`, `Function`, realm evaluation policy/implementation | T06, T08, T09, T12 |
| [T14](14-promises-jobs-async.md) | Promise jobs, async functions, async iteration | T04, T05, T06, T09 |
| [T15](15-generators-iterators-resource-management.md) | Generators, iterators/helpers, disposal | T04, T05, T08, T09 |
| [T16](16-arrays-and-array-builtins.md) | Array exotic semantics and complete Array API | T04, T05, T10 |
| [T17](17-typedarrays-binary-data-atomics.md) | ArrayBuffer, DataView, typed arrays, SAB, Atomics | T04, T05, T06, T10; async wait paths use T14 |
| [T18](18-strings-unicode.md) | ECMAScript strings and Unicode-correct String API | T04, T05, T10 |
| [T19](19-regexp.md) | ECMAScript RegExp syntax and semantics | T04, T05, T10, T18 |
| [T20](20-number-bigint-math-json.md) | Numeric semantics, BigInt, Math, JSON | T04, T05, T10 |
| [T21](21-symbols-collections-weakrefs.md) | Symbols, Map/Set, weak collections, WeakRef/finalization | T05, T06, T10 |
| [T22](22-date-temporal.md) | Date and Temporal | T04, T05, T06, T10, T18, T20 |
| [T23](23-intl402.md) | ECMA-402 Intl implementation | T06, T10, T18, T20, T22 |
| [T24](24-globals-errors-annexb-host.md) | Globals, native errors, Annex B, remaining host-visible builtins | T04, T06, T07, T09, T10 |

### Validation and closure

| ID | Task | Depends on |
|---|---|---|
| [T25](25-differential-fuzzing-performance.md) | Differential testing, fuzzing, timeout and code-size work | T01-T04; runs continuously |
| [T27](27-interpreter-quarantine-and-product-default.md) | Interpreter quarantine and Wasm-AOT product default | T02, T03; complete |
| [T26](26-zero-failure-conformance-closure.md) | Full pinned suite closure and release gate | All applicable tasks, including T27 |

### Repository ownership and identity

| ID | Task | Depends on |
|---|---|---|
| [T28](28-retire-legacy-js.md) | Retire the legacy JavaScript product and enforce the Rust-only boundary | T00; complete |
| [T29](29-lila-identifier-migration.md) | Coordinated Rust identifier migration to Lila; product cutover, version-6 Lila producer, and read-only version-4/version-5 decoder verified | T28; complete |

## Merge-conflict policy

T02 has landed initial boundaries, but several IR/lowering, object/operation and
builtin implementation files remain large shared hotspots. Coordinate broad
edits to those files. Feature work should continue moving code toward dedicated
IR, Wasm emitter, builtin and focused-fixture ownership. Shared ABI changes
belong in T04 and should land before dependent feature changes.

When two tasks require the same abstract operation, the first agent implements it in the shared operation layer with unit tests; the second consumes it. Do not copy slightly different `ToObject`, `ToLength`, `Get`, `Call`, iterator, descriptor, or completion logic into feature-specific code.

## Definition of done for a feature lane

A lane is complete only when:

- its real Test262 subtree is fully green for the Wasm-AOT backend and pinned revision;
- parser, early-error, runtime, backend, host-harness, timeout, and crash failures are all zero in that subtree;
- no test-specific semantic materialization remains for the covered behavior;
- descriptor metadata, subclassing/species, proxies, cross-realm behavior, abrupt completions, and coercion order have representative coverage;
- adjacent fake-suite and CLI regression tests remain green;
- README status is refreshed only if a complete real-suite publication was performed.

## Final acceptance target

`T26` owns the final evidence: a complete resumable Wasm-AOT matrix for the current pin, verified snapshot artifacts, zero crashes and bugs, no silent skips, no stale status claims, a green T27 interpreter-quarantine audit (no interpreter in product builds or emitted artifacts), and an explicit accounting of any dynamic-source cases permitted by `AGENTS.md`. Literal `passed == total` remains the project target; architecture exceptions must stay separately visible until the project deliberately resolves them.
