# Collection prototype `@@toStringTag` descriptors

## Scope

This contract owns the `Symbol.toStringTag` data properties installed on
`Map.prototype`, `Set.prototype`, `WeakMap.prototype` and
`WeakSet.prototype`. It does not own collection instance brands, receiver
validation or `Object.prototype.toString` itself.

## Descriptor law

Each of the four intrinsic prototype objects has one own data property keyed by
the realm's well-known `Symbol.toStringTag`. Its value is the matching family
name (`"Map"`, `"Set"`, `"WeakMap"` or `"WeakSet"`) and its attributes are
exactly:

- `[[Writable]]: false`;
- `[[Enumerable]]: false`; and
- `[[Configurable]]: true`.

The property is installed after the family's existing prototype methods and
accessors. This preserves their current observable property order while making
the tag the final collection-prototype property installed by the family.

## Typed installation authority

`CollectionPrototypeIntrinsic` is the closed four-family domain. Exhaustive
matches derive the prototype global, Realm intrinsic slot and tag value from
the same variant. The created-Realm WeakMap/WeakSet materializer therefore
cannot pair its family tag with the other weak family's Realm slot through
independent raw constants. Entry-Realm installers borrow the authority to load
the matching prototype global; the created-Realm weak-collection materializer
borrows it to select the Realm slot. Every producer then moves the authority
exactly once into the descriptor emitter. Entry bootstrap and created-Realm
Map/Set slot storage remain outside this Realm-slot projection. The domain
derives no cloning, copying, formatting, equality or default capabilities.

One emitter accepts that domain and owns the well-known-symbol key, String
value tag and descriptor flags. The four entry-Realm installers and the two
created-Realm weak-collection producers each call it exactly once. Adding a
fifth variant requires all exhaustive projections to be updated before the
backend builds; duplicating a raw collection tag descriptor outside the
authority is rejected by the structural regressions.

The helper reserves its temporary locals after the caller's retained
prototype/method locals and releases them in reverse order. It therefore
preserves the backend's local-stack discipline.

## Durable evidence

The source-structure regression recursively pins the ten domain mentions,
bans manual capability implementations, binds the six producers, checks the
three exact four-row projection tables, and preserves each producer's borrowed
projection before its single consuming emitter call. It also pins the exact
descriptor flags, property order and reverse local release. The entry-Realm CLI
fixture checks all four values and descriptors, plus the resulting built-in
object tags. The created-Realm weak-collection fixture checks the matching
WeakMap and WeakSet properties and their constructor-before-method order. The
four exact `prototype/Symbol.toStringTag.js` leaves remain the focused entry-Realm
pinned-suite witnesses. The original structure target passes `2/2`; the
created-Realm publication target passes `6/6`, and both exact CLI owners pass.
The Map, Set, WeakMap and WeakSet leaves pass all `8/8` sloppy/strict Wasm-AOT
executions, with every failure bucket at zero.

## Nonclaims

This authority extension does not change collection algorithms, iterator
closing, SameValueZero, Proxy behavior, weak reachability, GC or cleanup jobs.
Created-Realm WeakMap and WeakSet publication is owned by
[`weak-collection-created-realm-publication.md`](weak-collection-created-realm-publication.md).
No full collection-tree, T21-completion or published aggregate claim follows
from the descriptor authority alone.
