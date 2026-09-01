# T14 — Promise jobs, async functions and async iteration

**Status:** In progress — Promise/job machinery is substantial; suspended async closure remains

**Parallel group:** Feature lane  
**Depends on:** T03, T04, T05, T06, T09  
**Blocks:** Async modules in T12, async iterators in T15, Atomics.waitAsync in T17

## Current repository state

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
