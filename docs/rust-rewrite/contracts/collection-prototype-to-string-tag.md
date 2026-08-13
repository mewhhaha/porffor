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
matches derive both the prototype global and the tag value from the same
variant, so an installer cannot pair the Map prototype with the WeakMap
spelling through independent raw constants.

One emitter accepts that domain and owns the well-known-symbol key, String
value tag and descriptor flags. The Map, Set, WeakMap and WeakSet installers
each call it exactly once. Adding a fifth variant requires both exhaustive
projections to be updated before the backend builds; duplicating a raw
collection tag descriptor outside the authority is rejected by the structural
regression.

The helper reserves its temporary locals after the caller's retained
prototype/method locals and releases them in reverse order. It therefore
preserves the backend's local-stack discipline.

## Durable evidence

The source-structure regression pins the closed variants, exhaustive
prototype/tag projections, one descriptor emitter, exact flags, reverse local
release and one call from each installer. The CLI fixture checks all four
values and descriptors, plus the resulting built-in object tags.

Once the resource-bounded matrix releases Cargo and Test262, verification must
run the focused structure/CLI regressions, the four pinned
`prototype/Symbol.toStringTag.js` cases, and the adjacent
`Object.prototype.toString` built-in-tag cases. Static gates alone do not claim
runtime closure.

## Nonclaims

This seam does not change collection construction, `NewTarget` prototype
fallbacks, method behavior, iterator closing, SameValueZero, Proxy behavior,
weak reachability, GC or cleanup jobs. It does not make any full collection
tree green, complete T21, refresh the aggregate or justify a README status
change.
