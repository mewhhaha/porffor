# Exact NumberFormat provider

The NumberFormat provider accepts a validated configuration and exact
mathematical inputs, then returns scalar or range partitions. The same parts
produce the formatted string. Host/AOT consumers copy localized text unchanged:
only numeric fields substitute the selected digits, exactly once. Currency,
unit, notation and bidirectional literals are never transliterated as digits.
This provider does not itself publish the Intl constructor or any primitive
prototype method. Publication requires the complete coordinated intrinsic,
Realm, prepared-host, artifact, option-order and consumer implementation.

The numeric kernel owns normalization, decimal scaling, rounding modes and
increments, significant/fraction precision arbitration, signed zero, notation
carry and resource errors. RoundedDecimal::rounded_value is a rounded signed
mantissa, after percent scaling and notation division; it is not the original
unscaled mathematical input. Decimal strings and BigInts never pass through
binary64. Output bytes and part counts have separate checked limits and
fallible allocation paths.

## Locale proof and source inventory

NumberProfiles derives its 1,082-locale inventory from the complete pinned
CLDR47 sources. Lookup and the permitted prefix-based best-fit policy share
that inventory. The provider canonicalizer remains upstream. A resolved proof
checks that the formatting locale exists, that its numbering system is
admitted, that its resolved base is the same selected formatting locale, and
that the only retained extension is an exactly matching nu value. Unsupported
well-formed options use ordinary locale negotiation defaults. Supported-locales
results preserve the requested canonical tags.

The generated data's locale identities are independent of inheritance owners.
No unresolved pattern, plural operator, required label, alias or real main
locale can silently become English. Root inheritance and the documented
same-locale numbering-resource fallbacks remain explicit data operations.
The data README records the sole dangling synthetic default-content entry.

Primary source pins and archive hashes are in
crates/lila-intl/data/number-cldr-47/source-manifest.json. The package contains
CLDR47 commit `2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`, Unicode 16.0.0, and
selection evidence from ICU77.1 commit
`457157a92aa053e632cc7fcfd0e12f8a943b2d11`. Regeneration is offline and compares
canonical extracts, provenance, the binary profile, its fingerprint and the
1,719 pinned cardinal sample endpoints.

## Plural operands and literal patterns

Compact pattern selection uses exact rounded mantissa operands with c=e=0.
This preserves explicit-one and literal-only forms such as French and Italian
mille. The pinned ICU CompactHandler calls getPattern before adjustExponent;
see [number_compact.cpp](https://github.com/unicode-org/icu/blob/457157a92aa053e632cc7fcfd0e12f8a943b2d11/icu4c/source/i18n/number_compact.cpp).

Currency names and measurement units use the exact compact source-number
operands defined by [LDML47 plural operands](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-numbers.md#operands):
the compact exponent shifts the decimal point for n/i/v/w/f/t and remains c/e.
The pinned ICU LongNameHandler precedes CompactHandler, which independently
confirms that compact labels and measurement names have different selectors.

For scientific and engineering names, the provider deliberately uses the
rounded mantissa with c=e=0. ECMA-402
[PartitionNumberPattern](https://tc39.es/ecma402/#sec-partitionnumberpattern)
scales x, replaces it by RoundedNumber, and then supplies x to unit/currency
string selection. LDML47 defines c/e for compact notation, not scientific
exponents. Pinned ICU77 instead attaches the scientific exponent before its
LongNameHandler; that policy is not reproduced here. Literal English and
Russian tests distinguish this implementation-dependent name selection from
both source-magnitude selection and compact operands. The original numeric
kernel receipt remains immutable.

Literal-only compact and unit patterns are represented explicitly. A compact
pattern exactly 0 requests ordinary formatting of the original quantity with
the configured precision; it cannot accidentally print the scaled mantissa.
The provider preserves per-plural normal fallback, including Venetian's
one/other distinction. Arabic unit forms that omit the numeral remain literal
forms; sign selection still owns the formatted value's sign.

## Unit composition

Composition prefers an available precomposed sanctioned pair, then a
denominator perUnitPattern, then the locale's general per pattern. Plural and
case derivations are consumed from pinned supplemental grammar. Numerator
patterns retain their original placement and numeral omission.

[LDML47 compound units](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-general.md#compound-units)
does not supply a complete extraction for a denominator with meaningful
material on both sides of {0}. Pinned ICU77's extractCorePattern reports
PH_MIDDLE and its compound handler returns U_UNSUPPORTED_ERROR; see
[number_longnames.cpp](https://github.com/unicode-org/icu/blob/457157a92aa053e632cc7fcfd0e12f8a943b2d11/icu4c/source/i18n/number_longnames.cpp).
For this gap, the provider uses the same resolved locale and width's required
localized displayName as the denominator, then applies the localized per
pattern. This is an explicit implementation-dependent choice. It never trims
or joins the two affixes heuristically. Scalar and numerator patterns remain
unchanged. Missing displayName fails generation. The policy census and
Japanese meter-per-celsius/celsius-per-meter tests retain its exact boundary.

## Currency, signs and ranges

Standard currency precision and AOT defaults share CurrencyFractions.
Currency name placement follows ICU's pinned CurrencyUnitPatterns resource
mapping from each locale's default numbering system; it does not depend on an
explicit nu request. The pinned ckb resource has no such row and inherits root.
Currency-specific pattern/symbol overrides search their parent chain before
falling back to general locale symbols, as LDML47 Currencies specifies.

Currency alpha adjacency tests the semantic currency edge against pinned
Unicode General_Category L. Currency spacing evaluates the parsed CLDR
UnicodeSets, including supplementary digits and hanidec's non-Nd digits.
Spacing and bidirectional literals retain measurement ownership. Ordinary
signs, accounting affixes and notation symbols have their own ownership.

[LDML47 number ranges](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-numbers.md#number-range-formatting)
permits implementation-defined collapse policies. Measurement names share
semantically equivalent prefixes/suffixes, including one-character narrow
units, with plural-range reselection. Currency-symbol/code and percent patterns
instead own a paired prefix and suffix containing their signs and spaces.
Following the relevant
[ICU77 AUTO affix policy](https://github.com/unicode-org/icu/blob/457157a92aa053e632cc7fcfd0e12f8a943b2d11/icu4c/source/i18n/numrange_impl.cpp#L299-L350),
that complete group shares only when both sides agree and its visible extent
is more than one code point. Thus `$` repeats, `+$` shares, and the Portuguese
NBSP-plus-euro suffix shares. Accounting brackets cannot detach from their
currency, and opposite signs prevent the paired pattern from sharing.
Incidental bidi marks do not make a single visible symbol eligible; all retained
bidi text keeps its owner and source. Plain signs remain endpoint-specific.

Scientific and compact notation retain the LDML recommendation to repeat
notation at each endpoint. This deliberately differs from ICU AUTO's ability
to collapse some longer compact modifiers. Measurement names can share around
those repeated notation fields. Literal-only unit patterns remain whole
endpoint values. Uncollapsed text retains its original scalar partition and
source label. Shared punctuation and affixes use source shared; retained numeric
fields use start/end.

Identical scalar strings use approximatelySign at the parsed sign position.
Accounting places approximation before its opening sign affix. Approximation
parts are shared. Different signed zeros, descending ranges and infinities
retain input order. The optional spacing predicate remains the pinned LDML
White_Space/decimal-digit endpoint rule; inserted spaces use ASCII U+0020,
matching pinned Test262 and
[ICU77 separator insertion](https://github.com/unicode-org/icu/blob/457157a92aa053e632cc7fcfd0e12f8a943b2d11/icu4c/source/i18n/numrange_impl.cpp#L367-L385).
Existing localized spaces remain unchanged. Adjacent literal fragments with
the same source form one part, so `" – "` is one shared record. Coalescing checks
combined output bytes and reserves fallibly; final part limits count emitted
records. Other part kinds and different source labels never merge.
