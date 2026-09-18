# Intl canonical locale tag invocation authority

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed invocation roles

The structural locale canonicalizer accepts one private, move-only
`CanonicalLocaleTagInvocationLocals` authority. Its constructor requires seven
distinct roles: input, canonical tag, language, script, region, base name, and
validity. The structural canonicalizer and provider/component refresh each consume their
own invocation and project its roles once. The refresh copies its input before
calling the structural operation, whose output initialization forbids aliasing
the raw input and output locals.

Previously, every producer passed seven adjacent `u32` locals. A producer could
therefore transpose tag, language, script, region, base-name, and validity roles
while continuing to compile. That can publish a structurally valid
`Intl.Locale` with inconsistent component slots, negotiate DateTimeFormat from
the wrong subtag, or treat a string payload as the success flag. The distinct
Rust role types make those positional substitutions type errors.

The authority and role types derive no capabilities. The authority is marked
`must_use`, consumes itself exactly once, and is visible only within the
`builtins` module so unrelated emitters cannot manufacture a wider locale
canonicalization surface.

## Producers and semantics

There are ten complete authority construction sites:

- initial `Intl.Locale` structural validation;
- the constructor's two provider/component refresh invocations;
- each present entry in `Intl.getCanonicalLocales`;
- the three DateTimeFormat locale-list producers;
- variants-option structural validation; and
- provider-result structural component refresh; and
- likely-subtag result structural component refresh.

The original boundary was source-equivalent hardening. The constructor-options
batch adds provider alias resolution before and after overrides, with separate
input storage during component refresh. The role domain and move-only authority
still prevent adjacent local-role substitutions. No new locale data is added.

## Durable evidence

`intl_canonical_locale_tag_invocation_structure` uses a Rust lexical scanner
that excludes comments and every Rust string/character literal form. It pins
the private non-copyable role domain, recursive product-source census, all ten
complete construction sites, both typed consumers and their single projections.
The product census is 16 uses of the authority and each role type. Earlier
counts omitted the independently introduced language-options producer.

The public `wasm_intl_canonical_locale_tag_roles.js` fixture observes the
canonical tag and all four `Intl.Locale` component slots, the
`Intl.getCanonicalLocales` result, and DateTimeFormat locale/extension
resolution. The focused CLI test owns that fixture.

At the 2026-08-27 focused checkpoint, the authority structure target passes
`4/4`, the public CLI witness passes `1/1` with 749 tests filtered, and the
neighboring Locale string-slot target passes `3/3`. `cargo check -p
lila-aot-wasm --quiet` is green with the repository's existing warnings. The
targeted Rust format check and scoped diff check are clean.

This is not a complete Intl.Locale, DateTimeFormat, Intl402, or T23 closure
claim and changes no published conformance count.
