# Plain async switch continuation ownership

## 2026-10-04 dry nested-block asynchronous disposal

This complete follow-up passed the ref93 combined all-target Rust type check
and remains unexecuted. It supersedes the
nested-block `await using` refusal in the earlier historical sections; their
verification receipts retain their original source. It does not add disposal
to CaseBlock. Current [switch early errors](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-switch-statement-static-semantics-early-errors)
retain SyntaxError for direct resource declarations in CaseClause/DefaultClause;
a supported nested Block owns its own resource stack.

A switch-specific mandatory source proof admits only the existing plain-async
block resource grammar: identifier bindings and eager initializers. It runs
before any case state is allocated and treats nested functions as separate
activations. Yield, for-await, async resource loop heads, suspended resource
initializers and resource suspension under an unrelated loop remain refused.
Ordinary all/implicit-suspension source policies retain their prior meaning.

The private case-span constructor accepts only an actual AsyncFunction
capability. It validates the actual suffix starting at finalizer entry, requires
suffix exit plus one to equal disposal entry, then dispose+1=resume and
resume+1=exit, with checked arithmetic throughout. The whole capability span
belongs to that case. Nullish resources still have implicit awaits, so an
implicit-only case cannot fall back to the eager switch representation.

Existing block lowering registers nonempty resources under a private pending
owner, lowers the entire suffix, then allocates the three finalizer states and
consumes the owner. No second state planner is added. The existing switch
emitter keeps its break frame live while compiling the actual case sequence.
The resource finalizer saves and restores pending completion, including its
payload and target, across disposal awaits. Block cells remain attached through
resumption. Matching break reaches the switch epilogue after finalization;
normal disposal permits fallthrough; finalizer replacement retains the existing
Break/Return/Throw and SuppressedError rules. No backend, frame, ABI or opcode
change is needed.

IR controls reject wrong execution owners, suffix/successor gaps and incorrect
case ordering, and inspect the actual lowered implicit-only owner. Three finite
strict/sloppy Engine cohorts require WasmAot, Normal Number262 and one expected
print. They cover selection/default/skipping/fallthrough/shared cells; method
acquisition/async-first fallback/nested LIFO and pending completion replacement;
and original foreign throws, rejection, SuppressedError and called Realm.
These sources passed the combined Rust type checkpoint; emitted-Wasm and
grouped focused/pinned/broad checkpoints remain pending. Resource loop heads, suspended initializers, for-await, enclosing
loops and sync/async generator switches remain separate unsupported domains.

## 2026-10-03 dry suspended-selector follow-up

This follow-up is uncompiled and unexecuted. It supersedes the earlier
awaited-selector refusal for stageable awaits;
the candidate verification and proposal sections below retain their exact
historical scope. They do not verify these new selector phases.

Discriminant evaluation and GetValue precede the shared CaseBlock lexical
instantiation. Case selectors then run in source order until the first strict
equality match; a default in the middle is chosen only after the other tests
fail. Selected bodies fall through without testing later selectors. These
ordering rules follow [Switch evaluation and CaseBlockEvaluation](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-switch-statement-runtime-semantics-evaluation).

Lowering saves the once-evaluated discriminant in an existing activation-owned
binding outside CaseBlock. Every non-default selector of a switch containing
a selector await gets an `AsyncFunctionSwitchSelectorContinuationIr`. Its
fields are private; construction validates the actual statement prefix from
entry through ready state and derives the checked successor decision state.
The closed eager/resumable selector enum is consumed exhaustively. The switch
constructor rejects mixed ownership, selector gaps, an unretained discriminant
expression and body ranges inconsistent with the selector sequence. A distinct
fallback state lies after all selectors and before every body entry.

The emitter uses the existing async statement-sequence dispatcher for each
prefix. A failed test commits the next selector; a match commits that case's
body entry. Fallback commits default or switch exit. Prefixes and earlier
tests do not rerun after suspension, and fallthrough does not execute later
selectors. Wasm locals remain ephemeral: the saved discriminant is reloaded
only after a selector prefix reaches its decision state. CaseBlock is created
before selection and reattached on every resume, retaining shared lexical
cells, closures and unvisited TDZ bindings. Existing labels, break epilogues,
finally and rejection routing remain the body/completion owners.

Frame lexical/temp planning, global and function discovery, data collection,
throw inference and IR/source traversals consume the actual selector prefixes.
The retained discriminant and intermediate operands use the existing owned
environment cells; no host ABI, heap layout or builtin-operation change is
introduced. A selector await retains this owner even when all bodies are eager.
Wholly eager switch layouts and the earlier body-only continuation layouts are
preserved.

Selector lowering carries the effects of all earlier failed selectors. Default
entry uses the final failed-selection facts, and body lowering joins direct
entry with possible preceding-body fallthrough through the existing var/global
merge authority. No-match is included in the switch-exit join. Mutable scope
value facts and static binding/prototype caches are conservatively widened at
body and exit joins; shared lexical lifecycle/storage is not replaced by a
second flow representation. This corrects the initial patch's inherited reset
pattern exposed by independent source review.

Three new constructor controls, two lowering controls and ten paired Engine
sources are authored and UNRUN. They cover rejected ownership/state gaps,
first-match/default/no-match/fallthrough, discriminant identity and outer
scope, shared cells/TDZ, method Reference ordering, rejection and nested
awaited-finally labelled exits. Compilation and the combined focused/pinned/
broad checkpoint are pending. Branch-sensitive selector/discriminant awaits,
other than the conditional/logical values described below, implicit asynchronous
disposal/iteration, enclosing-loop composition and
sync/async generator switch suspension remain unsupported dependent work.

The integrated dry conditional and logical expression follow-ups use existing
If prefix state spans, already accepted by the selector constructor and
dispatcher. They retain the test or left GetValue once, suspend only the selected
arm, and join before the selector decision. The new Engine control checks
skipped effects, selection past a middle default and body fallthrough without
testing later selectors. These sources remain unexecuted. Optional/compound
selector branches retain their refusal. See the
[logical expression contract](logical-await-expression-ownership.md).

## Current candidate verification

The named async-switch IR target and paired Engine target pass. They cover
selection order, shared CaseBlock environments, fallthrough, exits, awaited
finalizers, rejection, synchronous disposal and explicit refusal boundaries.
No new switch-specific pinned packet was authored. The joined product uses
the actually resolved Wasmtime 47.0.0 lock and explicit Copying policy.

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


This is a source-only proposal based on the reviewed Locale/GC-wording source.
It is uncompiled, its runtime controls are unexecuted, and it has no MAIN or
conformance admission. The runtime remains the base Wasmtime version; the
separate Wasmtime 47 foundation and incomplete lock closure are dependencies of
neither this IR owner nor its Wasm emitter.

The relevant source semantics are switch CaseBlock evaluation and its shared
lexical instantiation, Await's queued resumption, and propagation of Break,
Continue, Throw and Return through finally and disposal. Existing front tests
preserve the switch early error for direct using/await-using declarations in a
CaseClause or DefaultClause; a nested block is a different valid source shape.
The reference sections are [Switch](https://tc39.es/ecma262/#sec-switch-statement),
[Await](https://tc39.es/ecma262/#await), and
[Try](https://tc39.es/ecma262/#sec-try-statement).

## Source and IR admission

Only plain async functions can create this owner. A private consuming source
carrier checks all eagerly executed selectors and implicit suspension forms
before case lowering. Its visitor treats nested function bodies and parameter
lists as separate activations while traversing class definition-time work.
Ordinary await is allowed in an admitted body. Awaited selectors, yield,
`await using`, and `for await` in the current activation are refused explicitly.
An awaited discriminant uses the existing staged-expression prefix, isolated
from case lowering; branch-sensitive heads that prefix cannot own are refused.

`AsyncFunctionSwitchIr` owns its discriminant, shared lexical environment,
hoisted functions, actual case bodies, entry state and exit state. Its private
constructors authenticate each case's actual same-activation continuation
sequence, checked arithmetic and ordering. They reject missing child owners,
uncomposed loop/iterator/disposal owners, independent per-case environments,
duplicate defaults, inconsistent try-clause ranges and state gaps. A case's
next entry is one state beyond its final child; every eager case therefore has
a distinct selection/fallthrough segment when this owner is needed. Switch
exit is distinct from the last child's exit. A break bypassing that child
cannot depend on its await to select the following statement.

If no actual case allocates a continuation, the consuming eager-case path
restores the ordinary switch representation and does not advance the enclosing
counter. This preserves existing eager switches in synchronous loops. Ordinary
try clauses allocate states even without source await, so they trigger the
checked switch owner. A source iteration postcondition refuses a resulting
switch owner under an enclosing loop whose dispatcher cannot compose it.

## Wasm selection, resumption and cleanup

On owner entry, the discriminant is evaluated in the enclosing environment.
Only then is the CaseBlock environment created and all direct lexical cells
and hoisted functions instantiated. Eager selectors run left to right until
the first match; a default anywhere in the list is the fallback after no
selector matches. Selection commits the selected case entry to the activation.
No-match commits switch exit. Wasm selection locals are used only at entry.

On resume, the saved shared environment is reattached and the current
activation state identifies the selected case segment. The emitter runs the
existing async statement-sequence dispatcher for that actual body. Normal
completion commits the next case entry, enabling fallthrough without another
selector evaluation. Each case range is half-open; switch exit is outside the
owner's active range.

The switch break target is created after attaching CaseBlock. Matching breaks,
including pending breaks dispatched after awaited finally, reach one epilogue
that commits switch exit and leaves that environment once. An ancestor labelled
break unwinds past CaseBlock and uses the ancestor region's existing exit.
An admitted awaited-while child owns its own break/continue reset. Rejection,
finalizer override and nested synchronous disposal continue through existing
completion routing and activation-backed resource owners.

## Explicit remaining boundaries and verification

Awaited case selectors need retained discriminant and selector phases and are
not admitted here. Implicit asynchronous disposal/iteration, a suspending
switch inside an enclosing loop, and sync/async generator switch suspension
also remain unsupported. The synchronous-loop, async-for-of and generator-for-of
body constructors reject the new owner where no composition exists. Caller-flow
proof returns false; an async switch must not preserve eager call assumptions.

Authored IR and Engine regressions exercise ordering, default-middle fallback,
no-match and early/late break, fallthrough, discriminant scope, shared cells and
unvisited TDZ, nested switch/label/while owners, finally/rejection, disposal,
eager-loop preservation, opaque nested functions and typed rejection boundaries.
They require future actual compilation and sloppy/strict Wasm-AOT execution.
No tests or suite filters have run in this proposal. A future joined source must
also regenerate every affected native source-identity closure and complete one
coherent focused/pinned/broad verification ladder. Host ABI, builtin operation
numbers and public source serialization are unchanged.
