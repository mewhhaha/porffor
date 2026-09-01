# T04 — Shared ECMAScript operations and completion ABI

**Status:** In progress — shared catalogs exist; migration is incomplete

**Parallel group:** Foundation  
**Depends on:** T02  
**Blocks:** Most semantic feature tasks

## Current repository state

The engine's Wasm execution/output contract now carries the private,
no-capability
`WasmExecutionMode::{Legacy, Structured}` domain without incidental debugging
or equality capability. Five entry points fix the mode: four ordinary run
paths select legacy rendering and the structured observation path selects
typed completion capture. Host output ownership and final result shape each
come from an exhaustive match over the same borrowed value across all three
internal execution seams, with host-state
construction pinned before result projection. The source-equivalent invariant
and mutation guard are documented in
`docs/rust-rewrite/contracts/wasm-execution-mode.md`; this changes no completion
ABI or runtime behavior. Its structure target passes `3/3`, three exact runtime
witnesses pass `3/3` in aggregate, independent dry review is clean, and
`cargo xc` plus repository checks are green.

`lila-ir/src/operations.rs` and
`lila-aot-wasm/src/operations.rs` provide shared operation catalogs and
emitters, while the backend has explicit ABI and control-flow modules. The 29
expression-shaped `SpecOperationIr` rows now come from one typed descriptor
declaration containing the name, family, operand domain, normal result and
abrupt capability. The backend validates that closed operand domain before
dispatch, and the former parallel family/result/abrupt matches are gone.

The 2026-08-29 catalog checkpoint adds a second macro-backed operation domain
for complete emitters that do not fit the expression or statement shapes.
`BackendSpecOperation::emitter_evidence` mints `BackendEmitterEvidence` from a
closed backend operation, and the backend joins
`BackendSpecOperation::ArraySpeciesCreate` exhaustively to
`emit_array_species_create`. Its descriptor returns `Object`, not `Array`,
because a custom `@@species` constructor may return an arbitrary object. The
catalog census is now 29 expression rows, 2 backend rows, 5 statement-emission
rows, and 10 tracked gaps, still totaling 46.

The shared `ArraySpeciesCreate` emitter is product-reachable only from Array
`slice` and `splice`. The exact direct-read census in `builtins/array.rs` is 9
`Symbol.species` reads: one in the shared emitter, five live local
`ArraySpeciesCreate` copies in Array `flat`, `concat`, `flatMap`, `map`, and
`filter`, three distinct `TypedArraySpeciesCreate` paths in typed-array `slice`,
`map`, and `filter`. Array `every` and `some` now emit neither species selection
nor an output Array allocation. The catalog promotion does not claim a single
species owner or universal caller migration. `SpeciesConstructor`,
`Completion`, and `UpdateEmpty` remain tracked gaps. The full boundary is in
§16 of the
[spec-operation catalog evidence contract](../docs/rust-rewrite/contracts/Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md).
At the central checkpoint, the catalog and neighboring structure targets pass
`10/10`, the filtered IR operation units pass `53/53`, the three Array slice
CLI controls pass `3/3`, `cargo check -p lila-aot-wasm` is green, and all
repository gates are green with 240 exact shortcut entries. No Test262 result,
published status count, or conformance gain is claimed.

At the 2026-08-29 cleanup checkpoint, the two bounded structure targets pass
`10/10`. The constructor/species non-observation fixture and four neighboring
Array/TypedArray core and resizable-buffer controls pass `5/5`. The runtime
fixture records the existing Every/Some semantics; the bounded source guards
distinguish the emitted-Wasm cleanup.

`ToPropertyDescriptor` is the second `BackendSpecOperation`. Its descriptor is
`Value -> PropertyDescriptor` and `MayThrow`; the exhaustive backend join names
the one `FunctionBuilder::emit_to_property_descriptor` definition. Exactly two
direct source call sites use it, from the `Object.defineProperty` and
`Object.defineProperties` static builtins. The conversion returns a
`#[must_use]`, non-`Copy` `ReservedPropertyDescriptorLocals` value whose
descriptor field and carrier type are private. The paired
`emit_from_present_property_descriptor` materializer consumes that value,
publishes only its present fields, and releases the owned locals in reverse
reservation order.

This promotion does not claim a general `FromPropertyDescriptor`
implementation: that catalog row remains a tracked gap. `Reflect.defineProperty`
and the other Proxy descriptor paths still use open-coded conversion or
materialization and are not covered by the shared backend evidence. The pinned
catalog census is `29 + 2 + 5 + 10 = 46`; §17 of the catalog evidence contract
records the carrier lifecycle, caller census, and nonclaims. At the 2026-08-29
checkpoint, its bounded structure target passes `7/7`, the existing Object
descriptor fixture passes `1/1`, the filtered IR operation units pass `53/53`,
and `cargo check -p lila-aot-wasm` is green with only the pre-existing vendored
parser warning.

The 2026-08-29 try-clause checkpoint seeds the backend's four completion locals
at every catch and finally entry. All six catch paths and all six finally paths
now start with the Undefined payload/tag pair, Normal kind, and zero auxiliary
value after preserving the incoming throw or try completion. A first
break/continue can no longer inherit that outside value, while an expression
earlier in the clause still replaces the seed and supplies the clause's
completion value. The exact nine owners and twelve calls are pinned by the
module-boundary check and documented in
[`try-clause-empty-completion-seed.md`](../docs/rust-rewrite/contracts/try-clause-empty-completion-seed.md).
At the central checkpoint, the structure target passes `2/2`, four independent
CLI programs pass through two integration tests, the two neighboring completion
controls pass `2/2`, `cargo check -p lila-aot-wasm` is green, and all repository
gates are green with 240 exact shortcut entries. This checkpoint does not
redesign the four-local ABI, add explicit value presence, promote the
`Completion` or `UpdateEmpty` gaps, establish complete try/catch/finally
support, or claim a Test262, published-status, or conformance-count change.

Typed abrupt routing now covers `GetV` inside `GetMethod`, the `ToNumber` of
`Number.prototype.toFixed` argument zero, and every caller of the shared tagged
`ToPrimitive` emitter. The sole tagged emitter requires a closed
`ToPrimitiveAbruptRoute`: route to the active handler, return the current
function, or close a named iterator and return. Adding a route requires an
exhaustive match update, and a new caller cannot omit the decision. The
duplicate tagged `_without_throw_propagation` entry point is gone.

The generic `AbruptRoute` is gone from the first `GetV` and `ToNumber`
migrations. Both variants had one fixed producer, so the shared finisher
admitted only wrong combinations: GetV could return the current function, and
builtin ToNumber could route to the active handler. Each named wrapper now owns
its exact descriptor or conversion, stack cleanup and completion continuation
in source order. Those mismatches are unrepresentable, and the obsolete
single-policy abstraction is deleted. The recursive source guard rejects both
generic symbols and pins both named sequences. This source-equivalent closure
is documented in
`docs/rust-rewrite/contracts/may-throw-operation-abrupt-route-ownership.md` and
changes neither emitted Wasm nor the completion ABI. The focused structure
target passes `4/4`; the exact Number builtin-family and abrupt Iterator-helper
dispatch CLI witnesses each pass `1/1`; and the shared `cargo xc` checkpoint
and repository hygiene gates are green.

The ordinary-object ToPrimitive emitter now requires the private,
capability-free `OrdinaryToPrimitiveReceiverKind::{Object, Function}` domain
instead of an arbitrary `ValueKind`. Two exhaustive projections own the exact
runtime tag and boxed-primitive-slot decision, so another heap-record family
cannot enter this algorithm without defining both policies. The unused public
Function-only wrapper and its private pending twin are deleted; the live tagged
path already selects Function directly, and the ordinary Object wrapper remains
the other entry. Invalid receiver kinds and a second Function producer are now
unrepresentable. The focused boundary is recorded in
`docs/rust-rewrite/contracts/ordinary-to-primitive-receiver-kind.md` and changes
no conversion behavior, completion route, error Realm, emitted Wasm or ABI.
The receiver-kind target passes `4/4`; the neighboring pending-completion and
conversion-Realm targets pass `3/3` and `4/4`. The existing Wasm-backend
ToNumber and Error ToPrimitive CLI controls pass `2/2`, and the shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.

Numeric-update IR now carries the closed
`NumericUpdateValueKind::{Number, BigInt, Dynamic}` domain instead of arbitrary
`ValueKind` fields. Identifier, global-property, ordinary-property and
Super-property updates can represent only a statically known Number, a
statically known BigInt or the runtime-dispatched result of `ToNumeric`. The
Wasm delta emitter matches those three variants exhaustively; the former
one-caller static delta emitter and every defensive impossible-kind
`unreachable!` arm are deleted. The bounded contract is
`docs/rust-rewrite/contracts/numeric-update-value-kind.md`. This changes no
coercion, Reference lifecycle, prefix/postfix result, completion route,
emitted numeric operation or ABI. The closed-domain target passes `4/4`; the
four neighboring numeric-update targets pass `21/21`. The ordinary-property,
script-global nested-update and global-object-environment CLI controls pass
`3/3`, and the filtered IR numeric-update tests pass `4/4`. The with-environment
control is not green: Wasmtime rejects its existing 6,630,529-byte generated
function as too large before execution. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.

The Number half of coercive arithmetic now matches the complete
`ArithmeticBinaryOp::{Add, Sub, Mul, Div, Mod, Exp}` domain in one exhaustive
branch after the existing operand evaluation, `ToNumeric` and mixed-kind
check. Each arm owns its exact Wasm instruction sequence. The former Mod/Exp
preclassification and nested partial match are gone, so there is no defensive
`unreachable!` arm and a new arithmetic operation cannot compile without a
Number algorithm. The BigInt half remains the exhaustive
`BigIntHelperOp::from_arithmetic` projection. The bounded contract is
`docs/rust-rewrite/contracts/coercive-number-arithmetic-operation.md`; this
changes no evaluation, coercion, error or emitted-instruction order. The
closed-domain target passes `3/3`, and the neighboring Number conversion-order
and BigInt helper targets pass `8/8`. The ordinary-property eager compound
reference CLI witness passes `1/1`, covering all six Number operations. The
shared `cargo xc`, formatting, diff, module-boundary and task-plan checks are
green.

Static `typeof` selection now matches the complete `ValueKind` domain directly
inside `compile_typeof_payload`. The deleted one-caller helper admitted Dynamic
and defended it with `unreachable!`; Object and Dynamic now explicitly select
the existing runtime tag path, while every statically projected kind owns its
exact text. Function retains payload evaluation and the HTMLDDA observation.
Adding a value kind cannot compile without a static-versus-runtime decision.
The bounded contract is
`docs/rust-rewrite/contracts/typeof-static-kind-domain.md`. The total-domain
target passes `3/3`, and the exact core `typeof` engine witness passes `1/1`.
The shared `cargo xc`, formatting, diff, module-boundary and task-plan checks
are green.

The singleton strict-equality fast path now classifies the complete `ValueKind`
domain exhaustively instead of sending every unmentioned kind through a raw
payload catch-all. Number, String and tagged Function/BigInt/Dynamic retain
their representation-specific algorithms. The identity/reference kinds retain
raw-payload equality; Dynamic now explicitly selects tagged equality. A new
value kind cannot compile without an equality decision. The bounded contract is
`docs/rust-rewrite/contracts/strict-equality-static-kind-domain.md`. The total
domain target passes `3/3`, and the exact strict-equality CLI witness passes
`1/1`. The pinned String, object-reference and BigInt controls pass all `6/6`
Wasm-AOT executions with every failure bucket at zero. The shared `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green.

The six object-only spec-operation emitters now consume one private,
capability-free `SpecOperationObjectTargetKind::{StaticallyObjectLike,
RuntimeDynamic, StaticallyPrimitive}` authority. One exhaustive `ValueKind`
projection replaces six wildcard primitive complements, so a new heap kind
cannot silently inherit primitive rejection. Each Get, HasProperty,
HasOwnProperty, DeletePropertyOrThrow, Set and CreateDataPropertyOrThrow arm
retains its own error and completion route. The bounded contract is
`docs/rust-rewrite/contracts/spec-operation-object-target-kind.md`. The
recursive structure target passes `4/4`, the exact HasProperty ordering CLI
witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. No behavior or conformance
change is claimed.

The same route is also mandatory at the lower object/function-specialized
ToPrimitive seam. Its byte-identical `_without_throw_propagation` twin is gone,
and the former generic raw-completion route is gone. Private raw emitters now
return a `#[must_use]` `PendingToPrimitiveCompletion` with private fields. Every
internal numeric/string composite consumes that token in its exact guarded
continuation; the runtime-helper generator reaches only a dedicated wrapper
that emits all four ABI result slots. `unused_must_use` is denied in the module,
so a new internal raw call that omits its continuation fails to build. Array
element stringification selects active-handler routing before coercion.

Primitive ToString now has the same closed ownership rule. Its sole emitter
requires a `PrimitiveToStringAbruptRoute`: active handler, current-function
return, or iterator-close-and-return with a complete local witness. The former
raw `_to_local_without_throw_return` copy is gone. Every consumer names its
policy, and adding a policy requires an exhaustive match update. This fixes the
shared `SpecOperationIr::ToString`, `String(object)` and array-element paths:
when an object's coercion hook returns a Symbol, the resulting TypeError now
reaches an enclosing catch just like a value thrown by the hook, instead of
unconditionally returning the whole function. Object.fromEntries and
Object.groupBy retain their iterator-close-before-return discipline.

The exceptional `ToLength` seam now has a similarly closed, deliberately
bounded owner set. The two RegExp execution paths must propagate a conversion
throw to the active in-function handler, while Array.fromAsync's array-like path
must reject and return its already-created promise. Those three consumers call
one routed emitter with an exhaustive `ToLengthAbruptRoute`; the former
`_without_throw_return` twin and the three caller-side completion checks are
gone. A throw is routed immediately after `ToNumber`, before the infallible
numeric normalization step, so a new exceptional caller cannot accidentally
continue matching, mutate state, or escape a promise-returning algorithm without
naming its completion owner. The ordinary `ToLength` wrapper and its 56 callers
retain their existing current-function policy and remain outside this bounded
migration.

The proxy-aware `Call` dispatcher now encodes its remaining internal
two-policy choice as a private, exhaustive `ProxyCallThrowRouting` domain:
return the current function's completion tuple, or leave the throw in that
tuple for the caller to inspect. The raw dispatcher is private to
`functions.rs`; its two named wrappers fix one variant each, and the outlined
runtime-helper generator reaches it through the leave-completion wrapper rather
than selecting a raw boolean from `emit.rs`. This domain is deliberately
separate from `PropagateCallThrow::ToActiveHandler`, which may branch to an
active in-function handler instead of returning the current function.

The shared primitive `ToNumber` emitter now encodes its internal two-policy
choice as a private, exhaustive `PrimitiveToNumberThrowRouting` domain. Its two
named wrappers fix either current-function return or leaving the throw in the
completion tuple for an enclosing composite; only those wrappers can reach the
raw emitter. Both the BigInt and Symbol TypeError branches consume the typed
policy projection immediately after creating the error and before emitting the existing
placeholder NaN, preserving their instruction order while making boolean
inversion impossible.

Both throw-routing domains now derive no incidental capabilities. Their
multiple generated throw sites borrow the one policy selected by the named
wrapper and project it only through exhaustive matches, so neither route can be
copied, compared or formatted into a parallel authority. The focused lexical
guards pin the closed variants, capability absence, borrowed projections,
private raw emitters and complete producer/consumer census. This is
source-equivalent Rust ownership hardening and does not change emitted Wasm or
completion behavior. See
`docs/rust-rewrite/contracts/throw-routing-capabilities.md`.

The value-to-BigInt Number-admission seam now carries a crate-private,
two-variant `BigIntNumberPolicy` instead of `allow_number: bool`. The
crate-visible value helper and its private primitive helper require that
policy; the value helper performs the same number-hinted `ToPrimitive` and
forwards the policy unchanged, while only the primitive Number branch projects
it through an exhaustive match. The two
`SpecOperationIr::ToBigInt` projections, typed-data low-word conversion and
three Temporal epoch-nanosecond conversions explicitly reject Number. Only the
`%BigInt%` function selects `NumberToBigInt`, retaining integral conversion and
non-integral `RangeError` behavior. The implementation, source contract and
bounded six-reject/one-admit mutation guard are independently reviewed. Under
the shared eight-core cap, `cargo xc` is green, the structural guard passes
`2/2`, the exact BigInt minimal-validation CLI witness passes `1/1`, and the
exact TypedArray `with` CLI witness passes `1/1` while exercising Number
rejection by a BigInt typed-data write. This verifies the bounded policy seam;
no broad BigInt, Temporal or Test262 refresh or conformance gain is claimed.

`BigIntNumberPolicy` now derives no incidental capabilities. Each named caller
moves one authority through the crate-visible value helper's sole forwarding
edge, and only the private primitive helper consumes it in the exhaustive
Number-branch match. The strengthened guard rejects derived or manual cloning,
copying, equality and debug implementations while retaining the exact
six-reject/one-admit census and conversion-before-projection order. This is
source-equivalent Rust ownership hardening; it changes no conversion behavior
or emitted Wasm. See
`docs/rust-rewrite/contracts/bigint-number-coercion-policy.md`.

The crate-visible `ToPrimitiveAbruptRoute`,
`PrimitiveToStringAbruptRoute` and `ToLengthAbruptRoute` domains now derive no
incidental clone, copy, debug or equality capability. Every named call site
constructs a distinct authority and moves it into the corresponding emitter;
one private finisher consumes each route through its existing exhaustive
match. The iterator-close local witness remains independently reusable across
distinct conversion operations, but the route selected for one operation
cannot be duplicated or inspected into a parallel authority. The focused
guard pins the exact domains, rejects derived and manual capabilities across
the source tree, and preserves the three exhaustive finishers. This is
source-equivalent ownership hardening; it changes no completion behavior or
emitted Wasm. The focused capability and neighboring conversion-Realm targets
pass `2/2` and `4/4`; the latter's stale marker now reflects the already-landed
borrowed primitive-ToNumber route projection. The shared `cargo xc` checkpoint
is green. See
`docs/rust-rewrite/contracts/conversion-abrupt-route-capabilities.md`.

`PendingToPrimitiveCompletion` now represents only a pending `ToPrimitive`
completion in its type-owned lifecycle. The private token stores exactly the
result payload and tag locals; every raw producer constructs that same token,
and the sole routed consumer reaches the private ToPrimitive finisher directly.
The other five consuming continuations no longer discard a redundant
operation field, and constructing this token for GetV, ToLength or ToNumber is
unrepresentable instead of guarded by a debug-only equality assertion.

The follow-up audit deletes the ignored `MayThrowOperation` marker. It admitted
mismatched constants at the specialized ToPrimitive and ToLength finishers
without affecting emission, so it was decoration rather than a proof. The
named operation boundaries now own identity: GetV selects its descriptor
directly, while the ToNumber, ToLength and ToPrimitive wrappers can reach only
their named private finishers. The focused recursive guard pins the marker's absence,
the two-local pending shape, three live producers, six consuming projections and the
named operation boundaries. This changes no emitted instruction,
completion route or Wasm ABI. The focused identity target passes `3/3`, and the
neighboring may-throw ownership and conversion-Realm targets remain green at
`4/4` each, and the conversion-capability target passes `2/2`. The shared
`cargo xc` and workspace hygiene gates are green. The exact ToLength owner and
Error ToPrimitive/ToString CLI witnesses each pass `1/1`. See
`docs/rust-rewrite/contracts/pending-to-primitive-operation-identity.md`.

The shared arithmetic Number-pair helper now accepts the existing closed
`ArithmeticBinaryOp` directly. The deleted `NumericBinaryOperator` wrapper
admitted a never-constructed `Bitwise` state and derived five incidental
capabilities despite all three callers wrapping an arithmetic operator. One
private exhaustive projection now owns the only order distinction: `Add`
applies ToPrimitive to both evaluated operands before either Number conversion,
while every other arithmetic operator converts left then right. A new
arithmetic variant must therefore make a compile-time ordering decision, and a
caller cannot construct an unowned bitwise state or pass an unlabeled Boolean.
The helper body and its three caller paths preserve their instruction order.
The recursive structure target passes `4/4`, and the neighboring unary-numeric
target remains green at `7/7`. The shared `cargo xc` checkpoint is green. The
pinned addition and multiplication order controls pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. See
`docs/rust-rewrite/contracts/arithmetic-number-conversion-order.md`.

The shared ECMAScript string-trim core now carries a private, exhaustive
`EcmaTrimMode::{Start, End, Both}` instead of independent `trim_start` and
`trim_end` Booleans. That is the complete `TrimString` `where` domain, so the
former unowned neither-end state is unrepresentable. Three named wrappers own
the only raw-core entries: String-to-BigInt selects Both; the static String
method fast path maps `trim` to Both, `trimStart`/`trimLeft` to Start and
`trimEnd`/`trimRight` to End; and the standard-builtin dispatcher applies the
same mapping to its three builtin identities. The existing receiver coercion,
abrupt-completion, scan, slice and temporary-local order is unchanged. The
normative source contract, implementation and hardened caller/alias mutation
guard are independently reviewed. Under the shared eight-core cap, `cargo xc`
is green, the structural guard passes `2/2`, and the exact String trim and
arbitrary-precision BigInt string fixtures each pass `1/1`. No broad String,
BigInt or Test262 refresh or conformance gain is claimed.

The synchronous DisposableStack value-return seam now names its remaining
two-policy choice with a private, exhaustive
`DisposableStackReturnDisposition`: return the current function from the early
nullish `use()` branch, or fall through after a completed `use()` / `adopt()`
path installs the normal result. The former raw Boolean is gone, so a new caller
must name that lifecycle decision and cannot silently transpose an unlabeled
Boolean or omit the choice. This closes one feature-local completion-routing
invariant; it does not migrate the stack to a shared completion operation or
change the tuple ABI. The implementation, source
contract and bounded caller-map guard pass the capped `cargo xc` gate, the
exact structural witness (`1/1`) and the existing exact CLI lifecycle fixture
(`1/1`). This verifies the routing-only seam; the 76-file inventory and broad
DisposableStack cohorts were not refreshed, and no conformance gain is
claimed.

The ArrayBuffer slice bound-normalization seam now carries the private, closed
`ArrayBufferSliceBound::{Start, End}` role instead of a caller-selected argument
index and default Boolean. Exhaustive projection fixes `Start` to argument zero
and default zero and `End` to argument one and the entry byte length; the sole
grouped body for ordinary, shared, and immutable slice writes `start_local`
before `end_local`. The implementation and strengthened caller/order guard are
independently reviewed. Under the shared eight-core cap,
`cargo fmt --all -- --check` and `cargo xc` are green, the structural guard
passes `3/3`, the exact species-capture CLI fixture passes `1/1`, and the exact
`start-default-if-undefined.js` and `end-default-if-absent.js` Test262 leaves
each pass `2/2` Wasm-AOT executions with all failure buckets zero under
`--jobs 1 --threads 1`. This verifies only the bound-role invariant: no broad
ArrayBuffer/Test262 refresh, shared-operation migration, copy-policy change, or
conformance gain is claimed.

The `Iterator.prototype.flatMap` outer-close helper now carries the private,
closed `IteratorFlatMapInnerState::{NotInstalled, Active}` lifecycle state
instead of `clear_inner_active: bool`. Its exhaustive projection preserves the
existing order: close the outer iterator while retaining the current throw,
mark the helper done, clear the inner-active marker only for `Active`, then
clear the executing marker. All eight callers remain in the sole flatMap-next
owner, with four active-inner step failures and four pre-installation failures;
the unique inner installation still stores the iterator and next method before
publishing the active marker and looping. The contract and swap-resistant
caller/lifecycle guard are independently reviewed. Under the shared eight-core
cap, `cargo fmt --all -- --check`, `git diff --check`, and `cargo xc` are green;
`iterator_flat_map_inner_close_state_structure` passes `3/3`, the exact
`iterator::run_wasm_backend_succeeds_for_iterator_prototype_flat_map_fixture`
CLI lifecycle witness passes `1/1`, and the exact
`close-iterator-when-inner-next-throws.js` and
`throw-when-inner-not-iterable.js` Test262 leaves pass `4/4` Wasm-AOT variants
in total with every failure bucket at zero under `--jobs 1 --threads 1`. This
verifies only the typed inner-lifecycle selection and preserved outer-close
order; it does not claim a flatMap algorithm change, broader Iterator/Test262
refresh, conformance gain, or completion of T04 or T15. Batch AA additionally
makes the state must-use and capability-free: producers can only move it into
the sole outer-close consumer, with no clone, copy, debug, default, comparison,
ordering or hashing escape. At the Batch AA checkpoint, the strengthened guard
passes `3/3`, `cargo xc` is green, the exact CLI witness passes `1/1`, and the
two pinned leaves pass all `4/4` Wasm-AOT variants with every failure bucket at
zero. No runtime behavior change is claimed.

The `%Iterator%` constructor's `GetPrototypeFromConstructor` fallback now uses
the closed `OrdinaryDefaultPrototype::Iterator` member and the required
resolved-Realm policy. The shared operation performs the observable
`Get(NewTarget, "prototype")` before resolving the original new target's
function Realm, recursively follows bound and Proxy targets, rejects revoked
Proxies, and consumes the required `%Iterator.prototype%` payload together with
its Object tag. Entry and created Realms publish that exact slot, and Iterator
routes directly to its owning body before generic construction can duplicate
the Get or allocation. The implementation, strengthened structural guard,
bound/nested-Proxy CLI controls and contract are independently source-audited;
the fixture passes `node --check`. The runtime gate exposed and repaired the
prerequisite empty-Function lifecycle: created-Realm `%Function%` is now
self-backed and its supported zero-argument result inherits the active
constructor's defining Realm. On 2026-08-24, the exact structural and CLI tests
passed `1/1` each and the single-file pinned Test262 gate passed `2/2` Wasm-AOT
variants with every failure bucket at zero. This closes only the
constructor-Realm operation seam, not dynamic Function source parsing or
broader iterator, generator, close or suspension debt.

The `%Iterator%` active-function rejection uses the private, shared
`ActiveStandardBuiltinFunction` domain to select the exact created-realm
self-backed constructor or the exhaustively mapped entry-realm global. Its
Iterator projection is bounded to one `IteratorConstructor` member and mapping
within the current two-member Iterator/RegExp domain. The Iterator arm rejects
only an undefined `NewTarget` or a Function-tagged payload equal to that exact
active object, and throws in the active function's Realm before its sole
prototype Get and tagged allocation. The shared construct dispatcher routes
Iterator directly to that owning body before generic preconstruction. The
strengthened structural guard pins the Function-tag/identity conjunction, both
identity publications, direct-return membership and dispatch/Get/allocation
order. The CLI fixture covers the raw two-Realm identity matrix, the two
cross-Realm observing Proxy directions, and same-Realm Proxy and bound wrappers
around the active Iterator in entry and created Realms. Every wrapper must
remain distinct and record exactly `prototype,return`; bound wrappers returning
`undefined` also exercise fallback through their target's function Realm. The
product source required no change. On 2026-08-24, the structural guard and CLI
fixture passed `1/1` each, while the direct pinned leaf passed both Wasm-AOT
variants (`2/2`) with every failure bucket at zero. `cargo check -p
lila-aot-wasm`, `cargo xc`, `node --check` and `git diff --check` are also
green. That pinned leaf covers only entry-realm undefined/self rejection, so
this checkpoint claims no measured baseline gain,
RegExp behavior change, broader Iterator closure, generator suspension,
IteratorClose, helper closing or resource-management closure.

The async-generator request-settlement seam now carries the crate-private,
closed `AsyncGeneratorCompleteStepKind::{Yielded, Completed}` lifecycle state
instead of an unlabeled `done: bool`. Only
`emit_complete_async_generator_step` projects that state through an exhaustive
match: `Yielded` becomes `false`, while `Completed` becomes `true`. The exact
owner census is fixed at eleven product calls: the sole yield-completion owner
selects `Yielded`, and all ten terminal body, queue-drain, awaited-return and
already-completed owners select `Completed`. The existing active-request,
capability, dequeue, active-clear, reject/resolve and temporary-lifetime order
is unchanged.

The focused
[complete-step kind contract](../docs/rust-rewrite/contracts/async-generator-complete-step-kind.md)
and swap-resistant source guard are implemented, independently reviewed and
focused-verified as of 2026-08-23. Under the shared eight-core, 22 GB cap,
`cargo fmt --all -- --check`, `cargo xc` and `git diff --check` are green; the
structural guard passes `4/4`, and the exact
`expression-yield-as-operand.js` Test262 leaf passes `2/2` Wasm-AOT variants
with every failure bucket at zero. The broader resumable-loop CLI candidate
remains red with byte-identical output on unchanged `HEAD` and this lane; its
lost loop/lexical continuation state is pre-existing T15 debt, not a
complete-step-kind regression or a passing result.

The earlier Proxy `Call` and primitive `ToNumber` migrations are likewise
invariant-only rewrites. Their former boolean selections already chose the
correct policies, all existing public wrapper call sites are unchanged, and the
policy-dependent emission points retain their exact return/leave branch and
instruction order. Focused source contracts pin each closed variant set,
exhaustive projection, private raw entry and named-wrapper route. Their static
source/diff/rustfmt gates are green; compile and the existing Proxy apply,
callable-trap abrupt-completion, JSON reviver and numeric-conversion runtime
fixtures remain queued behind centralized verification. No `Call`/Proxy or
`ToNumber` conformance gain, completion-ABI redesign or `exnref` migration is
claimed.

The numeric-conversion Realm seam now has one private, capability-free
`NumericConversionRealmAccess` authority. The former
`OutlinedNumericRealmArgument` and `NumericConversionErrorRealm` domains
projected the same three `NumericErrorRealmSource` rows independently, allowing
helper ABI parameter 6 and direct TypeError/RangeError construction to drift.
The sole exhaustive projection now fixes their shared environment-access
decision, while the three consumers remain distinct exhaustive effects. This
follow-up changes no helper argument, error call, instruction, local or
ordering. The dedicated structure target passes `4/4`, the exact projection
unit passes `1/1`, and the neighboring ToIndex Realm and conversion-Realm
targets pass `3/3` and `4/4`. The borrowed TypedArray-set CLI witness passes
`1/1`, and the shared `cargo xc` checkpoint is green. The pinned Array-source
and TypedArray-source negative-offset Set controls pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. See
[`numeric-conversion-realm-projection-capability.md`](../docs/rust-rewrite/contracts/numeric-conversion-realm-projection-capability.md).
Independent review, the shared workspace compile and every repository gate are
green.
This invariant does not claim a completion-ABI redesign, broad Test262 result
or conformance-count change.

The shared conversion-error Realm seam now uses a type-owned current-function
Realm proof across the ToPrimitive and primitive-ToString phases.
The private `ConversionErrorRealm` keeps its borrowed exhaustive 0/1 helper ABI
projection, while `ConversionErrorRealmSource` is borrowed through every
forwarding emitter. `CurrentFunctionRealmPrimitiveLocals` carries payload and
tag locals only. Its sole producer and consumer make the two fixed boundary
selections, so a builtin cannot carry or substitute a raw source policy between
the phases. The seven live main-Realm producers and outlined
runtime-helper producer retain their prior policies, and the helper decoder
still checks main Realm before current-function Realm and traps an invalid
word. This source-equivalent ownership invariant is recorded in
[`conversion-error-realm-source-lifecycle.md`](../docs/rust-rewrite/contracts/conversion-error-realm-source-lifecycle.md).
The dedicated structure target passes `4/4`, both exact
Error.prototype.toString CLI witnesses pass `1/1`, and the exact typed-phase
unit passes `1/1` after its source locator was updated to the active
`builtins/errors/prototype_to_string.rs` module. The scoped rustfmt and
owned-file diff checks pass. Independent review is clean after strengthening
the attribute boundary, all seven live named borrowed seams, the exact token census,
the two fixed selections, and the complete local-release lifecycle. The shared
workspace formatter, `cargo xc`, diff, module-boundary, and task-plan checks all
pass. No
completion-ABI redesign, conversion behavior gain, broad Test262 result or
conformance-count change is claimed.

The backend completion-kind registry now stores the six ordered
`CompletionKindIr` variants directly instead of duplicating each variant as an
independently editable name/code row. Registry consumers derive both projections
through `name()` and `abi_code()`, so contradictory rows are no longer
representable. On 2026-08-25, the bounded registry source target passes `2/2`
and the filtered ABI unit tests pass `3/3`. The shared workspace compile and
every repository policy gate pass. All 648 Wasm-golden artifacts remain
present; this registry invariant adds no emitted delta beyond the shared
Iterator realm repair recorded under T06/T15. No broader Test262 run was
performed for this invariant-only change.

The public IR completion-slot descriptor now stores only a private
`CompletionKindIr`. One `completion_kinds!` declaration emits the closed enum,
its exhaustive name/code/carriage projections, the ordered inventory and every
ABI slot, so adding a kind without adding it to either inventory is not a
representable edit. The abrupt-only completion domain and its ordered inventory
likewise come from one `completion_abrupt_kinds!` declaration instead of an
independently maintained enum and mask. Callers cannot construct a row whose
five projections disagree, and a compile-time dense-code assertion rejects
reordered or repeated ABI codes. Total `completion_abi_slot` construction replaces the optional
lookup that implied a closed completion kind might have no ABI row. Focused
verification passes: the IR structure target is `4/4`, the filtered IR ABI
tests are `3/3`, the backend registry structure target is `2/2`, and the
filtered backend ABI tests are `3/3`. Broader verification remains part of the
next shared checkpoint.

This migration also fixes the Temporal month-code coercion path: a user value
thrown by `toString` now escapes unchanged instead of being overwritten by the
later non-String TypeError check. Existing coercion and iterator-close order is
otherwise unchanged. These wrappers do not make the remaining property and
builtin-coercion sites authoritative: feature
emitters still contain substantial local coercion, property and completion
logic, and the large Test262 materialization layer shows that shared operations
are not yet authoritative across every family. The Wasm completion convention
also remains the existing tuple/current-completion mechanism rather than the
target `exnref` design.

The descriptor and migration boundary are specified in
[`docs/rust-rewrite/operation-descriptors.md`](../docs/rust-rewrite/operation-descriptors.md).
Keep new cross-family semantics in the shared operation layer and delete local
copies only as callers migrate.

The engine's top-level Wasm completion boundary now parses the raw exported
kind once into the private, non-derived
`WasmTopLevelCompletionKind::{Normal, Throw}` domain. Three exhaustive consumers
own the complete downstream policy: legacy thrown-text access, legacy
success-versus-error publication, and structured typed-completion publication.
The former unlabeled `is_throw` Boolean is gone, so these consequences cannot
be independently inverted or omitted, and adding a kind requires every owner
to handle it. The dedicated Rust-lexical structure guard pins the declaration,
single producer, exact source census and all three exhaustive consumers. The
contract is recorded in
[`wasm-top-level-completion-kind.md`](../docs/rust-rewrite/contracts/wasm-top-level-completion-kind.md).
This source-equivalent closure changes no completion ABI or runtime behavior;
the dedicated structure target passes `3/3`, both exact public runtime
witnesses pass `1/1`, and `cargo check -p lila-engine --lib --quiet` is green
with the repository's existing warnings. No broad suite or Test262 refresh was
run for this invariant-only change.

## Objective

Create one spec-shaped implementation path for common ECMAScript abstract operations and one uniform ABI for normal/throw/return/break/continue completions. Remove feature-local copies whose subtle differences cause evaluation-order, proxy, realm and abrupt-completion failures.

## Required operation families

### Conversion and comparison

- `Type`, `IsCallable`, `IsConstructor` and `IsPropertyKey`.
- `ToPrimitive` with correct hint and `@@toPrimitive` ordering.
- `ToBoolean`, `ToNumeric`, `ToNumber`, `ToBigInt`, `ToString`, `ToObject` and `ToPropertyKey`.
- `ToIntegerOrInfinity`, `ToLength`, `ToIndex`, integer/uint conversions and clamping.
- `SameValue`, `SameValueZero`, strict equality, abstract equality and abstract relational comparison.

### Object and invocation operations

- `Get`, `GetV`, `Set`, `HasProperty`, `HasOwnProperty`, `DeletePropertyOrThrow`.
- `CreateDataProperty`, `CreateDataPropertyOrThrow`, `DefinePropertyOrThrow` and descriptor conversion.
- `GetMethod`, `Call`, `Construct`, `OrdinaryCreateFromConstructor`, `SpeciesConstructor` and `ArraySpeciesCreate`.
- Iterator acquisition/step/value/close operations, with sync/async variants exposed for T14/T15.

### Completion model

Define a Rust representation and Wasm calling convention for:

- normal value;
- throw with value and realm-correct error identity;
- return;
- break/continue with optional target;
- empty completion and completion-value updates.

The convention must work across user functions, builtins, proxy traps, host imports and nested `try/finally` without relying on unstructured scratch globals.

## Design constraints

- Operations must preserve observable order and stop immediately on abrupt completion.
- Object operations must dispatch through the internal-method protocol from T10; static-shape fast paths require guards proving no observable trap/accessor/prototype difference.
- Avoid a runtime interpreter. These are compiler-emitted helpers or specialized Wasm functions generated from typed operation IR.
- Design the Wasm-level completion convention from the experimental Wasmtime lower bound: `exnref` exception handling, typed function references and reference types are available and may carry throw/abrupt paths. Do not maintain a second completion mechanism for runtimes that lack them.
- Keep operation signatures stable enough for feature modules to depend on them. Version or feature-gate ABI changes rather than silently changing tuple layout.
- Emit structured diagnostics when an operation cannot yet lower; do not panic.

The Rust reachability planner represents accessor-slot selection with the
private `ShapeAccessorReferenceSelection` domain. Optional chains and reads
select `Getter`, assignments and writes select `Setter`, and logical
assignments, numeric updates and eager compound assignments select
`GetterOrSetter`. Static-key lookup and dynamic-key prototype traversal project
the selection directly and exhaustively, so the former pair of Booleans can no
longer express a neither-slot query. This is planner state only and does not
alter property evaluation, accessor invocation or emitted Wasm ordering. The
isolated contract is
[`shape-accessor-reference-selection.md`](../docs/rust-rewrite/contracts/shape-accessor-reference-selection.md).
Conditional ordinary-property receivers now retain the accessor provenance of
each shaped branch even when no single merged `HeapShape` exists. A separate
all-branches-shaped proof prevents that state from being confused with an
unknown receiver, while effect-free Map/Set size getters avoid self-poisoning
the current function's signature. The joined lowering invariant and all four
joined planner regressions are green. A second closed receiver-provenance proof
distinguishes a carried non-Proxy heap shape from a receiver that may be a
Proxy, so the inherited `__proto__` getter retains its trap effects without
rooting unrelated accessors for proven non-Proxy receivers. The immutable
flattened receiver-leaf set also owns mutation-authority calculation and alias
invalidation for all four ordinary-property write carriers. Nested conditional
writes now invalidate every possible receiver and every descendant reached
through a prototype chain without degrading unrelated intrinsic prototype
facts.
The focused structure, planner, IR and CLI witnesses pass. The following shared
workspace semantic golden passes `2/2` in 696.00 seconds with 668 dumps, adds
only the expanded shape-accessor witness, removes none, and leaves 664 of 667
retained dumps equal after accounting normalization. The only retained
structural changes belong to the independently intended Array reduce, Promise
internal-callback Realm, and TypedArray constructor no-species witnesses.

The private `BuiltinGetterReceiverProvenance` proof now carries no incidental
clone, copy, debug or equality capability. Its ordinary-reference and direct
property-read routes borrow one proof, and the inherited `__proto__` getter
projects `ProvenNonProxy` and `MayBeProxy` through an exhaustive match without
changing the surrounding builtin classification. The source-equivalent
boundary and its focused commands are recorded in
[`shape-accessor-reference-selection.md`](../docs/rust-rewrite/contracts/shape-accessor-reference-selection.md).
The dedicated structure target passes `4/4`, the exact non-Proxy planner unit
passes `1/1`, and the exact Wasm-AOT shape-accessor CLI witness passes `1/1`.
Independent review added the exact six-use local-name census and source-wide
observer-route bans. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings. No emitted-Wasm, completion-ABI or conformance-count change
is claimed.

## Implementation sequence

1. Write a catalog mapping operation name to spec inputs, outputs and possible abrupt completions.
2. Introduce typed operation nodes/helpers in `lila-ir`.
3. Introduce shared Wasm helper generation and a registry that emits each helper once per module.
4. Convert representative property access, builtin argument coercion and tagged `ToPrimitive` paths.
5. Migrate remaining call sites incrementally, deleting old helpers as coverage moves.
6. Add operation-level differential tests against `spec-exec` using side-effecting coercion objects and proxies.

## Acceptance criteria

- There is one authoritative implementation for each listed operation or an explicit tracked gap.
- Side-effect/evaluation-order tests cover success and abrupt paths for every conversion family.
- Nested calls and builtins can propagate arbitrary thrown JavaScript values, not only error-name strings.
- `try/catch/finally`, proxy traps and cross-realm errors consume the same completion ABI.
- Representative Array, String, TypedArray, Date and Proxy tests use the shared operations rather than local coercion code.
- No operation silently maps unsupported object input to a primitive default.

## Required tests

```sh
cargo test -p lila-ir operations_ --quiet
cargo test -p lila-aot-wasm operations_ --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli wasm_ --quiet
```

Run real Test262 coercion-order cases from several builtins plus `language/statements/try`, `built-ins/Proxy`, and `built-ins/Object` to verify cross-family behavior.
