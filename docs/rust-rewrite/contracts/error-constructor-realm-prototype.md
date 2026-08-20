# Error constructor realm prototype

`Error` creates its result with
`OrdinaryCreateFromConstructor(NewTarget, "%Error.prototype%", ... )`.  The
observable `Get(NewTarget, "prototype")` is performed exactly once.  When that
value is primitive, `GetPrototypeFromConstructor` selects `%Error.prototype%`
from `GetFunctionRealm(NewTarget)`; a per-function cache and the entry-realm
global are not semantic substitutes.

The normative algorithms are
[`Error ( message [ , options ] )`](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-message)
and
[`GetPrototypeFromConstructor`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor).
The pinned regression is
`built-ins/Error/proto-from-ctor-realm.js`.

The 2026-08-13 low-RAM Wasm-AOT baseline, produced by an older compiler
artifact while this batch was being dry-written, reported 56 of 58 cases
passing in the `built-ins/Error` leaf.  One failure was the pinned constructor
realm case; the other was separately accounted dynamic Function-source debt.
That artifact is evidence for selecting this seam, not current-SHA closure.

## Closed fallback and allocation

`OrdinaryDefaultPrototype` is the closed set of ordinary-object intrinsic
fallbacks.  Its `Error` member maps to one required realm-intrinsics slot.
After a primitive prototype result, the shared
`RequiredResolvedRealmOrdinaryActive` transition must:

1. resolve the original new target with `GetFunctionRealm`;
2. route revoked or invalid function-realm outcomes before exposing a realm;
3. load the required `%Error.prototype%` slot, treating missing publication as
   an internal bootstrap invariant failure; and
4. install the payload and its Object representation tag together.

The Error constructor owns a private, non-`Copy`, `must_use` prototype
witness.  The only Error-instance allocator accepts that witness, and allocates
through the tagged prototype operation.  An explicit Object-, Function-,
Array- or Arguments-valued `NewTarget.prototype` therefore retains both
identity and representation; only the realm intrinsic itself is known to have
the ordinary Object tag.

Created-realm Error constructors store their own function identity in the
builtin environment slot.  The Error-specific required fallback policy uses
that active function as the effective new target when `NewTarget` is absent,
then performs the same observable `Get(activeFunction, "prototype")` as the
explicit-new-target path. Entry builtins select their entry constructor through
the zero-environment path. This keeps `other.Error()` in `other`'s realm and
preserves the common abstract-operation shape even though the intrinsic's own
`prototype` property is immutable, without weakening constructors whose
undefined-NewTarget behavior is different.

The generic construct dispatcher treats `Error` as a direct-returning
constructor.  Its builtin body already performs allocation and returns the
created Error object.  Letting the generic base-constructor path preallocate
would perform an additional observable prototype read before the body and
would use `%Object.prototype%` as the primitive fallback.  The direct route
makes the Error body the sole owner of `OrdinaryCreateFromConstructor`.

## Storage and publication

The realm-intrinsics record owns one pointer slot for `%Error.prototype%`.
Record size, layout metadata and the exhaustive
`NonArrayRealmIntrinsicSlot` mapping move together.  Both realm producers
publish the slot:

- entry bootstrap publishes `ERROR_PROTOTYPE_GLOBAL_INDEX`; and
- `$262.createRealm()` publishes its `error_prototype_local`.

The existing per-function native-error prototype snapshots remain in use by
other native-error constructors and current-function-realm throw helpers.
They are not consulted by this Error fallback.  Migrating the six native Error
subclasses, AggregateError and SuppressedError to required realm slots is
separate T24 work.

## Observable regression

The durable CLI fixture checks the primitive fallback against another realm,
direct call through a created-realm active Error function,
Object/Function/Array/Arguments custom prototype identity, Error branding and
message installation, one Proxy `prototype` read, exact abrupt propagation,
object-valued prototype use after revocation, and the revoked primitive
fallback TypeError. `Error.prototype` is an immutable own data property, so
the active intrinsic's otherwise unobservable `Get` transition is pinned by
the source-structure contract rather than a fictitious accessor mutation. The
same structure check pins one direct constructor dispatch, one required
resolved-realm selection and the absence of payload-only allocation in the
Error instance allocator.

## Deferred gates

The implementation batch performs only static source, formatting and diff
checks while the low-RAM current-pin matrix owns Cargo and Test262 resources.
After that runner releases them, verification must include:

```sh
cargo test -p lila-aot-wasm error_constructor_realm_ --quiet
cargo test -p lila-cli run_wasm_backend_uses_new_target_realm_for_error_prototype --quiet
./target/debug/lila test262 run built-ins/Error/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
```

The final current-SHA evidence remains the complete T24 native-error ladder
and the full low-RAM Wasm-AOT publication path.

## Non-claims

This seam does not implement dynamic Function source, complete the other
native-error constructor fallbacks, change Error property descriptors or
ordering, add `stack`, close cross-realm throw construction, or claim the
`built-ins/Error` tree or T24 complete.  It does not refresh snapshots or the
README.  Runtime and current-pin results remain unclaimed until the deferred
gates actually run.
