# `Intl.Locale` reserves its result before observing the tag

## Scope

This contract owns one observable part of ECMA-402 `Intl.Locale`
construction: `OrdinaryCreateFromConstructor` happens before the constructor
checks or coerces `tag`, and before it reads any locale option.

It does not implement `UpdateLanguageId`, the locale extension options,
provider-backed `Intl.Locale` canonicalization, or the rest of the Locale
prototype.

## Required order

[ECMA-402 `Intl.Locale`](https://tc39.es/ecma402/#sec-intl.locale) creates the
result object in step 6. The first tag type check is step 7, `ToString(tag)` is
step 9, and options coercion and reads begin after that. Therefore the Wasm
constructor performs these transitions in order:

1. Reject an undefined `NewTarget`.
2. Resolve `NewTarget.prototype`, including Proxy `[[Get]]`, and allocate the
   ordinary result object with that prototype's complete payload/tag identity.
3. Keep the allocated object private and unreachable.
4. Validate and coerce `tag`, then perform the implemented canonicalization.
5. Initialize the Locale record and brand on the reserved object.
6. Publish only the initialized object as the normal result.

The ordering has two abrupt-completion consequences:

- if `Get(NewTarget, "prototype")` throws, neither `tag` coercion nor any
  options property read occurs; and
- if tag coercion throws, the prototype read has already occurred, but the
  reserved object is never returned or otherwise exposed.

The latter path can leave unreachable allocation behind. That is an internal
allocation detail; no JavaScript reference to the partial object exists.

## Load-bearing representation

`builtins/intl.rs` has a private two-state lifecycle:

```text
ReservedIntlLocaleObjectLocal
    -> InitializedIntlLocaleObjectLocal
    -> published result
```

Both states are non-`Copy`, have private raw locals, and are constructed only
by their corresponding emitter transitions. The initializer consumes the
reserved state after successful tag processing, installs every currently
represented Locale slot plus the internal brand, and returns the only state
the publisher accepts. Publishing a reserved or partially initialized object
is therefore a Rust type error.

Prototype resolution continues to use the existing shared
`emit_new_target_prototype_to_locals` route with its existing `CurrentGlobal`
fallback policy. The reserved object local is allocated before the temporary
prototype payload/tag locals; those temporaries are released in reverse order
while the object remains live. Allocation consumes both prototype locals, so
Function, Array, and Arguments prototypes do not silently become Object-tagged.
This change moves the shared lookup to its specified observation point without
introducing a parallel prototype algorithm or changing its current Realm
fallback policy.

## Durable evidence

`wasm_intl_locale_construction_order.js` pins:

- a Proxy `NewTarget.prototype` read before tag `ToString`;
- successful initialization and branding behind a custom prototype;
- exact Function, Array, and Arguments prototype identity across tagged
  allocation;
- an abrupt prototype getter that wins over a primitive-tag TypeError and
  leaves options untouched;
- a primitive-tag TypeError from the called constructor's Realm after a
  successful prototype read; and
- an abrupt tag coercion that occurs after the prototype read and never reads
  options.

The adjacent pinned Test262 obligations are
`intl402/Locale/constructor-tag-tostring.js`,
`intl402/Locale/constructor-getter-order.js`, and
`intl402/Locale/subclassing.js`. The fixture isolates the allocation-order
boundary those broader tests rely on.

## Non-claims

`Intl.Locale` still ignores its options argument in this slice. The fixture's
options Proxy proves only that no future or partial option read may move ahead
of object reservation, and that a preceding abrupt completion leaves it
untouched. T23 still owns the full ordered `UpdateLanguageId` and extension
option sequence.

Created realms still do not install `Intl`, so this contract does not claim the
cross-Realm fallback cases in `intl402/Locale/proto-from-ctor-realm.js`. It also
does not claim complete alias data, likely-subtag behavior, Locale accessors,
or a green `intl402/Locale` subtree.

## Verification boundary

Static freeze consists of exact-file `rustfmt --check` for `builtins/intl.rs`
and the CLI module, `node --check` for the fixture, the colocated structural
guard proving LIFO reservation, tagged allocation, and the one-way
reserved/initialized/published transition,
`git diff --check`, file-scope review, and fresh independent review.

After the shared low-memory matrix releases Cargo, run the registered CLI
fixture, the three pinned Test262 witnesses above under Wasm AOT, the adjacent
`intl402/Locale` filter, and then the serialized broad batch ladder. No Cargo,
fixture execution, Test262, or snapshot result is claimed by this freeze.
