# T21 — Symbols, collections, weak collections and weak references

**Status:** In progress — symbols/collections implemented; weak reachability is explicitly unavailable

**Parallel group:** Feature lane; split internally by Symbol, strong collections and weak reachability  
**Depends on:** T05, T06, T10; iterators use T15; cleanup jobs use T14  
**Blocks:** Collection and weak-reachability portions of T26

## Current repository state

Symbol, Map, Set, WeakMap, WeakSet, WeakRef and FinalizationRegistry have
runtime records and builtin implementations, including ordered collection
storage and registered weak/ephemeron edges. Because the collector is not
executable, weak targets cannot clear and finalization cleanup jobs cannot be
driven by reachability. Strong collection coverage has advanced, but the full
weak-semantics and complete-tree criteria remain blocked on T05/T14.

Map and Set iterator result shape is now a closed persisted-wire domain rather
than five raw integer constants. Each constructor accepts only
`MapIteratorKind` or `SetIteratorKind`; their macro rows generate both the
stable wire word and the complete dispatch set. `next()` walks that set and
hands the selected variant to an exhaustive Rust match, then traps an invalid
record word. Adding an iterator kind therefore cannot silently inherit the old
Map-key or Set-value fallback.

The strong collection cursor is also a shared typed product seam. Its persisted
state is the closed `Scanning | Exhausted` domain, and `StrongCollectionCursor`
selects the complete Map/Set record layout through exhaustive matches. One
emitter now owns the mutation rules: reload the live append-only history and
backing pointer, persist the next position before testing a tombstone, and make
exhaustion irreversible while severing the exhausted iterator's collection
pointer. That preserves deletion and `clear()` positions, visits reinsertion
appended before exhaustion, and rejects invalid state words instead of treating
them as booleans. The representation law and pinned-suite evidence are recorded
in
[`ordered-collection-cursors.md`](../docs/rust-rewrite/contracts/ordered-collection-cursors.md).
This is not a claim that cross-realm or full pinned collection coverage is
closed. Exact engine contracts for both Map and Set are green across live
mutation, deletion/reinsertion and irreversible exhaustion.

Strong collection iterator receiver validation is now one shared typed seam.
`StrongCollectionCursor` exhaustively selects the Map/Set iterator brand and
preserved error message, while the closed
`CollectionReceiverError` domain distinguishes a non-object from
missing internal slots. The exhaustive receiver-representation domain permits
an internal-brand load only for the compatible Object-tag layout; Array,
Function and Arguments objects route directly to missing slots, while live and
revoked Proxies remain unbranded without target unwrapping or trap observation.
Both `next()` emitters use the same receiver-record helper, and its failures are
created from the active builtin function's realm. This makes a borrowed
other-realm `next` throw that realm's `%TypeError%` without changing successful
cursor behavior. The semantic and representation law is recorded in
[`collection-iterator-receiver-validation.md`](../docs/rust-rewrite/contracts/collection-iterator-receiver-validation.md).
This closes only the strong iterator receiver seam, not every collection error
path or T21's broader cross-realm acceptance criterion.

Ordinary collection data receiver validation is now the same representation-
safe mechanism without conflating its semantic domain with iterator cursors.
The closed `CollectionDataReceiverKind` domain selects Map, WeakMap, Set or
WeakSet brands and messages, while `CollectionReceiverRequirement` keeps those
four data-slot requirements distinct from the two strong iterator
requirements. One exhaustive `CollectionReceiverRepresentation` table owns the
runtime layout decision for both seams: Object-tag records alone may load the
ordinary brand offset; Array, Function and Arguments are Objects with no brand
layout; primitives (including the runtime-only heap BigInt tag) are non-object;
and compile-time-only Dynamic is unreachable. Constructor brand projections
and other Map/Set allocation sites also consume the same data-kind authority;
a source-structure regression pins each brand constant to that sole mapping.
The eighteen source consumers
that compile thirty-six ordinary collection builtins can no longer duplicate
or mismatch tag checks, brand loads, messages or error-realm selection. Live
and revoked Proxies remain unbranded without target unwrapping or trap
observation. The exact inventory, semantic law and deferred shared verification
gates are recorded in
[`collection-data-receiver-validation.md`](../docs/rust-rewrite/contracts/collection-data-receiver-validation.md).
This closes only the ordinary collection receiver seam, not the weak-
reachability blocker or T21's full-tree and cross-realm acceptance criteria.

The shared receiver-failure authority now derives no capabilities.
`CollectionReceiverError::{NonObject, MissingInternalSlots}` remains the exact
input to the exhaustive ordinary-data and strong-iterator message tables, and
the representation-safe validator remains its sole three-producer boundary.
The recursive, bounded guard pins the nineteen-mention ownership census, all
twelve message rows, typed forwarding and the exact brand-mismatch,
object-without-brand and non-object validator bodies. The focused invariant and
its existing receiver fixture sources are recorded in
[`collection-receiver-error-domain.md`](../docs/rust-rewrite/contracts/collection-receiver-error-domain.md).
This is source-equivalent capability closure, not new collection behavior or a
T21 completion claim. Its error-domain structure target retains the prior
`4/4` result, and the data-receiver fixture retains its prior `1/1` result.
Independent review confirmed the exact message tables, validator bodies and
brand-check/load/failure order.

The later T18 created-Realm publication boundary now gives Array, String, Map
and Set iterator `next` functions the metadata that their unchanged result and
receiver paths read. Four unit targets select the builtins. One private context,
constructed once from the Realm-function authority and five exact prototype
locals, exhaustively selects the matching publication prototype. Materializing
a target stores the function's own payload as its environment handle and the
context's `%TypeError.prototype%`, then returns a non-`Copy`, `#[must_use]`
token that owns both publication locals. Publication consumes that token.

The context localizes and couples the raw bootstrap trust boundary. It does not
let Rust prove that the six supplied Realm and prototype inputs belong to one
Realm because the five prototypes remain raw `u32` Wasm local indices. The
strengthened shared structure target passes `5/5`, the retained collection
receiver-domain target passes `2/2`, the created-Realm materialization inventory
passes `1/1`, and the Map/Set iterator-receiver fixture passes `1/1`. That
fixture witnesses defining-Realm provenance, both receiver-error
categories and successful controls. The shared boundary is specified in
[`created-realm-iterator-next-publication.md`](../docs/rust-rewrite/contracts/created-realm-iterator-next-publication.md).
This repair does not close other collection error paths, weak reachability, the
full Map/Set trees or T21.

Collection-created algorithm TypeErrors now have one closed realm-aware
authority. Separate Map/WeakMap and Set/WeakSet constructor-stage domains make
their distinct legal failures exhaustive (including the Map-only iterator
entry check), while the existing strong-collection domain selects the two
`forEach` callback checks. All fifteen source sites create errors from the
active builtin function's Realm through one typed emitter; no bounded source
site calls the entry-realm runtime-error helper directly. Created-realm Map and
Set constructors are self-backed and carry their Realm's TypeError prototype,
so they cannot lose that identity through missing function metadata. WeakMap
and WeakSet were initially only structurally covered; their created-realm
intrinsics and cross-realm runtime evidence are now owned by the publication
boundary below. The exact algorithm-error realm, ordering and source inventory
are recorded in
[`collection-algorithm-error-realms.md`](../docs/rust-rewrite/contracts/collection-algorithm-error-realms.md).
This closes only those algorithm-created TypeErrors, not successful
cross-realm construction, iterator closing, weak reachability or T21.

The four collection prototype `Symbol.toStringTag` descriptors now have one
closed installation authority. `CollectionPrototypeIntrinsic` exhaustively
derives both the prototype global and its matching `Map`, `Set`, `WeakMap` or
`WeakSet` String value and derives no cloning, copying, formatting, equality or
default capabilities. Each installer borrows the authority for its prototype
projection and then moves it exactly once into the emitter that owns the
well-known-symbol key and non-writable, non-enumerable, configurable
descriptor. The recursive ten-mention census, derived/manual-capability bans,
exact four-row tables and producer/order laws are recorded in
[`collection-prototype-to-string-tag.md`](../docs/rust-rewrite/contracts/collection-prototype-to-string-tag.md).
This closes only those four intrinsic data properties, not constructor-realm
fallbacks, weak reachability or the complete collection trees. The strengthened
structure target passes `2/2`, and the exact CLI owner passes `1/1`; the four
exact Map, Set, WeakMap and WeakSet `prototype/Symbol.toStringTag.js` leaves
pass all `8/8` sloppy/strict Wasm-AOT executions with every failure bucket at
zero. Independent dry review is clean, and the shared format, `cargo xc`, diff,
module-boundary and task-plan checkpoint is green with the workspace's existing
warnings.

The four `Map`/`WeakMap` get-or-insert entry points now select their argument
discipline through the private closed
`MapGetOrInsertValueSource::{ValueArgument, ComputedCallback}` domain instead
of a raw Boolean. The shared emitter matches that domain exhaustively both
when preparing the second argument and when deciding whether a missing entry
invokes user code. This preserves the distinct WeakMap validation order:
`getOrInsert` validates the weak key before loading the value local, while
`getOrInsertComputed` validates callback callability before the weak key and
never invokes the callback for an invalid key. Existing entries still bypass
callback invocation, and a callback's same-key mutation is still overwritten
by its returned value after the required second lookup.

The current-worktree checkpoint is green: `cargo xc`, `rustfmt` and
`git diff --check` pass, the bounded structure target passes `3/3`,
`node --check` accepts the fixture, and the exact Wasm-AOT CLI owner passes
`1/1`. The shared golden checkpoint retains all 646 existing artifacts
byte-for-byte; artifact 647 is solely this new fixture. Ten exact pinned Map and
WeakMap upsert files pass all `20/20` sloppy/strict Wasm-AOT executions with
every non-success bucket at zero. The matrix covers direct insertion,
present-key callback suppression, callback mutation overwrite, non-callable
callbacks and invalid weak keys without callback invocation. This is
compile-time argument-policy hardening and focused live-key behavior evidence;
it does not implement weak reachability or claim complete Map/WeakMap trees.

Batch AE gives that complete value-source lifecycle one private
`builtins/collections/map_get_or_insert.rs` owner. The domain, four semantic
entry points and raw parameterized emitter moved together byte-for-byte; the
parent can no longer name the raw policy or emitter, while all four product
calls remain unchanged. The five-line domain and 312-line method selection
retain SHA-256
`b5db66b00f27f10e45c4b98a31220473b159564a3d292e1c9ac765a6a7ae3873`
and
`00a687c5a16c6f0c9c2ffeeeb21f714b31cc58b6dcf9d0539f5ea4a12a54acc7`;
their combined hash is
`8666b1d64189818ecd0d108a521afdf4f0ccd9068be169436cc1c1697273d4e7`.
The resulting 6,491-line parent and 322-line child have SHA-256
`8d6c436a07bc388cf950cfaf35659d65f6de068101f2382f6e384b738c44ce9e`
and
`6022280ee176b5a20373e540763ec158a5c2914ce49fb2c8720c5f72df25d7d7`.
Recursive source policy pins ten domain mentions, eight qualified variants,
five raw-emitter sites and all four semantic methods and callers. The Batch AE
shared checkpoint passes `cargo xc`; the exact Map get-or-insert, Map
collection and Set collection structure targets pass `3/3`, `4/4` and `4/4`
(`11/11` aggregate); and the exact
`iterator::run_wasm_backend_preserves_map_get_or_insert_value_sources` CLI
owner passes `1/1`. The exact ten-file upsert cohort passes all `20/20`
sloppy/strict Wasm-AOT executions with every failure bucket at zero: Map
`getOrInsert` append-new-values; WeakMap `getOrInsert` adds-object-element;
paired Map/WeakMap `getOrInsertComputed`
does-not-evaluate-callbackfn-if-key-present,
overwrites-mutation-from-callbackfn and not-a-function-callbackfn-throws; and
WeakMap `getOrInsert` plus `getOrInsertComputed`
throw-if-key-cannot-be-held-weakly. No semantic golden was run for Batch AE.

The `Set`/`WeakSet` value-admission split for `add`, `delete` and `has` now uses
exhaustive matches over the existing private `SetCollectionKind::{Set,
WeakSet}` domain instead of equality checks that gave future variants Set's
unrestricted default. `Set` emits no weak-value check;
`WeakSet.prototype.add` retains its current-Realm TypeError, while `delete` and
`has` retain their early Boolean-false result for values that cannot be held
weakly. Receiver validation still precedes argument access, and lookup remains
after the policy decision. The source invariant and its weak-reachability
non-claim are recorded in
[`set-collection-weak-value-admission.md`](../docs/rust-rewrite/contracts/set-collection-weak-value-admission.md).
The bounded structure target passes `4/4`, and its neighboring Map
get-or-insert structure target passes `3/3`. The existing engine regression
passes `1/1`, and the three exact WeakSet invalid-value leaves paired with Set
controls pass all `12/12` sloppy/strict Wasm-AOT executions with every failure
bucket at zero. The coordinated `cargo xc`, rustfmt and diff checks are green.

The `Map`/`WeakMap` key-admission split for `delete`, `get`, `getOrInsert`,
`getOrInsertComputed`, `has` and `set` now uses exhaustive matches over the
existing private `MapCollectionKind::{Map, WeakMap}` domain instead of equality
checks that gave future variants Map's unrestricted default. `Map` emits no
weak-key check. WeakMap read/removal methods retain their early false or
undefined results, while insertion methods retain their current-Realm
TypeErrors and their distinct value/callback ordering. The domain no longer
implements equality; its existing by-value layout projections keep `Copy`.
The exact producer census, ordering law and weak-reachability non-claim are
recorded in
[`map-collection-weak-key-admission.md`](../docs/rust-rewrite/contracts/map-collection-weak-key-admission.md).
The bounded structure target passes `4/4`; the exact Map get-or-insert and Set
collection neighbors pass `3/3` and `4/4`, for `11/11` across the three Batch
AE structure targets. The existing engine WeakMap regression and exact Map
get-or-insert CLI owner each pass `1/1`; the twelve exact WeakMap invalid-key
and Map primitive-key Test262 controls retain their earlier `24/24`
sloppy/strict Wasm-AOT result with every failure bucket at zero. A separate
computed-callback probe remains `0/2` solely at the explicit T13
`new Function()` dynamic-source boundary and is not an admission witness.

The sole product Wasmtime policy now records
`WasmWeakReachabilityCapability::Unavailable` independently of its DRC
collector choice. Every product engine therefore carries the missing
weak/ephemeron facility as typed setup and reporting context; enabling or
changing a strong-reference collector cannot silently claim weak semantics.
This makes the blocker explicit without changing the current builtin surface
or treating the linear records as a weak implementation.

The shared `thisSymbolValue` receiver path now accepts one closed
`SymbolReceiverOperation` instead of an arbitrary diagnostic string.
Description, `toString`, `valueOf` and `[Symbol.toPrimitive]` exhaustively
project their existing exact TypeError messages, and all four prototype callers
must name their operation at compile time. `[Symbol.toPrimitive]` now consumes
that same receiver algorithm instead of duplicating its primitive/boxed/error
walk. The capability-free domain, seven-mention census, sole projection and
four producers are recorded in
[`symbol-receiver-operation-ownership.md`](../docs/rust-rewrite/contracts/symbol-receiver-operation-ownership.md).
Existing CLI fixtures remain the direct behavior witnesses for the first three
operations: optional property chaining covers Symbol `toString` and `valueOf`,
iterator property-name formatting reads `description`, and the object
`valueOf` fixture covers boxed and cross-realm Symbols. The exact non-object and
boxed-Symbol `[Symbol.toPrimitive]` leaves pass all `4/4` sloppy/strict
Wasm-AOT executions, with every failure bucket zero and all outcomes `Success`.
This invariant does not close Symbol identity, property ordering or T21's
weak-reachability blocker.

Batch AS makes the seven-entry outer family a private `SymbolBuiltin` with no
derived capabilities and exposes only seven fixed Symbol entries to standard
dispatch. The frozen 323-line domain/emitter selection has SHA-256
`3296276e16255ea9aaf39f05b54b77414320a0f71d5c0d4c1a61ed04c1cef9b2`;
restoring only the former derive and visibility reproduces that source exactly.
At the 2026-08-28 Batch AS checkpoint, `cargo xc` is green, the strengthened
receiver-operation structure target passes `4/4`, and the exact non-object and
boxed-Symbol `[Symbol.toPrimitive]` leaves pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. This source-equivalent
boundary claims no new Symbol behavior, broader conformance or published
conformance-count change.

Builtin shape producers now project typed `WellKnownSymbol` values through the
existing namespaced shape-key encoding instead of storing their human-readable
descriptions as ordinary string keys. Symbol reads, writes, inherited lookup
and post-`Set` publication use the same projection, so a string property such as
`"Symbol.iterator"` cannot supply or overwrite the `Symbol.iterator` shape
fact. The computed-key preservation, Symbol/string kind separation, false
function-target, setter-observation and non-writable-intrinsic controls each
pass `1/1`; the complete `lila-ir` unit suite passes `892/892`. The shape table
still uses its encoded key representation; this does not claim full
`Reflect.ownKeys`, Proxy or user-Symbol closure.

The implemented WeakRef surface is now independently published by created
realms. A private must-use token is the only input to global publication, so
the prototype Realm-slot write, constructor-first link, exact prototype
properties, self-backed callable identities and created-Realm TypeError
snapshots must all exist before `global.WeakRef` can be exposed. The exact
prototype own-key order is `constructor`, `deref`, `Symbol.toStringTag`. The
bounded structure witness and source-free runtime fixture are recorded in
[`weak-ref-created-realm-publication.md`](../docs/rust-rewrite/contracts/weak-ref-created-realm-publication.md).
The bounded structure target passes `5/5`, the exact CLI witness passes `1/1`,
and six selected non-GC pinned files pass `12/12` with every failure bucket at
zero. At the earlier checkpoint, the 682-dump semantic golden passed `2/2` in
685.75 seconds, added this witness plus the independent for-await
identifier-assignment fixture, removed none and left all 680 retained dumps
byte-identical. This is
intrinsic/Realm closure only: the target remains strongly retained by the
current unavailable weak-reachability backend. At that checkpoint,
created-Realm WeakMap, WeakSet and FinalizationRegistry remained open.

Batch AF gives that complete created-Realm `WeakRef` materialize-to-publish
lifecycle one private
`builtins/host/created_realm_weak_ref_intrinsics.rs` owner. The non-`Copy`,
must-use carrier, sole producer and consuming publisher moved together while
the parent retained its byte-identical inferred caller pair. Rust requires the
carrier and methods to be `pub(super)`, but the fields remain child-private and
recursive policy forbids parent naming, import, re-export, construction or
projection. The exact pre-move five-line carrier and 153-line method selection
retain SHA-256
`50b98c378f3a260b73ab69e22538e856b957c5203c470489baab4e0677568244`
and
`10a82a763c5b87ef5e28dbd72e26d62d1873675864462e7622b3dd06cbff7a68`;
their combined 158-line hash is
`24d820fe3c2b14085b1f3aa3373537fb1def4ef211ef410025aea1f68300f119`,
and the visibility-normalized selection has SHA-256
`9d17048c6ff4a8f4cacfd97b8c6ac0edc40c5039fe18f563843f931fe403e479`.
The resulting 8,941-line parent and 163-line child have SHA-256
`6ffaf8361a886420f7ee766a66154f6fc42bf9c5704cac6a2fc7e9e64e218b3a`
and
`5d460ca0be7e9eef7f81cee28ca258ac1fc9b4b6b655523651f4b64d4caea049`;
the unchanged 12-line caller pair retains SHA-256
`41419c46072b0e4ae037b6217d5312768a77c9ff4f6c780b55381be0253a96eb`.
The retargeted source target passes `4/4`. At the Batch AF shared checkpoint,
`cargo xc` is green, the exact created-Realm WeakRef CLI fixture passes `1/1`,
and the six exact pinned `proto-from-ctor-realm`,
`newtarget-prototype-is-not-object`, prototype `constructor`, prototype
`prop-desc`, deref `this-not-object-throws` and deref `custom-this` leaves pass
all `12/12` sloppy/strict Wasm-AOT executions with every failure bucket at
zero. Batch AF did not rerun the semantic golden; the earlier created-Realm
behavior checkpoint remains historical evidence.

A later correctness follow-up moved the reciprocal constructor link before
`deref` and `@@toStringTag`, matching entry-Realm own-key order. The
strengthened structure target passes `5/5`, and the live fixture passes `1/1`
with all seven primitive fallback types.

The six algorithm-created `FinalizationRegistry` TypeError categories now flow
through the private capability-free `FinalizationRegistryTypeError` domain.
Its sole exhaustive projection owns the exact diagnostics for construction,
registration, unregistration and receiver validation; the shared emitter no
longer accepts an arbitrary diagnostic string. All eight producers remain in
their existing observable order and still create errors from the active
builtin function's Realm. The focused ownership and ordering law is recorded
in
[`finalization-registry-error-domain.md`](../docs/rust-rewrite/contracts/finalization-registry-error-domain.md).
This does not change emitted Wasm or implement weak reachability, cleanup-job
scheduling or created-Realm FinalizationRegistry publication.

FinalizationRegistry cell presence is now the private persisted lifecycle
domain `FinalizationRegistryCellState::{Vacant, Occupied}` rather than an open
integer Boolean. One typed serializer owns all state writes. Registration and
unregistration publish the matching typed state, while cell-array growth and
unregistration admit only exact state words before exhaustive routing. An
invalid persisted word traps as an invariant failure instead of silently
becoming an occupied cell. The representation and source witness are recorded
in
[`finalization-registry-cell-state.md`](../docs/rust-rewrite/contracts/finalization-registry-cell-state.md).
This hardens the current record lifecycle only; weak reachability and cleanup
jobs remain blocked on the collector and job integration. The bounded cell-
state and neighboring error-domain structure targets each pass `4/4`; targeted
rustfmt and diff checks are green.

Created realms now publish FinalizationRegistry through a private move-only,
must-use materialization token. The materializer owns the closed Realm slot,
fresh prototype, exact `register`, `unregister` and `@@toStringTag` properties,
fresh constructor, reciprocal links, self-backed environments and created-
Realm TypeError snapshots. Only the consuming publisher can expose the global
binding. The constructor-first link preserves the exact `constructor`,
`register`, `unregister`, `Symbol.toStringTag` prototype order.
FinalizationRegistry materializes before WeakRef and publishes after it,
preserving global property order while releasing their retained temporary
locals in stack order. The bounded structure target passes `7/7`, the
source-free CLI witness passes `1/1`, and six pinned identity, descriptor,
receiver and cross-Realm fallback files pass all `12/12` sloppy/strict
Wasm-AOT executions with every non-success bucket at zero. Weak reachability,
cleanup scheduling and callback delivery remain open; created-Realm
WeakMap/WeakSet publication is closed by the following boundary. The focused
FinalizationRegistry boundary is
[`finalization-registry-created-realm-publication.md`](../docs/rust-rewrite/contracts/finalization-registry-created-realm-publication.md).

Created realms now publish WeakMap and WeakSet through one private move-only,
must-use materialization token. The materializer owns both closed Realm slots,
fresh prototypes, constructors, all nine methods, constructor-first property
order, self-backed callable environments and created-Realm TypeError snapshots.
It reuses the closed `CollectionPrototypeIntrinsic` authority to select each
Realm slot and emit both `@@toStringTag` descriptors. Internal and parent
reverse publication preserve temporary-local ownership; the observable
present-global subsequence is `Map`, `WeakMap`, `WeakSet`, `WeakRef`,
`FinalizationRegistry`, `Set`. The bounded structure target passes `6/6`, the
source-free CLI witness passes `1/1`, and sixteen pinned identity, descriptor,
method and cross-Realm files pass all `32/32` sloppy/strict Wasm-AOT executions
with every non-success bucket at zero. `cargo xc` is green and the broad
backend target retains its same seven unrelated baseline failures at
`367/374`. Weak reachability, cleanup scheduling, complete trees and aggregate
status remain open. The focused boundary is
[`weak-collection-created-realm-publication.md`](../docs/rust-rewrite/contracts/weak-collection-created-realm-publication.md).

## Objective

Implement Symbol identity and registries, insertion-ordered Map/Set collections, weak collections, WeakRef and FinalizationRegistry on top of the real object/GC/job models. Do not simulate weak behavior with strong maps or deterministic test-only collection.

## Symbol semantics

Implement:

- unique Symbol creation with optional descriptions;
- the process/agent-wide `Symbol.for` registry and `Symbol.keyFor`;
- every well-known Symbol required by the pinned ECMAScript/ECMA-402 revisions;
- Symbol wrapper objects, branding, `description`, `toString`, `valueOf` and `@@toPrimitive`;
- Symbol property keys throughout objects, descriptors, own-key ordering, proxies and reflection;
- cross-realm identity rules for well-known and registry symbols;
- correct TypeErrors for implicit string/number conversion and explicit `String(symbol)` behavior.

Use stable internal symbol IDs independent of string payloads. User symbols with equal descriptions remain distinct.

## Map and Set

Implement spec-shaped internal slots and insertion order for:

- `Map` and `Set` construction from arbitrary iterables with iterator closing;
- `get`, `set`, `has`, `delete`, `clear`, `size`, `forEach`, entries/keys/values and `@@iterator`;
- SameValueZero key semantics, including NaN and canonicalized signed zero;
- mutation during iteration/forEach, deletion/reinsertion and iterator liveness;
- subclassing, custom new target, method extraction and cross-realm behavior;
- all current Set composition methods and Map/collection additions present in the pin;
- `Map.groupBy`, `Object.groupBy` or related grouping APIs in the correct owning task/module when present.

Choose data structures that preserve insertion order and avoid accidental quadratic behavior. Hashing must work for every value kind while respecting identity and GC movement strategy.

The shared `Map.groupBy` / `Object.groupBy` compiler now carries its result
representation through the closed `GroupByResult` domain without equality
capability. Its two wrappers are the only producers, and all eleven semantic
decisions are direct exhaustive matches: seven diagnostics plus prototype
loading, result allocation, callback-key treatment and group storage. A future
result representation can no longer inherit Map or Object behavior from an
`if` / `else` default. The bounded structure target and finite Map-vs-Object
CLI witness are recorded in the
[GroupBy result-kind contract](../docs/rust-rewrite/contracts/group-by-result-kind.md).
They pass `3/3` and `1/1`. The completed shared checkpoint passes `2/2` in
717.58 seconds with 674 dumps, adds this witness plus the independent Promise
combinator Realm and Temporal overflow-options witnesses, removes none and
leaves all 671 retained dumps equal after accounting normalization. Broad
Test262 verification remains deferred.

The seven Set predicate and algebra methods now route through closed operation
domains without equality defaults. Predicate receiver and other iteration, and
algebra receiver iteration, each accept only their legal operation subset; the
public dispatchers construct those restricted values through exhaustive
matches. Algebra result initialization and receiver-iteration eligibility are
also exhaustive, while the all-operation other iterator retains a complete
four-arm match. A future operation must therefore select every independent
policy before the compiler builds, and an existing operation cannot enter an
invalid helper through a debug-only assertion. The bounded structure target and
finite two-size-direction CLI witness are recorded in the
[Set operation-domain contract](../docs/rust-rewrite/contracts/set-operation-domains.md).
They pass `4/4` and `1/1`. The shared 678-dump semantic golden passes `2/2` in
722.99 seconds, adds this witness plus the independent Array.fromAsync
callback-Realm, Object-policy and Promise-mode witnesses, removes none and
leaves all 674 retained dumps equal after accounting normalization. Broad
Test262 verification remains deferred.

## WeakMap and WeakSet

- Store keys through GC-supported weak/ephemeron edges, not strong references.
- Implement constructor iterable handling and all methods/branding/descriptors.
- Support every key category allowed by the pinned spec, including non-registered Symbols if applicable to the pin.
- Ensure entries disappear when keys become unreachable and values do not keep keys alive through cycles.
- Do not expose enumeration, size or deterministic collection timing.

## WeakRef and FinalizationRegistry

Implement:

- WeakRef construction, target validation, `deref` and keep-during-job semantics;
- FinalizationRegistry registration, unregister tokens, cleanup callbacks and holdings restrictions;
- cleanup jobs queued through T14 after GC discovers unreachable targets;
- correct behavior across realms, exceptions in cleanup callbacks and registry lifetime;
- host `gc()` as a real collection request without promising that every eligible object is immediately finalized.

Test support may repeat collection/job checkpoints, but it must not special-case known Test262 objects or force an otherwise-observable cleanup ordering.

## Integration requirements

- Symbols must participate in object shapes and `Reflect.ownKeys` without string conversion.
- Collection iterators use T15's common iterator protocol.
- GC tracing must retain strong collection entries and process ephemerons to a fixed point.
- Pending cleanup jobs and holdings must be rooted correctly.
- Proxies and wrappers retain object identity as keys.

## Acceptance criteria

- Full pinned Symbol, Map, Set, WeakMap, WeakSet, WeakRef and FinalizationRegistry trees are green.
- Symbol key ordering and coercion behavior pass across Object/Reflect/Proxy APIs.
- Map/Set iteration passes all mutation and reinsertion cases.
- Weak collections do not keep otherwise unreachable keys alive in GC stress tests.
- WeakRef keep-alive-within-job and finalization scheduling tests pass without deterministic shortcuts.
- Cross-realm well-known/registry/user-symbol identities are correct.
- No weak API is implemented by a permanent strong-reference table.

## Required tests

```sh
cargo test -p lila-aot-wasm symbol_ --quiet
cargo test -p lila-aot-wasm collection_ --quiet
cargo test -p lila-aot-wasm weak_ --quiet
cargo test -p lila-cli wasm_collection --quiet
./target/debug/lila test262 run built-ins/Symbol --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Map --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/Set --execution-backend wasm --timeout-ms 120000 --threads 4
```

Run weak-collection, WeakRef and FinalizationRegistry filters repeatedly with GC stress enabled, then rerun Object/Reflect/Proxy own-key tests.
