# Collection data receiver-validation contract

## Scope

This contract owns receiver validation for the ordinary strong and weak
collection prototype builtins that require one of `[[MapData]]`,
`[[WeakMapData]]`, `[[SetData]]` or `[[WeakSetData]]`. It does not own
`%MapIteratorPrototype%.next` or `%SetIteratorPrototype%.next`; those keep their
distinct iterator domain under
[`collection-iterator-receiver-validation.md`](collection-iterator-receiver-validation.md).

The source inventory is closed at eighteen receiver-helper consumers, which
compile thirty-six builtin identities:

- nine Map-family consumers: `clear`, `delete`, `get`, `getOrInsert` and
  `getOrInsertComputed`, `has`, `forEach`, `set`, `size`, and iterator creation;
- nine Set-family consumers: `add`, `clear`, `delete`, `has`, the three set
  predicates, the four set-algebra methods, `forEach`, `size`, and iterator
  creation.

Those consumers expand to twelve Map, six WeakMap, fifteen Set and three
WeakSet builtin identities. `Set.prototype.keys` is the same function object as
`Set.prototype.values`, so it introduces no separate compiled identity.

## Semantic law

ECMA-262's collection algorithms apply `RequireInternalSlot` to their `this`
value before using the collection record. `RequireInternalSlot` distinguishes
two failures:

1. a value whose ECMAScript Type is not Object throws a `TypeError` because it
   is not an object;
2. an Object without the required collection slot throws a `TypeError` because
   that slot is missing.

Arrays, functions and arguments objects have ECMAScript Type Object even
though the backend gives them distinct runtime tags. They must therefore take
the missing-slot route, never the non-object route. An ordinary Object-tag
Proxy also lacks its target's internal slots: validation does not unwrap the
target, inspect revocation state or invoke a Proxy trap. Both a live Proxy over
a compatible collection and a revoked Proxy take the missing-slot route.

Inline and heap-backed BigInt values both have ECMAScript Type BigInt and take
the non-object route. The heap-backed representation's runtime-only tag is not
a license to read an object header.

The error is created in the active builtin function's defining realm.
Borrowing another realm's collection method and applying it to an invalid
receiver therefore produces that realm's `%TypeError%`, not the caller's.

Pinned Test262 exposes the slot distinction through the collection
`prototype/*/this-not-object-*` and
`prototype/*/does-not-have-*-internal-slot-*` families. In particular, the
Map, Set, WeakMap and WeakSet files explicitly use Arrays as missing-slot
receivers. The product regression
`wasm_collection_data_receiver_realm.js` additionally distinguishes backend
Array, Function and Arguments layouts, both BigInt representations, and
non-observation of live and revoked Proxies across all four families. Its
created-realm Map and Set methods prove defining-realm provenance at runtime.
The separate created-Realm weak-collection fixture now supplies the matching
WeakMap and WeakSet runtime evidence in both borrowing directions; the shared
error emitter and private publication lifecycle are recorded in
[`weak-collection-created-realm-publication.md`](weak-collection-created-realm-publication.md).

## Rust invariant

`CollectionDataReceiverKind` is the closed ordinary collection slot domain:
`Map | WeakMap | Set | WeakSet`. Exhaustive matches select the required brand
and both receiver error messages. `MapCollectionKind` and `SetCollectionKind`
must project to that domain before a record can be loaded. Their constructor
brand accessors delegate to the same projection, and other strong Map/Set
allocation sites name a `CollectionDataReceiverKind` rather than spelling a
brand constant. A source-structure regression pins each of the four data-brand
constants to exactly one mapping authority.

`CollectionReceiverRequirement` preserves the semantic split between an
ordinary collection data slot and the existing strong iterator cursor while
allowing both to use one representation-safe validator. The validator accepts
only a requirement and a destination record local; there is no raw brand or
message parameter that a caller can mismatch.

`CollectionReceiverRepresentation` is the one shared runtime-layout domain:

- `ObjectTagBrandLayout` may read `HEAP_OBJECT_INTERNAL_BRAND_OFFSET`;
- `ObjectWithoutBrandLayout` is an ECMAScript Object but routes directly to
  the missing-slot failure;
- `NonObject` routes to the non-object failure;
- `NonRuntime` traps the compile-time-only `Dynamic` tag.

Its exhaustive `ValueKind` table maps Object to the brand layout;
Array, Function and Arguments to the object-without-brand-layout route; and
all primitive kinds to non-object. The validator starts in `NonObject`, so the
runtime-only heap BigInt tag cannot fall into an object layout. A valid brand
is checked before the boxed record payload is loaded.

Adding another `ValueKind`, receiver representation, ordinary collection
slot, strong collection iterator or receiver failure must update exhaustive
Rust matches before the backend builds. The four ordinary families and two
iterator families cannot silently drift into different tag classification,
brand loads, messages or error-realm behavior through duplicated control flow.

## Non-claims and deferred gates

This seam does not change successful collection algorithms, key equality,
mutation order, iterator cursor movement, weak reachability or cleanup jobs. It
does not own created-Realm bootstrap; WeakMap and WeakSet publication is the
separate boundary linked above. It does not claim every collection error path
is covered, that the full pinned collection trees are green, or that T21 is
complete.

The focused CLI fixture passes on the current working tree, alongside the
bounded source-structure checks and independent review. At the 2026-08-31
created-Realm weak-collection checkpoint, formatting is green and the broad
backend library target retains its same seven unrelated baseline failures at
`367/374`. The receiver-specific `collection_` target, bounded CLI
collection/iterator shard, focused pinned receiver filters and broad Test262
aggregate were not rerun for this documentation update.
