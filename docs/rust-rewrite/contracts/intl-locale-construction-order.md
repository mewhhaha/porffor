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

The private `builtins/intl/construction_lifecycle.rs` child owns the complete
two-state lifecycle:

```text
ReservedIntlLocaleObjectLocal
    -> InitializedIntlLocaleObjectLocal
    -> published result
```

Both states are non-`Copy`, have private raw locals, and are constructed only
by their corresponding emitter transitions. Rust requires the carrier and
transition names to be `pub(super)` for the parent's inferred handoffs, but
the parent neither names nor imports them and cannot construct or project
their private tuple fields. The initializer consumes the reserved state after
successful tag processing, installs every currently represented Locale slot
plus the internal brand, and returns the only state the publisher accepts.
Publishing a reserved or partially initialized object is therefore a Rust
type error.

The exact 15-line carrier and 97-line transition selections retain
visibility-normalized SHA-256
`f7515bf0b336e4307fac6cdefb699e32b4b3794bd0a6eff9e4f3d58113473725`
and
`7aea2daa1ccd0b8d9bdd8f5ac35eb2287eef2c686037a8e41f226c7ce659d0fa`;
their combined 112 selected lines retain
`ca374b7f75159c8b7c978d46ee0be44be1faafb9ac34d6b8e686a200ba5d4ac4`.
The 117-line child has SHA-256
`3ebcef67424bcff990b0e6f6ed519e5c40185288d3b3ab9c21311c9110dc1bd0`.
The source move alone reduces the 2,368-line parent snapshot to 2,256 lines;
the strengthened colocated guard brings the current file to 2,364 lines, with
2,171 lines before the test module. The unchanged reserve call and ten-line
initialize/publish block retain SHA-256
`3fd56c270a997572d0c093e16d933bc366f4ec3f5371fb20d36835affe92c3d9`
and
`44a6f0a8622f2073daad21bf287597fee7048bd60325c79d7f1606125b4b5b4e`.
The recursive guard pins zero parent production references to either carrier,
four child uses of each carrier, one child definition and one parent call for
each transition, and the sole one/two raw projections.

`functions.rs` classifies `Intl.Locale` in the closed direct-returning
constructor domain. The construct dispatcher therefore enters the builtin
before the generic constructor path can read `NewTarget.prototype` or allocate
a receiver. The lifecycle reserve transition remains the sole prototype `Get`
and result allocation for an explicit `NewTarget`; removing this classification
makes both the recursive guard and the construction-order fixture fail.

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
and `builtins/intl/construction_lifecycle.rs`, the colocated recursive
structural guard proving sole ownership, LIFO reservation, tagged allocation,
and the one-way reserved/initialized/published transition, the module-boundary
and task-plan source gates, `git diff --check`, and file-scope review.

At the 2026-08-28 Batch X checkpoint, `cargo xc` is green, the colocated
lifecycle/direct-dispatch guard passes `1/1`, and the registered construction
order fixture passes `1/1`. The fixture initially exposed a duplicate
`NewTarget.prototype` read from the generic construct path; the closed
direct-returning classification is the bounded correctness repair.

The pinned `constructor-tag-tostring.js` and `subclassing.js` leaves pass all
`4/4` Wasm-AOT executions. `constructor-getter-order.js` remains `0/2` with
Runtime bugs because this slice still ignores Locale options, as recorded in
the non-claims above. The adjacent Locale filter, semantic snapshot and broad
batch ladder were not run, and no published conformance count is changed.
