# Intl canonical locale tag invocation authority

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed invocation roles

The structural locale canonicalizer accepts one private, move-only
`CanonicalLocaleTagInvocationLocals` authority. Its constructor requires seven
distinct roles: input, canonical tag, language, script, region, base name, and
validity. The canonicalizer is the sole consumer and the sole place that
projects those roles back to raw Wasm locals.

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

Exactly five producers construct the complete authority:

- `Intl.Locale` construction;
- each present entry in `Intl.getCanonicalLocales`;
- the DateTimeFormat single-string locale path;
- the DateTimeFormat array-like locale path; and
- `Intl.DateTimeFormat.supportedLocalesOf`.

The structural algorithm, Wasm locals, instruction order, error paths, provider
call, locale matching, and result publication are unchanged. This boundary
does not add locale data or implement any open ECMA-402 service.

## Durable evidence

`intl_canonical_locale_tag_invocation_structure` uses a Rust lexical scanner
that excludes comments and every Rust string/character literal form. It pins
the private non-copyable role domain, recursive product-source census, all five
complete producers, the typed signature, and the sole consuming projection.

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
