# Collection algorithm TypeError realms

## Scope

This contract owns the TypeErrors created directly by the Map, WeakMap, Set
and WeakSet constructor algorithms, plus the non-callable callback checks in
`Map.prototype.forEach` and `Set.prototype.forEach`. Receiver-brand failures
remain owned by the separate collection receiver contracts.

## Realm law

An algorithmic TypeError is created in the Realm of the active builtin
function. Borrowing a collection constructor or `forEach` method from another
Realm therefore creates that Realm's `%TypeError%`; it must not silently use
the entry Realm merely because the Wasm module stores entry intrinsics in
globals.

`CollectionAlgorithmTypeError` is the sole source domain for these errors. Its
payload is closed further by family:

- `MapConstructorTypeError` contains exactly the seven Map/WeakMap constructor
  failure stages;
- `SetConstructorTypeError` contains exactly the six Set/WeakSet constructor
  failure stages; and
- `StrongCollectionCursor` selects the two `forEach` callback failures.

Keeping the Map-only iterator-entry failure out of the Set domain makes that
impossible state unrepresentable. Exhaustive name and stage-suffix projections
assemble every preserved diagnostic during compilation. The sole emitter
consumes this domain and delegates to the current-function-realm TypeError
operation; constructor and `forEach` bodies must not call the entry-realm
runtime-error helper directly.

The created-realm Map and Set constructors are self-backed function objects:
their environment handle names the constructor itself and that object stores
the created Realm's `%TypeError.prototype%`. This metadata is part of the
realm law, not fixture setup. Entry-realm functions may still use the existing
zero-environment fallback to entry globals.

## Ordering and abrupt completion

The realm correction does not move any check. Each constructor still observes
`NewTarget`, the adder, the iterable method, the returned iterator, `next`,
each iterator result and (for Map/WeakMap) each entry in the existing order.
User-code throws from property lookup or calls continue to propagate before a
Lila-created algorithm TypeError. Iterator closing behavior is unchanged.

## Durable evidence

The focused Wasm fixture borrows the created Realm's Map and Set constructors
and both `forEach` methods. It covers every constructor failure stage across
the Map and Set algorithms, the Map-only non-object entry failure, both
requires-new identities, both `forEach` checks, and an abrupt getter control.
Every Lila-created failure must inherit from the borrowed builtin Realm's
`TypeError.prototype` and not the entry Realm's. The source-structure evidence
also pins that the WeakMap and WeakSet wrappers delegate to those same closed
constructor bodies.

Created realms do not yet publish WeakMap or WeakSet constructors. Their
cross-realm runtime witnesses therefore remain deferred until those intrinsics
exist; inventing fixture-only weak constructors would not test the product
Realm bootstrap.

The source-structure regression pins the closed domains, exhaustive diagnostic
projections, one typed emitter, fifteen source call sites (seven Map constructor,
six Set constructor and two `forEach`), and the absence of raw runtime-error
construction in those bounded bodies.

## Nonclaims

This seam does not add created-realm WeakMap or WeakSet intrinsics, change
successful collection construction or iteration, receiver branding,
SameValueZero, weak reachability, iterator closing, callback invocation, Proxy
behavior, GC, cleanup jobs, broad Test262 counts or T21 completion.
