# Strong collection iterator receiver-validation contract

## Scope

This contract owns receiver validation for
`%MapIteratorPrototype%.next` and `%SetIteratorPrototype%.next`. It covers the
required internal-slot brand, extraction of the iterator record and the realm
of a receiver-validation `TypeError`.

It does not own cursor movement or iterator result shape. Those remain governed
by [`ordered-collection-cursors.md`](ordered-collection-cursors.md).

## Semantic law

Calling either strong collection iterator `next` builtin validates `this`
before reading an iterator record:

1. a receiver whose ECMAScript Type is not Object throws a `TypeError`;
2. a Map iterator `next` receiver without the Map iterator internal slots
   throws a `TypeError`;
3. a Set iterator `next` receiver without the Set iterator internal slots
   throws a `TypeError`;
4. a valid receiver yields its iterator record to the shared strong collection
   cursor without changing that cursor's state or successful result behavior.

Arrays, functions and arguments objects have ECMAScript Type Object, so they
take the missing-internal-slots route rather than the non-object route. Their
runtime layouts do not authorize a read from the ordinary Object internal-brand
offset. A Proxy likewise never inherits an iterator target's internal slots:
an Object-tag Proxy has the ordinary brand-layout position set to the unbranded
value and takes the missing-slots route without unwrapping its target, invoking
a trap or consulting revocation state. This holds for both live and revoked
Proxies.

The error is created in the active builtin function's defining realm. Borrowing
another realm's `next` and applying it to an invalid receiver therefore produces
an instance of that realm's `%TypeError%`, not the caller's `%TypeError%`.

The pinned receiver-shape evidence is:

- `MapIteratorPrototype/next/this-not-object-throw-*.js` and
  `SetIteratorPrototype/next/this-not-object-throw-*.js`;
- `MapIteratorPrototype/next/does-not-have-mapiterator-internal-slots*.js` and
  `SetIteratorPrototype/next/does-not-have-mapiterator-internal-slots*.js`.

Those tests establish the error category. The product regression
`wasm_collection_iterator_receiver_realm.js` additionally establishes exact
category messages, defining-realm provenance, safe Array/Function/Arguments
classification and Proxy trap non-observation because the pinned files do not
create another realm or distinguish the backend layouts.

## Rust invariant

`StrongCollectionCursor` is the closed Map/Set receiver domain. Exhaustive
methods on it select the required iterator brand and preserve the builtin's
existing receiver error messages.

`StrongCollectionIteratorReceiverError` is the closed receiver-failure domain:
`NonObject | MissingInternalSlots`. One receiver-record emitter consumes both
closed domains and is the only validation path used by the Map and Set iterator
`next` emitters. It creates failures through the current-function-realm
`TypeError` helper and loads the boxed iterator record only after the matching
brand is established.

`StrongCollectionIteratorReceiverRepresentation` exhaustively classifies every
`ValueKind`: only `Object` uses `ObjectTagBrandLayout`; `Array`, `Function` and
`Arguments` use `ObjectWithoutBrandLayout`; primitive kinds use `NonObject`;
and the compile-time-only `Dynamic` kind uses `NonRuntime`. Its closed generated
dispatch is the receiver-record helper's layout authority. Only the
`ObjectTagBrandLayout` arm may load the ordinary internal brand, while the
object-without-brand-layout arm routes directly to `MissingInternalSlots`.
Heap-backed BigInt's extra runtime tag falls through to the primitive
`NonObject` default.

Adding another `ValueKind`, receiver representation, strong collection
iterator or validation failure must therefore update exhaustive Rust matches
before the backend builds. Map and Set cannot silently drift into different
layout reads, brand checks, messages or error-realm behavior through duplicated
control flow.

## Non-claims

This contract does not change successful cursor advancement, tombstone
handling, exhaustion, persisted kind words or iterator result objects. It does
not make every collection builtin error realm-correct, establish full pinned
Map/Set coverage, or alter the weak-reachability blocker owned by T05/T14.
