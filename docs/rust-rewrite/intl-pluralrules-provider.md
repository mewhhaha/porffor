# PluralRules native provider

The AOT Wasm shell owns JavaScript coercions, option order, digit defaults,
object slots, and exceptions. The host consumes closed primitive requests and
pinned CLDR47 data; it never constructs a NumberFormat object or observes a
JavaScript value. Native dispatch uses four real operations: ResolvePluralLocale
(17), SupportedPluralLocales (18), SelectPlural (19), and SelectPluralRange (20).
Intl host ABI7 retains the Temporal operations 14–16.

## Exact numeric authority

PluralRules and NumberFormat use the same lossless numeric input codec,
ToIntlMathematicalValue normalization, signed decimal rounding, and generic
plural predicate evaluator. The shared nine-word rounding decoder validates
precision ranges, inactive fields, non-unit increments, mode, and trailing-zero
policy before either service receives a RoundingSettings value.

[ECMA402 ResolvePlural](https://github.com/tc39/ecma402/blob/e463f3c8b62e5c67f4846cd05eec40d8d5947ed0/spec/pluralrules.html)
invokes bare FormatNumericToString. Signed rounding comes before absolute CLDR
operand extraction. Minimum integer padding and retained fraction zeroes are
part of the visible comparison string. NumberFormat's localized partition and
ComputeExponent do not run on this service path. Huge BigInts retain exact
modulo arithmetic; a String with the same magnitude can normalize to infinity
under the specified mathematical-value interval.

## Data and notation policy

The offline NumberFormat source archive now also includes ordinals.xml from
CLDR47 commit 2ef784e3a4168bc2a43cd1b5b9839b6636f5899c. Its Git blob and SHA256 are
checked alongside every existing source. Payload schema 2 publishes separate
cardinal and ordinal tables, both category masks, per-locale associations,
complete cardinal range matrices, and the existing checked compact resources.
Ordinal identity participates in locale-profile pooling: 1082 admitted locales use
535 pooled profiles, 40 cardinal tables, 26 ordinal tables (25 XML groups plus
the empty fallback), and 10 complete cardinal range matrices.

PluralRuleSelect is implementation-dependent. This provider explicitly uses
bare, unscaled n/i/v/w/f/t with c=e0 for standard, scientific, and engineering.
Compact notation selects the resolved data locale's default numbering system
and requested short/long exponent table directly from the final bare rounded
magnitude. Below the table and zero use exponent 0; above its final row uses that
row. Only c/e metadata is attached: bare digits are never expanded again.
Rounding carry therefore affects the final magnitude once, without a scaled
NumberFormat carry probe or a second rounding operation.

The cardinal range tables complete absent cells with the end category.
CLDR47 supplies no ordinal range matrices; ordinal range selection deliberately
uses the end-category default. Both policies are checked against the resolved
locale category mask. Range equality compares unsigned bare visible strings,
so en -1..1 returns One and sl 1..101 can use One→One=Few. Descending, negative,
and infinite endpoints are admitted; NaN alone receives semantic rejection.

## Wire and lifecycle

Each message begins with two little-endian u64 words: version 1 and operation*2
plus request 0/response 1. Locale resolution takes a canonical locale list and
closed matcher and returns resolved/data locale strings followed by cardinal
and ordinal masks. SupportedLocales uses the same actual available locale set
without observing type, notation, or digit options.

Selection carries the two associated locale strings followed by 12 u64 fields:
type, notation, compactDisplay, minimumIntegerDigits, precision kind,
minimum/maximumFractionDigits, minimum/maximumSignificantDigits,
roundingIncrement, roundingMode, trailingZeroDisplay. It then carries one or
two shared numeric fields. Responses contain one category word 0..5.

All messages validate exact header, domains, text extents, inactive words,
locale/data/mask association, and final byte exhaustion. The Engine copies the
request before capacity probes or overlapping writes. Capacity failure writes
nothing. Only NaN range produces Rejected(-1); malformed protocol, invalid data,
and resource failures are native faults.

## Verification boundary

This implementation is prepared in a private future-tree stage. Authored Rust
kernel, ordinal endpoint, association, wire, and real host capacity controls are
not yet executed. Offline generation and source/data checks are independent
preparation evidence. Cargo, Engine, and pinned Test262 verification remain
required before any conformance or product-support claim.
