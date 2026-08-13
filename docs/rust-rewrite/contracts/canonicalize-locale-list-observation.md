# CanonicalizeLocaleList observes one index at a time

## Scope

This contract owns the observable Wasm shell for ECMA-402
`CanonicalizeLocaleList`. It does not own locale alias data, locale matching,
`Intl.Locale` options, or any formatting service.

The pure locale provider may see only a structurally validated String payload.
It must never receive a JavaScript object, decide whether an indexed property
exists, invoke a getter, or choose when `ToString` runs.

## Required order

For `Intl.getCanonicalLocales(locales)`, the shell performs these transitions:

1. `undefined` becomes an empty list.
2. A primitive String or an object with `[[InitializedLocale]]` becomes a
   one-element source without observing `length` or indexed properties.
3. Every other value passes through `ToObject` in the called builtin's defining
   Realm. Thus `null` throws that Realm's TypeError, while Number, Boolean,
   Symbol and BigInt primitives observe inherited `length` and indexed
   properties on that Realm's wrapper prototypes.
4. `LengthOfArrayLike` is observed exactly once. That length remains fixed even
   if a later getter or coercion changes the source's `length`.
5. For each integer index below that fixed length, in ascending order:
   1. form the decimal property key;
   2. perform `HasProperty` on the original source;
   3. skip an absent property without performing `Get` or coercion;
   4. for a present property, perform `Get`;
   5. require a String or Object;
   6. read an initialized `Intl.Locale`'s `[[Locale]]`, otherwise perform
      `ToString`;
   7. structurally validate and canonicalize the tag in Wasm;
   8. call the typed pinned locale provider;
   9. deduplicate the provider-canonical tag; and
   10. only then advance to the next index.
6. `CreateArrayFromList` publishes the accumulated tags in an Array whose
   prototype is the called builtin's defining-Realm `%Array.prototype%`.

The ordering is observable. In particular, element zero's `toString` may
create or replace element one, and the walk must see the new value. A Proxy
`has` trap may throw before its corresponding `get` trap. A hole is not the
same thing as a present property whose value is `undefined`.

`CanonicalizeLocaleList` is array-like, not iterable. It must not read
`Symbol.iterator` or use IteratorClose.

## Load-bearing representation

The array-like path carries a private `CanonicalLocaleListArrayLikeLocals`
record containing only:

- the original source as one `TaggedLocals` value; and
- the one snapshotted length local.

It deliberately has no element buffer and no snapshot-array payload. The only
consumer owns the ascending index loop. Each iteration calls the shared
`emit_object_has_property_i32` operation before the conditional
`emit_object_read`, and finishes conversion, provider canonicalization and
deduplication inside that same present-property branch.

That HasProperty operation is already backed by the closed,
exhaustively-consumed `ObjectInternalMethodBranch` domain: Proxy,
IntegerIndexed, Array, Arguments, BoxedString and Ordinary. Adding a branch to
that shared catalogue forces every exhaustive consumer, including
HasProperty, to decide how to handle it. The catalogue does not require every
future exotic to have a distinct branch when it intentionally shares an
existing representation and internal-method path.

The generic `emit_array_like_snapshot_payload` helper remains valid for callers
whose algorithms intentionally copy every indexed value before later work. It
is forbidden in `emit_intl_get_canonical_locales` because such a copy erases
both HasProperty and the per-element observation order above.

## Failure ownership

- `ToObject(null)` and a non-String/non-Object present element produce the
  current function Realm's TypeError.
- A malformed tag or the provider's closed `Rejected` result produces the
  current function realm's RangeError.
- Abrupt completion from `length`, Proxy `has`, indexed `Get`, or `ToString`
  propagates unchanged.
- Any host result other than `Written(u32)` or `Rejected`, or a written length
  beyond the supplied capacity, remains an ABI fault and traps.

## Durable evidence

`wasm_intl_canonical_locale_list_observation.js` pins:

- `length`, `has`, `get` and `toString` order;
- an absent hole and an inherited indexed property;
- creation of a later element by an earlier element's coercion;
- primitive wrapper prototype observation;
- exact propagation of a Proxy `has` abrupt completion; and
- provider alias deduplication as a control.

Defining-Realm primitive boxing, null TypeError and result Array prototype are
structural obligations of the emitter: it calls
`emit_value_to_current_function_realm_object_locals` and installs the Array
prototype loaded from `HEAP_FUNCTION_DEFINING_REALM_OFFSET`. They do not yet
have a cross-Realm runtime witness because `__lilaCreateRealm` does not install
the `Intl` namespace in created realms.

The adjacent pinned Test262 witnesses are
`intl402/Intl/getCanonicalLocales/has-property.js`, `get-locale.js`,
`to-string.js`, and `locales-is-not-a-string.js`.

## Non-claims

This contract does not make the whole `Intl/getCanonicalLocales` subtree green.
It does not complete CLDR/Unicode extension canonicalization, grandfathered tag
coverage, `Intl.Locale`, generated data images, any formatting service, or the
full `intl402` tree.

Errors created inside the shared Proxy internal-method machinery remain an
explicit nonclaim: that machinery still sources some errors from the main
Realm. The fixture pins only unchanged propagation of an error object thrown by
user code in a Proxy trap; it does not claim the right Realm for Proxy-generated
TypeErrors.

Cross-Realm execution of `Intl.getCanonicalLocales` is also an explicit
evidence gap, not a claimed pass. Created-realm Intl bootstrap remains a T06 /
T23 dependency; until it exists, the defining-Realm ToObject, null-error and
result-Array obligations are frozen by source structure and independent review
only.

The result array may retain capacity for the snapshotted source length even
when holes or duplicates make its published length smaller. This is an
unobservable allocation choice, not a second element snapshot.

## Conflict and risk boundary

The shared ToObject, object read, HasProperty and decimal-key operations are
consumed as-is. This seam does not modify their implementations or the generic
array-like snapshot helper. In particular, the Number/BigInt, iterator and
collection lanes may continue changing their own callers without sharing an
Intl-specific representation.

The principal implementation risk is control-flow drift: a later refactor
could place provider work outside the present-property guard or increment the
index before coercion completes. The private record excludes the old snapshot
buffer shape, while the fixture's exact event trace and mutation case guard the
remaining runtime order. No live conformance snapshot or published count is
owned by this seam.

## Verification boundary

Static freeze gates are exact-file `rustfmt --check` for the two touched Rust
files, `node --check` for the fixture, focused source searches proving that the
Intl emitter consumes the typed record and shared HasProperty operation without
calling `emit_array_like_snapshot_payload`, `git diff --check`, file-scope and
local-lifetime review, and fresh independent review.

After the active shared-runtime work releases the verification ladder, run in
order: a focused `lila-aot-wasm` check, the one registered CLI fixture, the four
pinned Test262 witnesses named above under Wasm AOT, their adjacent
`Intl/getCanonicalLocales` filter, and finally the serialized broad batch
ladder. Once T06/T23 installs Intl in created realms, add and run a cross-Realm
witness for primitive wrapper lookup, null TypeError identity and returned
Array prototype identity. Cargo, fixture execution, Test262 and snapshot
publication are deferred at this freeze; none is evidence for the current
patch yet.
