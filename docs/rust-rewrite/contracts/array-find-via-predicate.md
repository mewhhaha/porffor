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

`builtins/array/find_via_predicate.rs` is the sole owner of the shared Array and
TypedArray compiler family. `FindViaPredicateKind` is its four-kind selector;
exhaustive projections map every kind to:

| kind | direction | successful projection |
| --- | --- | --- |
| `Find` | ascending | value |
| `FindIndex` | ascending | index |
| `FindLast` | descending | value |
| `FindLastIndex` | descending | index |

The capability-free `FindViaPredicateKind` implements no clone, copy, debug,
default, comparison, ordering or hashing capability. Eight fixed entries move
exactly one kind into the private Array or TypedArray compiler, and that
compiler borrows the same authority through all seven exhaustive projections.
Standard dispatch cannot name the kind or either raw compiler. Duplicating the
kind, splitting its surface decisions across copied values or collapsing it
through equality no longer compiles.

The capability-free `FindDirection` is private and similarly owns the selected
ascending or descending traversal without clone, copy, debug, default,
comparison, ordering or hashing capability. Each compiler produces one
direction from its `FindViaPredicateKind`, then borrows that same authority for
both index initialization and index advancement. Initialization and advance
cannot be selected from independently copied directions.

The capability-free `FindProjection` is private and owns whether the method returns a
found value or its index without clone, copy, debug, default, comparison,
ordering or hashing capability. Each compiler produces one projection from its
`FindViaPredicateKind`, then borrows it for both the miss-result initialization
and successful-match projection. The default and success result shapes cannot
drift through independently copied policies.

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
enum, require all seven projections to borrow the capability-free kind, pin the
exact kind-to-direction mapping, require both direction consumers and all four
call sites to borrow one owned direction, pin the exact kind-to-projection
mapping, require both result-shape consumers and all four call sites to borrow
one owned projection, reject derived or manual capabilities, keep the predicate
wrapper private and non-`Copy`, and pin the sole validator and Proxy-aware
consumer.

After erasing only the new `&projection` call-site markers, the compiler bodies
reproduce the frozen Batch X raw hashes
`ece6c116f388ab7ca262b90d55ff58529e85a2d5ef5c2abfa0610c790ad797c9`
for TypedArray and
`21a37e37281c0528d4148d935f56196c14f2e58716e0784a9eea12960dbc136f`
for Array. Erasing both projection and direction borrow markers reproduces the
earlier semantic-body hashes
`5aaece4591126bfc317affcc137762a7f00bba4288ce5f8cd8e93dc6331fa32e`
for TypedArray and
`9f54a114dbee477e0c430d03e54159cd3a452247ac3f58a17969fdbf54622103`
for Array. The fully borrowed raw-source hashes are
`40be1db2dd3ccb1f35a9e022061f4fb23a8adc8fac8e446f06fdb93879b3e92d`
and `b71e9cfcea61c77cdbef9aeb68917c65e1e54ab1bbe735e49a4175d82f00673e`.
The unchanged kind-to-direction projection is
`ff78990936edbc59ba6caec6fc58a107f7ee318714ee3f5381adad04d35a866a`.
The borrowed initialization and advance consumers normalize to their frozen
instruction-arm hashes
`e33ff2bad904f64e169937a5cfa2eaf34e37cd79faf6e44bfc4e76f23438288e`
and `d2be00944522054e0575e4cde514488767125adc14e0cb2551476eb65dbb8259`.
The unchanged kind-to-result projection is
`bec47d1927099f9da9b358f71c884f90c28c5e24901ade28e3e122d2564db23a`.
The borrowed default-result and successful-match consumers normalize to their
frozen instruction-arm hashes
`09f3b0deba372b7c4a5af87d28d3ae9748686f4f852d29eeb25e8b2e6d513a78`
and `2b97905a18c0a0d42b705b6664002ef9fb5dbb1c00c614e1200d429182e50af5`.
The eight standard mappings remain byte-identical at
`13b2e609dd878f19762612dad1851febd9390c21b4bca021c3f41c71908ff1a8`.
The capability-hardened child module is
`59072414dbc8488ce29feb46271997f6bc9ad8ba65fafb3dc287c96a4a48157b`.
At the 2026-08-28 Batch X checkpoint, `cargo xc` is green, the strengthened
structure target passes `5/5`, and the exact Array, reverse Array and TypedArray
CLI controls pass `3/3`. The three pinned Wasm-AOT leaves pass all `5/5`
executions, covering resizable-buffer observation, strict callback `this` and
abrupt length completion, with every failure bucket at zero.
At the 2026-08-28 Batch Y checkpoint, projection hardening passes the same
structure target `5/5` and the same exact Array, reverse Array and TypedArray CLI
controls `3/3`. Its four projection-focused Wasm-AOT leaves pass all `8/8`
executions with every failure bucket at zero, and the shared `cargo xc`
checkpoint is green.

Batch BF makes `FindViaPredicateKind` and both raw family compilers private to
the child owner. The catalog can call only eight fixed entries, one for every
Array/TypedArray and find/findIndex/findLast/findLastIndex pairing. Restoring
only former visibility produces the exact original six-line kind selection
with SHA-256
`3989f2ebe1ce925d23b20d4e06eb35f00e1e840f7509b8226b9b425a639c4e5c`.
Restoring the former names and visibility of the 188-line TypedArray and
310-line Array raw compilers reproduces SHA-256
`40be1db2dd3ccb1f35a9e022061f4fb23a8adc8fac8e446f06fdb93879b3e92d`
and
`b71e9cfcea61c77cdbef9aeb68917c65e1e54ab1bbe735e49a4175d82f00673e`.
At the Batch BF checkpoint, `cargo xc` is green, the structure target passes
`5/5`, and the exact forward Array, reverse Array and TypedArray controls pass
`3/3`. Formatting, module-boundary, task-plan and shortcut gates are green.
This source-equivalent hardening has no new Array behavior and does not close T16.

## Nonclaims

This seam does not complete the other Array callback families, Array exotic
descriptors, species behavior, Proxy receiver traps, cross-realm callbacks or
full Array/Test262 conformance. It does not remove a Test262 materializer,
change published conformance counts, or establish a broader Array/Test262
baseline. Semantic snapshot and broad conformance verification remain
deferred.
