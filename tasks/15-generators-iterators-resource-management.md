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
