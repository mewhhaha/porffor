# Plain generator synchronous for-of continuations

Status: the earlier continuation batch passed its focused Wasm-AOT controls
and all 16 exact pinned executions. The 2026-10-04 local-control source batch
passed its ref77 combined all-target Rust type check; emitted Wasm and runtime
verification remain pending. Broad verification and full T15 acceptance remain
open.

## Eager lexical-pattern source batch — 2026-10-04

The current source also admits eager Let/Const array/object binding heads in
plain synchronous generators. The shared lexical-pattern proof owns the actual
TDZ names, source mode and complete fresh iteration cells. Mandatory generator
completion consumes its exact semantic initializer prefix together with the
checked body. The iterator-value sink remains private EntryLocal storage and
cannot survive suspension; head mode is supplied by the real environment owner.
Object initialization uses one semantic Get/default operation. The existing
iteration entry, close, local-control and yielding-finalizer lifecycle remains
the consumer.

Three new paired Engine cohorts retain all eighteen preceding cohorts. Actual
constructor/lowering controls and maintained architecture guards are authored.
This successor has no compilation, emitted-Wasm or runtime evidence. Eager
lexical patterns are distinct from still-refused Var/assignment patterns,
suspended heads/iterables, async generators and resource heads. The separate
[lexical-pattern contract](plain-generator-for-of-lexical-pattern-heads.md)
records the initializer/cell/sink proof and exact finite controls; older runtime
evidence below retains its recorded source scope.

## Ordinary-property assignment-head source batch — 2026-10-04

The complete successor admits eager `Simple` property heads, such as
`for (locate()[key()] of source) { yield value; }`, in plain synchronous
generators. The source producer consumes the existing ordinary-property
Reference plan and plain-assignment completion. Its fused PutValue retains raw
base/receiver, deferred computed-key conversion and source strictness; it does
not capture a property Get, emit a raw write or reconstruct a canonical key.
Expression assignment uses the same completion after its existing RHS effect
analysis. Setter dependency and alias/effect invalidation remain shared.

`GeneratorForOfAssignmentIr::identifier` and `::ordinary_property` authenticate
their own actual prefixes before a single consuming head/body plan can publish
EntryLocal storage. Identifier immutable-name justification remains separate.
The ordinary prefix must carry the full Dynamic/all-tag iterator sink as RHS,
and neither Reference operand may read, write or retain that sink. Shared
checks reject spellable, persistent, captured or body-escaping storage and
mismatched prefix/body pairing. No declaration binding mode or head environment
is manufactured for assignment.

The original PutValue executes once at the iteration entry state, before the
body lexical scope and inside the existing live close/finalizer frame. Resume
does not evaluate base/key/Set again or read the temporary sink. Head failures
close with the original Throw; iterator acquisition/step failures remain
outside this close owner. Local and injected completions still wait for the
selected completion of yielding finalizers. Public storage projections, backend,
async-head environment rules and checked `B + 1` state arithmetic are unchanged.

The existing constructor and actual lowering targets gain domain/operand/lifetime
controls and source witnesses for effectful and primitive/nullish bases,
shadowing, setters, strictness and yielding cleanup. Three new paired Engine
fixtures retain all fifteen earlier cohorts and the exact observation helper:

| Authored fixture | Expected witness |
| --- | --- |
| `property-head-order-and-resume.js` | Raw base/key/ToPropertyKey/Set order, no Get, original Proxy receiver, cached next across replacement, fresh captured body cells, shadowing and interleaved owners; no head replay on resume or terminal done. |
| `property-head-errors-close.js` | Original foreign base/key/coercion/Set throws and native nullish/strict Set errors close once before yielding outer cleanup; sloppy false Set enters the body; iterator step errors do not evaluate or close the head. |
| `property-head-receivers-and-finalizers.js` | Callable Proxy inherited setter receives raw primitive String; no inherited Get; Continue/Break and explicit/injected Return/Throw survive two yielding finalizers, retaining close precedence and once-only head evaluation. |

All code, controls, guards and documentation are authored and passed the ref105
combined all-target Rust type check. Emitted-Wasm/runtime acceptance remains
pending. The ref100 proof in the preceding identifier section retains its scope.
Suspended head/iterable, private and
super heads, nonlexical patterns, resources, async generators and foreign control owners
retain explicit boundaries. Full T15 and fresh conformance remain open.

## Identifier assignment-head source batch — 2026-10-04

The successor source batch admits a bare identifier assignment head in a plain
synchronous generator with an eager iterable. A private consumed head proof
selects either the existing validated declaration binding or the prepared
identifier PutValue. Assignment keeps the original target cell and carries no
head TDZ environment, fresh iteration binding or source BindingMode.

The prepared prefix retains the existing identifier-resolution owner, including
mutable writes, strict unresolved references, const/TDZ errors and the justified
sloppy immutable-name no-op. Its source-unspellable iterator-value sink has
entry-only Dynamic storage. Checked construction rejects a sink used by the
remaining body or retained operands; the producer also checks capture and owned
environment metadata. The existing exhaustive IR traversal performs the storage
query, including an indirect call's receiver. No source-shaped replacement
assignment or duplicate interpreter owner is introduced.

The iterator plan consumes the checked head and complete generator body.
Borrowed Activation, IterationEnvironment and EntryLocal projections make each
storage case explicit. Planning reserves the EntryLocal pair without hoisting
or registering a suspension-owned binding. The shared backend retains the async
head's existing environment rules; generator assignment has no declaration mode.

IteratorValue is stored and the eager prefix runs only at the iteration's entry
state, before the first Yield and inside the live close/finalizer frame. Resume
neither repeats PutValue nor reads the nonpersistent sink. A head write failure
uses the existing throw-preserving IteratorClose; local control, injected
Return/Throw and yielding finalizers retain their checked completion owners.
Acquisition and stepping failures retain their earlier no-close behavior.

Meaningful constructor and source-lowering controls extend their existing
owners. Paired fixtures extend the existing generator continuation Engine
target. Production, controls and documentation are authored as a complete
batch. The ref100 combined all-target Rust type check passed, including the
constructor, lowering and Engine test targets. Emitted Wasm, executable controls
and full task acceptance remain pending.

## Boundary

The `GeneratorForOfIterator` statement owns an eager synchronous
Iterator Record and a complete generator iteration body. The first batch
admits ordinary `var`, `let`, and `const` identifier heads, direct and delegated
yields, materialized body/catch lexical environments, and structured
try/catch/finally clauses. User statements still pass through parsing,
lowering, and Wasm emission.

The binding proof validates the head's physical storage and environment
layout. A separate `GeneratorForOfBodyIr` validates generator continuation
states and rejects an async continuation owner. The iterator plan consumes
those validated inputs and exposes immutable accessors.

The 2026-10-03 dry source follow-up requires the source identifier at the shared
async/generator binding constructor. For lexical heads it compares the sole TDZ
placeholder with the existing `TdzPlaceholderName` minted from that identifier,
then performs the existing environment and iteration-storage checks. Both
actual lowering routes derive source name, storage alias and environment from
the same `ForOfLoop`; review found no current mismatched caller. A future wrong
placeholder cannot be admitted into the validated plan, and a caller omitting
the source-name input no longer has a constructor it can call. Generated storage
names are not parsed to recover source identity. When recorded, this follow-up
was uncompiled and unexecuted. Its all-target Rust type check passed in the
ref74 composition; the dated runtime evidence below belongs to its earlier
source.

For entry state `E` and validated body exit `B`, the plan reserves checked loop
exit `B + 1`. Source admission and lowering must reserve the same state span,
including structured clause boundaries. A body suspension resumes within
`E..=B`; it does not acquire or step the iterator again. Normal body completion
leaves the iteration environment, resets state to `E`, and steps. Exhaustion
or a completed abrupt loop exit selects `B + 1`, so following statements run
once. The Iterator Record's iterator, cached next method, and done slots live
in the invocation activation. A captured lexical head gets a fresh iteration
environment; nested body and catch records are restored beneath that owner.

The Iterator Record's compiler-owned bindings extend only physical activation
storage. Eval's named environment is derived independently from analyzed source
bindings before lowering, so iterator, next, and done names never enter source
name resolution. Ordinary head bindings keep their source-visible cells.

The generator's saved lexical environment is distinct from its base invocation
environment. Yield and yield-star save the active lexical chain. The existing
pending-completion stack carries Return/Throw through a finalizer that yields
again. The finalizer's eventual completion reaches the loop's close region;
suspending the finalizer itself does not close the iterator.

The [ForIn/OfBodyEvaluation algorithm](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-runtime-semantics-forinofbodyevaluation)
orders cached iterator stepping, done/value extraction, head initialization,
and body evaluation. [GeneratorResumeAbrupt](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-generatorresumeabrupt)
supplies injected Return/Throw to the suspended body. [IteratorClose](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-iteratorclose)
preserves an existing Throw ahead of close errors; close errors replace a
pending Return. A finalizer may replace either pending completion first.

## Local control source batch — 2026-10-04

The plain synchronous generator route admits unlabelled `break` and `continue`
owned by its current identifier-headed for-of. The generator-specific source
visitor and mandatory body constructor both retain that ownership through
blocks, lexical blocks, eager If arms, and existing GeneratorTry clauses. An
actual child loop, switch, label, or parameter-initialization tree has a foreign
branch owner; a branch there cannot borrow the enclosing iterator's targets.
Labelled branches are rejected even when their spelling names the current loop.
The generic loop-control visitor used by other owners remains unchanged.
The T14 plain-async follow-up relocates this proof to one neutral shared
CurrentLoop/NestedStatement domain consumed by both source gates and mandatory
body constructors; async preflight runs before its eager-body shortcut.

The checked body retains every statement and validates the same exact Yield
and try/catch/finally states. Local branches reserve no state. The iterator plan
still consumes this body, the validated eager head, and its activation-owned
Iterator Record; it exposes no unchecked branch admission API. Continuation
entry `E`, body exit `B`, and checked loop exit `B + 1` keep their earlier roles.

The real generator emitter publishes the current break target and a body-end
continue target. A current-loop Continue passes through existing finally
owners, becomes Normal after its target is consumed, retires the active
iteration environment, resets state to `E`, and steps the cached next method.
It does not read or call the iterator's return method. Break reaches close only
after the yielding finalizer has selected its eventual completion, then exits
to `B + 1`. A finalizer may replace either local branch with Break, Continue,
Return, or Throw. An injected Return/Throw while cleanup is suspended replaces
the older pending local completion. The existing pending-completion stack and
control-target auxiliary slot carry these choices; no activation-frame layout,
ABI, or opcode is added. The T14 follow-up also gives plain async loops this
shared target lifecycle after mandatory ownership preflight.

IteratorClose runs for the selected exiting completion. A close getter/call
failure or non-object result replaces Break/Return; an existing Throw retains
its original identity. A close failure after local Break belongs to the
surrounding catch/finally. A body or finalizer Yield alone does not close, and
iterator acquisition, cached-next lookup, stepping, done/value extraction, and
ordinary exhaustion retain their earlier no-close behavior.

The existing `generator_for_of_iterator` constructor tests and the
`generator_branch_lexicals` lowering target now contain source controls for
accepted current-loop ownership, foreign/labelled rejection, exact clause and
following-statement states, and preserved captured-head environments. Three
new fixture sources extend `aot_generator_for_of_continuations` without changing
its earlier nine controls or its WasmAot/Normal262/sole-`ok` observation helper.
Their strict and sloppy executions are authored, not passing runtime evidence:

| New fixture source | Expected witness |
| --- | --- |
| `local-control-cached-next-cells.js` | Continue before/after Yield does not close; Break waits through two finalizer Yields; cached next, fresh captured head/body cells, interleaved owners, and completed-generator close count remain correct. |
| `finalizer-replaces-local-control.js` | Finalizer Break/Continue/Return/Throw and injected Return/foreign Throw select stepping, close, and completion precedence after local control was pending. |
| `local-break-close-route.js` | Getter/call/non-callable/non-object close failures reach the surrounding yielding catch/finally; missing or successful close reaches following code once. |

The earlier dated runtime and pinned cohort below describe their earlier source.
They do not establish acceptance of this follow-up.

## Runtime evidence

The new engine target is `aot_generator_for_of_continuations`. Its nine
standalone fixtures use the existing Test262 host surface, require WasmAot,
assert every observable result in JavaScript, print `ok`, and complete with
the Number `262`. The Rust target checks that exact structured completion and
output in separate sloppy and strict Script observations: nine tests and
18 product executions passed on 2026-09-30, including explicit return-yield and
return-yield-star cleanup controls. No fixture is added to the CLI semantic golden
corpus.

| Fixture | Expected witness |
| --- | --- |
| `protocol-cached-next-multiple-yields.js` | Pre-start Return performs no acquisition, step, or close; acquire once after next, cache next across replacement, step only between iterations, skip value on exhaustion, retain two body suspension segments. |
| `captured-iteration-body-catch-interleaved.js` | Distinct let-head/body/catch cells persist across two yields, later iterations, completion, and interleaved generator owners. |
| `const-head-captured-cell.js` | Const captures keep their iteration identity across suspension and after advancement. |
| `yield-star-try-finally-segments.js` | Each delegated sequence finishes before its yielding finalizer and the next iteration. |
| `injected-return-finally-reyield-close.js` | Injected Return, explicit return-yield, and return-yield-star all preserve the original return value through yielding cleanup, call close once with no arguments, and skip the next step. |
| `injected-throw-caught-and-escaping.js` | A catch may yield the exact thrown object without closing; escaping Throw(undefined) retains its identity over a close getter error. |
| `iterator-close-completion-precedence.js` | Getter/call/protocol failures replace Return and preserve Throw; missing/successful close retains the original completion. |
| `finally-replaces-pending-completion.js` | Finalizer Throw/Return replaces the prior completion before close precedence is selected. |
| `iterator-operation-errors-do-not-close.js` | Acquisition, cached-next lookup, next call, done getter, and value getter failures stop immediately and do not enter body close. |

The exact pinned baseline/replay cohort contains these eight files under
`language/statements/for-of/`:

- `yield.js`
- `yield-star.js`
- `yield-from-try.js`
- `yield-from-catch.js`
- `yield-from-finally.js`
- `yield-star-from-try.js`
- `yield-star-from-catch.js`
- `yield-star-from-finally.js`

Their unrestricted flags select 16 sloppy/strict executions. The fresh
2026-09-29 baseline records 16 Runtime NotImplemented outcomes with the reason
`generator suspension`, zero passes, and zero Bug/Crash outcomes. The retained
binary SHA-256 is
`d1249c1bc084b26ed3be3734cef2cf5d8773efb1f90feedb923c222247aedc08`.
The fresh post-implementation replay passes all 16 executions with every
non-success bucket at zero. Its compiler SHA-256 is
`4104fdc0e535105b0820354b70299a2d1c6170599cd5f5af4930249bff908515`.

## Verification and deferrals

Run the generator-for-of constructor/state tests, `generator_branch_lexicals`,
the focused engine target, retained async for-of continuation and generator
catch/delegation regressions, then the eight exact pinned files.
Root owns Cargo and the final shared verification checkpoint.

Suspension in the iterable or head, Var/assignment pattern heads, nested
resumable loops, labelled/nonlocal control and branches under foreign statement
owners, async-generator ownership, and resource loop heads remain explicit
unsupported families for this batch. Plain async for-of local control has the
separate dry T14 follow-up; other resumable loop owners retain their boundaries.
No forced-GC execution is claimed: the current Wasm-AOT `gc()` host emitter
rejects executable collection. This batch does not complete T15 or publish a
new conformance status.
