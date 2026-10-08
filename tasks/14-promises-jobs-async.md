# T14 — Promise jobs, async functions and async iteration

## ShadowRealm intrinsic import reactions — 2026-10-07 dry source

The native importValue path consumes a fresh defining-Realm intrinsic Promise
capability and the shared PerformPromiseThen owner, also used by the public
Promise.prototype.then implementation after its own species selection. Internal
module awaits consume actual Promise records without rereading public Promise
properties. Export and rejection continuations retain their Realm and exact
capture family; namespace objects do not pass through thenable assimilation.
Coercion/abrupt order, cached failures, mutated Promise properties and cyclic
TLA controls are authored and unrun. See the
[ShadowRealm contract](../docs/rust-rewrite/contracts/shadowrealm-implementation-sequence.md).

## Module resource disposal owns implicit Await — 2026-10-07 dry source

The original module scanner now includes `await using` declarations and
synchronous for-of `await using` heads in HasTLA. Classic-for resource heads
reach the same lexical-declaration visitor. Nullish resources, unreachable
declarations and empty iterables retain this syntactic ownership. Nested
functions and methods keep their own disposal; module-evaluated class names
still contribute their actual Await expressions.

The existing graph join therefore selects AsyncModuleActivation for a module
whose only suspension is asynchronous disposal and makes its importers wait for
the original resource completion. No activation protocol or disposal algorithm
is added. Record/graph controls and three Engine cohorts cover statement/block/
classic-for/for-of lifetimes, null disposal timing, importer ordering and exact
undefined rejection under both promise policies. These controls are authored
and unrun; compilation and runtime acceptance remain deferred until the whole
task source batch is ready. The two September 30 top-level `await using`
failures are historical leads, not refreshed conformance counts.

## Complete plain async classic phases — 2026-10-06 dry source

Plain async For/While/DoWhile now share the complete phase owner and native loop
with mixed generators. Checked protocol-tagged regions bind every actual phase,
Await point and whole completion cell. Suspended lexical/pattern heads use the
original TDZ and per-iteration records; nested branches, finalizers and labelled
control use the original completion machinery. Old direct-await restrictions
have positive source and runtime controls. Common resource capabilities now span
lexical lists, iterator heads, classic heads and CaseBlocks, with original
registration and finalizer ownership. Compilation and execution are deferred.
See the [contract](../docs/rust-rewrite/contracts/plain-async-classic-regions.md).

## For-await bodies with Await — 2026-10-04 dry source

Plain async `for await` loops with eager Var/Let/Const identifier binding heads
now share the completed iterator/body plan with synchronous for-of. A closed
execution view owns awaited Next and Close, checks successors around the actual
body span, and keeps retained protocol flags in the real storage census and
planner. The existing awaited iterator emitter consumes the completed view;
body resumes cannot repeat next Get/Call or head initialization.

The mandatory body proof preserves local Await/If/Try and completion ownership
and rejects materialized body/catch environments at every depth. Captured head
cells remain fresh per iteration. Three finite paired Engine cohorts and actual
IR controls cover interleaved async/sync fallback, cached next, repeated body and
finalizer awaits, Continue/Break/Return/Throw, close precedence, abrupt cutoffs and
native-error Realms. Sync fallback Gets and Awaits terminal value even when done
is true; true async terminal results omit value.

All current compilation, guards, fixture/runtime and pinned acceptance remain
unverified. Captured body/catch scopes, suspended operands, pattern/assignment or
resource heads, nested resumable loops and async-generator composition remain
explicit source gaps. Finish all task source before capped serial verification;
no task closure or published count follows. See the
[for-await contract](../docs/rust-rewrite/contracts/plain-async-for-await-body-continuations.md).

## Nested switch block disposal — 2026-10-04 dry source

The complete plain-async switch batch admits the existing supported nested-block
`await using` shape: identifier bindings and eager resource initializers. Its
mandatory source proof runs before any case states are allocated. The private
case span constructor accepts only an AsyncFunction disposal capability, checks
the actual suffix from entry, and requires exact checked adjacency through
dispose, resume and exit. An implicit-only case therefore retains a resumable
owner even without an explicit Await expression.

Existing acquisition, pending-completion and lexical-environment owners retain
resources and captured cells through disposal awaits. Matching switch breaks
route after the selected finalizer completes; fallthrough starts the next case
only after normal disposal. No emitter, activation, opcode or ABI change is
introduced. Three finite strict/sloppy WasmAot cohorts cover selection/cells,
completion/LIFO/finalizer replacement, and errors/rejection/Realms; IR controls
exercise actual lowerings and reject incorrect capability/state ownership.

The batch passed the ref93 combined all-target Rust type check and remains
unexecuted. Direct CaseClause/DefaultClause resource
declarations retain SyntaxError; resource loop heads, suspended initializers,
for-await and generator/enclosing-loop composition remain outside this owner.
Full T14/T15 and pinned conformance acceptance remain open. See the
[contract](../docs/rust-rewrite/contracts/plain-async-switch-continuations.md).

The ref80 `cargo xc --locked --offline` whole-workspace all-target Rust type
check passed on 2026-10-04, including the local async for-of follow-up and all
then-integrated invocation/branch owners and test targets. Formatting and
task-plan checks passed. Emitted Wasm, focused runtime regressions, broad suites
and full T14/T15 acceptance remain pending. Historical execution receipts below
retain their original source scope.

## Local async for-of control — 2026-10-04 dry source

Plain async synchronous for-of bodies admit only current-loop unlabelled
Break/Continue. Source admission and both mandatory resumable body constructors
consume one neutral closed CurrentLoop/NestedStatement ownership domain. The
async ownership preflight runs before the existing eager-body shortcut, so a
foreign loop, switch, label or parameter-initialization branch cannot bypass it.
Existing eager Try/resource and checked Await/If/Try state handling retain their
owners; no activation, IteratorRecord or continuation layout is added.

The shared iterator emitter publishes a body-end continue target and retains
the break target through IteratorClose and completion dispatch. An awaiting
finalizer selects the actual completion first. Matching Continue clears its
completion and auxiliary target before iteration cleanup/reset/step, without
close. Break closes once with existing throw precedence and outer clause
routing. Three finite strict/sloppy Engine cohorts cover interleaved activations,
cached next and captured cells, repeated finalizer awaits, replacement by local
control/Return/Throw/rejection, and all six close protocol routes through
awaiting outer handlers. The maintained generator source guard follows the
consumed ownership proof rather than its retired blanket rejection spelling.

This whole source batch passed the ref80 combined all-target Rust type check;
emitted Wasm and runtime acceptance remain pending. Async generators, for-await, suspended heads
and foreign or labelled control remain separate domains. Full T14/T15 acceptance
remains open. See the
[loop contract](../docs/rust-rewrite/contracts/async-for-of-continuations.md).

## Target-only grouped Property References — 2026-10-04 dry source

Grouped ordinary calls and tags whose optional chain ends in a Property now
admit suspension in the target before a synchronous tail. The existing checked
grouped source retains the actual terminal kind and the complete chain; its
private constructor validates either terminal role through the existing link
validator. Ordinary optional value admission keeps its link-Await requirement.
The existing evaluated-base producer retains the resolved target once and
captures only the terminal Get and raw receiver before outer arguments, spread,
GetTemplateObject or awaited substitutions. A recursively checked first optional
Call retains the same original inner receiver.

The existing three paired calls, templates and abrupt fixtures add target
thenables and mutations, Symbol/Proxy/Get order, strict primitive receivers,
synchronous Call-to-Property tails, optional first-Call skipping, frozen cached
templates and original foreign abrupt/finally observations. The two obsolete
target-only Property refusal rows become positive IR controls with actual
activation/state and capture-order assertions. This supersedes the target-only
Property boundary recorded below; private/super, delete, loops, mixed protocols
and generators retain their existing admission boundaries. Its combined
all-target Rust type check passed; emitted Wasm and these semantic controls
remain pending. Full T14 acceptance stays open.
See the [grouped Reference contract](../docs/rust-rewrite/contracts/grouped-optional-reference-await-ownership.md).

## Generator invocation sibling — 2026-10-04 dry source

The shared suspended-invocation path now consumes a complete checked generator
chain whose actual terminal link distinguishes Property Reference from Call
Value. A Property retains its terminal raw receiver across outer Yield; a Call
result remains receiver-free. This completes the bounded T15 sibling without
changing the async owner or its admission boundaries. The existing three paired
generator fixtures are extended with Property, skipping and abrupt controls.
The integrated batch passed the combined all-target Rust type check; execution
and full T14/T15 acceptance remain pending. See the
[grouped yield contract](../docs/rust-rewrite/contracts/grouped-optional-call-value-yield-ownership.md).

## Grouped terminal Call values — 2026-10-03 dry source

One private grouped source owns the actual finite chain and a closed terminal
Property/Call kind. Mandatory prefix admission validates plain async functions
outside all loops, with no Yield, before any continuation state is consumed.
Property mode retains its actual terminal Get and raw receiver. Call mode
consumes the completed full-tail Value and pins it with an undefined receiver
before every outer argument or GetTemplateObject/substitution. It also owns
target-only suspension, including a nested grouped first Call whose original
method receiver must survive. Existing ordinary value/link-await admission and
terminal Property Reference boundaries retain their domains.

The existing paired calls, templates and abrupt Engine fixtures now cover
returned function/tag identity, inner versus outer receivers, spread completion
before outer Await, unconditional outer evaluation, optional outer skipping,
cached frozen templates, saved intrinsic error Realms and arbitrary foreign
abrupt values. Meaningful IR sources cover actual publication, activation/state
ownership and mandatory preflight. Five superseded refusal inputs are retired;
delete, private/super, loops, generators and mixed protocols remain explicit.
The later target-only Property batch supersedes that original refusal.
Compilation, emitted Wasm and all
controls are unverified; full T14 acceptance remains open. See the
[Call Value contract](../docs/rust-rewrite/contracts/grouped-optional-call-value-await-ownership.md).

## Current invocation ownership — 2026-10-03 dry source

Calls, constructors and tags acquire their evaluated callee, original receiver
and direct-eval capability before staged awaited or yielded arguments. Factory-owned
argument snapshots retain spread iteration results before suspension. Grouped
optional property calls/tags publish the chain owner's conditional receiver;
a terminal Call supplies a Value with an undefined outer receiver;
nullish and noncallable outer calls still evaluate arguments before rejection.
The private IR owners, actual emitters and planning/traversal consumers are
integrated with seventeen await and fourteen yield regression sources. A shared
Await/Yield invocation owner consumes existing activation bindings and rejects
unsupported staged shapes before continuation-state consumption. The combined
all-target Rust type check passed; runtime verification remains pending. Mixed suspension, async-generator
branches, awaited bodies/other loop heads and super() staging remain separate
work. Bounded ordinary-generator values now have the separate
[T15 branch owner](../docs/rust-rewrite/contracts/plain-generator-value-branch-ownership.md). See the
[invocation contract](../docs/rust-rewrite/contracts/suspended-call-argument-ownership.md).

## 2026-10-03 dry implementation follow-up

Plain async conditional and logical values now reuse the existing If continuation owner
outside loop bodies and in checked awaited while conditions with eager bodies. One private completed-arm constructor keeps
its prefix and state range together; private arm scopes preserve the outer fact
domain while activation slots survive. A consumed result owns both final writes
and the resumed read. Checked source admission closes the actual expression
statement mis-hoist and the New/import option omissions. Switch selectors already
consume these If prefixes. Authored invocation-order, nested value, TDZ and
abrupt Realm controls passed the combined Rust type check and remain unexecuted;
unsupported optional
Reference forms, awaited bodies, other loop heads and
mixed Await/Yield and async-generator branch owners stay open. Ordinary
generator values use their dedicated bounded owner. See the
[conditional expression contract](../docs/rust-rewrite/contracts/conditional-await-expression-ownership.md).

Logical `&&`, `||` and `??` share that same branch/result owner. A consumed left
owner retains GetValue once and supplies both its condition and skipped value;
strict null/undefined comparisons preserve non-nullish values without coercion.
The selected RHS alone suspends. Existing Call/New/tag/property References and
switch selector prefixes compose with these branches. Four IR controls, three
paired strict/sloppy sources and a selector/fallthrough control are authored and
unexecuted. See the [logical expression contract](../docs/rust-rewrite/contracts/logical-await-expression-ownership.md).

Property-only optional values share the same branch/result owner. Checked
source tails reject private links; Call-bearing tails now have the additional
context admission described below. Each shorted link guards its
whole suffix. The original base and each intermediate GetValue remain in
activation bindings before a later computed key suspends. Raw keys preserve
the existing property-coercion and ordinary nullish-receiver checks. Outer
ordinary method/delete References, constructor values and call arguments
compose. Grouped optional callees/tags with awaited property keys now have
a consumed terminal Reference mode; the later target-only grouped batch also
admits awaited bases for callees/tags. Direct delete remains refused.
Target-only optional values retain the
synchronous-chain path. If static
nullish nested keys erase every suspension, an ordinary If commits the result
without advancing the enclosing state. Five IR controls and three paired
strict/sloppy semantic sources are authored and unexecuted; see the
[optional property contract](../docs/rust-rewrite/contracts/optional-property-await-ownership.md).

Grouped ordinary calls and tags now retain the original optional chain
Reference outside all loops. Each intermediate Get completes before a later
key suspends, and only the terminal Get publishes callee and raw receiver
together through the existing capture IR. A closed Call/Construct purpose keeps
constructors on their value route. Nullish groups skip the entire suffix but
still evaluate outer arguments or GetTemplateObject/substitutions before the
shared callability error. Existing private arm scopes, checked joins and erased
await fallback retain their owners. Six IR controls and three paired semantic
sources cover two-key child retention, Proxy/Symbol/getter hooks, primitive
receivers, outer spread/job order, template caching and foreign abrupt identity.
They passed the combined Rust type check and remain unexecuted. The later
grouped Call Value owner extends
terminal Call results used as outer callees/tags. Private/super, delete, loop and
generator References retain their boundaries; see the
[grouped Reference contract](../docs/rust-rewrite/contracts/grouped-optional-reference-await-ownership.md).

Optional Property/Call tails now consume a checked plain-async source owner
outside all loops. The actual first Call captures its original Reference; a
preceding property supplies the same actual Get and raw receiver to the existing
capture IR. Every optional nullish guard precedes arguments and skips the entire
suffix. Selected Calls reuse ordered suspended arguments, spread snapshots and
the existing call completion owner. Each Call result survives later keys or
Calls as a Value. Grouped Call/tag acquisition consumes the actual terminal
kind: a Property keeps its Reference, while a Call supplies a Value. A first
Call on an awaited grouped target passes the corresponding checked source before
any continuation state is allocated. Property-only checked while routes and
target-only synchronous tails remain unchanged. Maintained and new IR controls
and three paired strict/sloppy Engine sources are authored and unexecuted; see
the [optional Call contract](../docs/rust-rewrite/contracts/optional-call-await-ownership.md).

Checked awaited while conditions now admit those conditional/logical/property-only
optional value prefixes. A private context retains the real loop depth while staging,
restores it before the eager body, and is absent from fresh child activations.
The condition constructor checks exact recursive arm association and a fresh
exit; statically erased awaits retain their eager prefix inside each iteration.
Three paired strict/sloppy semantic sources and constructor/source controls are
authored and unexecuted. Other loop heads/bodies and logical assignment
conditions remain explicit gaps; see the [awaited while contract](../docs/rust-rewrite/contracts/plain-async-awaited-while-condition.md).

Awaited logical assignments outside loops now capture declarative identifiers
or ordinary property References with synchronous LHS operands. The actual Get
capture owns the normalized key, target and original receiver; only a normal
selected RHS reaches the consumed Put and result publication. Existing TDZ,
immutable writes, Set strictness, arbitrary exceptions and intrinsic error Realm
remain shared. Runtime/with/script-global-object/unresolved identifiers, suspended
LHS/private/super targets and all loop/generator contexts remain refused. New IR,
switch and three paired semantic sources are authored and unexecuted; see the
[logical assignment contract](../docs/rust-rewrite/contracts/logical-assignment-await-reference-ownership.md).

The suspended-selector, conditional, logical, optional-property/Call, grouped
Reference, while and logical-assignment follow-ups passed the combined
all-target Rust type check and remain unexecuted. The candidate results in the
next section describe the earlier eager-selector
source; they do not verify this implementation or close T14.

## Current candidate verification

The named async-switch IR and paired Engine targets pass in the same-Source
focused run. All seven IR controls pass with the awaited-discriminant
test checking the required lexical temporary, matching resume/read
binding, activation-owned cell, exact states and isolated case awaits.
The parent IR result with six passes and one failure remains dated history;
the correction changes no production lowering. No new switch-specific
pinned packet was authored. Awaited
selectors, implicit asynchronous disposal/iteration, enclosing-loop
composition and generator switches retain their explicit boundaries.
The predecessor 43-stage checkpoint remains dated history; its later broad
session 30300 is INCOMPLETE rather than still running. General T14 is open.

The candidate continuation revalidates 84 focused stages with 1,856 selected Rust test invocations on the exact same Source. Compilation,
one separate both-engine startup invocation and the default-features CLI
build belong to the original focused run. The continuation freezes that
CLI unchanged and executes all 151 selected pinned modes from 82 files.
The original pin-identity validation failure remains recorded.

This is candidate verification. MAIN installation and a fresh complete
MAIN broad checkpoint remain required. The earlier session 30300 is
INCOMPLETE without an owned terminal; its exit and cause remain unknown.
Full pinned Test262 conformance and task acceptance remain open. The
published status span is unchanged. The authentic continuation terminal is `bb22495c5187c67c269adeffd0e8efd91c3cbe97be18a79fb86c263945bc2798`; its Root-owned exit is `b399f9e5c16d0847ff4854a885e40c1b0b55f5428791d0186919db6dc64d0b1a`. The revalidated same-Source prefix is `7d0ce67ed3ce18e2646639461ba9eeb7f5d4e0793275b0937cf687ec96bf5bc9`; its original enclosing Root1 is `12a3deab22796d658bebdce50eaf263cf2a1443b1f03f0269dcdba951f9c77c1`.

The preparation and dated verification statements below retain their
original scope and failures. This checkpoint supersedes only the
unexecuted state of the named selected controls described above.


**Status:** In progress — Promise/job machinery is substantial; suspended async closure remains

**Parallel group:** Feature lane  
**Depends on:** T03, T04, T05, T06, T09  
**Blocks:** Async modules in T12, async iterators in T15, Atomics.waitAsync in T17

## Current repository state

The admitted NEXT batch gives an unconditionally reached awaited `while`
condition a checked contiguous activation-state owner and consumes a validated
eager source body before temporarily removing continuation state. Back edges and
`continue` restart the condition; ordinary completion routes preserve `finally`,
rejection and captured body environments. Non-loop labelled regions support
admitted Block/If/Try owners, ordinary awaits and awaited `while` conditions;
early labelled breaks bypass unentered conditions and advance to the region exit.
Switch cases/discriminants, implicit disposal/iterator body suspensions and bodies
without supported continuation owners report Unsupported. Nested functions retain
independent activations. The actual combined T12/Async/Locale checkpoint passes
43 stages/1,520 selected Rust tests, fresh CLI and 39 exact pinned files/65 modes,
then receipt acceptance and 107-path MAIN admission. The separate mandatory broad
checkpoint remains pending; this narrow result does not satisfy the whole
Promise/async zero-failure gate. See the [awaited while contract](../docs/rust-rewrite/contracts/plain-async-awaited-while-condition.md)
and [labelled region contract](../docs/rust-rewrite/contracts/plain-async-labelled-region.md).

The backend has Promise records, reaction/job queues, combinators, async
activation records and extensive focused real-suite coverage. README notes
still identify unsupported suspended-body and dynamic-source families, and
async generators, async iteration, module jobs and `waitAsync` share unfinished
boundaries with T12/T15/T17. The complete Promise/async filters have not met the
zero-failure acceptance gate for the current pin.

The IR caller-flow catalog now gives all 29 Promise builtins a closed
classification: 24 public/internal entry points can synchronously run user
code, while the value thunk, thrower, species getter, capability executor and
reject function are synchronously pure. A catalog partition test derives the
declared Promise set from the builtin function IDs, so adding an unclassified
entry fails. Promise construction bypasses invalidation only for ordinary Call
or a missing/definitely primitive executor and observes an exact executor only
on the matching callable Construct path. A resolving function preserves facts
only for a missing or primitive resolution; object resolution remains
conservative because reading `then` can invoke a getter. The partition and
seven focused behavior tests pass. This is compiler caller-flow correctness,
not a broader Promise/Test262 milestone.

Promise combinator element projection is now a closed
`PromiseKeyedElementProjection` choice: `Promise.all` can request only a
fulfilled value and `Promise.allSettled` can request only the settlement record
selected by its typed terminal direction. Pending reaction append likewise
accepts `PromiseReactionType` and exhaustively selects the matching fulfill or
reject list, so an arbitrary heap offset cannot be paired with a reaction local.
The two bounded structure targets pass `5/5`; the pre/post Wasm golden captures
are byte-identical. These are internal domain constraints, not a broader Promise
or suspended-async conformance claim.

Standard Promise combinator reaction routing now produces one private,
non-derived `PromiseCombinatorReactionPairLocals` per element. A single
exhaustive mode match selects both tagged callback roles together, and the
`then` invocation consumes the pair through its sole ordered projection. The
former independent fulfillment and rejection mode matches could drift while
still compiling, or transpose a payload and tag through their loose tuples.
The Rust-lexical
`promise_combinator_reaction_pair_ownership_structure` target pins the exact
three-row selection, five-mention authority census and one consuming call. The
retained all-mode Promise fixture is the semantic witness. This is a
source-equivalent T14 ownership closure, not a new Promise or job capability.

The callback-created allocation Realm boundary is now typed independently of
the combinator's constructor capability. Standard and keyed `allSettled`
records consume the self-backed callback's required defining-Realm Object
prototype, while `Promise.any` propagates the active combinator's AggregateError
prototype snapshot into its reject-element function and consumes a private
non-copyable allocation context for both the nonempty and empty rejection
branches. The bounded allocation target passes `6/6`; a four-branch
non-blocking fixture passes `1/1` with exact settlement-record descriptors/key
order and created-Realm Object/AggregateError prototypes. The same common
standard-combinator path now allocates its `all`/`allSettled` result Array and
`any` errors Array from the executing borrowed method's Realm through the
existing non-copyable Array-prototype proof. Returned Promise construction
remains independently owned by constructor `C`. Keyed/race behavior, general
AggregateError construction, PromiseResolve, async allocation and broader job
Realm switching remain outside this batch.

The four `Array.fromAsync` iterator-result continuations now select only the
closed `ArrayFromAsyncIteratorResultProperty::{Done, Value}` domain instead of
supplying arbitrary property-name strings. The domain now derives only Clone
and Copy, so it cannot be collapsed through equality or a Boolean default. Its
exhaustive projection owns the two observable keys, while all eight reads
retain their existing order and immediately following rejection routes. The
bounded structure target passes
`3/3`, and the async-value and iterator-closing CLI witnesses each pass `1/1`.
`cargo xc` is green, and the 647-artifact Wasm golden has an empty recursive
pre/post diff. This typed boundary claims no async-iteration conformance change.
The focused boundary is
[`array-from-async-iterator-result-property-domain.md`](../docs/rust-rewrite/contracts/array-from-async-iterator-result-property-domain.md).

`Array.fromAsync` result publication now converts failed ordinary index
definition and strict length Set into TypeError from the executing method's
Realm, independently of constructor `C` and result object `A`. The closed
object-mutation authority is threaded through outlined Set helpers, and
setter/Proxy-trap throws retain their original identity. Two bounded structure
targets pass `4/4` each, the focused fixture passes `1/1`, and six exact pinned
files pass `12/12` sloppy/strict executions. The isolated contract is
[`array-from-async-result-definition-error-realm.md`](../docs/rust-rewrite/contracts/array-from-async-result-definition-error-realm.md).
The shared 679-dump semantic golden passes `2/2` in 800.46 seconds, adds only
that Array.fromAsync witness and removes none. Of 678 retained dumps, 677 are
equal after accounting normalization; the expanded Promise internal-callback
Realm witness is the sole structural change.

The AOT pending-job record now has one closed Rust `PromiseJobKind` domain for
the two job shapes the product path actually enqueues: Promise reactions and
thenable resolution. Both producers encode that type, and the main-export drain
derives its comparison chain from the domain before selecting a handler through
an exhaustive match. An unknown word traps instead of silently running as a
thenable job. A private payload-bearing `PromiseJobToEnqueue` now requires each
producer to supply its argument and realm policy before the sole FIFO append;
new job shapes cannot grow a second queue-order implementation.
The job and reaction-callback enum, ordered `ALL` set and stable wire word now
come from the same macro row, with a const dense-range proof; there is no second
hand-written variant list that can omit a new row.

The private payload-bearing `PromiseJobToEnqueue` authority now derives no
cloning or copying capability. Its two complete reaction/thenable shapes are
constructed at exactly two producers and consumed once by the sole exhaustive
FIFO append, so reusing one selected job becomes a Rust move error. A recursive
lexical guard pins the exact six mentions, fully populated thenable record, both
payload arms and the complete Realm/kind/next/head/tail/reverse-release order.
This source-equivalent ownership hardening is recorded in
[`promise-job-to-enqueue-ownership.md`](../docs/rust-rewrite/contracts/promise-job-to-enqueue-ownership.md);
its structure target passes `3/3`, the two exact engine witnesses and one CLI
witness each pass `1/1`, and the two exact Test262 leaves pass `4/4`
sloppy/strict executions with every failure bucket at zero. No broad-suite,
semantic-golden or README claim is added. Independent review is
clean after the guard was strengthened to exact producer bodies and
alternate-call-route closure. The shared workspace formatter, `cargo xc`,
diff, module-boundary, and task-plan checks all pass.

Promise reaction callback words are also one closed six-variant Rust domain.
Reaction construction writes that typed word once, rather than initializing a
default and repairing internal async continuations afterward, and the runner's
ordered comparison chain selects behavior through an exhaustive match. Default
reaction jobs derive `GetFunctionRealm(handler)` at enqueue time or carry the
specification's null realm for an empty handler; internal async continuations
carry their captured realm. Thenable jobs derive the `then` callback realm.
Both callback lookups select the enqueue-time current realm for a revoked Proxy,
and the drain maps a null job realm to its saved host-checkpoint realm instead
of installing zero or leaking the preceding job's realm.

The reaction record's `[[Type]]` is now a separate closed
`PromiseReactionType::{Fulfill, Reject}` domain rather than a raw Promise-state
word. All three producer pairs must select the type before construction. The
reaction-job runner decodes the stable wire words 1/2 once into a normalized
rejection flag, traps an unknown word, and threads that flag through all six
callback shapes. No callback independently treats an invalid word as its own
fallback. This is a record-integrity boundary; valid reaction behavior and the
wire encoding are unchanged.

Promise `finally` completion preservation now has its own closed
`PromiseFinallyCompletion::{Fulfill, Reject}` domain. The `ThenFinally` /
`CatchFinally` continuation stage and the later `ValueThunk` / `Thrower` stage
consume the same typed choice through exhaustive matches, while four named
zero-choice wrappers keep naked booleans out of the standard-builtin
dispatcher. This closes a representational hole in which one inverted boolean
could compile and silently restore the wrong original completion. Existing
valid behavior and ordering are unchanged.

`PromiseFinallyCompletion` is now non-`Clone`, non-`Copy`; its two consuming
projections make a second by-value policy observation an E0382 move error. A
Rust-lexical guard pins all eight lexical mentions, the four exact wrapper
producers, and the two consuming projections together with their existing
ordering. Complete normalized-body fingerprints close both consumer emitters
against inserted, duplicated or reordered emission. The continuation and
restoration runtime stages remain independently constructed, so this capability
closure does not claim that one Rust token crosses the Promise job boundary.
Emitted instructions and valid behavior are unchanged. The dedicated structure
target passes `4/4`, the exact created-Realm Promise internal-callback CLI
witness passes `1/1`, and scoped Rust formatting is green.
Independent review added complete normalized-body fingerprints for both
consumer emitters. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings. Broader Promise Test262 verification was not rerun.

Ordinary async-function activations now store the completion supplied by
`Await` through one closed `AsyncFunctionResumeCompletion::{Normal, Throw}`
domain. The raw offset and stable words 0/1 are private to the heap boundary;
activation initialization and the reaction continuation must use the typed
store. Ordinary `await` and both `for-await-of` resume sites use the sole strict
decoder, which normalizes to one `is_throw` flag and traps an unknown word
instead of treating it as fulfilment. The shared `for-await-of` emitter now has
a closed async-function/async-generator layout choice, so the generator's
separate five-way resume-kind behavior stays explicit rather than being folded
into an integer tuple. This is also a record-integrity boundary: the existing
valid 0/1 behavior is unchanged, while illegal internal words fail closed.
Batch AB further makes `ForAwaitActivationLayout` a must-use, capability-free
owner. Three borrowed offset projections, two borrowed strict-decoder calls and
four borrowed exhaustive projections now derive every resume layout, rejection
Promise Realm and reaction-sequence decision from that single owner. The copied
layout capabilities and `is_async_generator` Boolean carrier are gone. This
changes no emitted Wasm or runtime behavior. At the Batch AB checkpoint,
`cargo xc` is green, the dedicated structure target passes `3/3`, and the exact
ordinary-rejection, iterator-close and async-generator rejection engine
controls pass `3/3`. Test262 and semantic goldens were not rerun for this
capability-only closure.

Async-generator activations now store `[[AsyncGeneratorState]]` through the
exact closed
`AsyncGeneratorExecutionState::{SuspendedStart, SuspendedYield, Executing,
DrainingQueue, Completed}` domain. The former backend-only suspended-await word
is retired: Await remains `Executing` as ECMA-262 requires, while the separate
body-status word continues to carry the Await phase. All seventeen writers use
the typed heap store. The prototype dispatcher and two Promise reaction jobs
strictly decode one stable snapshot through an opaque non-`Copy` token, so an
unknown state traps before reaching the executing/draining fallthrough. The
bounded lifecycle, request-completion and await-using structure targets each
pass `5/5`; the exact lifecycle/delegation CLI cohort passes `5/5`, and its five
pinned Test262 files pass `10/10` Wasm-AOT variants under `--jobs 1 --threads
1`. This is a state-word invariant, not broader suspended-async closure.

The distinct async-generator body-status field is now closed over
`AsyncGeneratorBodyStatus::{Idle, Running, Await, Yield, Complete, Throw}`.
All fifteen product writers cross one typed heap boundary. The body driver and
the two Promise reaction jobs are the only readers; each strictly validates one
stable snapshot and traps an unknown word before routing, while an opaque
non-`Copy` token prevents raw-local reuse. Await still pairs body status
`Await` with execution state `Executing`, so this backend protocol does not
widen `[[AsyncGeneratorState]]`. The bounded owner/ordering guard and
`docs/rust-rewrite/contracts/async-generator-body-status-word.md` are
focused-verified. The four related structure targets pass `20/20`, the exact
lifecycle/delegation CLI cohort passes `5/5`, and its five pinned Test262 files
pass `10/10` Wasm-AOT variants with every non-success bucket at zero.

The async-generator activation's resume-kind word now has the closed
`AsyncGeneratorResumeKind::{Normal, Return, Throw, Fulfill, Reject}` domain.
Nine runtime branch selections across six writer paths use one typed store.
Four control-flow readers strictly validate one heap snapshot, compare it
through an opaque non-`Copy` token and trap an unknown word before Normal-like
fallthrough. The delegation resume branch strictly validates, copies and
releases its activation snapshot before the branch joins; the fresh branch
initializes the wider pending-kind transport from typed Normal. Every
post-join route uses that wider transport, whose backend close-throw word 5
cannot be written back through the private activation offset. Resume-state
labels, request completion, execution state and body status remain separate
types. The focused structure target and four neighboring guards pass `27/27`;
the exact lifecycle/delegation CLI cohort passes `5/5`, and its five pinned
Test262 files pass `10/10` Wasm-AOT variants with every non-success bucket at
zero.

Promise records now store `[[PromiseState]]` through one closed three-variant
`PromiseState::{Pending, Fulfilled, Rejected}` wire domain. The raw offset is
private to typed initialization, terminal-store and strict-load helpers, and an
unknown word traps instead of falling through as rejection. The separate
`PromiseSettlement::{Fulfill, Reject}` domain is accepted by every terminal
producer and Promise-direction helper, so `Pending` or an arbitrary integer can
no longer be supplied where a terminal choice is required. Promise reaction
`[[Type]]` remains distinct despite sharing the two terminal wire words.

One exhaustive reaction-pair router now owns the pending/fulfilled/rejected
behavior shared by ordinary `then`, async `await` and async-generator
return-await. Terminal settlement captures the selected reaction list, stores
the result, clears both obsolete lists, stores the typed state, performs
rejection tracking when required, and only then enqueues the captured reactions.
This closes the Promise lifecycle record and transition-order boundary; it is
not a claim of broader queue ownership, suspended-body support, GC completion or
full Promise conformance.

Main Script completion now has one closed exit policy. While source statements
are emitted, every otherwise-terminal abrupt completion targets a code-sink
tracked host-checkpoint block instead of returning from the Wasm export. The
checkpoint drains jobs and then publishes the original Script completion;
internal functions retain their direct four-word completion return. The drain
also preserves the thrown error-name/message globals alongside the completion
tuple, so an error raised by a queued job cannot overwrite the identity or
message of an already-pending top-level throw. A durable engine regression
requires the queued job's print side effect, secondary rejection diagnostic and
primary throw identity; the focused engine contract passes `1/1`.

The main-export rejection checkpoint now detaches and walks a finite snapshot
of the complete candidate FIFO. After strict state and handled-mark rechecks, a
Normal Script completion keeps the oldest unhandled rejection as its exported
Throw and prints every later rejection value in FIFO order. If a top-level
abrupt completion is already primary, the checkpoint prints every unhandled
snapshot rejection and preserves that completion. Heap-backed modules import
the existing line-oriented host printer even when source never names `print`,
so the diagnostic path cannot disappear behind builtin reachability. Symbols
use non-coercing descriptive rendering. A throwing ordinary `ToString` emits a
fixed visible failure marker, restores the primary completion diagnostics and
continues the FIFO; a host-print failure is not caught. A `ToString` that calls
`Promise.reject` appends to a fresh live tracker which is neither traversed nor
cleared by the current checkpoint, so recursive rejection cannot extend the
snapshot indefinitely. The bounded source guard passes `5/5`, and the two
public CLI fixtures pass `2/2`; the tracker remains process-global rather than
realm-owned.

This closes the current record/ordering/realm-source boundary; it does not yet
provide the broader realm/agent-owned host queue contract. Async continuations
still ride on reaction records, while module and finalization-cleanup jobs
remain outside this two-kind queue. Full execution-context switching and
realm-correct allocation across the complete builtin surface also remain T06
work, so this is not a claim of complete cross-realm Promise conformance.

Created-Realm Promise publication now has a typed foundation. Realm intrinsic
records contain a required `%Promise.prototype%` slot, and both entry and
created bootstrap populate it. Main and created Realms consume the same closed
three-method prototype and ten-method static publication catalogs. Created
constructors, methods and the species getter receive fresh defining-Realm
function identities, self environments and TypeError/RangeError captures.
Allocation now accepts only an opaque context coupling its selected
`[[Prototype]]` with the executing constructor Realm; the constructor uses a
required resolved-Realm Promise fallback, while resolving functions inherit the
stored Promise Realm's Function and error prototypes. A focused non-blocking
CLI fixture proves the published descriptors and identities, created
constructor/`Promise.resolve` result prototypes, resolving-function Function
prototype and constructor TypeError Realm. It deliberately does not drain jobs.
The bounded source target passes `5/5` and the focused CLI consumer passes
`1/1`. The remaining cross-Realm Promise method errors and callback
execution-context switching are not closed by this foundation.
`Atomics.waitAsync` now consumes the opaque intrinsic Promise allocation
context directly in its async emitter. Both result emitters consume a private,
non-copyable Object-prototype proof from the executing Atomics function Realm,
so the async wrapper and Promise use that Realm's required Object and Promise
prototypes; the result defines enumerable
`async` then `value` CreateDataProperties with exact writable/configurable
attributes. The bounded result contract passes `4/4`; a distinct non-blocking
fixture passes `1/1` while taking not-equal, timeout-zero and immediate-notify
async branches and observing the resolved `"ok"` value through the created-
Realm Promise method. The consolidated semantic golden passes `2/2` in 733.38
seconds and contains 660 fixture dumps. Relative to the preceding 658-dump
checkpoint it adds only the two focused Promise/`waitAsync` witnesses, removes
none, and preserves every retained structural summary after normalizing
emitted-function byte sizes.

The central feature-enabled CLI compile covers the consolidated job machinery.
The typed callback-word/realm policy's durable layout contract is green, as are
the engine contracts proving that reaction jobs run after synchronous code in
registration order and that thenable-resolution jobs are asynchronous and
settle once. At the 2026-08-25 coordinated checkpoint, the exact Promise
lifecycle and ordinary async resume-completion heap tests each pass `2/2`; the
two Promise engine regressions and three ordinary-async/`for-await-of` engine
regressions each pass `1/1`. Three exact current-pin async leaves pass all
`6/6` sloppy/strict Wasm-AOT executions at vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure and
non-success bucket at zero. These checks are not a substitute for the full
Promise/async Test262 filters.

Plain async functions now retain a captured lexical `for-of` iteration record
across a body `await` rather than hoisting every iteration into one activation
cell. The durable fixture calls the closures only after the loop, so a reused
cell observably produces `6,6,6,6,6,6`; it additionally requires clean job-queue
drain with no uncaught asynchronous throw. The two exact current-pin witnesses,
`built-ins/Array/fromAsync/asyncitems-asynciterator-not-callable.js` and
`built-ins/Array/fromAsync/asyncitems-iterator-not-callable.js`, exercise the
newly admitted lowering shape but invoke their capture in the same iteration,
so they do not replace the distinct-environment fixture. The integrated
current-SHA consumer gate passes, and the two exact pinned witnesses report
`4/4` under Wasm-AOT. The complete 95-file `Array.fromAsync` leaf was not rerun.

Promise resolving functions, the capability executor, both finally stages and
all keyed/standard combinator element functions now share one typed
materialization boundary. All fourteen escaping closures receive their
defining Realm's Function, TypeError and RangeError prototypes before exposure,
store algorithm state outside the environment slot and self-back the function
identity used by error and Proxy operations. Existing reaction-job
`GetFunctionRealm` selection therefore observes the corrected callback Realm.
The focused finite fixture captures every family and exercises capability and
self-resolution TypeErrors. Callback-created AggregateError/Object results,
dynamic async Promise allocation and two nonescaping PromiseResolve surrogates
remain outside this batch.

The four direct async Promise allocations and compiler-owned captured reactions
now consume an opaque `AsyncExecutionRealmContext` instead of the dynamic
current-Realm global. Async invocation derives the context from the callee and
stores it in a traced ordinary activation slot; async generators derive it from
their existing retained function object. Default reactions keep their
`GetFunctionRealm(handler)` or null policy, while the five async continuation
kinds store activation-owned Realm authority through a distinct API. A bounded
source target and finite created-Realm job fixture cover both activation
families without blocking. PromiseResolve constructor catalogs and other async
builtins remain separate batches. The consolidated semantic golden passes `2/2` in 677.52
seconds and contains 663 dumps. It adds only the async-execution,
callback-created-allocation and internal-callback Realm witnesses to the
preceding checkpoint, removes none, and preserves all 660 retained structural
summaries after expected code-size and local-accounting fields are normalized.

Promise reaction initialization now carries its default or captured-async Realm
policy through one private non-`Clone`, non-`Copy` domain. Four named producers
construct the choice; intrinsic Await owns it once and borrows it through the
exhaustive PromiseResolve-authority projection and the ordered fulfill/reject
reaction initializers. The reaction initializer separately exhausts the same
choice for stored Realm and callback kind. This removes the former capability
to duplicate the policy while the guarded emitter retains one borrowed input
across all three projections. The separate five-way `AsyncAwaitContinuation`
now also derives no cloning or copying capability: Await borrows it for exact
Realm selection and its five-row callback projection before moving it once into
the reaction-initialization policy. Six producers remain exact, including
fulfill-before-reject AwaitReturn construction. These ownership changes alter no
heap word, emitted Wasm local, Wasm instruction or evaluation order. The
dedicated and two neighboring
structure targets pass `13/13`; the package formatting check is green.
The exact existing created-Realm CLI witness passes `1/1`. Independent review
confirmed the capability/mention closure, producer mappings, shared-policy
borrowing and projection order. The earlier reaction-initialization checkpoint
and this continuation extension now share a green coordinated checkpoint:
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module-boundary
check and the task-plan check. The bounded contract is
[`promise-reaction-initialization.md`](../docs/rust-rewrite/contracts/promise-reaction-initialization.md).

Async-generator request capability allocation now uses the canonical
`%Promise%` constructor catalogued by the executing request method's defining
Realm. One opaque non-copyable constructor proof fixes the Function tag and is
consumed by capability construction before receiver validation. The shared
`next`/`return`/`throw` arm no longer reads the entry Promise global, while its
three entry method identities are self-backed before publication and its
request record and activation layouts remain unchanged. PromiseResolve's two
nonescaping surrogates and other async-builtin allocations remain deferred. The
bounded contract passes `4/4`, the exact Realm fixture passes `1/1`, and the
664-dump semantic golden passes `2/2` in 707.34 seconds. It adds only the
Temporal field-mode fixture, removes none and preserves all retained
non-accounting summaries except the strengthened async Realm witness's five
intentional internal/name entries.

The standard combinator outer Array now belongs to the executing `all`,
`allSettled` or `any` method's defining Realm independently of constructor `C`.
The common allocation consumes the existing opaque current-function Array
prototype proof, including both `Promise.any` terminal paths, while keyed
combinators and `race` remain outside the boundary. The bounded allocation
target passes `7/7`, the finite cross-Realm fixture passes `1/1`, and the
following 665-dump semantic golden passes `2/2` in 707.16 seconds. It adds only
the RegExp result-mode witness, removes none and changes no retained
non-accounting summary except the intentionally expanded Promise witness's two
internal/named functions and four main-function locals.

`Promise.withResolvers` now allocates its outer ordered record from the
executing method's defining-Realm `%Object.prototype%`, independently of the
constructor `C` that owns the capability Promise and resolving functions. A
private must-use proof preserves capability-before-result order, traps missing
nonentry catalog state and is acquired only after the fallible raw shell
allocation. The bounded Realm contract passes `5/5`, the retained publication
contract passes `5/5`, and the finite borrowed-method fixture passes `1/1` in
both Realm directions without queuing reactions. The following 666-dump
semantic golden passes `2/2` in 704.11 seconds, adds only the array
key-selection witness, removes none and preserves every retained non-accounting
summary.

`Promise.try` now handles a non-callable callback with TypeError from the
executing method's defining Realm. A private must-use prototype proof selects
the entry snapshot only for zero-environment builtins, traps missing self-backed
snapshots and is consumed before the existing capability rejection. Capability
creation and forwarded-argument construction retain their specified order; the
invalid branch does not return early. The bounded contract and retained
publication target pass `5/5` each, and the FIFO created-Realm callback fixture
passes `1/1`. The following 667-dump semantic golden passes `2/2` in 702.89
seconds, adds only the iterator-policy witness and removes none. The sole
retained non-accounting change is the deliberately expanded callback witness's
one internal/named function and two main-function locals.

`Promise.prototype.then` and `Promise.prototype.finally` now pass a private,
must-use defining-Realm context into their shared SpeciesConstructor lowering.
The paired proof owns the default `%Promise%` and both validation TypeErrors,
precluding an impossible mixed-catalog selection while preserving constructor
Get, `@@species` Get, validation and capability order. The isolated contract is
[`promise-species-realm-context.md`](../docs/rust-rewrite/contracts/promise-species-realm-context.md).

The direct receiver failures in `Promise.prototype.then` and
`Promise.prototype.finally` now use a private one-shot TypeError-prototype proof
from the borrowed method's self-backed Realm snapshot. The closed two-variant
error domain owns the diagnostics, and proof acquisition remains confined to
the invalid branches before SpeciesConstructor. Borrowed
`Promise.prototype.catch` now performs current-function Realm ToObject and an
abrupt lookup checkpoint. Both `catch` and `finally` then cross a private
non-`Copy` validated delegated-Call boundary: its two-caller validator performs
Proxy-aware callability and owns the current-function Realm TypeError, while its
two-caller consumer preserves the original receiver and two arguments. Errors
inside the later callable-Proxy Call remain T11 work. The isolated contract is
[`promise-prototype-receiver-error-realm.md`](../docs/rust-rewrite/contracts/promise-prototype-receiver-error-realm.md).
The following shared workspace semantic golden passes `2/2` in 696.00 seconds
with 668 dumps, adds only the independently expanded shape-accessor witness,
and removes none. After accounting normalization, 664 of 667 retained dumps
are equal; the only structural changes are the intended Array reduce, Promise
internal-callback Realm, and TypedArray constructor no-species witnesses.

The two formerly raw PromiseResolve functions now have one-shot executing-Realm
ownership. Intrinsic await paths use a paired proof that obtains the canonical
`%Promise%` constructor and self-backed resolve function from the same catalog;
async-generator await-return borrows its activation Realm, while `finally`
continuations use their executing closure Realm with the earlier species
constructor kept as the separate `C` authority. Abrupt await fallback reuses
the paired constructor, and NewPromiseCapability failures use the operation
function's Realm. The expanded borrowed-`finally` witness proves the resulting
TypeError prototype. Seven overlapping structure targets pass `37/37`, the
finite CLI witness passes `1/1`, and the following 669-dump semantic golden
passes `2/2` in 771.49 seconds. It adds only the independent Temporal
arithmetic witness, removes none and leaves 667 of 668 retained dumps equal
after accounting normalization; the expanded Promise callback witness is the
sole retained structural change. Test262 verification remains deferred. The
contract is
[`promise-resolve-realm-context.md`](../docs/rust-rewrite/contracts/promise-resolve-realm-context.md).

The private `PromiseResolveRealmAuthority::{CurrentFunction, AsyncExecution}`
selection now derives no incidental capability. Each operation or intrinsic
context factory owns the selection, forwards it once and lets the shared
exhaustive materialization selector consume it; duplicating one chosen Realm
authority is now an E0382 move error. The Rust-lexical guard pins the exact ten
identifiers, three by-value factory parameters, four semantic producer routes,
single forwarding in both outer factories and sole two-arm consumer. This is
source-equivalent ownership hardening and adds no runtime or conformance claim.
The dedicated structure target passes `4/4`, the existing PromiseResolve Realm
context target passes `4/4`, the neighboring reaction-initialization target
passes `4/4`, and the created-Realm Promise internal-callback CLI witness passes
`1/1`. Broad Promise, Test262, golden and workspace verification remain
deferred. The boundary is recorded in
[`promise-resolve-realm-authority-ownership.md`](../docs/rust-rewrite/contracts/promise-resolve-realm-authority-ownership.md).

The complete PromiseResolve Realm-context lifecycle now has one private child
owner. Its three factories preserve the exact `4/5/1` authority split across
the parent, Realm-context child and finally-completion child. The split has
zero import/re-export paths. No parent or sibling can project either carrier.
At the coordinated checkpoint, the Realm-context and authority-ownership
structure targets each pass `4/4`, and the internal-function Realm-context
target passes `6/6`, for `14/14` focused structure checks. The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green. Semantic goldens were not
rerun because this is a source-equivalent owner move.

The complete shared Promise internal-function materialization lifecycle now has
one private child owner. Its non-`Copy`, must-use four-local carrier, three
factories, borrowing materializer, closure-context loader and consuming release
move together; private fields prevent parent or sibling construction and
projection. A narrow child-owned capability replaces PromiseResolve's final raw
Realm projection without changing the emitted Realm-intrinsics load. The
recursive guards pin eleven carrier identifiers and the exact
`4/7/2/11/9/9/2` lifecycle/capability census. At the coordinated Batch AG
checkpoint, the internal-function, PromiseResolve Realm-context and
callback-created-allocation structure targets pass `6/6`, `4/4` and `7/7`, for
`17/17` focused structure checks. The exact
`functions::run_wasm_backend_preserves_created_realm_promise_internal_callbacks`
and
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witnesses each pass `1/1`, and shared `cargo xc` is green. No Test262
cohort or semantic golden was run because this is a source-equivalent owner
move; no new behavior or conformance claim is made.

The Promise returned by `Array.fromAsync` and its two possible await throwaway
capabilities now share one typed executing-method Realm context. The context
selects the entry `%Promise%` only for a zero environment and otherwise requires
the self-backed method's defining-Realm intrinsic catalog. Both branch helpers
borrow the proof instead of accepting raw Promise-constructor payload/tag
pairs, and one consuming release closes its local lifecycle. Constructor `C`
still independently selects the result Array. A finite created-Realm fixture
covers both authority directions and a rejected invalid mapper. The bounded
structure target passes `5/5`, the finite CLI witness passes `1/1`, and the existing
`array_from_async` CLI cohort passes `4/4`. The following shared
671-dump semantic golden passes `2/2` in 697.36 seconds, adds only this witness
and the independent Temporal plain-difference witness, removes none and leaves
all 669 retained dumps equal after accounting normalization. Test262
verification remains deferred. The contract is
[`array-from-async-promise-realm-context.md`](../docs/rust-rewrite/contracts/array-from-async-promise-realm-context.md).

The same non-copyable `Array.fromAsync` execution context now owns the fixed
fulfilled/rejected callback pair. One materializer installs the defining Realm,
default Function prototype, TypeError prototype, self-backed environment and
GC-visible continuation-state link together. The array-like and iterable
branches can no longer allocate ambient-entry-Realm callbacks or use their
environment slot as raw state. The state record shrinks from 184 to 176 bytes,
and all nine await scheduling sites retain the rooted pair. The new and updated
bounded targets pass `10/10`, and the four-path CLI witness passes `1/1`.
The shared 678-dump semantic golden passes `2/2` in 722.99 seconds, adds this
witness plus the independent Object-policy, Promise-mode and Set-domain
witnesses, removes none and leaves all 674 retained dumps equal after
accounting normalization. Test262 verification remains deferred. The focused
boundary is
[`array-from-async-internal-callback-realm-context.md`](../docs/rust-rewrite/contracts/array-from-async-internal-callback-realm-context.md).

Promise combinator algorithmic failures now use the executing borrowed
method's Realm independently of constructor `C`. One private non-copyable
context pairs the defining Realm's TypeError and RangeError prototypes; the
`race`, keyed and standard combinator lowerings acquire it after `C.resolve`,
borrow it across the exact six/two/seven failure-site census and release it
once. The focused created-Realm witness covers `all`, `allSettled`, `allKeyed`,
`allSettledKeyed`, `any` and `race` with the entry Promise constructor. The
unbounded maximum-length RangeError is structure-only, while the dead static
settle validation and callback materialization remain explicit follow-on work.
The bounded structure target passes `5/5` and the focused CLI witness passes
`1/1`. The shared 674-dump semantic golden passes `2/2` in 717.58 seconds, adds
this witness plus the independent Temporal overflow-options and GroupBy
result-kind witnesses, removes none and leaves all 671 retained dumps equal
after accounting normalization. Test262 verification remains deferred. The
isolated contract is
[`promise-combinator-algorithm-error-realm.md`](../docs/rust-rewrite/contracts/promise-combinator-algorithm-error-realm.md).

The standard and keyed combinator lowerings now accept distinct closed mode
domains. Standard `all`, `allSettled` and `any` policy remains a three-case
exhaustive projection, while keyed `allKeyed` and `allSettledKeyed` use a
restricted two-case domain that cannot represent keyed first-fulfillment
semantics. Neither domain implements equality. Three keyed and seven standard
policy decisions are direct compile-review points rather than equality or
default branches. The bounded structure target and finite all-five-mode CLI
witness are recorded in
[`promise-combinator-mode-domains.md`](../docs/rust-rewrite/contracts/promise-combinator-mode-domains.md).
The same shared 678-dump checkpoint adds this witness and preserves all 674
retained dumps after accounting normalization. Broad Test262 verification
remains deferred.

## Objective

Implement the ECMAScript job model, complete Promise semantics, async functions and async iteration with deterministic host integration suitable for Test262 and embedders.

## Job queue contract

- Define realm/agent-owned FIFO job queues and host enqueue/drain hooks.
- Keep promise reaction jobs, thenable jobs, async continuation jobs, module jobs and finalization cleanup jobs distinct where observably required.
- Drain jobs at specified host checkpoints; do not run them eagerly inside `then`/resolution.
- Preserve realm and incumbent/active execution context needed by each job.
- Integrate Test262 `$DONE`, timeouts and rejection reporting without treating an empty queue as success before async completion.

## Promise implementation

Implement:

- internal state/result/reaction lists and resolving functions;
- thenable assimilation, self-resolution rejection and already-resolved guards;
- `then`, `catch`, `finally`, species and derived promises;
- constructor executor ordering and abrupt completion;
- `resolve`, `reject`, `all`, `allSettled`, `any`, `race`, `withResolvers` and current pinned additions;
- iterator closing and AggregateError behavior;
- metadata/descriptors and cross-realm error ownership.

All combinators must use shared iterator operations rather than array-only shortcuts.

## Async functions

Lower async bodies to resumable state machines whose calls return promises immediately. Implement:

- `await` conversion/then behavior;
- suspension/resumption through queued jobs;
- return/throw/finally across suspension;
- lexical environment and `this`/arguments/new-target retention;
- async arrows/methods and class methods;
- async stack cleanup and GC rooting.

## Async iteration

Provide `AsyncFromSyncIterator`, async iterator acquisition/close, async `for-await-of`, and the interfaces required by async generators/iterator helpers in T15.

## Host and blocking behavior

`can_block` must affect Atomics/host behavior, not Promise ordering. Provide a deterministic test driver that can run jobs until completion or a deadline and report pending jobs/rejections on timeout.

### Unhandled rejections now surface (fixed 2026-08-02)

A promise that rejected with no handler used to produce no diagnostic and exit
status 0. Combined with top-level await wrapping module bodies in an async
function, that meant a `flags: [module]` Test262 case whose assertion FAILED was
scored as a PASS - the measurement reporting green on red.

Fixed: rejected-with-no-handler promises are tracked on a list, and after the
job-drain loop the main export's completion kind is set to Throw carrying the
rejection value. Verified in both directions, which is the part that matters -
a fix that reported *handled* rejections would have turned passes into failures:

| case | reported | exit |
|---|---|---|
| `(async () => { throw ... })()` | yes | 1 |
| `await 0; throw new Test262Error(...)` | yes | 1 |
| immediate `.catch` | no | 0 |
| `.catch` attached in a *later* job | no | 0 |
| `try`/`catch` around `await` | no | 0 |
| `Promise.all` rejected then caught | no | 0 |

Implementation note: the promise record grew from 64 to 72 bytes for the list
link, and the global registry gained two slots.

The former oldest-only diagnostic hole is now focused-verified: the oldest
rejection remains the failing completion when there is no primary Script throw,
and the existing host line-output ABI reports every other unhandled value in
the checkpoint snapshot. With a primary Script throw, every snapshot value uses
host output and the Script throw remains primary. Diagnostic coercion operates
on a detached finite snapshot, leaving reentrant rejections on the fresh live
tracker. `cargo xc` is green; the bounded source guard passes `5/5`, the engine
checkpoint regression passes `1/1`, and the public CLI fixtures pass `2/2`.

One hole remains in the same story:

- The rejection list is process-global rather than per-realm, so cross-realm
  (`$262.createRealm`) promises share one tracker. Untested territory rather
  than a known break - cross-realm is the one feature still failing the probe.

Also fixed 2026-08-02: `await` inside a loop body was miscompiled - state living
across a suspension point inside a loop was not restored, so
`for (let i = 0; i < 3; i++) { t += await Promise.resolve(i); }` summed to 0
instead of 3. Now correct, including the `const v = await ...` and `for-of`
variants.

## Acceptance criteria

- Promise state and resolution tests pass, including hostile thenables and side-effect ordering.
- Combinators pass iterator-close, species and subclassing tests.
- Async functions preserve environments and finally semantics across multiple awaits.
- Async Test262 cases pass/fail based on `$DONE` or returned async completion, with duplicate completion detected.
- Cross-realm promises and errors use correct intrinsics.
- No busy-loop polling is required for ordinary promise progression.
- The pinned Promise/async-function/async-iteration filters reach zero failures.

## Required tests

```sh
cargo test -p lila-runtime job_ --quiet
cargo test -p lila-aot-wasm promise_ --quiet
cargo test -p lila-cli wasm_async --quiet
./target/debug/lila test262 run built-ins/Promise --execution-backend wasm-aot --timeout-ms 120000 --threads 4
```

Also run async function, `await`, `for-await-of`, async iterator and top-level-await filters, plus intentionally hanging/duplicate `$DONE` harness tests.


### 2026-10-02 source-only plain async switch continuation proposal

The isolated T14 proposal is based on the reviewed Locale/GC-wording source and
keeps the admitted 43-stage/1,520-test, fresh-CLI and 39-file/65-mode checkpoint
as historical evidence. Its mandatory MAIN broad run is still separate; no pass
is inherited by this proposal.

A consuming source-admission carrier refuses awaited selectors and implicit
`await using`/`for await` suspension before any case is lowered. A private checked
IR owner couples each actual case body to its continuation segment. Eager cases
receive separate segments whenever a child allocates a continuation, including
ordinary `try` clauses without an `await`; wholly eager switches retain their
existing synchronous representation. The Wasm emitter selects once after shared
CaseBlock instantiation, retains selection in the activation state, reattaches
the saved environment on resume, and commits switch exit after matching breaks.
An awaited discriminant is staged before creating that environment. Body await,
admitted If/Try/Block/label/awaited-while children, fallthrough, finalization and
nested synchronous `using` use the existing real compiler/emitter paths.

Authored IR and paired Engine controls cover selector order/default placement,
no-match/early-break exits, nested owners, captured cells/TDZ, rejection identity,
awaited finalizers and synchronous disposal. They are **UNCOMPILED/UNEXECUTED**.
Targeted Rust formatting and source-bound inventory checks are separate from
verification. Full compilation, meaningful runtime regressions, pinned primary
filters and the mandatory broad checkpoint remain future Root work. Awaited
selectors, implicit asynchronous disposal/iteration in cases, enclosing-loop
composition and generator switches still require their own typed owners. This
proposal does not close the general T14 acceptance criteria or change published
suite counts. See [the contract](../docs/rust-rewrite/contracts/plain-async-switch-continuations.md).

### 2026-10-03 async switch IR carrier assertion repair

The fresh UTF8 verification attempt retained an actual Root exit1 after all
eight Locale cases passed: async-switch IR stage51 reported six passed tests
and one failed assertion that required its discriminant await to be the first
prefix statement. Inspection of the exact compiled IR confirmed the required
activation-owned result cell before that await, followed by the switch owner.
The test-only repair now requires the exact cell/await/switch sequence, the
same declared, resumed and discriminant-read binding, and its owned environment
slot. It separately checks discriminant states `0→1`, case states `2→3` and
following-await states `4→5`, preserving switch entry `1` and exit `4`. Production
lowering is unchanged. The failed receipt remains historical evidence; focused
execution and complete verification of the repaired Source are pending.

### 2026-10-03 dry suspended switch selector ownership

Plain async switches now lower stageable case-selector awaits through the
existing async statement prefix and activation dispatcher. The discriminant
is evaluated once and stored in an activation-owned binding before CaseBlock
instantiation. All selectors in that switch have checked selection segments;
their private constructors validate the actual prefix, ready state and next
decision state. Exhaustive selector matches require callers to handle both
eager and resumable forms. An independently derived fallback state precedes
every body entry, so a first-case match cannot be confused with no match.

The shared lexical environment and its TDZ cells are instantiated before
selection and reattached on resume. Selectors run in source order until a
match, while default selection waits for every failed test. Existing body
segments preserve fallthrough without evaluating later selectors, matching
breaks, nested awaited finally and rejection propagation. Frame/data/function
planning, throw inference and source traversals visit each actual prefix;
saved discriminant and operand cells use the existing activation layout.

Fifteen new Rust regression declarations are authored: three constructor
ownership controls, two lowering controls and ten paired sloppy/strict AOT
sources. They cover state gaps/overflow/mixed owners, first-match/fallthrough,
default/no-match, discriminant identity and scope, retained cells/TDZ, method
receiver order, rejection and labelled breaks through awaited finalizers.
The previous awaited-selector refusal becomes a branch-sensitive-selector
refusal; implicit asynchronous disposal/iteration, enclosing-loop composition
and generator switch suspension remain explicit dependent work.

Independent source review found that the inherited eager-switch reset pattern
could erase writes from a failed awaited selector before lowering the next
test. The correction carries sequential var/global facts, takes default-entry
facts only after all selectors fail, joins each body with possible fallthrough,
and includes no-match in the exit join. Mutable scope values and static caches
are conservatively widened at body and exit joins while retaining the existing
lexical lifecycle/storage owner. Four additional paired sources cover the
reported `x = 'abc'` then `x.length` selector, default/no-match writes, direct
entry versus awaited fallthrough, and skipped lexical value effects. The first
patch and its blocking source review remain retained; no runtime repair is
inferred from this source correction.

Formatting and patch checks are source checks only. Compilation, focused
execution and the combined verification checkpoint remain pending under the
implementation-first workflow. No runtime pass or published count changes are
claimed. See [the contract](../docs/rust-rewrite/contracts/plain-async-switch-continuations.md).
