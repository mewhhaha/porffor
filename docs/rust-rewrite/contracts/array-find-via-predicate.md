# Array and TypedArray `FindViaPredicate`

## Semantic boundary

ECMA-262's
[`FindViaPredicate`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-findviapredicate)
abstraction has one four-way surface:
`find`, `findIndex`, `findLast` and `findLastIndex`. The method selects an
ascending or descending index walk and projects either the found value or its
index. Array and TypedArray entry points differ in receiver preparation, but
they share the predicate contract and the four direction/projection pairings.

For the generic Array methods, `ToObject(this)` and `LengthOfArrayLike` happen
before the predicate is tested with `IsCallable`. For the TypedArray methods,
`ValidateTypedArray` and its length snapshot happen before that test. Each
visited element is then passed to `Call(predicate, thisArg, « value, index,
receiver »)`. A callable Proxy is callable because its target is callable; a
revoked callable Proxy still passes `IsCallable`, then throws from its Proxy
`[[Call]]` when invoked. That callability shape is installed by
[`ProxyCreate`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-proxycreate).

The generic Array emitter previously represented this operation with three
booleans. One boolean selected a TypedArray-only branch that had no caller,
while the live generic branch accepted only the internal Function tag. That
rejected callable Proxies before `Call` and left the live Array methods with a
different predicate boundary from their TypedArray counterparts.

## Closed compiler shape

`FindViaPredicateKind` is the sole four-kind selector shared by Array and
TypedArray dispatch. Exhaustive projections map every kind to:

| kind | direction | successful projection |
| --- | --- | --- |
| `Find` | ascending | value |
| `FindIndex` | ascending | index |
| `FindLast` | descending | value |
| `FindLastIndex` | descending | index |

The same exhaustive kind also supplies the exact Array and TypedArray method
names and predicate errors. Result initialization, index initialization,
index advancement and successful-result projection accept the closed
direction or projection types rather than independent booleans. Adding a
fifth kind therefore fails exhaustive-match compilation until all semantic
projections are defined; the exact-inhabitant and dispatcher structural gates
also reject any unmatched surface expansion.

Predicate validation has one private constructor. It loads argument zero and
emits the general `IsCallable` operation only after the owning entry point has
finished its receiver and length observations. Its result is a private,
non-`Copy` `ValidatedFindPredicateLocals` value. The only consumer takes that
value by ownership and emits the Proxy-aware `Call` path with the exact
`thisArg` and three arguments. Raw tagged locals cannot be passed to this call
boundary, and one validated value cannot be silently reused for another
emitted call.

The generic entry retains its borrowed-TypedArray observation path. That path
implements generic `LengthOfArrayLike` and per-index integer-indexed reads; it
does not become the validated `%TypedArray%.prototype%` entry rule.

## Durable evidence

The existing forward and reverse Array fixtures exercise all four methods
with callable Proxies. They pin the apply-trap target, exact `thisArg`, three
argument positions, receiver identity, ascending or descending visit order,
call count, and value-versus-index projection. They cover both a non-callable
Proxy, which fails the `IsCallable` gate, and a revoked callable Proxy, whose
specified failure occurs in Proxy `[[Call]]`.

Rust mapping tests fix all four direction/projection/name/error projections.
Bounded source-structure tests keep both dispatcher families on the shared
enum, keep the predicate wrapper private and non-`Copy`, and pin the sole
validator and Proxy-aware consumer.

## Nonclaims

This seam does not complete the other Array callback families, Array exotic
descriptors, species behavior, Proxy receiver traps, cross-realm callbacks or
full Array/Test262 conformance. It does not remove a Test262 materializer,
change published conformance counts, or establish runtime verification; the
expensive Cargo and Test262 gates remain deferred to the coordinated batch
checkpoint.
