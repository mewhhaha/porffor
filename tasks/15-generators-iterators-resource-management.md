# T15 — Generators, iterators, iterator helpers and resource management

## Iterator materialization retains runtime protocol — 2026-10-07 dry source

The remaining source-text iterator caches are removed. They replaced
Array.from/Iterator.prototype.toArray calls with parsed generator yield values,
discarding actual iterator consumption, next effects and own method selection.
They also treated a non-iterable object with a next method as iterable and
trusted shadowed Array/Iterator spellings. Calls now use the existing acquired
callee and actual runtime iterator owners.

The generator-call override map had only an unsupported-expression producer;
its map, projections and no-op stubs are removed without adding a replacement
execution path. Generator expressions continue through their checked source
suspension plan and normal FunctionObject creation. Orphan source parsers and
all declaration/assignment/branch cache bookkeeping are removed with the folds.

Two existing IR controls now require real nested calls. Three paired Engine
cohorts cover non-iterable fallback, consumed-state retention, next effects and
abrupt values, own/shadowed/replaced methods and fresh generator invocation.
Compilation and runtime verification remain deferred; this source batch claims
no new passing Test262 cases.

## Array Yield/Await, With and complete ForIn source owners — 2026-10-06

Ordinary-generator array patterns retain their actual GC IteratorRecord and cached
next/done state across yielding target/default work. Original GetIterator, Step,
Put and IteratorClose owners remain shared, including nested patterns and whole
injected Return/Throw. Complete With retains the original boxed object record;
the head runs before entering that record and resume reattaches it before body
injection. Class abrupt cleanup and invocation-entry anchoring preserve original
parent lexical/private environments and nested finalizer ordering.

Complete ordinary ForIn retains four checked invocation cells and one GC cursor
for current object, remaining accepted keys, index and visited String keys.
Head TDZ, original Var/lexical/pattern/Reference initialization, per-iteration
environments and local/outward completion routing are joined. Admission requires
an actual retained-key write or the original sloppy immutable-binding Ignore
Reference proof. OwnKeys and descriptor observations preserve StatementList V;
prototype traversal remains lazy and ForIn performs no IteratorClose.

Plain async Array patterns now consume an opaque complete owner through lexical,
var and used/discarded assignment paths. The actual source Await/operation tape,
branch ranges and original cell inventory are checked once, including nested
arrays, recursive objects, optional tails and class evaluation operands. General
state readers continue to reject bare iterator operations. The native Generator
and Async entries share one physical acquisition/body/close pipeline; rejected
Await enters the reconstructed close scope and normal Await retains original
References and iterator storage.

Plain async With now checks the actual ToObject codomain, original object record,
head publication and complete Await tape. Plain async ForIn checks the exact AST
head identity, four original invocation cells and the shared physical retained-key
initialization proof. Head/body ranges and continue-to-advance routing share the
original ordinary-generator native pipeline. Rejection reconstructs original
cleanup scopes before injection; ForIn performs no IteratorClose.

Mixed async-generator classic loops, If/value regions, complete With, Switch and ForIn
have independent source reviews. One source allocator checks all eager and
suspended phase entries and the exact Await/Yield tape. Private certificates
retain invocation anchoring and captured Block/Try ancestry; the original
ForAwait iteration environment and implicit disposal boundaries remain joined.
The native code uses the original loop, Reference, With and Switch algorithms. Ordinary,
plain async and mixed With share one private validated ToObject/Object
Environment proof. The old proxy-control successor corrects four expected
HasProperty counts from the actual Object Environment algorithms.

Mixed Switch checks its complete discriminant, lazy selectors, fallback and
source-order bodies against the exact mixed tape. Its original two retained cells
and shared CaseBlock are validated once. Discriminant work precedes CaseBlock
TDZ/function instantiation; default selection follows every actual selector.
The shared operand finish callback reads generated terminal values inside their
actual temporary scope, including original and mixed loop callers.

Mixed ForIn shares the original four-cell storage and retained-key initialization
proof. Head TDZ records remain distinct from fresh per-key records, including
closures created before enumeration. Its original cursor performs no IteratorClose.

These source epochs and meaningful AST/IR/native/Engine controls remain
uncompiled and unrun. Mixed patterns, nested or suspended iterator heads and
suspended resource continuations remain source work before joined verification.

## Switch, Throw, object-pattern and class-name source owners — 2026-10-06

Ordinary Switch stages its complete discriminant before the CaseBlock environment,
then evaluates selectors in order and retains fallthrough completion in original
activation cells. Its private source and IR types validate state ranges, lexical
ownership and local Break handling. Empty statement completion wraps the entire
original item. Yielding Throw operands stage their whole selected value before
the existing Throw operation; injected Return/Throw retain the existing finalizer
and pending-completion transport.

Object patterns now suspend in computed names, assignment targets and complete
undefined-only defaults. Ordered raw/boxed source and normalized PropertyName
factories feed the same native GetV, CopyDataProperties and target Put owners.
WriteOnly Identifier References and raw member base/key cells survive suspension;
lexical declarations and scoped classic For heads initialize original cells.
Recursive objects and eager nested arrays keep their actual semantic owners.
The patched frontend cover converter retains computed nested patterns and actual
NamedEvaluation. Class labels now use parser name-scope provenance: inferred
names preserve outer reads and TDZ, explicit names retain inner class cells.

The complete source joins, retained controls and boundary guard have independent
reviews. New IR and paired Wasm fixtures cover ordering, TDZ, GC, primitive
receivers, retained With References, iterator closing and yielding finalizers.
Compilation, guard execution and runtime remain unrun. Array-owned Yield/Await,
broader ForIn/With, nested/suspended iterators, mixed async-generator and resource
continuations remain open before the combined capped verification.
See the [object-pattern contract](../docs/rust-rewrite/contracts/generator-object-patterns.md)
and [class-name contract](../docs/rust-rewrite/contracts/class-name-source.md).

## Earlier optional and eager assignment source joins — 2026-10-06

Eager assignment patterns now stage their whole RHS before target keys, Gets,
defaults/rest and the existing assignment emitter. Scoped classic For lexical
heads retain the actual declaration-to-activation mapping.

Optional chains own the complete base plan and guarded key/argument regions,
including multiple/delegated Yields. First Calls retain their actual Reference
receivers; later Gets and spreads keep their established order. Terminal property
Delete retains raw base/key, skips its getter and returns true when shorted.
Ordinary and suspended Delete share the existing native deletion owner. All retained
source control names/cohorts remain, with complete-region state assertions.

These owners have independent source reviews and authored GC/whole-completion
controls; compilation and execution remain unrun. Object-owned suspensions and
Switch now have the source owners above. Array-owned suspensions, broader ForIn/With,
nested/suspended iterators and mixed async/resource continuations remain source work. See the [optional region contract](../docs/rust-rewrite/contracts/generator-optional-regions.md).

## Checked ordinary generator loops — 2026-10-06 source

The ordinary generator path has a source-owned phased For/While/DoWhile graph
and checked conditional regions. The backend reconstructs actual loop and
label targets on each resume, stores the whole loop completion in its owned
activation cell, recreates captured For iteration cells at the specified
boundaries, and uses the existing pending-finalizer completion transport.
Multiple yielding heads/body/update phases, nested branches/loops/finalizers,
GC collection and whole Return/Throw identities have authored Engine controls.

Yielding logical selectors and suspended Identifier assignments have separate
consumed expression owners. Runtime/global/With assignments retain one actual
selected Reference. Compound/logical assignments retain the original whole
GetValue result before the RHS; plain assignments use WriteOnly capture without
GetValue. Coercion follows the complete RHS and logical skipped regions release only the
private record. Completion cleanup preserves live records across delegated
yields and retires them before committed Return or Throw transfer. Meaningful
catch/finally, deletion/unscopables and GC controls are written. Broader
iterator/resource coverage and full pinned acceptance remain open. The joined source is not yet
compiled or executed; isolated formatting and review are source evidence only.

## Complete ordinary expression regions — 2026-10-06 source

Yielding selectors and complete conditional/logical arms retain source-owned
state ranges and publish only after Normal evaluation. Eager operator operands
and template substitution coercion use the actual ordinary semantic owners.
Eager binding patterns stage their entire initializer while lexical names remain
in TDZ; their existing single-Get/default/rest and binding IteratorClose owners
run afterward. Var hoisting and whole abrupt values through yielding finally
remain attached to existing environment/completion transport.

Object literals retain one actual allocation. Each completed key is normalized
before its value; the shared property body handles spread, prototype setters,
data, methods, accessors and computed naming. Prepared class naming has a private
source-property proof and cannot be inferred from generic operand caching.
Both returned and discarded literals consume the same owner. These scopes have
independent source peers and meaningful strict/sloppy GC/order controls, all
unrun. Further pattern/control/iterator/async/resource source joins and the
combined capped verification remain open.

## Object bindings observe each property once — 2026-10-04 dry source

Ordinary lexical/var object patterns and shared synchronous/async/generator loop
heads now consume the existing ObjectDestructure operation. The cloned simple
lexical default and separate var statement optimizer are retired. The same
private initializer keeps actual source modes and renamed storage; arrays keep
their existing binding iterator owner. The backend's GetV lookup boxes a primitive
while preserving the original value as getter Receiver. Rest keeps its separate
CopyDataProperties rules.

Seven existing IR controls are maintained and new actual head/storage controls
and three paired finite Engine cohorts are authored. They distinguish one Get
on either default branch, primitive strict getter receivers, TDZ and var policy,
fresh/interleaved async cells, original abrupt cutoffs, finally and iterator close.
Compilation, runtime, guards and full-task acceptance remain unverified. Finish
all remaining task source before verification under a confirmed 4096 MiB aggregate
kernel cap, zero swap and serial defaults. No task or published count closes.
See the [single-Get contract](../docs/rust-rewrite/contracts/object-binding-single-get.md).

## Generator eager lexical-pattern heads — 2026-10-04 dry source

Plain synchronous generators now admit eager Let/Const array/object binding
patterns in synchronous for-of heads. The real source gate rejects Yield/Await
in the initializer and retains eager iterable/current-body ownership. Analysis
provides every source binding's fresh cell, including uncaptured cells.
The consumed shared proof validates real mode, matching TDZ names, complete
environment layout and semantic BindingInitialization. The generator head binds
its exact prefix to the checked body before exposing EntryLocal storage.

The incoming Dynamic/all-tag sink is private and cannot be retained, captured,
read by nested pattern operands or used by resumed body metadata. Actual source
mode belongs to head-environment setup independently from that sink. Eager
initialization executes once per entered iteration, before the body's lexical
scope; resume does not repeat property Get, computed key, default or inner
iterator work. The semantic object-pattern owner performs one Get/default
decision. Existing state, close, local-control and yielding-finalizer owners
remain the consumers, with no new ABI or activation layout.

Eleven production/IR-control paths, four Engine paths and eight documentation/
guard paths form the complete source batch. Constructor/lowering controls cover
prefix lifetime, exact state/cell ownership, empty/nested/rest/computed/default
patterns and genuine refused shapes. Three new paired cohorts retain all
eighteen previous cohorts and cover cached next, interleaved captures, TDZ,
original foreign abrupt identity, inner-before-outer close, no-close step
failures and local/injected completions through two finalizer yields.

All source is authored and independently reviewed; it is uncompiled and
unexecuted. Var/assignment pattern heads, suspended heads/iterables, async
generators, resource heads and foreign control owners remain explicit work.
The shared object-binding successor above now retires the ordinary/Async
optimized default producer and corrects primitive GetV receivers. Full T15 and
pinned acceptance remain open. See the
[lexical-pattern contract](../docs/rust-rewrite/contracts/plain-generator-for-of-lexical-pattern-heads.md).

## Generator ordinary-property assignment heads — 2026-10-04 dry source

The complete successor source admits eager Simple property heads in plain
synchronous generators. The actual producer retains the fused ordinary-property
Reference, raw base/receiver, raw computed key and source strictness. It shares
the consumed plain-assignment completion with expression assignment, including
setter dependencies and effect invalidation. No property Get, raw PropertyWrite
or canonical-key substitute is introduced.

Private identifier and ordinary-property constructors validate their own prefix
domains and share the mandatory lifetime and exact prefix/body proof. The
Dynamic iterator-value sink remains entry-only; neither raw Reference operand
nor resumed storage may read, write or retain it. The original property PutValue
runs before the lexical body opens, once per entered iteration inside the live
close/finalizer frame. Declaration binding modes, async lexical-pattern storage,
state arithmetic and iterator close remain their existing owners.

Constructor/lowering controls and three finite strict/sloppy Engine cohorts are
authored with production and documentation. They cover raw key order, no Get,
callable Proxy Set, inherited raw-primitive setters, shadowing/captured body
cells, cached next mutation, native/foreign head errors, no-close step failures,
local Continue/Break and explicit/injected Return/Throw through two yielding
finalizers. This complete source passed the ref105 combined all-target Rust type
check in 30 seconds of watched wall time (Cargo 17.07 seconds). Emitted-Wasm/
runtime and full T15 acceptance remain pending. Suspended head/iterable,
private/super, nonlexical pattern and resource heads, async generators and
foreign control owners remain explicit work. See the
[continuation contract](../docs/rust-rewrite/contracts/plain-generator-synchronous-for-of-continuations.md).

## Generator identifier assignment heads — 2026-10-04 dry source

Plain synchronous generators admit bare identifier assignment heads with an
eager iterable. A private consumed head proof distinguishes a validated source
binding from a prepared identifier PutValue. Assignment retains the actual
outer target and has no source declaration mode, head TDZ scope or fresh
iteration binding. Its source-unspellable iterator-value sink uses temporary
entry storage. Checked construction and the existing exhaustive IR visitor
reject reads, writes and retained storage that could escape into a resume state.

The lowered eager write runs before the lexical body opens, once per entered
iteration and inside the existing close/finalizer frame. Resume does not replay
the write. Const/TDZ/unresolved failures, the justified sloppy immutable-name
no-op, original thrown completions, local control and yielding cleanup retain
their existing owners. Declaration heads and async lexical-pattern behavior use
their actual binding modes; assignment creates none.

Meaningful constructor and lowering controls join three paired strict/sloppy
Engine fixtures covering captures and interleaved instances, head failures and
local control through yielding finalizers. Code, controls and documentation
are authored together and passed the ref100 combined all-target Rust type check
in 15 seconds of watched wall time (Cargo 11.08 seconds). Emitted Wasm and
runtime checks remain pending. Var/assignment patterns and resource heads, suspended head/iterable forms
and async generators remain explicit separate work. Full T15 stays open. See the
[continuation contract](../docs/rust-rewrite/contracts/plain-generator-synchronous-for-of-continuations.md).

## Iterator.from acquisition and wrapper Call — 2026-10-04 dry source

The complete Iterator.from family now has a private consumed implementation
leaf. Acquisition observes Symbol.iterator once with the original receiver,
uses general callable Proxy dispatch, validates the final iterator after both
nullish fallback and method Call, then caches next without an eager callability
check. Only this completed record can reach identity or wrapper publication.

Wrapper next and return validate the existing private fields without observable
receiver traps. Next calls the cached method; return gets its method afresh and
creates a fresh defining-Realm done result only for a nullish method. Both
forward arbitrary Call results, including primitives, and original thrown
values. Native failures use the called intrinsic Realm. Existing layouts,
prototype walk and identity behavior remain the implementation owners.

Two paired strict/sloppy WasmAot cohorts extend the existing Iterator consumer
target while preserving its five earlier controls. Code, meaningful controls,
invariants and documentation are authored. The ref97 combined all-target Rust
type check passed; emitted-Wasm/runtime checks remain pending. Other helpers and
full T15 acceptance remain open. See the
[completed-record contract](../docs/rust-rewrite/contracts/iterator-from-completed-record.md).

## AggregateError shared iterator consumer — 2026-10-04 dry source

AggregateError joins the capability-free SyncIteratorConsumer domain as its
sixth consumer. One borrowed authority selects its four native protocol
messages while the shared acquisition and IteratorStepValue owners retain
original throws, callable Proxy dispatch, cached next and done-before-value.
IteratorToList never reads return or closes on acquisition or step failure.
The error projection remains separate from consumer wording and selects the
called builtin Realm. The published errors Array also uses that intrinsic Realm.

Production, finite paired Engine controls and maintained architecture guards
are authored. The current inventory is 24 diagnostic rows, 17 typed projector
calls, 43 error identifiers, eight consumer constructions and 22 borrows.
The ref97 combined all-target Rust type check passed; emitted-Wasm/runtime
verification remains pending. See the
[AggregateError contract](../docs/rust-rewrite/contracts/aggregate-error-iterator-list.md)
and [consumer contract](../docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md).

## Plain-async switch block resources — 2026-10-04 dry source

Supported nested blocks inside a plain-async switch now use the existing
AsyncFunction disposal capability. The pre-state source proof admits identifier
`await using` with eager initializers while retaining independent nested
function ownership and the existing loop/iterator/generator boundaries. The
switch constructor authenticates the actual suffix and adjacent finalizer
states; implicit nullish disposal cannot be lowered as an eager case.

The existing resource owner retains acquisition order, async-first lookup and
sync fallback, reverse disposal, pending Break/Return/Throw and SuppressedError
precedence. The switch frame and block lexical environment stay live until
those implicit awaits finish. Meaningful IR controls and three paired Engine
cohorts are authored; the ref93 combined all-target Rust type check passed,
while emitted-Wasm/runtime verification remains pending. Direct resource
declarations in a CaseClause or DefaultClause remain front-end SyntaxError.
Async/sync generator switches, resource heads, suspended initializers and
for-await/enclosing-loop composition remain separate work. Full T15 remains
open. See the
[contract](../docs/rust-rewrite/contracts/plain-async-switch-continuations.md).

## Local generator for-of control — 2026-10-04 dry source

The generator-specific source admission and mandatory body constructor now
prove ownership of unlabelled Break/Continue for the current synchronous
for-of. Blocks, lexical blocks, eager If arms and existing GeneratorTry clauses
retain that ownership; foreign nested loop/switch/label branches and labelled
branches are rejected. The blanket loop-control check for other owners stays
unchanged. Exact Yield/Try state validation remains mandatory and local branches
allocate no new continuation state.

The real shared iterator emitter publishes this loop's break target and a
body-end continue target. Continue passes through the common iteration
environment cleanup and state reset before stepping the cached next. It never
reads return. Break/Return/Throw wait for the completion actually selected by a
yielding finalizer, then close once with the existing Throw-preserving precedence
and outer catch/finally routing. The checked body, IteratorRecord and lexical
activation owners are consumed together; no opcode, ABI or frame representation
is added. The async adapter's admission and behavior stay unchanged.

Three finite Engine fixtures extend the existing nine-fixture target, retaining
its exact Wasm AOT Normal262 and sole `ok` observation in strict and sloppy
modes. They cover Continue before/after Yield, Break through repeated finalizer
Yields, cached next after mutation, fresh captured iteration/body cells,
interleaved owners, finalizer replacement by Break/Continue/Return/Throw,
injected Return/foreign Throw during pending local control, all close error
classes through outer catch/finally, and completed-generator no-repeat close.
Existing iterator-operation no-close controls remain. Constructor and actual IR
lowering controls preserve both accepted ownership and explicit refusals.

This is plain synchronous Generator with eager var/let/const identifier heads
and the existing body grammar. Suspended heads, async generators, resource heads,
labelled/nonlocal branches and foreign nested resumable owners remain outside
this batch. Its combined all-target Rust type check passed in ref77; emitted
Wasm and runtime acceptance remain pending. Full T15 stays open. See the
[continuation contract](../docs/rust-rewrite/contracts/plain-generator-synchronous-for-of-continuations.md).

## Grouped Property References — 2026-10-04 dry source

A private checked grouped source now distinguishes the actual terminal Property
Reference from a terminal Call Value. Both retain the complete finite optional
chain and its guarded state plan. Only the final selected Property Get consumes
the receiver destination; ordinary outer calls and tags pin the completed callee
before their arguments, spreads and template substitutions. Nullish inner
shorting leaves undefined callee and receiver while ordinary outer operands
still run. Constructors retain the existing Value route.

The existing three paired Engine fixtures are extended with retained Proxy,
Symbol and primitive receivers, mutation across several suspensions, spread
order, frozen template identity and injected Throw/Return before and after
callee capture. Original foreign abrupt identity, native error Realm, finally
and prior assignment are checked. Meaningful IR controls cover actual capture,
activation cells, state joins and the surviving admission boundaries. Yielded
bases, optional outer targets, loops/iterator regions, mixed Await/Yield,
async generators, delegated Yield, private/super and unowned branch operands
remain outside this owner. No new frame, ABI, opcode or backend is introduced.
Its all-target Rust type check passed in the ref74 composition; emitted Wasm,
runtime controls and full T15 acceptance remain pending. See the
[grouped yield contract](../docs/rust-rewrite/contracts/grouped-optional-call-value-yield-ownership.md).

## Grouped terminal Call values — 2026-10-03 predecessor

A yielded optional chain ending in a Call now supplies its completed Value to
an ordinary outer Call or tag in an ordinary generator's owned region outside
loops. A private actual-AST terminal-Call source contains the complete existing
finite chain. The mandatory planner appends that guarded state plan before outer
operands, and the consumer completes the same chain before pinning the callee
with no outer receiver. Inner Gets keep their real receiver before yielded
arguments; grouping ends inner shorting, so ordinary outer operands still run
when the completed inner result is undefined. Tags keep their original frozen
cached template site before substitutions.

The existing three paired Engine fixtures now cover returned functions/tags,
mutation across yields, outer spread snapshots, primitive inner receivers,
unconditional outer operand order and injected Throw/Return at inner/outer
suspensions. Foreign marker identity, finally and prior assignment survive
abrupt factory/Call/tag/spread completion. Meaningful IR controls inspect actual
state joins, owned cells, callee pin and receiver disposition. One superseded
Call refusal was removed in this predecessor. Its terminal-Property refusal
is superseded by the owner above. Yielded base/optional outer target,
loops/iterator regions, async generators, mixed Await/Yield and unowned branch
operands retain their boundaries. Construct retains its separate route.
No IR/frame/ABI/opcode/backend is added. Source review is complete, compilation
and runtime acceptance remain pending, and full T15 stays open. See the
[yield ownership contract](../docs/rust-rewrite/contracts/grouped-optional-call-value-yield-ownership.md).

## 2026-10-03 dry implementation follow-up

The shared suspended-switch and Await/Yield invocation owners are uncompiled and
unexecuted. Yield-only Calls, constructors and tags now retain completed callees,
original receivers and private argument snapshots through the same invocation
authority and existing activation bindings. Source-plan preflight rejects
unsupported staged shapes before continuation-state consumption; direct Yield
targets stage their nested operands. Fourteen semantic fixture sources and
explicit mixed/unowned-conditional/prototype-setter refusal inputs are authored.
super() argument staging and general generator control-flow/disposal remain
open. See the [invocation contract](../docs/rust-rewrite/contracts/suspended-call-argument-ownership.md).
The candidate results in the next section remain scoped to the
earlier source and do not establish general T15 acceptance.

The shared plain-async branch admission also closes the async-generator
yielded-await prefix bypass by requiring the same checked source admission.
Mixed Await/Yield and async-generator branch values retain their checked
boundaries. This shared admission change is source-reviewed and integrated;
compilation and runtime verification remain pending. See
[the conditional-await contract](../docs/rust-rewrite/contracts/conditional-await-expression-ownership.md).

Bounded ordinary-generator conditional and logical values have a dedicated
source owner outside all loops. An eager selector or left value precedes at
most one plain Yield in each selected arm. Complete finite optional Property/Call
chains now have their own consumed checked source and state cursor. The eager
base and actual shorted links precede computed keys and arguments, each eager
or containing one plain nondelegated Yield through the existing staging grammar.
Several such operands may occur across the chain. Each selected Yield uses one
existing scalar GeneratorIf and a fresh join; eager guarded steps between them
contain no nested suspension. The retained live cell skips the full suffix.

Each completed Get captures its actual raw receiver and callee once before
arguments. Spread snapshots finish before later Yields; callability follows
argument evaluation, and a completed Call resets a subsequent Call's receiver.
Constructor values consume the completed chain without a call receiver. Private
scopes preserve outer facts and activation-owned cells. Normal-only publication
retains injected Throw/Return, arbitrary foreign markers and existing finally
routing. The complete source plan checks all state arithmetic and final count
before publishing suspension points.

This chain owner leaves yielded bases, nested branch operands, multiple Yields
inside one operand, delegated Yield, loop regions, mixed Await/Yield, async
generators, private/super targets and private links outside its domain. Existing
independently admitted target-only/synchronous paths retain their routes.
Grouped yielded-chain terminal-Property outer callee/tag References now use
the owner above. Direct delete, logical assignment and suspended identifier
Reference writes remain explicit boundaries.
IR controls, the three earlier paired value fixtures and three new paired chain
fixtures are authored and unexecuted. They cover state identity, complete
skipping, retained Proxy/primitive/Symbol References, consecutive Call results,
spread snapshots, ordinary noncallable argument order and abrupt completion.
See the [generator value contract](../docs/rust-rewrite/contracts/plain-generator-value-branch-ownership.md)
and [optional chain contract](../docs/rust-rewrite/contracts/generator-optional-chain-yield-ownership.md).

## Current candidate verification

The same-Source async-switch controls pass their synchronous-disposal composition
and typed refusal checks. This verifies the bounded switch owner, with no
claim of general generator switch suspension, async-disposal acceptance or
iterator-close closure. The earlier dated T15 proposal records remain history.

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


**Status:** In progress — iterator helpers and generator records exist; general suspension remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T08, T09; async portions also depend on T14  
**Blocks:** Iterator-dependent closure across arrays, collections, promises and modules

## Current repository state

The 2026-10-04 remaining invocation source retires ArrayIteratorNext's
canonical next relookup. The acquired function, original receiver and complete
ordinary/spread arguments retain the existing indirect owner. Its actual
catalog row records synchronous user code from array-like length/element Gets;
captured caller facts cannot survive those getters as though the call were pure.
Native iterator branding, iteration and error Realm behavior retain their body.

Actual lowering and paired Engine controls are authored for alias/callee capture,
arguments, getter effects, errors and abrupt cutoffs. No compilation or runtime
acceptance follows from this source; general generator/suspended/resource work
and full T15 stay open until the full-task dry pass and capped verification.
See the [contract](../docs/rust-rewrite/contracts/remaining-invocation-reference-ownership.md).

The 2026-09-29 continuation adds ordinary identifier-headed synchronous
`for-of` loops in plain generators. A validated binding, generator body and
checked continuation span feed the shared resumable iterator emitter; yields
restore the saved lexical chain, and yielding finalizers finish before
IteratorClose. Nine independent engine fixtures cover 18 sloppy/strict
executions, all passing on 2026-09-30. All-target checking and the full IR suite
also pass, along with all 39 neighboring async-loop and generator continuation
tests. Broad verification remains pending. The eight exact pinned yield/delegation
files improve from a fresh baseline of 16 Runtime NotImplemented outcomes to
16 passing Wasm-AOT executions, with every non-success bucket at zero. Pattern or
suspending heads, nested resumable loops and loop break/continue remain open.
See the [continuation contract](../docs/rust-rewrite/contracts/plain-generator-synchronous-for-of-continuations.md).

Review repaired explicit `return yield` so it dispatches its Return through
yielding finalizers and IteratorClose; the existing fixture now also witnesses
`return yield*`, with both controls passing on that earlier source. The
2026-10-03 dry shared binding constructor now requires the original source name
and checks its canonical TDZ placeholder before physical layout validation.
Both async and generator callers retain that name separately from renamed
storage. Existing callers were already correctly derived from the same
`ForOfLoop`; this closes a construction invariant. Independent source review is
complete, while compilation and runtime verification remain pending.

Generator activation/delegation records, iterator operations, several iterator
helpers, async-iterator support and resource-management builtins now exist.
The README records unsupported suspended/control-flow families. General
sync/async generator state machines, iterator-close coverage
across all consumers and complete resource-management filters remain open.

The paired Iterator prototype `constructor` and `Symbol.toStringTag`
weird-setter materializers have been removed. Their unchanged pinned sources
now execute with the full assertion harness; all four sloppy/strict Wasm-AOT
executions pass. The `Iterator.from` return-method materializer has also been
removed. Its three unchanged pinned sources execute with the complete vendored
`temporalHelpers.js` and `compareArray.js` harness; all six sloppy/strict
Wasm-AOT executions pass. The `Iterator.prototype.toArray` materializer has
also been removed after result allocation was bound to the active builtin
function's realm. Its four unchanged pinned physical cases pass all 8/8
sloppy/strict Wasm-AOT executions, including cross-realm Array identity, Proxy
iteration and abrupt-value behavior. The token-aware generated inventory
assigns 32 observations to T15, split between 28 semantic shortcuts and four
diagnostic guards, across the remaining iterator and generator selectors and
their contract guards. Twenty physical metadata branches across
`every`, `some`, `find`, `reduce`, `map`, `filter`, `flatMap` and `take` are now
removed. Their pinned-source matrix covers both Script modes, exact original
bytes, the complete LocalMerged assertion/property preludes and the full
vendored sta/assertion/property preludes. The eight enclosing selector tables
retain other materializers, so their audited observations remain while their
fingerprints narrow. The increase from the earlier 19-entry census reflects
exact rewrite-call, source-guard and match-table coverage, not restored
materializers.

The complete seven-case `Iterator.prototype.forEach` dispatcher and
path-selector body are also gone, while the compiler's standard builtin
remains. The replacement invariant covers the one built-in and six staging
sources in sloppy and strict Script modes, rejects any self-contained rewrite,
and pins exact original bytes plus LocalMerged/vendored assertion, `sta.js`,
`compareArray.js` and active-realm-host provenance. The earlier dated `27/27`
built-in and `12/12` staging results were rewrite-backed; a raw 14-execution
Wasm-AOT replay now passes `14/14`, and the exact replacement-invariant test
`iterator_for_each_cases_preserve_pinned_sources_and_exact_preludes` passes
`1/1`. This focused retirement evidence does not refresh a broader T15 or
published conformance result.

The shadowed TypedArray iterator source matcher and its fingerprint guard are
also gone. Their 17 exact paths were already members of the closed TypedArray
literal-plan authority, so removing them changed no materialized bytes. The
shared 41-path iterator/find invariant covers both Script modes and both
prelude stores, and the representative post-delete product replay passes all
`12/12` executions. This removes one T15 semantic shortcut and one diagnostic
guard; it does not close the remaining Iterator-helper materializers.

On 2026-09-28 the remaining T15 rewrite layer was deleted in full: every
stale path rewrite across `every`, `some`, `find`, `reduce`, `map`, `take`,
`drop` and `zip`, plus the `$262`-dependent realm rewrites, with their
dispatcher arms, source guards and fingerprint tests, and then the emptied
shared dispatcher itself. The enabling compiler fix routes the
`every`/`some`/`find`/`reduce` close-on-noncallable sites through the shared
GetMethod close helper. All 98 affected T15 paths pass exact `lila test262
run` replay under suite pin
`a124fd191443a4e2c7ae2ecad71fee7c9375973e` with every non-success bucket at
zero, and the token-aware inventory now assigns 0 observations to T15. General
suspension and full iterator-close coverage across all consumers remain open.

The shared workspace compile and every repository policy gate pass. The
Wasm-golden corpus remains at 648 artifacts with no additions or removals: 646
dump summaries change only emitted-function/total-size attribution from the
realm-aware Iterator builtin body, with no import, export, runtime-root,
helper-count, memory or data-segment contract change.

Direct synchronous String `for-of` no longer has a code-point-walk IR or Wasm
emitter. Ordinary String heads now use the same generic synchronous iterator
record as other iterables, with a `Dynamic` yielded value. Primitive property
lookup uses the current function Realm's String wrapper prototype but keeps the
primitive receiver for the observable `@@iterator` accessor and method call.
The focused fixture replaces both `String.prototype[Symbol.iterator]` and
`%StringIteratorPrototype%.next`, and its first iterator yields a Number before
break drives `IteratorClose`. `cargo check -p lila-aot-wasm` passes; the String
structure target passes `3/3`, the affected companion structures pass `19/19`,
the IR `for_of` target passes `17/17`, and the CLI witness passes `1/1`. The BMP,
astral, and truncated astral leaves pass `6/6` Wasm-AOT executions with every
failure bucket at zero. This direct-path checkpoint does not complete
protocol-error Realm ownership; the later 15-check boundary documented in
`docs/rust-rewrite/contracts/direct-synchronous-for-of-protocol-error-realm.md`
supersedes that historical nonclaim. A synchronous String loop whose body
directly awaits now uses the activation-backed Iterator Record checkpoint
described below. The focused direct-path contract is
`docs/rust-rewrite/contracts/synchronous-string-for-of-iterator-protocol.md`.

The staging `Iterator.prototype.flatMap` materializer is now gone. Its eight
unchanged pinned bodies use the ordinary assertion route: four use the shared
SameValue-only prelude and four use the complete LocalMerged assertion helper.
All sixteen sloppy and strict Wasm-AOT executions pass.

The `Array.prototype.keys/resizable-buffer.js` materializer was removed first.
The entries counterpart and the four keys/entries grow- and
shrink-mid-iteration materializers are now gone as well. The five newly raw
vendored bodies execute with the full assertion, compare-array and resizable
ArrayBuffer helpers; only the separately owned T13 static substitution for the
helper's dynamic subclass constructors remains. They cover the complete
constructor fan-out and fixed-length, fixed-offset, length-tracking and
offset-tracking view matrices. All ten sloppy and strict Wasm-AOT executions
pass. The entries base case's second hidden transform in
`rewrite_wasm_aot_known_static_for_of` is also deleted, so its destructuring
loop and `Array.from` calls remain byte-for-byte vendored. These three
retirements remove seven semantic selectors from the source. At that
2026-08-23 checkpoint, the token-aware inventory assigned 36 T15 observations
in the 409-entry census for the broader coverage described above.

The synchronous generator heap word now has the closed
`GeneratorState::{SuspendedStart, Executing, Completed, SuspendedYield}` domain.
Its exhaustive private word projection is the sole integer encoding; a strict
one-load decoder traps unknown words and returns a private non-`Copy` token, and
typed store/compare/release helpers own every product access to the private heap
offset. The bounded guard pins all allocation, resume and terminal owners plus
their lifecycle ordering, so raw states, wrong-domain locals and unknown-word
fallthroughs are structurally rejected.

The implementation and guard were independently reviewed and focused-verified
on 2026-08-23. Under the shared eight-core, 22 GB cap, the structure suite
passes `4/4`, four exact CLI lifecycle/suspension/heap fixtures pass `4/4`, and
the six exact generator-state Test262 leaves pass `12/12` Wasm-AOT variants
with every failure bucket at zero under `--jobs 1 --threads 1`. This is a typed
state-word boundary, not a claim that general continuation, generator, iterator
or T15 closure is complete. The source of truth is
`docs/rust-rewrite/contracts/generator-state-word.md`.

The distinct synchronous-generator resume-kind word now has the closed
`GeneratorResumeKind::{Normal, Return, Throw}` domain. Generator allocation
and the suspended-yield prototype dispatcher use one typed store boundary.
Plain `yield` strictly validates one activation snapshot before Return/Throw
routing. Delegation preserves its fresh-path Normal behavior through an opaque
typed transport; only the resumed branch strictly loads, copies and releases
the activation snapshot, and all three post-join comparisons use that
transport. The private offset, stable projection, unknown-word trap, exact
owner census and fresh/resumed source order are pinned by the focused contract
and structure guard. `cargo xc` is green; the new guard passes `7/7`, the
neighboring generator-state guard passes `4/4`, four exact CLI controls pass
`4/4`, and the six exact generator-state Test262 leaves pass all `12/12`
Wasm-AOT variants with every failure bucket at zero. This domain remains
distinct from generator state, resume-state labels and the completion ABI.

Async-generator request settlement now carries the closed
`AsyncGeneratorCompleteStepKind::{Yielded, Completed}` lifecycle state rather
than an unlabeled Boolean. Its sole exhaustive projection lives at the
iterator-result materializer: the one yield-completion owner selects
`Yielded -> done: false`, while the ten terminal body, queue-drain,
awaited-return and already-completed owners select `Completed -> done: true`.
The focused
[complete-step kind contract](../docs/rust-rewrite/contracts/async-generator-complete-step-kind.md)
and owner/lifecycle guard are implemented, independently reviewed and
focused-verified as of 2026-08-23. Under the shared eight-core, 22 GB cap,
`cargo fmt --all -- --check`, `cargo xc` and `git diff --check` are green; the
structural guard passes `4/4`, and the exact
`expression-yield-as-operand.js` Test262 leaf passes `2/2` Wasm-AOT variants
with every failure bucket at zero.

Async-generator activation state now has the exact five-value ECMA-262
`AsyncGeneratorExecutionState` domain. The redundant raw suspended-await word
is removed; promise-owned Await continuations persist `Executing` and retain
their finer Await phase in the distinct body-status field. A private typed
store owns all seventeen allocation, body, yield, Await, drain and completion
writers. The three readers take one strict snapshot, compare it only through
an opaque non-`Copy` token and trap an unknown word before routing. This also
separates the standard builtin's receiver-brand scratch local from its later
state token. The durable owner/lifecycle guard and
`docs/rust-rewrite/contracts/async-generator-execution-state-word.md` are
focused-verified. The three related structure targets pass `15/15` in total,
the exact lifecycle/delegation CLI cohort passes `5/5`, and the five pinned
request-order/state files pass `10/10` Wasm-AOT variants under `--jobs 1
--threads 1` with every non-success bucket at zero.

The backend body protocol now has the separate closed
`AsyncGeneratorBodyStatus::{Idle, Running, Await, Yield, Complete, Throw}`
domain. Its raw offset and stable 0-through-5 projection are private to typed
store and strict-load operations. All fifteen writers are typed; the body
driver and two Promise reaction readers validate one snapshot, compare it only
through an opaque non-`Copy` token and trap unknown words. The body driver also
uses a distinct resume-state local instead of recycling one raw local across
domains. The focused contract and bounded structure guard are focused-verified:
the four related structure targets pass `20/20`, the exact lifecycle/delegation
CLI cohort passes `5/5`, and its five pinned Test262 files pass `10/10`
Wasm-AOT variants with every non-success bucket at zero. This is a backend
body-protocol invariant, not another `[[AsyncGeneratorState]]` value or a
general suspension claim.

Async-generator resumption now uses the distinct closed
`AsyncGeneratorResumeKind::{Normal, Return, Throw, Fulfill, Reject}` activation
domain. Its private stable projection is the sole authority for nine typed
store selections and five strict readers. Unknown words trap before routing,
and each reader consumes an opaque snapshot after its comparisons or validated
transport copy. The delegation resume branch copies its snapshot into the
wider pending-kind transport through a single named bridge, while the fresh
branch initializes that transport from typed Normal. All routing after the
branch join uses pending-kind operations; the transport's close-throw word 5
is not an activation variant. The focused structure target and four neighboring
guards pass `27/27`; the exact lifecycle/delegation CLI cohort passes `5/5`, and
its five pinned Test262 files pass `10/10` Wasm-AOT variants with every
non-success bucket at zero. This does not type resume-state labels or close
general suspension debt.

Classic async-generator `for` loops with one direct body `yield` now give both
their lexical initializer and their direct body lexical declarations
activation-owned storage. Lowering registers the names carried by
`ForInitIr::Lexical` and `ForInitIr::LexicalBlock`, plus the direct
`StatementIr::Lexical` declarations on either side of the suspension, through
the existing `add_suspension_owned_binding` authority before emitting
`StatementIr::GeneratorLoop`. Resumed requests therefore read the loop counter
and body locals from the async-generator activation instead of fresh Wasm
locals.

The existing
`async_generator::wasm_backend_resumes_async_generator_loops_for_zero_one_and_many_iterations`
CLI owner is the exact runtime gate. Its fixture covers zero, one and three
iterations, an abrupt update, a fresh TDZ on each iteration and a body lexical
read after `yield`. At pre-batch commit `60a5e79ff31dac17b16f8ebfd391977b77f34b59`
it reported `0/1`: the three-iteration case yielded only its first value before
the terminal value, and the post-yield lexical check remained false. At current
Test262 content tree `aa55200d1310384c5cf69ea95b2a2ecba457007b`, these three
exact `Array.fromAsync` files likewise reported `0/6`, with every sloppy and
strict execution classified as `Runtime/Bug`:

- `built-ins/Array/fromAsync/asyncitems-asynciterator-exists.js`;
- `built-ins/Array/fromAsync/mapfn-async-iterable-async.js`; and
- `built-ins/Array/fromAsync/mapfn-sync-iterable-async.js`.

The adjacent
`language/expressions/await/async-generator-interleaved.js` control remains
green at `2/2`. At the 2026-08-25 coordinated checkpoint, the new IR regression
passes `1/1`, the bounded structure target passes `4/4`, the exact CLI rerun
passes `1/1`, and the three exact Test262 files pass `6/6`, with every
non-success bucket at zero.

This is direct-`yield`, storage-only classic-`for` closure. Captured
per-iteration environments, break and continue, suspension in the loop head,
multiple or nested suspensions, `while`, `do`, `for-of`, `for-await-of`, general
async-generator continuation and GC layout remain explicit nonclaims. This
batch does not change published conformance counts or complete T15.

The complete synchronous `%DisposableStack%` lifecycle now extends the real
constructor/brand foundation. `DisposableStackState::{Pending, Disposed}` and
`DisposableStackEntryKind::{Use, Adopt, Defer}` are distinct closed heap-word
domains, and the entry kind's exhaustive `dispose_call` projection is the only
authority for the three callback conventions. `use`, `adopt` and `defer`
validate before publishing an entry; `move` consumes a private non-`Copy`,
`#[must_use]` capability transfer into a fresh base-prototype instance; and
`dispose` sets the state before callbacks, walks strict LIFO order, continues
after errors and folds later observations into the specified nested
`SuppressedError` chain. A single function object backs both `dispose` and
`Symbol.dispose`, while the `disposed` accessor observes the same state word.
The fixture pins acquired-method identity, all three call conventions,
disposed-before-callback re-entry, transfer ownership, exact single-error
identity and multi-error suppression order.

At 2026-08-25, the seven `%DisposableStack%` TypeError reasons emitted by the
lifecycle owner are the private closed `DisposableStackTypeError` domain rather
than arbitrary static strings. Each guard selects one named reason and the
shared emitter projects the existing messages exhaustively without a wildcard.
The bounded structure target passes 3/3 and the exact lifecycle CLI witness
passes 1/1. The shared workspace compile and every repository policy gate pass;
the 648-artifact Wasm golden shows no DisposableStack-specific delta. A broader
DisposableStack/Test262 run was not performed for this source-equivalent
invariant.

The catalog, result-shape inference, arity planner, dispatcher, dependency
closure, intrinsic installer and pooled error strings are wired as one batch.
The current pin contains exactly 76 synchronous lifecycle files: 19 `use`, 12
`adopt`, 11 `defer`, 13 `move`, 13 `dispose`, seven `disposed` and one
`Symbol.dispose` witness. Together with the constructor shell this covers 92
of 93 `%DisposableStack%` files; the sole remaining
`proto-from-ctor-realm.js` file constructs dynamic source through another
Realm's `Function` constructor and remains explicit T13 Wasm-AOT policy debt.
The integrated current-SHA checkpoint is green: `cargo xc`, eight lifecycle
structure tests, the AOT/IR focused slices and the consumer fixture all pass.
Pinned Wasm-AOT evidence is 52/52 executions: the complete `use` subtree
(38/38), six exact lifecycle witnesses (12/12) and the staging re-entry witness
(2/2). This is focused evidence, not a claim that the complete 76-file
lifecycle inventory has run. The construction boundary and full lifecycle
contract live in
`docs/rust-rewrite/contracts/disposable-stack-construction-brand.md` and
`docs/rust-rewrite/contracts/disposable-stack-synchronous-lifecycle.md`.

The adjacent non-resumable synchronous `using` source batch now covers direct
children of ordinary Block and function-body statement lists through one
dedicated `StatementIr::SyncDisposableScope`. Its `SyncDisposableResourcesIr`
carrier is statically non-empty, retains declaration order and makes each
resource entry the sole lexical-binding initializer after validation,
`@@dispose` acquisition and registration. Interleaved statements become nested
suffix scopes rather than a generic `TryFinally`. The Wasm backend consumes
private non-`Copy` pending-completion and acquired-resource witnesses, walks the
registered entries in reverse, continues after every disposer throw, folds a
new `SuppressedError` only over an already-Throw completion and restores the
final normal/throw/return/break/continue completion once. The bounded source
test and CLI fixture cover acquisition once, TDZ ordering, nullish skipping,
receiver identity, normal and throwing LIFO, subsequent initializer failure,
single-error identity, Return preservation/replacement and nested suppression
over a body error. `await using`, resumable bodies, loop heads, Switch
CaseBlocks, modules and dynamic source remain explicit non-claims under
`docs/rust-rewrite/contracts/synchronous-using-scope-ir.md`. The integrated
current-SHA checkpoint is green: `cargo xc`, 3/3 focused IR tests, 4/4 source
structure tests and the CLI consumer pass. The exact 18-file non-dynamic
lifecycle cohort is 36/36 under Wasm-AOT. This focused result does not claim the
complete 78-file `language/statements/using` directory or full pinned aggregate.

The selected plain-generator extension is implemented around a required
`SyncDisposableScopeExecutionIr::{Immediate, PlainGenerator}` owner rather than
an optional resumability flag. Analysis exhaustively classifies ordinary,
generator, async-function and async-generator owners; only the generator route
can mint the private `PlainGeneratorSyncDisposableCapabilityIr` through the
suspension-owned binding allocator. The AOT consumer publishes the capability
when execution first reaches the declaration, retains the heap record across
`GeneratorYield`, and consumes a non-`Copy`
`PlainGeneratorSyncDisposeCapabilityStorage` into a detached capability only
when the scope exits. Detachment marks the record disposed and clears its live
entry count before the existing reverse disposal and completion-folding path
runs. The durable CLI fixture covers no acquisition or disposal before start,
no disposal while yielded, normal completion, external `return()` and
`throw()`, acquisition failure, nested capabilities, LIFO, disposer failures,
nested `SuppressedError` order and exactly-once terminal disposal.

The exact current-pin inventory for this batch is one unflagged file and two
sloppy/strict Script executions:

- `language/statements/using/initializer-disposed-at-end-of-generatorbody.js`.

At pre-batch source commit `904da7b355811ad399ff284bf0ddeac47d2cc9c2`,
both executions reported `Runtime/NotImplemented` with the diagnostic `using
declaration in a generator or async function`. The integrated current-SHA
checkpoint is green: the workspace/all-target check and `cargo xc` pass after
correcting one stale exhaustive lowering match, the focused IR invariant is
`1/1`, the bounded structure suite is `6/6`, and the generator CLI fixture is
`1/1` in 55.90 seconds. The passing fixture retains a nested non-yielding scope;
only the unsupported nested-yield shape was removed. The exact Test262 witness
is now `2/2` with zero unsupported, crash or bug results, and the retained ordinary
synchronous-using fixture remains `1/1` in 42.05 seconds. This is not a claim
about async functions or generators, `await using`, resource heads in classic
`for` or `for-of` beyond their separate batches, modules, dynamic source, the
complete 78-file `language/statements/using` directory or the full pinned
aggregate. The lifetime contract lives in
`docs/rust-rewrite/contracts/plain-generator-synchronous-using-scope.md`.

The adjacent plain-async-function synchronous-`using` batch implements
around a third required execution owner,
`SyncDisposableScopeExecutionIr::AsyncFunction`. Only lowering's
suspension-owned allocator can mint its private
`AsyncFunctionSyncDisposableCapabilityIr`; backend crates can inspect the
activation binding name but cannot manufacture the proof from a `String`. The
AOT consumer then exhaustively converts the separate plain-generator and
plain-async-function proofs into `ActivationSyncDisposeOwner`, whose variants
select the exact execution kind, resume-state offset, body compiler and
completion continuation. The async-function variant uses
`HEAP_ASYNC_RESUME_STATE_OFFSET`, publishes its capability only when execution
first reaches the declaration, retains it across `AsyncAwait`, and reaches
`DispatchAsyncFunction` only after detachment, reverse disposal, suppression
folding and completion restoration. It does not push the generic
async-finalizer pending-completion stack around the synchronous disposal walk.

The exact current-pin inventory is one async-flagged file and two Script
executions:

- `language/statements/using/initializer-disposed-at-end-of-asyncfunctionbody.js`.

At pre-batch source commit `1f27bc71f678d5b27e08d2719c660b9777021af4`,
both executions reported `Runtime/NotImplemented` with the diagnostic `using
declaration in an async function or async generator`. The durable consumer
fixture covers no acquisition before call, first-await retention, normal and
explicit-return completion, source throw, rejected-await resumption,
acquisition failure after a prior registration, nested non-await scopes, LIFO,
`SuppressedError` order and exactly-once disposal. The shared
workspace/all-target check and `cargo xc` are green; the focused IR invariant is
`1/1`; the async and retained generator structure executables are `7/7` and
`6/6`; the async CLI lifecycle oracle is `1/1` in 15.21 seconds; and the
retained generator oracle remains `1/1` in 55.21 seconds. The exact async
Test262 witness is now `2/2` with zero unsupported, crash or bug results. Async
generators, `await using`, `await` inside a `using` initializer,
resource-bearing loop heads, modules, dynamic source, every async shape rejected
by the existing linear plan, the complete 78-file `language/statements/using`
directory and the full pinned aggregate remain outside this batch. The source contract is
`docs/rust-rewrite/contracts/plain-async-function-synchronous-using-scope.md`.

The adjacent async-generator synchronous-`using` batch is verified around the
fourth required owner,
`SyncDisposableScopeExecutionIr::AsyncGenerator`. Only lowering's
suspension-owned allocator can mint its private
`AsyncGeneratorSyncDisposableCapabilityIr`, using the fixed
`async.generator.dispose.capability.` prefix. The backend's exhaustive
`ActivationSyncDisposeOwner` projects that proof to
`FunctionExecutionKind::AsyncGenerator`,
`HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET`, the shared async state walkers and
`SyncDisposeCompletionContinuation::DispatchAsyncGenerator`. The activation
binding must be an owned environment slot; temporary-local fallback is absent.
The capability is published only when a request first reaches the declaration,
retained across both `GeneratorYield` and `AsyncAwait`, and detached before the
existing reverse disposal and `SuppressedError` fold. Only after the folded
completion is restored does the async-generator dispatcher return it to the
driver, which settles the current request and drains the queue.

The exact current-pin inventory for this batch is one async-flagged file and
two Script executions:

- `language/statements/using/initializer-disposed-at-end-of-asyncgeneratorbody.js`.

At pre-batch source commit `a5606a73cbbb2a8ffd81c0c2e2dee945bb2b9a4b`,
both executions reported `Runtime/NotImplemented` with the exact diagnostic
`unsupported in lila wasm-aot first slice: using declaration in an async
generator`; the path has no Wasm-AOT rewrite, mask or known-failure entry. The
durable CLI fixture covers no disposal before start, while yielded or while
awaiting; normal completion; external `return()` and `throw()`; awaited
rejection; acquisition failure after a prior
registration; a nested non-suspending scope; LIFO and nested `SuppressedError`
order; exactly-once disposal; queued request order; and a request synchronously
enqueued by a disposer while the generator remains executing. Its observable
trace places both promise reactions after disposal and the queued request's
reaction before the current request's reaction. The shared
workspace/all-target check and `cargo xc` are green; the focused IR invariant
is `1/1`; the async-generator,
retained async-function and retained generator structure executables are
`7/7`, `7/7` and `6/6`; and their CLI lifecycle oracles are `1/1` in 16.81,
13.09 and 53.84 seconds respectively. The exact async-generator Test262 witness
is now `2/2` with zero unsupported, crash or bug results. Central verification
also fixed the dispatcher preflight and suspension scanner to recurse through
the typed async-generator scope. `await using`, async disposers,
resource-initializer suspension, resource loop heads, modules, dynamic source,
nonlinear async-generator forms,
the complete `using` tree and the full pinned aggregate remain outside this
batch. The source contract is
`docs/rust-rewrite/contracts/async-generator-synchronous-using-scope.md`.

The adjacent plain-async-function `await using` batch is implemented around a
separate `StatementIr::AsyncDisposableScope`. Its
`AsyncDisposableResourcesIr` is statically non-empty and cannot be converted
to a synchronous resource list. Only the exhaustive plain-async owner can mint
the private `AsyncFunctionAsyncDisposableCapabilityIr`; its private
`AsyncDisposableFinalizerPlanIr` requires strictly ordered entry, dispose,
resume and exit states and advances the enclosing async state cursor beyond
all four. This makes the capability binding, resource cursor and saved
completion activation-owned across each required disposal Await.

Acquisition evaluates initializers in source order, reads `@@asyncDispose`
before conditionally reading `@@dispose`, validates and registers the selected
method, and only then initializes the immutable binding. The synchronous
fallback is represented separately: its spec wrapper calls `@@dispose`,
discards a normal result even when it is thenable, converts an abrupt call into
a rejected Promise and awaits the wrapper result. Empty resources also await
`undefined`; declarations that execution never reaches allocate and await
nothing. Finalization detaches once, walks entries in reverse, awaits each
before starting the next and folds rejections through `SuppressedError` before
restoring normal, return or throw completion.

The exact selected current-pin inventory is two async-flagged files and four
sloppy/strict Script executions:

- `language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncfunctionbody.js`;
- `language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncfunctionbody.js`.

At pre-batch source commit `7a89e27ec79fe6210fff04a58b6bb3eace535e09`,
both files reported `0/2`; all four executions were
`Runtime/NotImplemented` with the exact diagnostic `unsupported in lila
wasm-aot first slice: await using declaration`. Neither path has a Wasm-AOT
rewrite, mask or known-failure entry. The durable CLI fixture covers direct
async acquisition, synchronous fallback and ignored thenable return, receiver
identity, registration/binding TDZ order, later acquisition failure, evaluated
versus unreachable empty-resource scheduling, sequential reverse awaits,
normal/return/body-throw/disposer-rejection completion, nested scopes, LIFO,
`SuppressedError` order and exactly-once disposal. Central verification is green
for `cargo check --workspace --all-targets`, `cargo xc`, the focused `lila-ir`
`await_using` tests (`2/2`, including capture ownership), and the bounded IR/AOT
source executable (`6/6`). The complete CLI lifecycle fixture is `1/1` in 13.10
seconds, while the retained synchronous-using CLI family filter is `6/6` in
58.29 seconds. The two exact Test262 paths are now `4/4` with zero unsupported,
crash or bug results.

The other 47 positive plain-async statement-list files form an explicit
49-file regression inventory rather than a `49/49` claim. Async generators,
resource loop heads in classic-`for` and `for-of`, modules, dynamic source,
suspension inside an initializer, nonlinear async control flow, the syntax
subtree, the complete `await using` directory and the full pinned aggregate
remain outside this batch. The source contract is
`docs/rust-rewrite/contracts/plain-async-function-await-using-scope.md`.

The adjacent async-generator `await using` path is verified against
`AsyncDisposableScopeExecutionIr::AsyncGenerator`. Its distinct, lowerer-minted
capability is activation-owned across both `GeneratorYield` and `AsyncAwait`,
while the shared asynchronous resource protocol still acquires
`@@asyncDispose` before the `@@dispose` fallback, registers before immutable
binding initialization, awaits entries sequentially in reverse and folds every
rejection through `SuppressedError`.

The durable CLI fixture covers no acquisition before the first request, no
disposal while yielded or awaiting, normal completion, external return and
throw, body-Await rejection, direct async and synchronous-fallback methods,
ignored fallback thenables, later acquisition failure, nested capabilities,
LIFO, error suppression, exactly-once disposal, queued requests and synchronous
reentrancy from an async disposer. The awaited-disposer oracle records the
current-request reaction before the queued reaction, with both after disposal;
this is intentionally distinct from the retained synchronous-disposal order.

At pre-batch source commit `5ad393f3d0`, these exact unmasked files reported
`0/4`:

- `language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncgeneratorbody.js`;
- `language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncgeneratorbody.js`.

All four sloppy/strict Script executions were `Runtime/NotImplemented` with the
diagnostic `unsupported in lila wasm-aot first slice: await using declaration
in an async generator`; neither path has a rewrite, mask or known-failure
entry. Central verification is green for `cargo check --workspace --all-targets`
and `cargo xc`; the focused `lila-ir` `async_generator_await_using` tests pass
`2/2` in 12.34s, including the exact state-collision invariant; the new bounded
structure executable passes `5/5`; and the retained plain-async structure
executable passes `6/6`. The async-generator lifecycle fixture passes `1/1` in
23.63s, while the retained plain-async await-using and synchronous
async-generator using fixtures pass `1/1` in 11.96s and `1/1` in 16.56s. The
two exact paths now each pass `2/2`, for `4/4` total with zero unsupported,
crash or bug outcomes. The runtime-discovered state collision was closed by
reserving three implicit finalizer states per await-using boundary before the
following suspension; AOT also asserts that each resumable statement entry
continues the preceding segment exit. Classic-`for` and `for-of` resource heads,
modules, dynamic source, binding patterns, suspension inside a resource
initializer, nonlinear async-generator forms, the complete `await using`
directory and the full pinned aggregate remain outside this batch. The source
contract is
`docs/rust-rewrite/contracts/async-generator-await-using-scope.md`.

The next adjacent batch gives a plain-async classic `for` initializer the
closed `ForInitIr::AsyncDisposable(AsyncDisposableForInitIr)` capability. The
private carrier pairs a statically nonempty resource list with an
activation-owned async-function finalizer only after test, update and body
lowering have allocated their source states. The containing node remains a
direct `StatementIr::For`, so labels and loop control are not hidden behind a
synthetic block. Every head binding enters TDZ before acquisition; the
capability then spans the test, every body and update, and all terminal or
abrupt completions before the loop lexical environment is restored.

The durable CLI fixture has no explicit Await or Yield expression beyond the
`await using` declaration. It covers async-first method lookup, synchronous
fallback with an ignored thenable return, body-before-disposal, normal exit,
local break/continue, labelled control targeting the resource loop, return,
throw, abrupt test and update,
later-initializer failure after earlier registration, later-binding TDZ, reverse
disposal and nested `SuppressedError` identity, an outer binding plus captured
loop binding, and exactly-once disposal.

At clean pre-batch commit `bca90f2ff9`, these exact raw files reported `0/8`:

- `language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-forstatement.js`;
- `language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-forstatement.js`;
- `language/statements/await-using/initializer-Symbol.asyncDispose-called-if-subsequent-initializer-throws-in-forstatement-head.js`;
- `language/statements/await-using/initializer-Symbol.dispose-called-if-subsequent-initializer-throws-in-forstatement-head.js`.

All eight sloppy/strict Script executions were `Runtime/NotImplemented` with
the exact diagnostic `unsupported in lila wasm-aot first slice: await using
declaration`; none has a Wasm-AOT rewrite or known-failure entry. Central
verification is green for `cargo check --workspace --all-targets`, `cargo xc`,
the focused IR test (`1/1` in 12.11s), and the bounded structure executable
(`5/5`). The new CLI lifecycle fixture passes `1/1` in 22.81s. Retained
plain-async and async-generator await-using fixtures pass `1/1` in 12.00s and
`1/1` in 22.60s, and the retained synchronous classic-for using fixture passes
`1/1` in 30.22s. The four exact paths now each pass `2/2`, for `8/8` total with
zero unsupported, crash or bug outcomes.

Runtime verification caught a regression in the retained plain-async fixture:
generic Labelled state recursion incorrectly scheduled an unreachable
await-using child after an earlier labelled break. The scanner now treats only
a label chain ending directly in an async-disposable For as transparent, which
keeps direct labelled resource loops resumable without admitting labelled
blocks. Async generators, ordinary and generator owners, modules, dynamic
source, binding patterns, `for-of` and `for-await-of`, source suspension in the
initializer, test, update or body, the complete `await using` directory, outer
labelled-block or enclosing-loop control, repeated or nonlinear re-entry of the
same resource-loop node, and the full pinned aggregate remain outside this
batch. The source contract is
`docs/rust-rewrite/contracts/plain-async-function-classic-for-await-using.md`.

The adjacent batch gives a plain async function's synchronous `for-of`
head a distinct asynchronous DisposeCapability. The durable source-free CLI
fixture forces the generic iterator protocol and covers async-first lookup,
the synchronous fallback whose normal return/thenable is ignored, fresh
captured iteration bindings, head TDZ and immutability, body-before-disposal,
sequential disposal before the next iterator step, local continue without
IteratorClose, disposal before break/return/throw/IteratorClose, a later
iteration's acquisition failure after the prior iteration was disposed, nested
implicit-finalizer resume followed by a direct read of the outer head binding,
nested LIFO `SuppressedError` identity and exactly-once disposal.

At clean pre-batch commit `009219b28`, these exact raw files reported `0/10`:

- `language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-each-iteration-of-forofstatement.js`;
- `language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-each-iteration-of-forofstatement.js`;
- `language/statements/for-of/head-await-using-bound-names-fordecl-tdz.js`;
- `language/statements/await-using/syntax/await-using-invalid-assignment-statement-body-for-of.js`;
- `language/statements/await-using/syntax/await-using-valid-for-await-using-of-of.js`.

All ten sloppy/strict Script executions were `Runtime/NotImplemented` with the
exact diagnostic `unsupported in lila wasm-aot first slice: await using
declaration in for-of`; none has an exact Wasm-AOT rewrite or known-failure
entry. Central verification is green for `cargo check --workspace
--all-targets`, `cargo xc`, the focused IR test (`1/1` in 12.17s), and the
bounded structure executable (`5/5`). The new CLI lifecycle fixture passes
`1/1` in both a cached central rerun (`0.23s`) and an uncached focused run
(`14.25s`). The retained async await-using fixtures pass `4/4` in 37.83s, and
the retained synchronous using-for-of fixture passes `1/1` in 48.82s. Each of
the five exact files now passes `2/2`, for `10/10` total with zero unsupported,
crash or bug outcomes.

This is focused evidence only. The Module-only
fresh-binding-per-iteration witness, `for-await-of`, async-generator owners,
binding patterns, dynamic source, the complete `await using` directory and the
full pinned aggregate remain outside this focused claim. The source contract is
`docs/rust-rewrite/contracts/plain-async-function-for-of-await-using.md`.

The next bounded source batch extends that same synchronous disposal lifecycle
to classic `for` initializer heads. The producer uses the closed, statically
non-empty `ForInitIr::SyncDisposable(SyncDisposableResourcesIr)` variant and
keeps the containing node as a direct `StatementIr::For`, preserving labelled
break/continue ownership without a synthetic outer Block. Every head binding
is initialized to TDZ before acquisition; when a captured binding materializes
a for-head environment, it remains current across acquisition, test, body,
update and disposal. Continue retains the capability, while normal exit, break,
return or throw consumes it through the existing reverse completion fold. The
focused CLI oracle covers labelled continue/break, nullish
acquisition, outer/inner binding isolation, later-binding TDZ during the first
resource GetMethod, false-test LIFO, later-initializer failure, suppression
order and immutable-binding update failure.

The exact adjacent current-pin vendored inventory is:

- `language/statements/using/syntax/using-for-statement.js`;
- `language/statements/using/syntax/using-invalid-assignment-next-expression-for.js`;
- `language/statements/using/syntax/using-outer-inner-using-bindings.js`;
- `language/statements/using/initializer-disposed-at-end-of-forstatement.js`;
- `language/statements/using/initializer-disposed-if-subsequent-initializer-throws-in-forstatement-head.js`.

The first three are adjacent grammar or binding evidence. The last two are the
focused disposal-timing and abrupt-initialization lifecycle witnesses. The
current-SHA checkpoint is green: `cargo xc`, 4/4 focused IR tests, 5/5 bounded
structure tests and the end-to-end CLI oracle pass; the five files above report
10/10 sloppy/strict Wasm-AOT executions. This is a focused batch result, not a
claim about the complete 78-file directory or the full pinned aggregate.
`await using`, resumable bodies, modules, `for-in`/`for-of`, Switch CaseBlocks
and dynamic source remain explicit non-claims under
`docs/rust-rewrite/contracts/synchronous-using-classic-for.md`.

The synchronous resource path supports a single BindingIdentifier in a
`for (using x of iterable)` head. Direct synchronous Array and String heads now
use the generic iterator node, which exhaustively owns
`ForOfIteratorHeadIr::{Assignment, SyncDisposable}`. The synchronous head's
private one-name carrier has no
binding mode, initializer, async plan or protocol flag, so it cannot enter a
specialized index walk or be combined with asynchronous iteration. Its
per-iteration capability acquires and initializes one immutable fresh binding,
disposes before the next iterator step, retains a local continue without
closing and disposes every other completion before IteratorClose.

The exact current-pin cohort is three unflagged files and therefore six
sloppy/strict Script executions:

- `language/statements/for-of/head-using-bound-names-fordecl-tdz.js`;
- `language/statements/for-of/head-using-fresh-binding-per-iteration.js`;
- `language/statements/using/syntax/using-invalid-assignment-statement-body-for-of.js`.

At pre-batch commit `681ca415ba1e74c220fa8a5982cba1e7adedc151`, focused
inspection rejected all three through the Wasm-AOT `for-of initializer`
boundary. The integrated current-SHA checkpoint is green: `cargo xc`, 3/3
focused IR tests, 5/5 bounded structure tests and the generic-iterator CLI
lifecycle oracle pass; the three files above report 6/6 sloppy/strict Wasm-AOT
executions with every failure bucket at zero. This remains a focused batch
result, not a claim about the complete using directory or the full pinned
aggregate. Resource heads are BindingIdentifier-only; pattern-looking source
such as `using[resource]` is an
ordinary element-access assignment head, not a resource binding pattern.
`await using`, `for-await-of`, resumable owners, modules, `for-in`, Switch
CaseBlocks and dynamic source remain explicit non-claims under
`docs/rust-rewrite/contracts/synchronous-using-for-of.md`.

The generator-yield IR now distinguishes `yield` from `yield*` with the closed
`YieldForm` domain. Its delegation case carries a one-inhabitant
`GeneratorDelegationProtocol`, which is compile-time tied to all four iterator
obligations and to the sync/async delegation emitter family. The parser-facing
delegation boolean is converted exactly once where `StatementIr` is built;
backend consumers match the closed form exhaustively. This seam is covered by
the green central feature-enabled CLI compile without changing
`generator_delegation.rs`, and the final 620-test CLI inventory includes the
sync suspension-reference regression plus all five async `yield*` wrapper,
initialization and `next`/`return`/`throw` validation fixtures. This is not a
claim that the broader generator or iterator-close acceptance criteria are
complete.

The generator-delegation property reader now accepts the private closed
`GeneratorDelegateProperty` domain instead of an arbitrary `&str`. Its seven
variants project exhaustively to either a well-known Symbol key or an ordinary
String key, so a misspelled call no longer compiles and adding a property
without choosing its exact representation is `E0004`. All fourteen sync and
async delegation reads name that domain directly: one `AsyncIterator`, two each
of `Iterator`, `Next`, `Throw`, `Done` and `Value`, and three `Return` reads.
The existing Symbol-marker and ordinary-string emitter paths, abrupt completion
propagation and observable read order are unchanged. The bounded structure
target passes `3/3`; the five matching async-generator delegation CLI witnesses
pass `5/5`. `cargo xc` is green, and the 647-artifact Wasm golden has an empty
recursive pre/post diff. No conformance improvement is claimed for this typed,
behavior-preserving boundary.

The same property boundary now derives no cloning, copying, debugging,
equality or default capability. Its owned reader borrows the sole seven-row key
projection and exhaustively consumes the two key representations. A
Rust-lexical recursive guard pins the 24/11 identifier censuses, exact ordered
fourteen-call forwarding and both complete reader bodies; see
[`generator-delegate-property-capability.md`](../docs/rust-rewrite/contracts/generator-delegate-property-capability.md).
The property and neighboring async-delegation structure targets pass `3/3` and
`4/4`, and the five retained CLI witnesses pass `5/5`. This is source-equivalent
capability hardening, not new generator behavior or T15 closure; no Test262 or
semantic-golden run was performed for this follow-up. Independent dry review is
clean, and the following shared workspace compile, formatter, module-boundary,
task-plan and diff gates all pass.

Async-generator delegation now carries the closed
`AsyncGeneratorDelegationKind::{YieldStar, ForAwaitYield}` policy from its two
control-flow producers through eight exhaustive emitter matches. Those matches
own pending Throw and Return forwarding, delegate `throw` lookup, close
eligibility and completion, close arguments and the ordinary `next` argument;
there is no equality test or implicit default for a future kind to inherit.
All eight independent projections now borrow the same non-derived policy; the
domain has no cloning, copying, equality or default capability. The bounded
structure target passes `4/4`, and the five exact async-generator delegation
CLI witnesses pass `5/5`. The six exact `yield*` and `for await` Test262 leaves
pass all `12/12` sloppy/strict Wasm-AOT executions with every failure bucket at
zero. Independent review found and closed a guard gap in the first three
projection bodies; final re-review, the shared workspace compile and every
repository gate are green. The semantic golden remains deferred, and no
behavior or conformance change is claimed for this source-equivalent boundary.

The `for await` iterator-method acquisition boundary now accepts the private
closed `ForAwaitIteratorSymbol::{AsyncIterator, Iterator}` domain instead of an
arbitrary `&str`. Its exhaustive canonical-name projection is the only producer
of the two Symbol pool names, so a misspelled call no longer compiles and adding
a protocol symbol without naming it is `E0004`. The two consumers still read
`@@asyncIterator` first and read `@@iterator` only after the existing nullish
gate; the dynamic Symbol-tagged property path and abrupt propagation are
unchanged. The bounded structure target passes `3/3`. The exact direct
`@@asyncIterator` and array-fallback `@@iterator` Test262 witnesses each pass
both sloppy/strict variants, for `4/4` total. A separate assignment-head witness
remained red `0/2` at that selector-only checkpoint because it observed
`undefined` where Test262 requires `7`; the later assignment-head batch recorded
below owns that independent defect. `cargo xc` is green, and the 647-artifact
Wasm golden has an empty recursive pre/post diff. No behavior or conformance
change is claimed for this typed boundary.

Call-argument spread now has the same compile-enforced boundary:
`ExprIr::SpreadArgument` carries a `SpreadArgumentIr`, whose required
one-inhabitant `SpreadArgumentProtocol` is tied through the iterator-operation
catalog to `emit_call_args_vector`. The witness credits only the operations the
emitter performs (`GetIterator`, `IteratorStep`, `IteratorValue`) and records
the no-`IteratorClose` path as an implementation fact. Adding a new spread IR
construction without that protocol is therefore a build error; the backend
does not branch on the witness, so evaluation order and emitted control flow
are unchanged. The central feature-enabled CLI compile and the exact
`run_wasm_backend_uses_iterators_for_call_argument_spread` contract are green
in the complete 620-test CLI inventory.

Array-literal spread now uses the direct general iterator accumulator, deleting
the unprovable shortcut rather than encoding an unreachable `ProvenDense`
variant. A spread-bearing literal lowers to `ExprIr::ArrayAccumulation`; each
spread carries the one-inhabitant `ArraySpreadProtocol`, tied at compile time to
the emitter that performs `GetIterator`, `IteratorStep` and `IteratorValue`.
Uninterrupted no-spread literals retain `ExprIr::ArrayLiteral` and their static
shape. A staged generator literal that crosses a nested yield instead uses
suspension-owned ArrayAccumulation even without a spread, because its partial
array and logical index must survive the suspension.

`ArrayAccumulationTargetIr` distinguishes an uninterrupted `Fresh` expression
from `SuspensionOwned(ArrayAccumulatorSlots)`. The latter contains distinct
array and `ArrayAccumulatorU64NextIndexSlot` types, is initialized before the
first element, and flushes every evaluated prefix before a nested generator
suspension. The compiler-private index carrier stores exact raw `u64` state;
it is never recovered through an ECMAScript Number, and the emitter rejects a
contribution at `u64::MAX` rather than wrapping. This is an explicit backend
bound, not a claim to implement the spec's unbounded mathematical counter. The
logical index is separate from array `length`: direct fresh
array writes cover indexes through `4294967294`, index `4294967295` and later
values become ordinary named data properties without growing `length`, and an
elision at or beyond that boundary throws `RangeError`. Every spread observes
`@@iterator`; there is no dense fast path and, matching ArrayAccumulation, no
`IteratorClose` claim. The focused IR evidence now requires the evaluated
prefix and resumed suffix to reuse identical typed array and raw-index slots.
The focused CLI fixture observes prefix evaluation before suspension, iterator
acquisition and every `next` before the suffix, then checks the final
consecutive indexes. The direct pinned cohort is the exact
`language/statements/generators/yield-spread-arr-{single,multiple}.js` pair.
Prior-pin `aa55200d1310384c5cf69ea95b2a2ecba457007b` snapshots reported `1/1`
per leaf on 2026-07-20; they predate this implementation and are historical
baselines only. At the current pin each leaf has only `flags: [generated]` and
materializes through the normal harness as two ordinary sloppy/strict
executions. The 2026-08-24 central checkpoint is green: the batch `cargo check`
and `cargo xc` gates pass, the focused exact lila-ir test passes `1/1`, the
exact CLI suspension fixture passes `1/1`, and the two generated leaves pass
all four materialized executions `4/4` with every failure bucket at zero. This
is bounded suspension-ownership closure, not broader generator, iterator,
IteratorClose or full Test262 closure.

Array destructuring now carries the closed
`ArrayDestructuringEvaluationIr::{BindingInitialization, AssignmentEvaluation}`
domain instead of an `assignment: bool`. All five lowering contexts name the
abstract operation they implement. The result emitter, direct lexical
initializer, result-tag planner, lexical counter, hoisted-variable collector
and product-name collector bind the field and match it exhaustively, so adding
an operation without stating its result and declaration ownership is `E0004`.
The per-pattern `ArrayPatternProtocol` remains orthogonal and unchanged.
Assignment evaluation still returns the original RHS payload and tag; binding
initialization still returns `undefined`. The module-boundary guard now closes
the exact five-producer/six-consumer product inventory and rejects inactive CLI
registrations. On 2026-08-24, `cargo check -p lila-aot-wasm` and `cargo xc`
passed, the six exact `lila-ir` scenarios passed `6/6`, and the exact array
destructuring CLI fixture passed `1/1`. No emitted-Wasm byte comparison or
focused Test262 cohort was run; `ArrayPatternProtocol`, IteratorClose, object
destructuring and broader generator/resource behavior remain open.

The shared array-destructuring iterator step now consumes the private,
non-derived `DestructuringIteratorStepKind::{Elision, Value}` domain. The
elision producer performs `IteratorStep` without reading `value`; the target
and rest producers select the value-reading arm. One exhaustive match owns the
complete ordered `IteratorValue` emission, so a future step kind cannot inherit
elision behavior through a false `matches!` result. The bounded recursive guard
pins the exact three-producer/one-consumer inventory and the existing abrupt
fixture observes that an elided result's `value` getter is not called. This is
a source-equivalent type closure expected to leave emitted Wasm byte-identical;
the structure target passes `4/4`, and the exact ordinary and abrupt CLI
witnesses pass `2/2`. Independent review confirmed the exact declaration,
producer inventory, value-read branch and preserved instruction order.
Coordinated `cargo xc`, formatter, diff and repository policy checks are green.
It does not extend destructuring, `IteratorClose` or broader T15 behavior.

The shared `%IteratorHelperPrototype%` dispatcher now carries the private closed
`IteratorHelperPrototypeOperation::{Next, Return}` domain instead of an
`is_return: bool`. The two shared-prototype builtin arms are its only producers,
and every concrete helper-brand target choice matches it exhaustively. Adding
an operation without a target is therefore `E0004`; passing the former boolean
is `E0308`. One CLI fixture borrows the shared `next` and `return` methods and
exercises all seven brands through eight creation surfaces: concat, zip,
zipKeyed, map, filter, flatMap, take and drop. On 2026-08-24, the executable
structure target passed `4/4`; that matrix and the existing prototype/drop CLI
witnesses each passed `1/1`. Four unrewritten current-pin Test262 leaves passed
all `8/8` ordinary Wasm-AOT executions with every failure bucket at zero. The
representation change preserves the existing target builtins and instruction
selection, but emitted bytes were not compared. This is invariant hardening
and bounded dispatch evidence, not broader helper semantics, close precedence
or IteratorClose closure.

The flatMap-specific abrupt outer-close boundary now uses the private,
exhaustive `IteratorFlatMapInnerState::{NotInstalled, Active}` domain instead
of a raw `clear_inner_active` Boolean. The sole shared helper retains exactly
eight calls in `IteratorFlatMapNext`: four `Active` calls for abrupt inner
`next`, result-object validation, `done` access and `value` access, and four
`NotInstalled` calls for abrupt or invalid inner-iterator acquisition before
the unique installation sequence. Its existing observable order remains outer
IteratorClose with the original throw preserved, `Done`, the state-selected
inner-active clear, then `Executing`. The contract and swap-resistant lifecycle
guard are independently reviewed. Under the shared eight-core cap,
`cargo fmt --all -- --check`, `git diff --check`, and `cargo xc` are green; the
`iterator_flat_map_inner_close_state_structure` executable passes `3/3`, the
exact
`iterator::run_wasm_backend_succeeds_for_iterator_prototype_flat_map_fixture`
CLI lifecycle witness passes `1/1`, and the exact
`staging/sm/Iterator/prototype/flatMap/close-iterator-when-inner-next-throws.js`
and
`staging/sm/Iterator/prototype/flatMap/throw-when-inner-not-iterable.js`
Test262 leaves pass `4/4` Wasm-AOT variants in total with every failure bucket
at zero under `--jobs 1 --threads 1`. This focused result verifies the typed
lifecycle boundary only; it does not generalize to other helper families, close
an inner iterator on these paths, refresh a broad Iterator cohort, claim a
conformance gain, or complete T15. Batch AA further makes the state must-use and
capability-free. The sole consumer now receives an owned decision that cannot
be cloned, copied, formatted, defaulted, compared, ordered or hashed. At the
Batch AA checkpoint, the strengthened structure target passes `3/3`,
`cargo xc` is green, the exact CLI witness passes `1/1`, and the two pinned
leaves pass all `4/4` Wasm-AOT variants with every failure bucket at zero. No
runtime behavior change is claimed.

Batch AC makes
`SyncDisposeCompletionContinuation::{Dispatch, DispatchAsyncFunction,
DispatchAsyncGenerator, DeferToIteratorClose}` a must-use, capability-free
one-way dispatch authority. Activation-backed generator, async-function and
async-generator owners select their exhaustive continuation; the two ordinary
scope/loop exits select direct dispatch, and synchronous `using`-for-of selects
deferred IteratorClose. Every producer moves the choice into the shared
disposal walk. Its sole ownership-consuming continuation match runs after
completion restoration and states all four terminal actions without a wildcard.
This changes no emitted Wasm or runtime behavior. The
[focused contract](../docs/rust-rewrite/contracts/sync-dispose-completion-continuation.md)
and three-test recursive guard pin the boundary. At the shared Batch AC
checkpoint, `cargo xc` is green, that guard passes `3/3`, the neighboring
synchronous-using-for-of target passes `5/5`, and the exact synchronous-scope,
plain-generator, plain-async-function, async-generator and using-for-of CLI
lifecycle witnesses pass `5/5`. No Test262 cohort or semantic golden was run
for this source-equivalent capability closure.

Batch AD replaces the destructuring backend's independently supplied prepared
target plus raw IR target pair with one private, must-use, capability-free
six-variant `PreparedDestructuringTarget`. Preparation exhaustively mirrors
binding, identifier Reference, property, private, nested-array and
nested-object targets; the write consumes that value without a parallel IR
discriminant or mismatch `unreachable!`. Property receivers and computed keys
remain evaluated before source/default observation, while borrowed IR facts
remove the former target/key clones. The focused contract and recursive guard
are in
[`prepared-destructuring-target.md`](../docs/rust-rewrite/contracts/prepared-destructuring-target.md).
`cargo xc` passes. The prepared-target and neighboring iterator-step structure
guards pass `8/8`; the array-iterator, rest-setter-after-completion and
private-reference-order CLI witnesses pass `3/3`. No Test262 cohort or semantic
golden was run because this invariant claims no new destructuring,
IteratorClose or conformance behavior.

The prepared-source work later added `EnvironmentIdentifier`, making the target
seven-variant, but left `AssignmentIdentifier` carrying the IR Reference, so the
write re-matched its disposition and rejected the already-prepared environment
case with `unreachable!`. `AssignmentIdentifier` now carries the private,
must-use `PreparedIdentifierWrite` domain, which cannot spell the environment
case; the parallel IR discriminant is gone again.

Batch AE made the then-two-variant synchronous iterator error authority
capability-free. Iterator acquisition and stepping owned their selection, and
every internal protocol check, iterator-completion helper, and exhaustive
projection borrowed that same authority instead of relying on implicit copies.
The current five-consumer replacement is recorded in
[`sync-iterator-consumer-capability.md`](../docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md).
At the historical Batch AE checkpoint, `cargo xc` passed. The authority,
neighboring `Math.sumPrecise`, and protocol-error structure targets passed
`14/14`; the Array spread and `Math.sumPrecise` CLI witnesses passed `2/2`. No
selector-specific Test262 cohort or semantic golden was run because that
source-equivalent invariant claimed no new iterator or conformance behavior.

Batch AF makes the private 18-field `DestructuringIteratorLocals` reservation
bundle capability-free. Its eleven-field synchronous iterator projection now
borrows the bundle, as do all pattern-element and iterator-step consumers, while
the enclosing array-destructuring compiler remains the sole reverse-order
release owner. The recursive guard and contract are in
[`destructuring-iterator-locals-ownership.md`](../docs/rust-rewrite/contracts/destructuring-iterator-locals-ownership.md).
`cargo xc` passes. The ownership and neighboring `Math.sumPrecise` and
synchronous-protocol structure targets pass `14/14`; the exact
array-destructuring iterator and abrupt-close CLI witnesses pass `2/2`. No
Test262 cohort or semantic golden was run because this source-equivalent
invariant claims no new destructuring, iterator or conformance behavior.

Batch AG separates synchronous iterator protocol access from temp-local release
authority. `SyncIteratorLocals` is now a capability-free borrowed view, while
only the non-`Copy`, must-use `ReservedSyncIteratorLocals` returned by the
reservation operation can be consumed by reverse release. Array spread and
`Math.sumPrecise` borrow that owner for acquisition and stepping before moving
it once into release; array destructuring can borrow its eleven-field protocol
projection but cannot pass that projection to the release API. The recursive
guard and contract are in
[`sync-iterator-locals-release-ownership.md`](../docs/rust-rewrite/contracts/sync-iterator-locals-release-ownership.md).
`cargo xc` passes. This guard and the destructuring-owner,
iterator-consumer and `Math.sumPrecise` structure targets pass `18/18`;
the exact array-accumulation suspension and `Math.sumPrecise` CLI witnesses
pass `2/2`. The two generated `yield-spread-arr-{single,multiple}.js` leaves
pass all `4/4` sloppy/strict Wasm-AOT executions with every failure bucket at
zero. No semantic golden was run because this source-equivalent invariant
claims no new iterator, Array, Math or conformance behavior.

The `%Iterator%` constructor now selects its primitive
`NewTarget.prototype` fallback through the closed
`OrdinaryDefaultPrototype::Iterator` domain and the required resolved-Realm
policy. The observable prototype Get precedes function-Realm resolution; abrupt
and revoked Proxy routes remain explicit. Allocation consumes the prototype
payload and representation tag together, preserving exact Object, Function and
Array custom-prototype identity. Bound and nested Proxy new targets traverse to
the defining Realm only after the observable Get. The realm slot and both
publication paths already existed, so this seam changes no heap layout. A
strengthened structural guard and one CLI fixture pin the typed policy, original
new-target ownership, tagged allocation, exact entry/created-Realm publication,
six primitive cross-Realm fallbacks, bound/Proxy traversal, one-read/abrupt
ordering and revoked-Proxy behavior. This is not a claim that generator state
machines, IteratorClose, helper closing or the whole Iterator tree are complete.
The seam is independently source-audited and its fixture passes `node --check`.
Its first runtime gate exposed and repaired the supported empty Function's
defining-Realm lifecycle, which is a prerequisite for truthful
`GetFunctionRealm` results. On 2026-08-24, the exact structural and CLI tests
passed `1/1` each and the single-file pinned Test262 gate passed `2/2` Wasm-AOT
variants with every failure bucket at zero. Dynamic Function source parsing and
complete Iterator-tree closure remain nonclaims.

The `%Iterator%` active-function rejection now uses the private closed
`ActiveStandardBuiltinFunction::IteratorConstructor` identity instead of
comparing `NewTarget` directly with the entry-realm constructor global. The
typed emitter selects the self-backed created-realm function from the builtin
environment when present and otherwise selects the exhaustively mapped entry
global. The closed domain is shared with RegExp, but the Iterator projection is
bounded to one variant, one mapping and one direct constructor-arm call. This
preserves exact object identity: constructing a realm's Iterator with itself
throws before prototype lookup, while using either realm's distinct Iterator or
a Proxy/bound wrapper around the active Iterator as `NewTarget` remains a valid
subclass-style construction. The shared construct dispatcher classifies
Iterator as direct-returning, so its body performs the sole prototype Get and
allocation after active-function rejection instead of inheriting generic
preconstruction.

The strengthened structural guard pins the Function-tag/payload-equality
conjunction, both identity producers, exact direct-returning membership and
dispatch/Get/allocation source order. The CLI fixture covers the raw two-Realm
identity matrix, one observable prototype Get in both distinct cross-Realm
Proxy directions, and same-Realm Proxy and bound wrappers around the entry and
created active Iterators. Each same-Realm wrapper must remain distinct and
record exactly `prototype,return`; bound getters returning `undefined` also pin
fallback through their target's function Realm. The product source required no
change. On 2026-08-24, the structural guard and CLI fixture passed `1/1` each,
while the direct pinned leaf passed both Wasm-AOT variants (`2/2`) with every
failure bucket at zero. `cargo check -p lila-aot-wasm`, `cargo xc`, fixture
`node --check` and `git diff --check` are also green. The pinned leaf covers
only entry-realm undefined/self rejection, so this remains direct
specification/source closure rather than a measured baseline gain. It does not
change RegExp behavior, generalize active
identity to every builtin, or complete generator suspension, IteratorClose,
helper closing, resource management or broader T15 semantics.

The `for await (identifier of iterable)` head now remains an assignment target
instead of collapsing with `var identifier` into one synthetic loop-owned
binding. The private closed `ForOfBareIdentifierHead` domain routes the bare
form through a fresh iterator-result slot and a per-iteration prefix using the
ordinary checked identifier Reference write, including `with` selection,
immutable-binding `TypeError`, and strict/sloppy global-write policy. `var`,
`let`, and `const` declarations retain their existing declaration storage and
per-iteration behavior. Capture analysis records the bare assignment head even
when its function never reads the target. Focused IR assertions distinguish
that write-only capture, mutable assignment, immutable failure, and lexical
declaration storage; a bounded structure guard pins the closed domain, separate
AST arms, fresh slot, checked write path, and capture-analysis registration.
The CLI fixture covers outer mutation to `7`, immutable rejection without
mutation, lexical shadowing, and `var` declaration behavior. The exact pinned
leaf `language/statements/for-await-of/head-lhs-async.js` passes both Wasm-AOT
variants (`2/2`) with every failure bucket at zero. On 2026-08-27, the bounded
structure target passed `4/4`, focused IR tests passed `3/3`, and the CLI fixture
passed `1/1`; fixture syntax and repository policy gates are also green. The
shared 682-dump semantic golden passes `2/2` in 685.75 seconds, adds this witness
plus the independent created-Realm WeakRef fixture, removes none and leaves all
680 retained dumps byte-identical. No broad T15 filter was run. This is not a
claim that other T15 iterator, generator, helper, or resource-management work
is complete.

Array and TypedArray iteration now share one closed, macro-backed
`ArrayIteratorKind::{Key, Value, KeyAndValue}` authority instead of raw numeric
constants and `u64` constructor parameters. Producers select named kinds,
storage alone serializes the stable word, and both `next` paths decode through
the complete row table before an exhaustive three-arm semantic match. The
ordinary named-property carrier retains its incompatible-receiver `TypeError`
for corrupt words; the private TypedArray record traps on an impossible word.
Focused verification for this source-equivalent invariant batch is recorded in
`docs/rust-rewrite/contracts/array-iterator-kind-wire-domain.md`; it does not
complete the remaining T15 surface. The following shared 683-dump semantic
golden passes `2/2` in 655.10 seconds, adding and removing none. Its sole
retained structural change is the intentionally expanded Array corruption
witness; the other 682 retained summaries differ only in accounting fields.

The shared sync iterator protocol-error authority was first closed as a
private, capability-free four-row domain consumed once by the exhaustive
diagnostic/Realm projection. At that direct-`for-of` checkpoint, three
consumer classes produced twelve mapping rows, fifteen producer checks, and 29
`SyncIteratorProtocolError` mentions. `compile_for_of_iterator` and
`compile_async_disposable_for_of_iterator` each own five inline checks, while
`compile_async_function_for_of_iterator` delegates its five checks to the
shared acquisition and stepping emitters. All three select
`SyncIteratorConsumer::ForOf`; the async-disposable owner also boxes primitive
lookup through the current function Realm. Diagnostics, throw propagation, and
order remain pinned by the focused structure guard. The authority is recorded in
`docs/rust-rewrite/contracts/sync-iterator-protocol-error-authority.md`;
before this full-boundary expansion, the ownership and neighboring
`Math.sumPrecise` structure targets passed `4/4` and `6/6`, while the exact
destructuring-abrupt and `Math.sumPrecise` Wasm-AOT CLI fixtures each passed
`1/1`. At the expanded boundary, the focused and affected structure targets
pass `37/37`, the exact error fixture plus four success-path CLI controls pass
`5/5`, and four direct `for-of` leaves pass all `8/8` Wasm-AOT executions with
every failure and non-success bucket at zero. The result is recorded in
`docs/rust-rewrite/contracts/direct-synchronous-for-of-protocol-error-realm.md`.
No complete Test262 directory or broad T15 filter was run.

The ordinary direct synchronous `for-of` owner now treats callable Proxies as
callable for both `@@iterator` and cached `next`. Its two stale Function-tag
gates and Function-only calls use the same general `IsCallable` and
function-or-Proxy `Call` operations already used by the direct
async-disposable and resumable shared owners. Receivers, empty argument lists,
and the existing post-call propagation order remain unchanged. The bounded
Rust guard pins the two general checks and calls, forbids Function-only routing
in this owner, and checks each receiver and result-validation sequence. The
entry-Realm CLI fixture covers successful and throwing apply traps,
non-callable Proxy diagnostics, revoked callable Proxies, once-only `next`
lookup, and no close on abrupt stepping. It also retains 13 initialized
captured bindings and requires both primitive and non-callable Proxy iterator
methods to use the entry `%TypeError.prototype%`; this prevents lexical slot
12 from being interpreted as the function-layout Realm prototype field. The
fixture does not establish the
Realm of a cross-Realm Proxy-internal error, and the rewritten Proxy apply
null-handler Realm case remains T11 work. The existing protocol-error census
and 46-row operation catalog do not change; see the callable-Proxy follow-up in
`docs/rust-rewrite/contracts/direct-synchronous-for-of-protocol-error-realm.md`.
The affected all-target compile and formatting check pass; five structure
targets pass `23/23`; five exact CLI controls pass `5/5`; and eight unchanged
direct iterator/Proxy Test262 leaves pass all `16/16` executions with every
failure bucket zero. The module, task-plan, shortcut, and diff guards are
green, and the shortcut inventory remains 240.

The shared `emit_iterator_close` owner now creates both of its algorithm
TypeErrors in the current function Realm. All 68 external entry routes share
that rule: 16 direct, 49 preserving-current-Throw, and 3
preserving-saved-Throw. The preserving wrappers still restore the incoming
Throw after close, and a zero `current_env_local` still selects the main Realm
fallback for entry code. At this close-only checkpoint, ordinary direct
`for-of` acquisition and stepping errors remained separate work. The later
15-check boundary now routes all three direct synchronous `for-of` owners
through `SyncIteratorConsumer::ForOf`; its focused and affected structure targets pass
`37/37`, its CLI cohort passes `5/5`, and its four pinned leaves pass `8/8`.
`cargo check -p lila-aot-wasm` passes; the focused structure target passes
`4/4`; the exact
created-Realm CLI test passes `1/1`; and the affected `iterator_close` CLI sweep
passes `6/6`. The two direct `for-of` Test262 leaves pass all `4/4` Wasm-AOT
executions with every failure and non-success bucket at zero. No semantic
golden, published-status refresh, complete Test262 leaf, or broad workspace
suite was run. The boundary and exact commands are recorded in
[`iterator-close-error-realm.md`](../docs/rust-rewrite/contracts/iterator-close-error-realm.md).
The later protocol-error boundary is recorded in
[`direct-synchronous-for-of-protocol-error-realm.md`](../docs/rust-rewrite/contracts/direct-synchronous-for-of-protocol-error-realm.md).

Generator delegation now has the separate private, capability-free
`GeneratorDelegateProtocolError` authority for all eight direct `yield*`
protocol failures. Its eighteen sync/async producers select a named failure;
the shared callability and object-result checks no longer accept arbitrary
messages, and the sole exhaustive projection owns every diagnostic and raw
runtime-error emission in the module. Adding a failure without a diagnostic is
`E0004`, while passing a raw or misspelled message is `E0308`. The focused
contract and structure guard live in
`docs/rust-rewrite/contracts/generator-delegate-protocol-error-authority.md`.
The standalone structure executable passes `4/4`, `rustfmt --check` passes for
both changed Rust files, and the four-file invariant diff passes
`git diff --check`. No Cargo compile, CLI fixture, Test262 cohort or semantic
golden was run for this source-equivalent authority closure; it changes no
generator behavior or broader T15 status.

The parked `%AsyncDisposableStack%` disposal walk now threads the private,
capability-free `AsyncDisposableStackDisposeCompletionKind::{Normal, Throw}`
domain instead of an unlabeled `has_error` Boolean. Its sole exhaustive
projection owns the stable 0/1 heap encoding. Initialization and error folding
use one typed store; suppression and terminal settlement use one strict load
that traps unknown words before returning a non-`Copy`, `#[must_use]` local for
an explicit `Throw` or `Normal` comparison. The completion-kind offset has no
other product access. The focused structure executable passes `4/4`,
`rustfmt --check` passes for both changed Rust files, and the four-file diff
passes `git diff --check`. No Cargo compile, CLI fixture, Test262 cohort or
semantic golden was run for this source-equivalent lifecycle closure. It
preserves Await timing, single-error identity and suppression order and does
not complete `%AsyncDisposableStack%` or T15. The source contract is
[`async-disposable-stack-dispose-completion-kind.md`](../docs/rust-rewrite/contracts/async-disposable-stack-dispose-completion-kind.md).

Direct synchronous Array `for-of` no longer has an index-walk IR or backend
emitter. `StatementIr::ForOfArray`, `compile_for_of_array`, and the synchronous
`ARRAY_INDEX_WALK` witness are deleted. Exact Arrays now lower through
`StatementIr::ForOfIterator` with an ordinary assignment head, no async plan,
and `SYNC_ITERATOR_PROTOCOL`. The yielded value is `Dynamic`, because a
replaceable `@@iterator` can yield a value unrelated to the source Array's
inferred element shape. The focused runtime witness covers length growth, an
inherited indexed getter, and a prototype `@@iterator` that yields a String and
receives one `return` call after `break`. At the direct-path checkpoint,
`cargo check -p lila-aot-wasm` passed; the two focused structure targets passed
`3/3` and `4/4`, the IR `for_of` target passed `16/16`, the planner and two CLI
targets each passed `1/1`, and four pinned Array length-mutation leaves passed
`8/8` Wasm-AOT executions with every failure bucket at zero. No
semantic-golden result or published-status refresh is claimed.

The plain-async synchronous `for-of` body-`await` path now has a separate
`StatementIr::AsyncFunctionForOfIterator` whose closed
`AsyncFunctionForOfIteratorPlanIr` owns the synchronous Iterator Record, the
single body suspension, the ordered entry/resume/exit states, and the head and
per-iteration environment lifecycles. Lowering records its yielded value as
`Dynamic` and no longer classifies the source as Array. The backend acquires
`@@iterator` and `next` once on entry, stores the Iterator Record in the async
activation, and reloads it after each body await. It closes after abrupt body
completion with ECMAScript completion precedence, but not after a `next`,
`done`, or `value` error.

This deletes `AsyncForOfArrayWalkForm`,
`lower_async_for_of_array_with_body_await`,
`ARRAY_INDEX_WALK_RESUMABLE`, and the old index synthesis. Focused Array and
String protocol fixtures, close/error fixtures, the six-capture environment
oracle, and a bounded structure target cover the replacement. `cargo check -p
lila-aot-wasm` passes. The five focused structure targets pass `19/19`, the
`lila-ir` `for_of` target passes `18/18`, and the four exact CLI oracles pass
`4/4`. All four fixtures pass `node --check`, and the two pinned
`Array.fromAsync` leaves pass `4/4` Wasm-AOT executions with every failure and
non-success bucket at zero. The complete 95-file `Array.fromAsync` leaf,
semantic golden, and published-status refresh were not run.
The admitted form remains a plain async function with one direct body `await`
and a simple single-name declaration or bare identifier assignment head.
Direct `break`/`continue`, pattern and property heads, a captured head TDZ,
iterable suspension, async generators, and `for await` remain separate paths or
explicit rejections. See
[`synchronous-array-for-of-iterator-protocol.md`](../docs/rust-rewrite/contracts/synchronous-array-for-of-iterator-protocol.md).

The same activation-backed plan now admits static, computed, and private
member-reference heads. The yielded value enters `$forof.access`; the existing
property-write prefix then re-evaluates the Reference once per entered
iteration, inside IteratorClose and before the body `await`. Resume skips that
prefix. Capture analysis now scans the member base and computed key, including
when those are outer bindings used nowhere else. The focused
`wasm_plain_async_sync_for_of_member_heads.js` fixture covers changing public
targets and keys, setter failure, valid and wrong-brand private writes,
IteratorClose counts, and Throw precedence. Declaration and assignment
patterns, resource heads, `super`, suspending member operands, direct
`break`/`continue`, nonlinear body suspension, captured head TDZ, suspending
iterables, async generators, and `for await` remain nonclaims. Focused
verification passes: `cargo fmt --all -- --check` and `cargo check -p lila-ir
-p lila-aot-wasm -p lila-cli --all-targets`; `21/21` IR `for_of` tests plus
the `1/1` rejection matrix; `25/25` focused and affected structure tests; and
`2/2` exact member-head and retained capture CLI tests. The fixture passes
`node --check`. No matching pinned Test262 cohort, semantic golden, or
published-status refresh is claimed. See
[`plain-async-synchronous-for-of-member-heads.md`](../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-member-heads.md).

The plan now also admits assignment patterns and `var` binding patterns.
Their existing Array/Object destructuring prefixes execute once in
`before_await`, inside IteratorClose and before suspension. `var` BoundNames
remain activation-owned; assignment References are prepared and consumed
before the await. Capture analysis now exhaustively scans object assignment
patterns and object/array nesting in both directions, including computed
source keys, defaults, rest targets, and public/private target operands. The
source-free `wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js` oracle
covers Array and Object `var` forms, computed assignment order, once-only
effects, inner and outer close counts, and Throw precedence. The relevant
all-target compile and formatting check pass; the IR `for_of` filter and
explicit rejection matrix pass `24/24` and `1/1`; six focused and affected
structure targets pass `25/25`; and the new plus three retained CLI oracles
pass `4/4`. The fixture passes `node --check` and its Node semantic baseline.
The later lexical-pattern checkpoint below supersedes this historical
`let`/`const` rejection with a complete fresh per-iteration environment and
TDZ model. No matching pinned Test262 cohort is claimed. See
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

The plan now also admits `let` and `const` array and object binding patterns.
The closed head input carries the lexical mode, compiler-only IteratorValue
entry local, exact iteration-storage and TDZ-placeholder name sets, and the
BindingInitialization prefix. Its constructor derives the public three-case
storage lifetime and rejects incomplete name sets, duplicate names or slots,
invalid layouts, wrong modes, assignment targets, and empty-pattern
environment claims. Capture analysis materializes every BoundName before it
computes slots and hops, while lowering predeclares all final iteration
storage before defaults or computed keys. The backend publishes one complete
fresh Environment Record before initialization and leaves it before outer
IteratorClose.

The source-free
`wasm_plain_async_sync_for_of_lexical_pattern_heads.js` oracle covers nested
patterns, defaults, computed object keys, array and object rest, before- and
after-await closures, uncaptured reads, mutable `let`, forward and captured-head
TDZ, `const` writes, empty patterns, and inner plus outer close Throw
precedence. The relevant all-target
compile and formatting check pass; the IR `for_of` filter and rejection
witness pass `27/27` and `1/1`; six focused and affected structure targets
pass `28/28`; and the new plus four retained CLI oracles pass `5/5`. The
fixture passes `node --check` and its Node semantic baseline. The pinned
Test262 checkout has no exact lexical-pattern/direct-await leaf, so no
Test262 count is claimed. See
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-lexical-pattern-heads.md).

The synchronous iterator path now uses the private, non-`Copy`
`SyncIteratorConsumer::{ArrayDestructuring, ArrayAccumulation, ForOf,
MathSumPrecise, ListFormat, AggregateError}` domain. The four protocol errors form
one exhaustive 24-row diagnostic projection. The confirmed source census is
17 typed projector calls and 43 error identifiers: the declaration, typed
projector parameter, 17 producers, and 24 mapping rows. Each semantic owner constructs one consumer,
and the structure guard pins the same borrow through acquisition and stepping.

Consumer selection now controls wording only. Primitive acquisition boxes
through the current function Realm. Algorithm-created synchronous protocol
TypeErrors exhaustively project the builder body source: a standard builtin
may use its trusted self-backed current Realm, while main, user, host, and
runtime-helper bodies use the main Realm. A nonzero lexical environment is
never interpreted as Realm metadata. Array
destructuring threads its named consumer through its custom step owner, which
keeps the typed pre-call `next` check, post-call object-result check, `done`
read, and conditional `value` read in order. ArrayAccumulation uses a distinct
consumer and four exact `array spread` diagnostics. Its abrupt `next`, `done`,
and `value` paths propagate without IteratorClose, matching the 2026
[`ArrayAccumulation`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-arrayaccumulation)
operation.

ListFormat is the fifth semantic consumer. Its private StringListFromIterable
owner borrows the shared acquisition and step checks, propagates acquisition
and step abrupts without close, and adds one preserving-current-throw close
only for a non-string element. The 2026-10-01 source-only structure follow-up
pins the exact four new diagnostic pairs, same-consumer borrows, private error
authority, and the current 73 external close routes. Focused execution of
these corrected witnesses is pending.

The destructuring and ArrayAccumulation fixtures run the syntax-owning function
in the entry Realm. They pin diagnostics, completion identity, no-close
behavior, and primitive String prototype lookup, but they cannot distinguish
current-function from main-Realm error identity. Wasm AOT does not dynamically
compile the created-Realm user function needed for that witness, so no
cross-Realm runtime result is claimed. This checkpoint also does not claim the
current function Realm's `%Array.prototype%` for a fresh Array literal or
Array-rest result.

At the historical four-consumer checkpoint, the all-target compile and formatting
check passed. Nine focused and affected
structure targets pass `42/42`; seven exact Wasm-AOT CLI witnesses pass `7/7`;
and five pinned Array-spread plus four Array-destructuring leaves pass all
`18/18` sloppy/strict executions with every failure and non-success bucket at
zero. The new fixture passes `node --check`, and the module boundary guard is
green. No semantic golden, published-status refresh, complete Test262 prefix,
or broad workspace suite was run. See the
[`sync-iterator-consumer-capability.md`](../docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md)
contract and §26 of the combined iterator evidence contract.

Generator and async-generator calls now select the instance `[[Prototype]]`
the way EvaluateGeneratorBody and EvaluateAsyncGeneratorBody do: after
FunctionDeclarationInstantiation completes normally, one observable
`Get(functionObject, "prototype")` runs, and a non-Object result selects
`%GeneratorPrototype%` or `%AsyncGeneratorPrototype%` from the generator
function's own Realm. Previously the call path read the function header's
creation-time prototype snapshot, which assignments did not update, and fell
back to the entry-realm global. The private `GeneratorInstanceFamily` domain
projects exhaustively onto two new `OrdinaryDefaultPrototype` variants and
reuses the required resolved-Realm fallback. The instance is allocated with a
null placeholder, and the prototype payload and tag are stored together, so a
callable `prototype` keeps its identity. The
`generator_instance_prototype_structure` guard pins the owner, the ordering
after parameter initialization and the absence of header or global reads. The
`aot_generator_instance_prototype` engine target covers replaced, null and
primitive prototypes, class and literal methods, initializer-time replacement,
abrupt initialization and cross-Realm fallback for both families.

## Objective

Implement resumable generator execution, the complete iterator protocols, iterator helpers and explicit resource management through reusable state-machine and iterator-operation layers.

## Generator state machines

Lower generator bodies into explicit states with stored environments, operand values, completion records and finally/handler state. Cover:

- generator function/method object creation and prototypes;
- `next`, `return`, `throw`, suspended-start/yield/completed/executing states;
- `yield` and `yield*` delegation, including missing methods and iterator closing;
- re-entrancy errors;
- `try/catch/finally`, return and throw across suspension;
- captured variables, `this`, `arguments`, `super` and private environment;
- async generators by composing the state machine with T14 jobs/promises.

Do not implement generators by interpreting a stored AST at runtime.

## Iterator operations

Complete and centralize:

- `GetIterator`/`GetIteratorFromMethod` for sync and async hints;
- iterator records, `IteratorNext`, `IteratorComplete`, `IteratorValue`, `IteratorStep`;
- `IteratorClose` and `AsyncIteratorClose` with correct completion precedence;
- `%IteratorPrototype%`, `%AsyncIteratorPrototype%` and identity methods;
- array, string, typed-array, Map/Set and custom iterator interoperability.

All consumers (`for-of`, spread, destructuring, Promise combinators, constructors and builtins) must use these operations.

## Iterator helpers

Implement the pinned Iterator/AsyncIterator helper APIs, including lazy helper objects, `map`, `filter`, `take`, `drop`, `flatMap`, `reduce`, `toArray`, `forEach`, `some`, `every`, `find`, helper `return`, close behavior, limits/coercions and branding.

## Explicit resource management

Implement current standardized syntax/builtins present in the pin:

- `using` and `await using` declaration lowering;
- `Symbol.dispose`/`Symbol.asyncDispose`;
- `DisposableStack`, `AsyncDisposableStack` and `SuppressedError` integration;
- LIFO disposal, move/adopt/defer/use, abrupt-completion chaining and async disposal jobs.

Coordinate syntax/early errors with T07 and error objects with T24.

## Acceptance criteria

- Generator protocol/state/re-entrancy tests pass.
- `yield*` handles sync and async delegates, return/throw absence and close precedence.
- All iterator consumers close iterators on the exact required abrupt paths.
- Iterator helpers are lazy and pass mutation/close/branding tests.
- Explicit resource management preserves suppression order and disposal timing.
- Generator/iterator objects remain valid across GC cycles.
- Pinned generator, iterator-helper and resource-management filters reach zero failures.

## Required tests

```sh
cargo test -p lila-ir generator_ --quiet
cargo test -p lila-aot-wasm iterator_ --quiet
cargo test -p lila-cli wasm_iterator --quiet
./target/debug/lila test262 run built-ins/Iterator --execution-backend wasm --timeout-ms 120000 --threads 4
```

Also run language generator/`yield`, `for-of`, spread/destructuring, AsyncIterator, DisposableStack, AsyncDisposableStack and explicit-resource-management filters.


### 2026-10-02 source-only switch/disposal composition proposal

The separate plain-async switch proposal retains nested synchronous `using`
blocks through the existing activation-backed disposal owner, including a
switch break after an ordinary await and cleanup through finalizers. Direct
CaseClause/DefaultClause `using` and `await using` declarations retain the front
end's syntax rejection; wrapping them in a block changes that grammar boundary.
Valid block-form `await using` and case `for await` remain typed Unsupported
because this switch batch does not own their implicit suspension protocols.
The synchronous/async-for-of/generator-for-of body validators explicitly refuse
the new plain-async switch owner where continuation composition is absent.

All new Rust/Engine controls are uncompiled and unexecuted. General generator
switch suspension, async disposal and iterator-close acceptance remain open;
this source proposal claims no T15 filter or runtime pass. The admitted previous
checkpoint and every dated history above remain unchanged.

### 2026-10-03 shared suspended selector protocol follow-up

The plain-async switch owner now includes checked selector prefixes and a
distinct no-match/default decision state. It reuses the existing activation
dispatcher, retained bindings and CaseBlock environment rather than adding a
separate iterator, disposal or generator state machine. Existing synchronous
disposal and pending-completion routing in admitted bodies are preserved.
Selector rejection and labelled breaks through nested awaited finally have
new authored regression sources, but none has been compiled or executed.

This is a consumed T14/T15 continuation seam, not generator-switch or
asynchronous-resource completion. Case `await using`/`for await`, a suspended
switch within an enclosing loop and sync/async generator switch suspension
still need their own composed owners; the existing validators continue to
reject them. General T15 acceptance and runtime verification remain open.

## Native GC iterator helper source successor — 2026-10-05

The atomic semantic-GC source draft physically replaces all thirty iterator
helper dispatch arms and their obsolete manual helper methods with seven
concrete helper owners. The closed operation/brand domains, cached whole
Iterator Records, explicit whole completions, captured Realm invocation and
consuming Concat/Zip entry publication are on the actual compiler path.
Seventeen finite paired Engine cohorts are authored for lazy and terminal
methods, concat, zip/zipKeyed, close/error order, reentrancy and borrowed Realms.
The three manual-layout spelling guards retire; existing behavioral controls
remain, and the current contracts describe the GC successor.

This is source progress in the isolated atomic draft, not task acceptance or
new conformance evidence. MAIN and the generated README status are unchanged.
Compilation, fixture parsing, emitted Wasm validation and runtime controls are
unexecuted. The full remaining task source pass comes before verification;
later verification requires the confirmed aggregate 4096 MiB cgroup cap,
zero swap and one worker. See the [native helper contract](../docs/rust-rewrite/contracts/gc-iterator-helper-execution.md).
