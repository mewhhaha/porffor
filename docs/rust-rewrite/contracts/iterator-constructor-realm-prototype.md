# Iterator constructor realm prototype

Status: focused-verified on 2026-08-24 for the Wasm-AOT constructor-Realm
fallback seam.

## Scope

This contract spans the closed realm-prototype selection and generic construct
routing in `crates/lila-aot-wasm/src/functions.rs`, the Iterator body in
`crates/lila-aot-wasm/src/builtins/standard.rs`, the structural guards in
`crates/lila-aot-wasm/src/lib.rs`, and the focused CLI fixture/test. Existing
bootstrap and created-realm publication remain verified inputs rather than heap
layout changes.

`Iterator` creates its result with
`OrdinaryCreateFromConstructor(NewTarget, "%Iterator.prototype%")`. The shared
`GetPrototypeFromConstructor` rule is observable when
`NewTarget.prototype` is not an object: the fallback is the
`%Iterator.prototype%` intrinsic of `GetFunctionRealm(NewTarget)`, not the entry
realm's Iterator prototype.

The normative algorithms are
[`Iterator ( )`](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-iterator)
and
[`GetPrototypeFromConstructor`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor).
The pinned case is
`built-ins/Iterator/proto-from-ctor-realm.js`.

The 2026-08-13 current-pin Wasm-AOT baseline passed seven of the eight tests in
the `built-ins/Iterator` matrix leaf. That case was its exact failure: the first
observable primitive fallback selected the entry-realm prototype, while the
case requires the other-realm Iterator prototype for six primitive values.

## Closed fallback domain

`OrdinaryDefaultPrototype` is the closed domain of ordinary-object intrinsic
defaults consumed after `GetFunctionRealm`. `Iterator` is a member alongside
Object, String, Number, Boolean and Date and maps to exactly one realm-intrinsic
slot. Array remains separate because `%Array.prototype%` has an Array layout and
tag.

The generic new-target prototype operation has a distinct
`RequiredResolvedRealmOrdinary` policy. The Iterator constructor's
primitive-prototype arm must:

1. perform the observable `Get(NewTarget, "prototype")` exactly once;
2. resolve the original new target's function realm only after that Get returns
   a primitive;
3. route abrupt, revoked and invalid realm results before exposing a realm local;
4. load the selected required realm slot, trapping missing bootstrap state; and
5. install the prototype payload together with its Object representation tag.

It may not use the generic `CurrentGlobal`, optional realm-slot/global fallback
or function-snapshot policies. An object-valued `NewTarget.prototype` still wins
without consulting the fallback. Its representation tag travels with its
payload into allocation so Object, Function and Array prototypes keep their
exact identity and behavior.

The shared `[[Construct]]` dispatcher classifies Iterator as a direct-returning
constructor. Its dispatch invokes the Iterator body and leaves the generic
construct block before that generic path reads `NewTarget.prototype` or
preallocates a receiver. This makes the body the sole owner of the observable
Get, fallback resolution and allocation; without that routing, the generic path
and body would perform the operation twice.

## Storage and publication

The realm-intrinsics record already owns and publishes the
`%Iterator.prototype%` pointer for both producers:

- entry-realm bootstrap publishes `ITERATOR_PROTOTYPE_GLOBAL_INDEX`; and
- `$262.createRealm()` publishes its newly allocated Iterator prototype.

A missing slot in a resolved realm is an internal bootstrap invariant failure,
not permission to substitute the entry global. This seam therefore changes no
heap layout, bootstrap protocol or host-realm record.

The supported empty-function materialization used by the focused fixture is a
prerequisite to that lookup. A created-Realm `%Function%` is self-backed, and
the zero-argument function it creates inherits the active constructor's
defining Realm. The durable source guard pins both transitions so an
other-Realm function cannot silently report the entry Realm to
`GetFunctionRealm`. Dynamic source generation remains unsupported.

## Observable regression

The durable CLI fixture first proves the created Realm's Iterator prototype is
distinct from the entry intrinsic, then repeats the pinned six-value primitive
matrix through an other-realm function and checks exact prototype identity.
Bound and nested Proxy new targets require `GetFunctionRealm` to traverse to
that same function Realm after the observable Get; both Proxy traps observe
exactly one `prototype` read. Object-, Function- and Array-valued custom
prototypes pin representation-tag preservation. A separate Proxy getter pins
one observable `prototype` read, a thrown getter pins abrupt completion before
allocation, and a revocable function Proxy pins the required
`GetFunctionRealm` TypeError route after the getter returns a primitive.

The structural guards bound the shared new-target prototype operation, Iterator
constructor arm and shared construct dispatcher. They require the observable
prototype Get before the required-Realm arm, require that arm to resolve the
original new-target payload and tag without a global fallback, require exactly
one typed required-Realm selection and one tagged allocation in the body,
reject the legacy payload-only helper, `CurrentGlobal` selection and
payload-only allocator there, and pin exactly one direct-returning Iterator
membership before the generic prototype Get and receiver allocation. The entry
and created-Realm publication guards name the Iterator slot and the exact realm
and prototype locals, rather than accepting an unbounded store-helper call.

## Focused verification

The implementation and strengthened witnesses are independently source-audited.
On 2026-08-24, `node --check` passed for the CLI fixture, the exact structural
test passed `1/1`, the exact CLI runtime test passed `1/1`, and the unflagged
`built-ins/Iterator/proto-from-ctor-realm.js` leaf passed both sloppy/strict
Wasm-AOT variants (`2/2`). Parser, early-error, lowering, runtime, Wasm-backend,
host-harness, unsupported, not-implemented, crash and bug buckets were all
zero. The focused commands were:

```sh
cargo test -p lila-aot-wasm --lib \
  tests::iterator_constructor_realm_prototype_is_required_tagged_and_published \
  -- --exact --test-threads=1
cargo test -p lila-cli --test cli -- \
  --exact iterator::run_wasm_backend_uses_new_target_realm_for_iterator_prototype \
  --test-threads=1
./target/debug/lila --jobs 1 test262 run \
  built-ins/Iterator/proto-from-ctor-realm.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --timeout-ms 180000 --threads 1
```

The final current-SHA closure remains the complete T15 ladder and full low-RAM
current-pin publication path.

## Non-claims

This seam does not implement general generator suspension, `yield*`,
`IteratorClose`, `AsyncIteratorClose`, iterator-helper close behavior, explicit
resource disposal or GC validation. It does not claim that the complete
Iterator tree is green. It changes Iterator construction's primitive
`NewTarget.prototype` fallback, preserves the exact representation tag of an
explicit Object, Function or Array prototype through result allocation, and
routes Iterator directly to that owning body instead of pre-running generic
construction. It does not implement dynamic `Function` source parsing or claim
complete `Function` constructor semantics; only the already-supported empty
function's active defining-Realm lifecycle is part of this checkpoint.
