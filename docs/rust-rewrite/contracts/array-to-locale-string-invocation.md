# Array and TypedArray `toLocaleString` element invocation

## Semantic boundary

ECMA-262 specifies
[`Array.prototype.toLocaleString`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.tolocalestring)
by applying
[`Invoke(element, "toLocaleString")`](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-invoke)
to every non-nullish element. `Invoke` first performs `GetV` and then calls the
result with the exact original element as `this` and an empty argument list.
The distinct
[`%TypedArray%.prototype.toLocaleString`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.tolocalestring)
method validates its receiver and obtains its length differently, then uses the
same element algorithm.

`Call` applies the general `IsCallable` operation. A callable Proxy therefore
reaches Proxy `[[Call]]`; a non-callable value throws a TypeError before any
call. That TypeError is created in the current Realm Record of the running
built-in function. Borrowing a created realm's Array or TypedArray method must
therefore throw that realm's TypeError, not the entry script's TypeError.

The shared emitter already preserved the original element across the temporary
object used for `GetV`, and its call path admitted callable Proxies. Its local
non-callable branch, however, constructed a process-entry-realm TypeError. The
error was observably wrong only when a foreign realm's method was borrowed.

## Closed compiler shape

`ToLocaleStringReceiverKind` has exactly two inhabitants: the generic
Array-like entry and the validated TypedArray entry. Exhaustive projections
supply their method names and their element-method-not-callable messages.

After `GetV`, the general `IsCallable` gate is emitted by one validator. The
validator's success value is a private, non-`Copy`
`ValidatedToLocaleStringInvocationLocals` token containing both the exact
tagged method and the exact original element receiver. Its failure path always
uses the current-function-realm TypeError helper.

The token's only consumer takes ownership and emits the Proxy-aware call with
the token's receiver and an empty argument list. Raw method and receiver locals
cannot be passed to that boundary independently, so a later refactor cannot
silently validate one value and call another or substitute the temporary boxed
lookup target for the original receiver.

## Durable evidence

The focused CLI fixture exercises main- and foreign-realm Array methods with a
non-callable element method, and the foreign TypedArray method through its
realm-local Number prototype. It also fixes the successful boundary with a
callable Proxy, exact receiver identity and zero arguments, and proves that a
revoked Proxy retains a callable `typeof` shape before the invocation throws.
Together with the structural assertion that the validator uses the general
`IsCallable` helper, that case proves the throw comes from revoked Proxy
`[[Call]]` rather than an early representation-specific validator rejection.

A bounded Rust source-structure test keeps the receiver domain closed, the
token private and non-`Copy`, and the validator and consuming call boundary
unique. It rejects the entry-realm error helper in the validator, raw call or
callability operations in the shared loop, a non-empty argument list, and a
call that precedes validation.

## Baseline disclosure and nonclaims

The available `built-ins/Array/prototype` baseline predates the T10
`Object.prototype.toLocaleString` repair. It has two remaining
`toLocaleString` failures for primitive elements because the old Object path
used its boxed lookup target as the getter and call receiver. T10 now
statically preserves the original primitive through GetV and Proxy-aware Call;
focused runtime and pinned Test262 execution remain deferred, so this carries
no current-SHA baseline-delta or full-subtree-green claim.

This change does not complete compiler-wide `GetV`, ECMA-402 locale formatting,
Array exotic descriptors, species or constructor realms.
It removes no Test262 materializer, changes no published conformance count, and
does not claim the Array, Array prototype or TypedArray trees are green.
Runtime verification remains deferred to the coordinated batch checkpoint.
