# T15 — Generators, iterators, iterator helpers and resource management

**Status:** In progress — iterator helpers and generator records exist; general suspension remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T08, T09; async portions also depend on T14  
**Blocks:** Iterator-dependent closure across arrays, collections, promises and modules

## Current repository state

Generator activation/delegation records, iterator operations, several iterator
helpers, async-iterator support and resource-management builtins now exist.
Some generator and helper behavior is still handled through focused
materialization, and the README records unsupported suspended/control-flow
families. General sync/async generator state machines, iterator-close coverage
across all consumers and complete resource-management filters remain open.

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
`for (using x of iterable)` head. Array and String
index-walk nodes accept only `ForOfAssignmentIr`; the generic iterator node
instead exhaustively owns `ForOfIteratorHeadIr::{Assignment,
SyncDisposable}`. The synchronous head's private one-name carrier has no
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
Plain no-spread literals retain `ExprIr::ArrayLiteral` and their static shape.

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
`IteratorClose` claim. The implementation is dry-written in this batch;
Cargo, focused runtime and pinned Test262 gates remain pending for the central
verifier.

Array destructuring now carries the closed
`ArrayDestructuringEvaluationIr::{BindingInitialization, AssignmentEvaluation}`
domain instead of an `assignment: bool`. All five lowering contexts name the
abstract operation they implement. The result emitter, direct lexical
initializer, result-tag planner, lexical counter, hoisted-variable collector
and product-name collector bind the field and match it exhaustively, so adding
an operation without stating its result and declaration ownership is `E0004`.
The per-pattern `ArrayPatternProtocol` remains orthogonal and unchanged.
Assignment evaluation still returns the original RHS payload and tag; binding
initialization still returns `undefined`. This seam is dry-written and
statically checked in this batch; Cargo and focused runtime gates remain
deferred to the central verifier.

The shared `%IteratorHelperPrototype%` dispatcher now carries the private closed
`IteratorHelperPrototypeOperation::{Next, Return}` domain instead of an
`is_return: bool`. The two shared-prototype builtin arms are its only producers,
and every concrete helper-family target choice matches it exhaustively. Adding
an operation without a target is therefore `E0004`; passing the former boolean
is `E0308`. A single CLI fixture borrows the shared `next` and `return` methods
and exercises both on concat, zip, map, filter, flatMap, take and drop helpers.
The representation change preserves the existing target builtins and emitted
instruction order. This is invariant hardening, not a claim that broader helper
semantics or IteratorClose coverage are complete. The seam is dry-written and
statically checked; Cargo, runtime and pinned Test262 gates remain deferred to
the central verifier.

The `%Iterator%` constructor now selects its primitive
`NewTarget.prototype` fallback through the closed
`OrdinaryDefaultPrototype::Iterator` domain and the required resolved-Realm
policy. The observable prototype Get precedes function-Realm resolution; abrupt
and revoked Proxy routes remain explicit. Allocation consumes the prototype
payload and representation tag together, preserving exact Object, Function and
Array custom-prototype identity. The realm slot and both publication paths
already existed, so this seam changes no heap layout. A structural guard and one
CLI fixture pin the typed policy, tagged allocation, six primitive cross-Realm
fallbacks, one-read/abrupt ordering and revoked-Proxy behavior. This is not a
claim that generator state machines, IteratorClose, helper closing or the whole
Iterator tree are complete. The seam is dry-written and statically checked;
Cargo, runtime and pinned Test262 gates remain deferred to the central verifier.

The `%Iterator%` active-function rejection now uses the private closed
`ActiveStandardBuiltinFunction::IteratorConstructor` identity instead of
comparing `NewTarget` directly with the entry-realm constructor global. The
typed emitter selects the self-backed created-realm function from the builtin
environment when present and otherwise selects the exhaustively mapped entry
global. This preserves exact object identity: constructing a realm's Iterator
with itself throws before prototype lookup, while using either realm's distinct
Iterator as the other realm's `NewTarget` remains a valid subclass-style
construction. The shared construct dispatcher now classifies Iterator as
direct-returning in `crates/lila-aot-wasm/src/functions.rs`, so its body performs
the sole prototype Get and allocation after active-function rejection instead
of inheriting generic preconstruction.
Structural guards pin both identity producers, exact direct-returning membership
and dispatch/Get/allocation source order. One CLI fixture covers the raw
two-realm identity matrix and one observable prototype Get in both distinct
directions. This is direct specification/source closure rather than a measured
pinned Test262 failure. It does not generalize active-function identity to every
builtin or complete broader T15 semantics. The seam is dry-written and
statically checked; Cargo, runtime and pinned Test262 gates remain deferred to
the central verifier.

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
