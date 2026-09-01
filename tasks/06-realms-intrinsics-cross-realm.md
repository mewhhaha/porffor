# T06 — Realms, intrinsics and cross-realm semantics

**Status:** In progress — typed callable Function-prototype and created-realm function foundations exist; full allocation and isolation remain

**Parallel group:** Core foundations  
**Depends on:** T03, T04, T05  
**Blocks:** T11-T14, T17, T21-T24

`Reflect.defineProperty` now carries the observable Proxy-trap descriptor
allocation through a private, non-cloneable Realm Object-prototype proof. The
entry route selects the entry intrinsic; a self-backed created-Realm Reflect
method must load its populated defining-Realm slot and traps instead of falling
back when Realm state is absent. The descriptor allocator consumes that proof,
so an unrelated raw prototype local cannot be passed at the guarded site. The
Rust-lexical closure and scoped non-claims are recorded in
[`reflect-descriptor-object-realm.md`](../docs/rust-rewrite/contracts/reflect-descriptor-object-realm.md).
The focused structure target passes `4/4`, and the engine witness passes `1/1`
for distinct entry/created descriptor prototypes. This checkpoint changes no
intrinsic registry row, Wasm ABI, host surface or published conformance count.
The complete two-line proof, 53-line producer and 12-line consuming allocator
now have one private `reflect/descriptor_object_prototype.rs` owner. Their
visibility-normalized SHA-256 values remain
`b1c715d874f23c0d210ee092b547457eead1cb42557eaff40124f4fe59ba68a0`,
`0ed08206648a1d4f58e9aa3683448dd738d85ea9000d48402e66eed4b34d74f9`
and
`ed599d485865a47e4b425a5ed23630ca3b4c1e3c5c01e18a14869dc39fac1bf2`;
the unchanged six-line parent call pair retains
`770f6319489eb9aa746a3e1147f7484a550413db9c6c4bb4e8bc2da018fd40e5`.
The 73-line child has SHA-256
`30522774764563257635779bbb9c4f59639af31b98136d119cc42ca9dd38688f`
and reduces the concurrent Reflect parent from 2,415 to 2,347 lines. The
recursive guard now pins five child-only carrier identifiers, sole construction
and consumption, zero import/re-export paths and the unchanged parent call
pair. The retargeted structure target passes `4/4`, the engine Realm witness
passes `1/1`, and the shared `cargo xc` checkpoint is green.

## Current repository state

Realm IDs, realm records, intrinsic metadata and realm-owned prototype
references are present in the runtime/backend. The current 23 intrinsic rows now
live in one declarative registry that generates `IntrinsicKind`, ordered
descriptors, callable `name`/`length` properties and 46 ordered property
templates. A closed `IntrinsicLink` relation distinguishes internal
`[[Prototype]]` inheritance from constructor/prototype own-property links.
Const validation pins the exact row/property counts, role compatibility and
reciprocal relationships, so incomplete registry additions fail compilation.
The design contract is recorded in
`docs/rust-rewrite/realm-intrinsics.md`.

The `%ThrowTypeError%` `length`, `name` and property-order metadata
cases now execute their unchanged sources and full `propertyHelper` harness for
6/6 sloppy/strict variants. Removing their stale rewrite authority retired all
four T06-owned semantic shortcut observations. This is metadata closure, not
full realm allocation or isolation closure.

Wasm-AOT created-realm function materialization now takes a private typed
`RealmFunctionMaterializationContext` that contains the `RealmRecordLocal`
minted only by realm-record allocation together with the realm's exact local
Function-prototype payload/tag. The 83 bootstrap sites that previously
allocated a function under the current realm and then repaired pieces of its
header now go through one in-realm choke point; the canonical
`parseInt`/`parseFloat` installer delegates to the same path. Ordinary builtins
therefore receive both their defining realm and that realm's
`%Function.prototype%` before their destination local is exposed. Generator,
async and async-generator contexts fail explicitly until their realm-local
prototype families exist.
Environment/self-backing remains a separate choice because it has distinct
execution semantics. `GetFunctionRealm` now returns opaque result locals that
cannot expose their realm until a consuming route has handled both nonresolved
states. Constructor/default-prototype routes preserve the specified revoked
Proxy `TypeError`; Promise-job creation explicitly selects the specification's
current-realm fallback for a revoked callback. Every route traps a missing
defining realm or unknown callable representation as an internal invariant
failure instead of silently selecting a prototype.

The private `FunctionRealmOutcome::{Resolved, Revoked, Invalid}` authority now
derives no incidental capabilities and no longer exposes Rust representation or
discriminant order as its run-time ABI. One borrowed exhaustive projection owns
the unchanged raw Wasm codes 0/1/2 at all three writers and both router
comparisons. A Rust-lexical guard pins the Revoked-before-Invalid routing,
outcome-local release and all five Get/route pairs with their existing one
branch, three return and one current-Realm policies. This source-equivalent
hardening is recorded in
[`get-function-realm-outcome-code.md`](../docs/rust-rewrite/contracts/get-function-realm-outcome-code.md);
its structure target passes `4/4`, and the generic-construct, Iterator fallback
and Promise callback CLI witnesses each pass `1/1`. No broad, Test262 or
semantic-golden result is claimed for this follow-up. Independent dry re-review
is clean after the complete router tail and all five owner-bounded Get/route
pairs were pinned. The following shared workspace compile, formatter,
module-boundary, task-plan and diff gates all pass.

Created-realm `%Array.prototype%` bootstrap now has a closed typed seam. A
reserved local must be consumed by Array-layout initialization before it can be
published, receive Array named properties, form the realm-local `%Array%` /
`%Array.prototype%` links, or be released. The general intrinsic writer accepts
a closed non-Array slot domain, while the Array slot has dedicated typed
created-realm and hard-coded entry-realm publication operations. The initialized
Array exotic points at the created realm's `%Object.prototype%`; its constructor
is born through a realm-aware `BootstrapSupplied` choke point without an
automatic plain prototype, and its links use the Array-aware descriptor path
and the exact ECMAScript attributes. Resolved-realm Array default-prototype
fallback requires the resolved realm's populated Array slot and preserves the
Array tag, with no entry-global substitution or payload identity heuristic.
`Iterator.prototype.toArray` now consumes an opaque current-function-realm
Array-prototype witness when allocating its result. Entry-realm calls retain
the entry intrinsic, while a created realm's borrowed method selects that
method's defining realm rather than the receiver's realm. The four unchanged
pinned physical cases that formerly used a source rewrite pass all 8/8
sloppy/strict Wasm-AOT executions, including the direct cross-realm prototype
identity assertions.
The shared workspace compile and every repository policy gate pass. The
Wasm-golden corpus remains at 648 artifacts with no additions or removals: 646
dump summaries change only emitted-function/total-size attribution from the
realm-aware Iterator builtin body, with no import, export, runtime-root,
helper-count, memory or data-segment contract change.
The ordinary-object defaults selected by construction now use a separate closed
slot domain and a non-copyable loaded-prototype witness. Object, String, Number,
Boolean and Date construction require their resolved realm's populated
intrinsic slot and consume the witness together with its Object tag; missing
realm bootstrap state traps instead of selecting an entry-realm global. Date
reuses the same required fallback policy after its arity-specific value
calculation rather than the shared direct-constructor dispatcher.

RegExp construction now extends that closed ordinary-default domain with its
already-published realm slot. Calling a created realm's borrowed RegExp
constructor normalizes undefined `NewTarget` to the self-backed active function;
explicit new targets retain their own identity. The RegExp body then owns one
observable prototype Get, required `GetFunctionRealm` fallback and tagged
allocation, while direct-return classification prevents the generic construct
path from repeating the Get or preallocating a discarded receiver. The
source-free focused contract is
`docs/rust-rewrite/contracts/regexp-constructor-realm-prototype.md`; the pinned
realm case remains coupled to unsupported dynamic Function source generation.

This remains metadata foundation rather than full realm bootstrap. Intrinsic
objects are not yet independently allocated from these templates across the
complete ECMAScript set, the registry is not yet shared with `lila-ir`, and the
eleven focused `lila-runtime` contracts are green. `lila-engine` re-exports the
typed link relation with the rest of the public realm vocabulary.
Dynamic-source-dependent cross-realm cases remain explicit
unsupported cases, and no current complete Wasm-AOT aggregate proves the full
realm acceptance matrix. Complete intrinsic allocation, host-capability
scoping, teardown, borrowed builtins and realm-correct errors therefore remain
active work. The current batch implements `%Function.prototype%` as one
catalogued, non-constructable function value in the entry realm and every
created realm. Its call body returns `undefined`, its internal prototype is the
same realm's Object prototype, and the Function constructor publishes that
exact Function-tagged identity with the required descriptors. A non-copyable
created-realm context couples the intrinsic to its defining realm before other
builtins can consume it. The focused CLI consumer covers callability, tag,
native source, non-constructability, descriptors, entry identity and two
distinct created-realm identities; a bounded structural witness pins the
catalog, body, rooting and both materialization routes. The source-free
contract and five exact selected Test262 paths are recorded in
`docs/rust-rewrite/contracts/callable-function-prototype.md`. `cargo xc`, the
eight bounded source invariants, two realm-materialization unit tests and the
CLI consumer are green. The five selected current-pin files pass 10/10 strict
and sloppy Wasm-AOT executions; the adjacent non-constructability case passes
2/2. The bounded constructor seams do not repair every intrinsic family or
unrelated partial-bootstrap prototype loader.

The created-realm materializer's private internal-prototype policy now derives
no cloning, copying, debugging, equality or default capability. One exhaustive
four-row producer maps ordinary execution to the realm's Function prototype and
generator, async and async-generator execution to the existing explicit
unsupported path; the production consumer remains exhaustive and rejects that
path before allocation. Both unit observations are exhaustive matches. The
recursive lexical guard pins the complete authority census, producer rows,
unit bodies, exact error and allocation order. This is source-equivalent T06
capability hardening, not support for the deferred specialized intrinsic
families. The focused structure, owner unit and positive created-realm CLI
fixture pass `3/3`, `1/1` and `1/1`. The adjacent callable-prototype structure
target runs `7/8`; its unrelated planning-root assertion observes zero
occurrences where it expects one, outside this policy lane. Independent review
is clean after the guard was expanded to the complete materializer body. The
shared workspace formatter, `cargo xc`, diff, module-boundary, and task-plan
checks all pass.

Positive binary-data bounds now use the executing standard builtin's Realm at
the remaining direct Atomics and DataView RangeError sites. Four Atomics owners
and ten DataView getter/setter owner groups are pinned by bounded source guards.
The created-Realm DataView constructor is published with a self environment
handle and its Realm-owned TypeError and RangeError prototypes. Its prototype now consumes one
closed ordered publication plan containing all three implemented accessors, all
22 numeric methods and `@@toStringTag`. Callable names come from the
`StandardBuiltinId` catalog; every callable receives its created-Realm function
prototype, self environment, TypeError prototype and RangeError prototype before
its exact accessor or method descriptor is exposed. The structural publication
target passes `3/3`. A borrowed getter/setter fixture passes `1/1`, covering
created-Realm identities, descriptors and positive-bound RangeErrors on an
entry-Realm DataView. The six-branch borrowed-constructor fixture also passes
`1/1`. The remaining three direct constructor TypeErrors and the three-route
DataView current-length validator now use the same current-function Realm; an
11-call-site grouped source census covers 24 published accessor/method
callables. A borrowed invalid-receiver, invalid-buffer, detachment and
out-of-bounds fixture passes `1/1` while preserving coercion/prototype-read
order. All 16 direct Atomics algorithm TypeErrors are likewise source-pinned to
the executing builtin Realm and their entry-Realm fixture passes `1/1`.
Proxy `[[Set]]` now projects its executing-error Realm through a closed source
domain as well. Standard builtin bodies and exactly five helper bodies may
expose `current_env_local` as Realm storage: ObjectWrite, both receiver-side
helpers and both OrdinarySet variants. Each helper caller first projects its
environment to a trusted Realm record or zero, so ordinary user and host
lexical environments select the main-Realm fallback. Nested `Reflect.set`
materialization consumes that same typed trust. Borrowed created-Realm Array
and Reflect methods therefore construct direct and prototype-forwarded revoked,
non-callable, strict-falsy and incompatible-descriptor TypeErrors with the
borrowed builtin's TypeError prototype. Assignment keeps its strict-mode guard;
both Array push paths unconditionally throw for internal `Set(..., Throw=true)`.
The source/unit invariants and focused
ten-branch CLI fixture pass `4/4`, `1/1` and `1/1` respectively.
Created realms now publish the complete 14-method Atomics surface. Main and
created realms consume the same closed `AtomicsBuiltin::PUBLICATION_ORDER`, its
exhaustive projection selects only Atomics `StandardBuiltinId`s, and names
remain owned by the builtin catalog. The namespace object, global binding,
methods and `@@toStringTag` use their exact descriptors; every method has a
fresh created-Realm function identity, self environment and defining-Realm
TypeError/RangeError prototypes before publication. The bounded publication
target passes `3/3`, and a non-blocking borrowed-`add` fixture passes `1/1`
while covering all 14 identities, names, lengths and descriptors plus
defining-Realm TypeError and RangeError behavior. It inspects but never invokes
`wait` or `waitAsync`. The separate `waitAsync` result repair makes both entry-
and created-Realm Atomics functions self-backed, then derives one private,
non-copyable Object-prototype proof from the executing function's defining
Realm. Both result wrappers consume that proof; the async branch separately
acquires the opaque intrinsic Promise allocation context from the same
executing function Realm and traps rather than substituting an entry global
when required Realm state is absent. Both result shapes define enumerable
`async` then `value` CreateDataProperties with writable/configurable attributes.
A bounded source target passes `4/4`, and a distinct immediate-notify fixture
passes `1/1`; together they cover the not-equal, timeout-zero and async
branches, wrapper/Promise prototypes,
descriptors, key order and resolved `"ok"` value without blocking. The latest
shared semantic golden passes `2/2` in 703.95 seconds
and contains 658 fixture dumps. Relative to the prior 656-dump checkpoint it
adds only `wasm_atomics_created_realm.js` and
`wasm_proxy_set_error_realm.js`, with no removal. All retained dumps preserve
their roots, builtin/helper counts, locals, imports, exports, globals,
memories, data segments and name counts; their only changed fields are the
expected emitted-function byte sizes from the shared Realm/set-path plumbing.
The preceding shared 656-dump semantic golden added the four focused
Atomics/DataView Realm fixtures with no removal. The 652 retained dumps preserve
every structural field except the expected main-local/largest-function changes
in the expanded Number and Proxy fixtures; their code-size deltas partition
exactly into the Number formatter, Atomics Realm and DataView Realm/publication
bodies and their combinations.

Created realms now publish the implemented Promise foundation: a fresh
constructor, `%Promise.prototype%`, `then`/`catch`/`finally`, all ten current
static methods, `@@species` and `@@toStringTag`, with catalog-derived names and
exact descriptors. `%Promise.prototype%` is a required typed Realm intrinsic;
constructor fallback cannot substitute an entry-global prototype. Every
published callable has the created Realm's Function prototype, self environment
and TypeError/RangeError captures. Promise allocation consumes one opaque
`{ [[Prototype]], executing Realm }` context, and resolving functions inherit
that Realm's Function and error prototypes before they reach an executor. The
non-blocking CLI witness covers fresh identities, descriptors, constructor and
`Promise.resolve` result prototypes, resolving-function identities and a
defining-Realm constructor TypeError without running reactions. The bounded
source target passes `5/5` and the focused CLI consumer passes `1/1`. Broader
Promise method error routing and job execution-context switching remain
follow-on work; created-Realm `Atomics.waitAsync` result ownership is closed by
the typed Object-prototype proof and immediate-notify fixture above. The
consolidated semantic golden passes `2/2` in 733.38 seconds and contains 660
fixture dumps.
It adds only the created-Realm Promise and `Atomics.waitAsync` witnesses to the
preceding 658-dump checkpoint, removes none, and preserves all 658 retained
dumps after emitted-function byte accounting is normalized.

Promise combinator callback allocation now builds on the self-backed internal
function boundary without enlarging its algorithm-context records. Standard
and keyed `allSettled` records consume a private defining-Realm Object
prototype context; `Promise.any` copies the active combinator's existing
AggregateError prototype snapshot into its reject-element function and uses an
opaque allocation context for both the last-rejection and empty-input paths.
The bounded allocation target passes `6/6`, and a finite non-blocking fixture
passes `1/1` across both settlement directions and both `Promise.any` branches
with exact descriptors and key order. General AggregateError construction,
async allocation and broader Promise job Realm switching remain separate.

The outer standard-combinator Array boundary is now closed independently.
`Promise.all`, `Promise.allSettled` and both `Promise.any` terminal paths share
one allocation using the executing method's defining-Realm Array intrinsic,
while constructor `C` continues to own only the returned Promise. Zero-env
entry selection is explicit; missing self-backed Realm catalog state traps.
Keyed combinators, `Promise.race`, general AggregateError construction and
PromiseResolve remain separate. The following 665-dump semantic golden passes
`2/2` in 707.16 seconds, adds only the RegExp result-mode witness and removes
none. Of 664 retained dumps, 663 preserve every non-accounting summary; the
expanded Promise witness alone gains two internal/named functions and four
main-function locals.

`Promise.withResolvers` now separates the same two Realm authorities. Its
capability Promise and resolving functions belong to constructor `C`, while the
subsequent ordered result record consumes a private one-shot Object-prototype
proof from the executing method's defining Realm. Capability creation and raw
shell allocation precede proof acquisition; missing self-backed Realm catalog
state traps without a current-Realm fallback. The bounded contract passes
`5/5`, the retained Promise publication contract passes `5/5`, and the finite
two-direction created-Realm fixture passes `1/1`. The following 666-dump
semantic golden passes `2/2` in 704.11 seconds, adds only the array
key-selection witness, removes none and preserves all 665 retained
non-accounting summaries.

The non-callable callback branch of `Promise.try` now consumes a private
one-shot TypeError-prototype proof from the executing method's self-backed Realm
snapshot. The entry zero-environment path is explicit; a missing nonentry
snapshot traps without current-Realm or constructor fallback. Capability and
argument-vector creation still precede callback validation, and the resulting
Throw completion flows through the existing capability reject path. The
bounded contract passes `5/5`, retained publication remains `5/5`, and the
FIFO created-Realm callback fixture passes `1/1`. The following 667-dump
semantic golden passes `2/2` in 702.89 seconds, adds only the iterator-policy
witness and removes none. Of 666 retained dumps, 665 preserve every
non-accounting summary; the expanded Promise callback witness alone gains one
internal/named function and two main-function locals.

Borrowed `Promise.prototype.then` and `Promise.prototype.finally` now select
SpeciesConstructor's default `%Promise%` and both validation TypeErrors through
one private, must-use context from the executing method's defining-Realm
catalog. Entry publication remains an explicit zero-environment route; missing
self-backed Realm, intrinsics, Promise-constructor or TypeError-prototype state
traps without receiver, constructor or active-job fallback. The isolated
boundary is documented in
[`promise-species-realm-context.md`](../docs/rust-rewrite/contracts/promise-species-realm-context.md).

The direct incompatible-receiver branches of borrowed
`Promise.prototype.then` and `Promise.prototype.finally` now consume a distinct
one-shot TypeError-prototype proof from the executing method. The proof is
acquired only after the receiver check fails, uses an explicit entry route for
zero environments and traps on a missing nonentry self-backed snapshot.
Borrowed `Promise.prototype.catch` now uses the executing function's Realm for
ToObject, checkpoints an abrupt `then` lookup, and shares with `finally` one
private validated delegated-Call value. Its two-caller validator accepts
callable Proxies and owns the current-function Realm TypeError for non-callable
methods; its two-caller consumer alone invokes the validated method with the
original receiver and exact argument pair. Later callable-Proxy Call errors
remain T11 work. The boundary is
documented in
[`promise-prototype-receiver-error-realm.md`](../docs/rust-rewrite/contracts/promise-prototype-receiver-error-realm.md).

Shared ordinary descriptor, `CreateDataPropertyOrThrow` and Set rejection
paths now select synthesized TypeError prototypes through the closed
`ObjectMutationErrorRealmSource` domain. Standard builtins may carry only their
self-backed executing-Realm environment, the five outlined Set helpers may
carry only that trusted argument, and ordinary bodies use the explicit entry
fallback. `Array.fromAsync` therefore keeps constructor/result-object Realm
authority separate from errors created by its executing method. The focused
boundary is
[`array-from-async-result-definition-error-realm.md`](../docs/rust-rewrite/contracts/array-from-async-result-definition-error-realm.md).
The shared 679-dump semantic golden passes `2/2` in 800.46 seconds, adds only
the Array.fromAsync result-definition Realm witness, removes none and leaves
677 of 678 retained dumps equal after accounting normalization. The expanded
Promise Realm witness is the sole retained structural change.

The adjacent `%Function.prototype%[@@hasInstance]` source batch installs one
exact catalogued function value under the closed `WellKnownSymbol::HasInstance`
key in both the entry realm and every created realm. Created-realm publication
uses the existing non-copyable function-materialization context, assigns the
created realm and its TypeError prototype before publication, and applies the
required all-false property attributes without exposing the raw intrinsic
payload. `cargo xc`, the five bounded structure checks and the created-realm
CLI consumer are green. The complete eleven-file intrinsic leaf passes 22/22
strict and sloppy Wasm-AOT executions; this is focused evidence and does not
replace the current-pin aggregate publication.

Temporal namespace bootstrap now also has a closed planning boundary. A bare
`Temporal` binding roots the shared ordered constructor and `Temporal.Now`
member lists before a private `TemporalNamespaceMembers` witness can exist;
bootstrap requires that witness and has no partial per-member installation
path. The bounded structure target and focused planning, artifact-import and
engine runtime witnesses pass `3/3`, `1/1` and `1/1`; the previously blocked
ZonedDateTime CLI consumer also passes `1/1`, and `cargo xc` is green. The
647-artifact semantic golden changes only 22 fixture dumps that already carried
a Temporal root, plus its two manifest summaries. This closes the
partial-intrinsic bootstrap state for the currently advertised Temporal
namespace only; independent realm allocation of the complete intrinsic set
remains open.

All fourteen escaping compiler-owned Promise closures now materialize through
one private non-copyable Realm context. Resolving functions use the Promise
record Realm; capability, finally and combinator closures use the active
Promise function with a canonical entry-Promise fallback and no dynamic
current-Realm authority. The choke point installs the defining Realm, its
Function/TypeError/RangeError prototypes, a self environment and a distinct
algorithm context before exposure. Capability duplicate-call and Promise
self-resolution TypeErrors use the same Realm authority. AggregateError and
allSettled result-object prototype ownership remain explicit follow-on work.

Async execution no longer treats the Realm installed for the currently running
Promise job as ownership authority. A private non-copyable context derives the
invoked async function's Realm, stores it in the ordinary activation, and
recovers async-generator authority through that activation's retained function
object. The returned Promise, three rejected-Promise control-flow wrappers and
captured async reactions all borrow that context; default reactions retain
their handler-or-null policy. PromiseResolve constructor-catalog selection and
other async builtins remain explicit follow-on work. The consolidated semantic
golden passes `2/2` in
677.52 seconds, adds only the three focused callback/async Realm witnesses to
the preceding 660-dump checkpoint, removes none and preserves all retained
structural summaries after expected code-size and local-accounting fields are
normalized. The source-free contract is
`docs/rust-rewrite/contracts/async-execution-realm.md`.

The Realm intrinsic table now catalogs the canonical `%Promise%` constructor
as well as its prototype. Both entry and created bootstrap publish the exact
constructor identity. The three `%AsyncGeneratorPrototype%` request methods
consume a non-copyable proof loaded from their executing function's defining
Realm, so their capability and invalid-receiver rejection Promise cannot fall
back to the entry constructor or the current Promise-job Realm. PromiseResolve
surrogates and Promise allocation in other async builtins remain separate. The
664-dump semantic golden passes `2/2` in 707.34 seconds, adds only the Temporal
field-mode fixture, removes none and preserves all retained non-accounting
summaries except the strengthened async Realm witness's five intentional
internal/name entries.

PromiseResolve now has explicit executing-Realm ownership. A private
non-copyable operation context owns the temporary self-backed resolve function;
the paired intrinsic context derives that function and canonical `%Promise%`
constructor from one Realm catalog. Shared await normalization, its abrupt
rejection fallback, async-generator await-return and both `finally`
continuations consume these contexts, and NewPromiseCapability failures use the
operation function's Realm. The borrowed-`finally` fixture observes the created
Realm TypeError when a species constructor omits capability initialization.
The seven overlapping Promise/async Realm structure targets pass `37/37`, and
the finite CLI witness passes `1/1`. The following 669-dump semantic golden
passes `2/2` in 771.49 seconds, adds only the independent Temporal arithmetic
witness, removes none and leaves 667 of 668 retained dumps equal after
accounting normalization. The expanded Promise internal-callback Realm witness
is the sole retained structural change. Test262 verification remains deferred.
The isolated contract is
[`promise-resolve-realm-context.md`](../docs/rust-rewrite/contracts/promise-resolve-realm-context.md).

`Array.fromAsync` now acquires the canonical `%Promise%` constructor once from
the executing method's Realm. One private non-copyable context is borrowed by
the returned capability and both runtime branches' await throwaway capability,
then consumed after branch emission. The zero-environment entry route remains
explicit; a nonentry method must supply its defining Realm and intrinsic table,
and constructor `C` remains solely the result-Array authority. The focused
created-Realm fixture exercises both independent directions and the rejected
invalid-mapper path. Other async builtins remain separate. The bounded
structure target passes `5/5` and the finite CLI witness
passes `1/1`. The following shared 671-dump semantic golden passes `2/2` in
697.36 seconds, adds only this witness and the independent Temporal plain-
difference witness, removes none and leaves all 669 retained dumps equal after
accounting normalization. Test262 verification remains deferred. The contract
is
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

The six Promise combinator static methods now create their algorithmic
TypeErrors and RangeErrors from one private paired context owned by the
executing method's Realm. `race`, the two keyed methods and the shared
`all`/`allSettled`/`any` lowering acquire the context only after the observable
`C.resolve` lookup, borrow it at exactly fifteen live failure sites and consume
it once per lowering. Constructor `C` continues to own only the returned
Promise. A finite created-Realm fixture covers all six method identities and
both Realm directions for Promise versus TypeError; the maximum-length
RangeError remains structural. The dead raw validation in static
`Promise.resolve`/`reject` and unrelated callback errors remain separate. The
bounded structure target passes `5/5` and the focused CLI witness passes
`1/1`. The shared 674-dump semantic golden passes `2/2` in 717.58 seconds, adds
this witness plus the independent Temporal overflow-options and GroupBy
result-kind witnesses, removes none and leaves all 671 retained dumps equal
after accounting normalization. Test262 verification remains deferred. The
contract is
[`promise-combinator-algorithm-error-realm.md`](../docs/rust-rewrite/contracts/promise-combinator-algorithm-error-realm.md).

Created realms now also materialize the implemented WeakRef family through a
private non-copyable publication token. The materializer stores the new
prototype in the closed Realm intrinsic slot, links `constructor` first, then
installs `deref` and `@@toStringTag`, preserving that exact prototype own-key
order. It couples the fresh constructor and method to the created Realm's
Function and TypeError prototypes before the token can expose the global
binding. The constructor's `prototype` descriptor retains its WeakRef-specific
all-false attributes. A source-free fixture covers identity, descriptors,
construction, borrowed errors and all seven primitive foreign-NewTarget
fallbacks. The bounded structure target passes `5/5`; its CLI
registration and runtime witness pass `1/1`, while six selected non-GC pinned
files pass all `12/12` sloppy/strict executions. The shared 682-dump semantic
golden passed `2/2` at the earlier checkpoint in 685.75 seconds, adding this
witness plus the independent for-await identifier-assignment fixture, removing
none and leaving all 680 retained dumps byte-identical. WeakMap and WeakSet were
still separate at that checkpoint and are closed by the publication boundary
below; weak reachability remains separate. The focused boundary is
[`weak-ref-created-realm-publication.md`](../docs/rust-rewrite/contracts/weak-ref-created-realm-publication.md).

Created realms now publish the implemented FinalizationRegistry family through
its own private non-copyable token. The Realm prototype slot, exact prototype
properties, three self-backed callable identities and defining-Realm TypeError
snapshots are complete before global exposure. Constructor-first linking
preserves the exact `constructor`, `register`, `unregister`,
`Symbol.toStringTag` prototype order. FinalizationRegistry and WeakRef
materialize in reverse publication order so their retained locals obey the
backend's stack lifecycle without changing observable global property order.
The bounded structure target passes `7/7`, the source-free CLI witness passes
`1/1`, and six pinned identity, descriptor, receiver and cross-Realm fallback
leaves pass all `12/12` sloppy/strict executions. Weak reachability, cleanup
jobs and created-Realm WeakMap/WeakSet publication were still open at that
checkpoint. The focused boundary is
[`finalization-registry-created-realm-publication.md`](../docs/rust-rewrite/contracts/finalization-registry-created-realm-publication.md).

Created realms now publish the implemented WeakMap and WeakSet families
through one private non-copyable token. Both Realm slots, fresh prototypes,
two constructors, nine methods and the shared typed `@@toStringTag` authority
are complete before global exposure. Constructor-first linking preserves the
entry-Realm prototype key order. Reverse materialization and publication obey
the backend local stack while the observable present-global subsequence follows
`Map`, `WeakMap`, `WeakSet`, `WeakRef`, `FinalizationRegistry`, `Set`. The
bounded structure target passes `6/6`, the source-free CLI witness passes
`1/1`, and sixteen pinned identity, descriptor, method and cross-Realm files
pass all `32/32` sloppy/strict Wasm-AOT executions. `cargo xc` is green and the
broad backend target retains its same seven unrelated baseline failures at
`367/374`. Weak reachability, cleanup jobs, full created-global ordering and
complete weak-collection trees remain open. The focused boundary is
[`weak-collection-created-realm-publication.md`](../docs/rust-rewrite/contracts/weak-collection-created-realm-publication.md).

## Objective

Turn the minimal Rust `Realm` shell and backend-specific prototype slots into a first-class ECMAScript realm model with independently allocated intrinsics, global environment, host hooks and realm-correct error creation.

## Required model

Each realm must own or reference:

- a unique realm ID and agent association;
- the global object, global `this` value and global environment record;
- an intrinsic table containing every constructor, prototype, iterator prototype, well-known function and `%ThrowTypeError%`;
- template maps for builtin properties and exact descriptors;
- job queue/host-defined data interfaces;
- locale/time-zone hooks used by Date/Intl/Temporal;
- module registry and host loader hooks;
- dynamic-source policy from T13.

Do not encode realm identity as a collection of one-off function header fields. Use a general reference from functions and builtin objects to their defining realm.

## Intrinsic bootstrap

The runtime registry is now the single source for its current rows and property
templates. Expanding it to the complete intrinsic set and making `lila-ir`
consume the same registry remain part of this work item.

- Generate intrinsic installation from one declarative registry shared with `lila-ir` builtin metadata.
- Define constructor/prototype links, method `name`/`length`, writable/enumerable/configurable attributes and well-known-symbol properties in data, not repeated emitter code.
- Allow feature modules to register their intrinsic families without editing one giant bootstrap match.
- Validate that all references resolve, property keys are unique and every builtin function has a defining realm.

## Cross-realm behavior

Implement and test:

- `OrdinaryCreateFromConstructor` fallback to the new target's realm;
- error objects created in the realm required by the invoked function/operation;
- cross-realm prototype and `instanceof` behavior;
- calling borrowed builtin methods across realms;
- realm-local `%Array.prototype%`, `%TypeError.prototype%`, iterator prototypes and species constructors;
- object identity and wrapper behavior across `$262.createRealm()`;
- teardown that cannot invalidate still-reachable objects.

## Host integration

Extend `lila-runtime::HostHooks` or replace it with typed capability traits. Host hooks must be scoped by realm/agent and may not expose spec-exec engine objects to product Wasm semantics. `createRealm` must produce a truly separate global and intrinsic graph.

## Acceptance criteria

- Two realms have distinct global objects and intrinsic identities.
- Cross-realm constructor/prototype fallback and thrown-error prototype tests pass without exact-test materialization.
- Builtin descriptors are generated from one registry and verified by unit tests.
- A function always retains the correct defining realm after binding, storage, proxy wrapping or cross-realm transfer.
- Realm destruction releases host resources only after JavaScript reachability allows it.
- No fallback returns the current realm when realm creation is unavailable; failures are explicit.

## Required tests

```sh
cargo test -p lila-runtime --quiet
cargo test -p lila-ir intrinsic_ --quiet
cargo test -p lila-aot-wasm realm_ --quiet
cargo test -p lila-spec-exec realm_ --quiet
cargo test -p lila-engine --quiet
```

Run real Test262 cases containing `createRealm`, `proto-from-ctor-realm`, `newtarget-proto-fallback`, cross-realm error constructors, species and borrowed builtins.

The runtime intrinsic registry no longer stores a separately writable
`length_name_configurable` flag. One exhaustive `IntrinsicKind` projection
lists all 23 registered intrinsics, assigns the fixed name/length template only
to `%ThrowTypeError%`, and supplies the configurable template to every other
kind; both descriptor builders consume their existing owner. The bounded
`intrinsic_function_property_attributes_structure` target passes `2/2`, the
focused runtime template test passes `1/1`, and the existing Wasm
`%ThrowTypeError%` intrinsic-properties fixture passes `1/1`. The shared
workspace compile and every repository policy gate pass, and all 648
Wasm-golden artifacts are byte-identical to the post-Iterator baseline. No
broader conformance run was performed for this registry-only change.

The runtime registry's callable role is now one closed
`IntrinsicDescriptorShape`: constructor, ordinary function, callable
prototype, or non-callable prototype. Function metadata retains only `name`
and `length`; role, callability and constructability are exhaustive projections
from the shape, so contradictory combinations cannot be registered. The 23
rows retain their 10/2/1/10 shape census. Focused structure, runtime registry
and Wasm-AOT constructor/nonconstructable witnesses cover this invariant. The
shared workspace compile and every repository policy gate pass, and all 648
Wasm-golden artifacts are byte-identical to the post-Iterator baseline. No
broader conformance run was performed for this registry-only change.

Realm shell identity now has one authority: the retained `HostHooks` provider.
`Realm` no longer caches a public mutable `String`, so a caller or cloned realm
cannot make the reported shell name disagree with the shared provider. Runtime
and engine accessors project the hook value directly. The bounded source guard
passes `2/2`, and the focused runtime and engine witnesses each pass `1/1`.
The shared workspace compile and every repository policy gate pass, and all
648 Wasm-golden artifacts are byte-identical to the post-Iterator baseline. No
broader conformance run was performed for this host-only change.

`RealmId` is now the sole stored realm identity. `Realm` derives its intrinsic
and global views from that ID, and `RealmGlobal` retains one ID while
synthesizing the fixed global-object, global-this and environment projections.
The runtime can no longer retain contradictory realm IDs across those views.
The bounded structure target passes `3/3`, and the focused intrinsic-identity
and global-identity witnesses pass `1/1` each; both focused registry-reference
witnesses also pass `1/1`. Broad workspace, golden and conformance gates remain
deferred to the centralized verification pass.

Intrinsic property templates now have one of four closed descriptor shapes:
function name, function length, constructor prototype, or prototype
constructor. Owner, key, value and attributes are exhaustive projections from
that shape, so the registry cannot combine a semantic property role with a
contradictory key, value kind or attribute template. The dedicated bounded
structure target passes `3/3`, the overlapping function-attribute structure
target passes `2/2`, and the generated registry retains its `13/13/10/10`
shape census. The focused name/length and constructor-link runtime witnesses
each pass `1/1`, and the existing Wasm `%ThrowTypeError%` intrinsic-properties
fixture passes `1/1`. Broad workspace, golden, policy and conformance gates
remain deferred to the centralized verification pass.

Realm allocation now reserves zero in the type rather than by convention.
`RealmId` contains a private `NonZeroU64`, and the sole builder allocation path
uses checked atomic advancement. Exhausting the integer domain fails with the
last counter value before wraparound can publish zero or reuse an earlier Realm
identity. The source boundary is recorded in
[`realm-id-allocation.md`](../docs/rust-rewrite/contracts/realm-id-allocation.md).
The bounded structure witness passes `2/2`; the focused runtime behavior
witness remains part of the centralized verification checkpoint. This does not
close Realm teardown or complete intrinsic allocation.
