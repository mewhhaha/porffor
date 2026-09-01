# Spec-operation catalog contract

This area's contract lives in one document, because the catalog's
`StatementEmission` rows are witnessed by the iterator contract's `EmissionSite`
values and the two would drift if split:

**[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)**

Start at §3 (type mapping, Part A) for the catalog half.

As built: `crates/lila-ir/src/operations.rs`. §12 (encoder addendum) records
the four deviations, the three added mistake classes, and ledger entries L6–L8.

**§13 is the dry-run discrepancy pass and supersedes §§1–12 where they
disagree.** For this half, read 13.2 (the catalog entry was forgeable), 13.3
(`ALL` is now macro-generated; the L1 test and `catalog_index` are deleted),
13.9 (`sites` is a slice), 13.10 (the single-source `TaskId` enum closes owner
membership over T00–T29) and 13.11 (what `EmitterEvidence` actually proves, and
how L2 must be scoped).

## Current catalog checkpoint (2026-08-29)

§16 supersedes the earlier status and census claims. `ArraySpeciesCreate` is a
macro-backed `BackendSpecOperation`, with unforgeable `BackendEmitterEvidence`
joined exhaustively to the existing
`emit_array_species_create` function. Its canonical normal result is `Object`,
because a custom species constructor may return an arbitrary object. The 46-row
catalog now consists of 29 expression rows, 2 backend rows, 5 statement rows,
and 10 tracked gaps.

The shared emitter is complete but is product-reachable only from Array
`slice` and `splice`. Of the 9 direct `Symbol.species` reads in
`builtins/array.rs`, one belongs to that emitter and eight remain outside it.
§16 classifies those eight sites and records why this promotion is not a
single-owner or universal-migration claim. `SpeciesConstructor`, `Completion`,
and `UpdateEmpty` remain gaps. The two bounded structure targets pass `10/10`,
and the new runtime fixture plus four neighboring Array/TypedArray controls pass
`5/5`. No Test262 result or published count is claimed.

§17 records the second backend row in detail. `ToPropertyDescriptor` has the
typed contract `Value -> PropertyDescriptor` and `MayThrow`, with exactly one
shared converter definition and two direct Object-static-builtin call sites:
`Object.defineProperty` and `Object.defineProperties`. Its private-field,
non-`Copy`, `#[must_use]` reserved-locals carrier must be consumed by the
present-descriptor object materializer, which releases the carrier's locals in
reverse reservation order. This is not evidence for the general
`FromPropertyDescriptor` operation, whose row remains a gap; the separate
Reflect and Proxy descriptor paths remain open-coded nonclaims. The census is
still `29 + 2 + 5 + 10 = 46`. The bounded evidence target passes `7/7`, the
existing Object descriptor fixture passes `1/1`, the filtered IR operation
units pass `53/53`, and `cargo check -p lila-aot-wasm` is green.

---

## INTEGRATOR stage — I7 applied

The lane note's **I7** (optional, sequenced last) is now **applied**: the dead
`ForOfArray` async path is deleted.

The evidence was re-measured in the tree before deleting, not taken from the
note. `StatementIr::ForOfArray` has exactly **one** construction site
(`lowering.rs`, the array index-walk head lowering) and it set
`async_plan: None`; `AsyncForOfPlanIr` had **zero** construction sites
workspace-wide — only a definition, a field type, an import and a parameter
type. So `compile_async_for_of_array` (448 lines) was unreachable from the
product path, which AGENTS.md says should fail to build and did not, because it
was `pub(crate)` and reached from arms fed by a field that is always `None`.

Deleted: `AsyncForOfPlanIr` and `ForOfArray.async_plan` (`ir.rs`),
`async_plan: None` at the construction site (`lowering.rs`),
`compile_async_for_of_array` plus its import and the two
`async_plan: Some(plan)` entry/exit-state arms (`control_flow.rs`), and the two
`ForOfArray { async_plan: Some(_), .. }` arms in `emit.rs`. The two dispatch
arms now bind `..` and call `compile_for_of_array` unconditionally.

`for await` over an array is unaffected: it does not reach `ForOfArray` at all.
`lower_for_of` routes any `for_of.r#await()` to `ForOfIterator`, whose
`AsyncForOfIteratorPlanIr` is a different type and is genuinely constructed —
which is *why* the `ForOfArray` path was dead.

`cargo check -p lila-ir`, `cargo check -p lila-aot-wasm` and `cargo xc`
are all clean after the deletion, and no warning appeared or disappeared, so
nothing else depended on it. Rung G is expected to diff empty and has not been
run here.

## Current synchronous Array `for-of` checkpoint (2026-08-29)

The immediate synchronous Array shortcut is now retired. This checkpoint
supersedes the integrator note above wherever it still describes a live
`StatementIr::ForOfArray` or `compile_for_of_array`.

Exact Arrays that reach direct synchronous statement lowering now use
`StatementIr::ForOfIterator` with
`IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL`. The catalog's
`GetIterator`, `IteratorStep`, `IteratorValue`, and `IteratorClose` rows use the
existing `StatementEmission(SyncForOfIterator)` path for these loops. The
catalog census does not change.

The generic iteration value is `Dynamic`, because a replaced `@@iterator` can
yield a value unrelated to the Array's inferred element shape. The focused
runtime witness covers live length, inherited indexed `Get`, a prototype
`@@iterator` that yields a String, and break-driven `IteratorClose`.

Focused verification passes: the two structure targets are `3/3` and `4/4`,
the IR `for_of` target is `16/16`, the planner and two CLI targets are each
`1/1`, and the four pinned Array length-mutation leaves are `8/8` on Wasm-AOT
with every failure bucket at zero. The former plain-async Array index rewrite
has since been deleted by the resumable synchronous iterator checkpoint below.
See
[the focused contract](./synchronous-array-for-of-iterator-protocol.md) and
§18 of the combined evidence contract.

## Current synchronous String `for-of` checkpoint (2026-08-29)

The immediate synchronous String shortcut is now retired.
`StatementIr::ForOfString`, `compile_for_of_string`,
`IteratorProtocolWitness::STRING_CODE_POINT_WALK`, and its two String-specific
premises are deleted. Ordinary String heads now use
`StatementIr::ForOfIterator` with
`IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL`. The four catalog rows keep
their existing `StatementEmission(SyncForOfIterator)` evidence, and the
catalog census does not change.

The generic value is `Dynamic`. Primitive property lookup boxes through the
current function Realm, while the observable accessor and iterator-method
receiver remain the original primitive String. The focused fixture mutates
both replaceable protocol methods and restores their full descriptors.
Focused verification passes: the String structure target is `3/3`, affected
companion structures are `19/19`, the IR `for_of` target is `17/17`, the CLI
witness is `1/1`, and three pinned native String controls are `6/6` on Wasm-AOT
with every failure bucket at zero. This direct-path checkpoint made no
complete iterator-error Realm claim; the later full-boundary checkpoint below
supersedes that historical nonclaim. Its former directly awaiting String-loop
nonclaim is superseded by the plain-async checkpoint below. See
[the focused contract](./synchronous-string-for-of-iterator-protocol.md) and
§19 of the combined evidence contract.

## Current plain-async synchronous iterator checkpoint (2026-08-29)

The catalog's four iterator rows now have a separate statement-emission owner
for a synchronous `for-of` whose body directly awaits in a plain async
function. `IteratorProtocolWitness::RESUMABLE_SYNC_ITERATOR_PROTOCOL` is fully
emitted by `EmissionSite::ResumableSyncForOfIterator`, and the backend's
exhaustive name-resolution join maps that site to
`compile_async_function_for_of_iterator`.

The IR is `StatementIr::AsyncFunctionForOfIterator` with a required closed
plan. Its activation-backed Iterator Record makes `GetIterator`,
`IteratorStep`, `IteratorValue`, and `IteratorClose` real operations across
resume rather than premises discharged by an Array index walk. The catalog
census remains `29 + 2 + 5 + 10 = 46`; this adds an emission site consumed by
the existing four statement rows, not catalog rows.

`AsyncForOfArrayWalkForm`, `lower_async_for_of_array_with_body_await`, and
`ARRAY_INDEX_WALK_RESUMABLE` are deleted. The result value is `Dynamic`, and no
Array type gate remains, so directly awaiting synchronous String iteration uses
the same protocol owner. Bounded evidence includes a runtime bare-identifier
assignment oracle; the five structure targets pass `19/19`, the IR `for_of`
target passes `18/18`, and the four exact CLI oracles pass `4/4`. A single-name
declaration or bare identifier assignment is admitted.
Direct `break`/`continue`, pattern and property heads, captured head TDZ,
iterable suspension, async generators, and `for await` are nonclaims. See §20
of the combined evidence contract.

## Current resumable synchronous member-head checkpoint (2026-08-29)

The property-head nonclaim above is superseded for non-suspending static,
computed, and private member References. No catalog row or emission owner
changes: `AsyncFunctionForOfIteratorPlanIr::before_await` carries the existing
`PropertyWrite` or `PrivateWrite`, and
`EmissionSite::ResumableSyncForOfIterator` still owns the surrounding iterator
operations. The write executes once on entry inside IteratorClose, before the
body await. Capture analysis now scans the member base and computed key. The
runtime oracle is `wasm_plain_async_sync_for_of_member_heads.js`. The relevant
all-target compile, `21/21` IR filter, `1/1` rejection matrix, `25/25`
structure tests, and `2/2` exact CLI tests pass; the fixture passes
`node --check`. No catalog count changes and no matching pinned Test262 cohort
is claimed. See §23 of the combined evidence contract and
[`plain-async-synchronous-for-of-member-heads.md`](./plain-async-synchronous-for-of-member-heads.md).

## Current resumable synchronous nonlexical-pattern checkpoint (2026-08-29)

Assignment patterns and `var` binding patterns now use the existing
`ResumableSyncForOfIterator` emission owner. This adds no catalog row or
emission site: the activation-owned IteratorValue slot feeds the ordinary
Array/Object destructuring prefix in `before_await`, within the same
IteratorClose frame. Capture analysis exhaustively scans both assignment
pattern shapes and their recursive nesting. The source-free runtime oracle is
`wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js`; no matching pinned
Test262 cohort is claimed. The relevant compile and formatting check pass; the
focused IR checks, six structure targets, and four CLI oracles pass `25/25`,
`25/25`, and `4/4`, respectively. The fixture passes `node --check` and its
Node semantic baseline. The lexical-pattern checkpoint below supersedes this
historical multi-binding rejection. See §24 of the combined evidence contract
and
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](./plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

## Current resumable synchronous lexical-pattern checkpoint (2026-08-29)

`let` and `const` binding patterns use the existing
`ResumableSyncForOfIterator` emission owner and add no catalog row or emission
site. The closed head witness supplies exact iteration and TDZ names plus
BindingInitialization; the plan derives a compiler-only entry local and a
complete fresh iteration Environment Record. Defaults, nested destructuring,
rest, empty-pattern semantics, and their abrupt completions therefore run
inside the existing IteratorClose frame before the body await.

The source-free runtime oracle is
`wasm_plain_async_sync_for_of_lexical_pattern_heads.js`; the pinned checkout
has no exact Test262 leaf. The relevant compile and formatting check pass; the
IR filter and rejection witness pass `27/27` and `1/1`, six structure targets
pass `28/28`, and five exact and retained CLI controls pass `5/5`. The fixture
passes `node --check` and its Node semantic baseline. The catalog census stays
unchanged. See §25 of the combined evidence contract and
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).

## Current shared IteratorClose error-Realm checkpoint (2026-08-29)

The existing `IteratorClose` catalog row and catalog census do not change. The
shared backend owner now creates its two algorithm TypeErrors in the current
function Realm across 67 external entry routes: 16 direct, 48
preserving-current-Throw, and 3 preserving-saved-Throw. Preserving routes keep
the original Throw, and entry code with a zero `current_env_local` keeps the
main Realm fallback.

At this close-only checkpoint, ordinary direct `for-of` acquisition and
stepping errors remained outside the change. The source-structure target
passes `4/4`, the exact created-Realm CLI test passes `1/1`, the affected close
sweep passes `6/6`, and the two direct `for-of` Test262 leaves pass `4/4`. The
later full boundary below supersedes that historical nonclaim. See
[the focused contract](./iterator-close-error-realm.md) and §21 of the combined
evidence contract.

## Current direct synchronous `for-of` protocol-error Realm checkpoint (2026-08-29)

The existing four synchronous iterator catalog rows and the catalog census do
not change. Their three direct execution owners now route five acquisition and
stepping checks each through one closed body-Realm projection, for 15 checks
in total. `compile_for_of_iterator` and
`compile_async_disposable_for_of_iterator` own five inline checks each;
`compile_async_function_for_of_iterator` delegates its five checks to the
shared acquisition and stepping emitters. All routes use
`SyncIteratorConsumer::ForOf` and the exhaustive
`SyncIteratorProtocolError` diagnostic projection. Primitive property lookup in the two
inline owners, including the async-disposable path, boxes through the current
function Realm. Algorithm-created errors from main and user bodies use the
main Realm; only a standard-builtin body may read its trusted self-backed
current environment as Realm metadata.

The entry-Realm CLI fixture exercises all five errors and a valid control, but
does not distinguish current-function from main-Realm construction. A
Realm-distinguishing runtime witness requires a compiled user function defined
in a created Realm, which the Wasm-AOT dynamic-code boundary does not provide.
The focused and affected structure targets pass `37/37`, the CLI cohort passes
`5/5`, and four pinned direct leaves pass all `8/8` Wasm-AOT executions with
every failure and non-success bucket at zero. Shared IteratorClose,
`for await`, Array destructuring, ArrayAccumulation, and `Math.sumPrecise`
retained separate owners at that checkpoint. The current consumer checkpoint
below supersedes its synchronous-consumer nonclaim. The catalog census remains
`29 + 2 + 5 + 10 = 46`. See
[the focused contract](./direct-synchronous-for-of-protocol-error-realm.md) and
§22 of the combined evidence contract.

The ordinary direct owner now also routes `@@iterator` and cached `next`
through the general `IsCallable` and Proxy-aware `Call` operations. Its bounded
Rust guard forbids the former Function-tag gates and Function-only calls while
pinning the original iterable and iterator receivers, empty argument lists,
and post-call propagation before result validation. The source-free fixture
covers callable, throwing, non-callable, and revoked Proxy methods plus the
no-close stepping rule. The direct async-disposable and resumable shared
owners already had this shape. No operation row or error-producer count
changes, and no cross-Realm Proxy-internal TypeError result is claimed. The
affected structure, CLI, and unchanged Test262 cohorts pass `23/23`, `5/5`,
and `16/16`, respectively, with every failure bucket zero.

## Current synchronous iterator consumer checkpoint (2026-08-29)

The four existing synchronous iterator catalog rows still do not change. The
backend now passes one of
`SyncIteratorConsumer::{ArrayDestructuring, ArrayAccumulation, ForOf,
MathSumPrecise}` through acquisition and stepping. The non-`Copy` domain owns
diagnostic selection only. Its product with four protocol-error variants is 16
exhaustive rows, backed by 17 typed producers and 35 error identifiers.

Primitive acquisition boxes through the current function Realm.
Algorithm-created protocol TypeErrors use the exhaustive builder Realm-source
projection: standard builtins may use their self-backed current Realm, while
main, user, host, and runtime-helper bodies use the main Realm. A nonzero
lexical environment is not Realm metadata. Destructuring's custom step keeps its typed checks and
`next`, result, `done`, then conditional `value` order. ArrayAccumulation keeps
distinct `array spread` diagnostics and no IteratorClose path for acquisition
or step failures, as required by the 2026
[`GetIterator`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-getiterator),
[`IteratorStepValue`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-iteratorstepvalue),
and
[`ArrayAccumulation`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-arrayaccumulation)
operations.

The runtime fixtures execute in the entry Realm and therefore cannot prove a
current-function versus main-Realm distinction. No created-Realm runtime result
or fresh Array literal/rest `%Array.prototype%` result is claimed. The
all-target compile and formatting check pass; the structure cohort passes
`42/42`; the exact Wasm-AOT CLI cohort passes `7/7`; and nine pinned
Array-spread/destructuring leaves pass all `18/18` sloppy/strict executions.
See
[`sync-iterator-consumer-capability.md`](./sync-iterator-consumer-capability.md)
and §26 of the combined evidence contract.
