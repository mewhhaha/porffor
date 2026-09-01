# Intl.Locale string-slot dispatch

Status: implemented and verified for the current Wasm-AOT Locale string
accessor surface.

## Invariant

`IntlLocaleStringSlot::{Tag, Language, Script, Region, BaseName}` is a private, non-derived domain
in `builtins/intl.rs`. Exhaustive projections bind every
variant to one heap offset and one optionality policy. Only `Script` and
`Region` may turn an absent payload into `undefined`; the other three slots
remain required strings.

The raw slot-selecting emitter is private. The shared catalog dispatcher can
call only five fixed entries for `language`, `script`, `region`, `baseName`
and `toString`; it cannot name the slot domain, select a variant, or call the
raw emitter. The structural target pins the exhaustive projections, the fixed
entry-to-variant mappings and all five catalog routes.

## Source-equivalence witnesses

No instruction-emitting statement changed. Reconstructing only the former
derive attribute, crate visibility and consuming projection receivers produces
the exact original 30-line slot-domain selection with SHA-256
`00486705af5ad3a89c1386f4ca8b3088d5531ca676a582aa643ca90bca658d6a`.
Reconstructing only the former visibility of the 43-line raw emitter produces
its exact original SHA-256
`4b346dcd2c819c503603ed7c08842e577d4b893dc98aa1f33c5f7d2c864cd134`.

## Verification

- `cargo xc` passes; existing workspace warnings remain.
- `intl_locale_string_slot_domain_structure` passes `4/4`.
- The neighboring Locale heap-slot and canonical-tag invocation structure
  targets pass `8/8`.
- The exact canonical locale tag roles CLI control passes `1/1` and observes
  all five string-returning routes.
- Formatting, module-boundary, task-plan and exact Test262 shortcut gates pass.

## Nonclaims

This is source-equivalent compiler hardening with no new Intl behavior,
Test262 pass or published-status change. Locale options and the remaining
ECMA-402 surface stay open, and this does not close T23.
