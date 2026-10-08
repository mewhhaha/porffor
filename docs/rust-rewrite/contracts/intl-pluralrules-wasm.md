# PluralRules Wasm shell and primitive protocol

This batch authors the complete `Intl.PluralRules` JavaScript shell through the
ordinary Rust JS-to-Wasm compiler. Its five connected builtin bodies implement
construction, `supportedLocalesOf`, `resolvedOptions`, `select` and
`selectRange`. The native provider receives primitive locale/configuration and
lossless numeric messages. It never receives JavaScript objects, callbacks or
executable source. Compilation and product verification of this batch are
pending; this contract does not establish support or pinned-suite PASS.

## Selected immutable number data

PluralRules consumes the same admitted `NumberProfilesDataImage` owner as
NumberFormat. The image contains the actual pinned native `LNF47` schema-2
payload: 1082 locale associations, 40 cardinal rule tables, 26 ordinal rule
tables, 10 complete cardinal range matrices, and each locale's default
numbering-system short/long compact exponent resources. These are the existing
CLDR47 rule expressions and exact decimal operands. No ICU plural constructor
or narrowed operand conversion replaces them.

`ResolvedPluralLocale` retains an `Arc<NumberProfiles>` together with its two
category masks. Resolution and complete wire decoding require that admitted
owner. Scalar and range selection reject a configuration from another owner
before numeric normalization, rounding, nonfinite handling or data lookup.
Wire messages keep their existing primitive fields; the selected kernel's
decoder reconstructs the proof against its retained image rather than a global
default profile. Independent image owners remain distinct even when their
pinned payload bytes and locale names match.

RelativeTimeFormat also carries this shared number owner through its checked
configuration and primitive decoder. Its constructor checks the retained
NumberFormat locale owner before creating the cardinal configuration; its
partition entry checks the plural owner before an automatic literal can
return. The successor [RelativeTime data image](intl-relative-time-data-image.md)
also retains the actual template catalogue. Its selected formatting entry
checks the exact template locale owner and uses the captured Number owner
before an automatic literal can return.

Current Minimal and Custom-named images admit the exact pinned payload;
Unfiltered Custom names do not select a different data set. The coupled Number/
Plural projection selects a real public domain with retained private dependencies.
The complete Conformance producer admits the full pinned source; whole-Intl
placement is Embedded. Runtime and pinned acceptance remain unverified.
Three new source controls exercise two independently admitted number images,
exact 400-digit ordinal selection and cardinal range data, selected wire
decoding, and the relative-time automatic-literal path. They are authored and
unexecuted until the coherent batch's capped verification checkpoint.

## Observable operations

A plain constructor call throws `TypeError` before either argument is observed.
Construction performs `Get(newTarget, "prototype")` and the resolved NewTarget
Realm fallback before locale-list canonicalization or options coercion. The
reserved ordinary result stays unpublished until its complete private record
and brand have been installed. The shared construct dispatcher enters this
builtin directly, avoiding a second prototype read or generic preallocation.
The intrinsic prototype is an ordinary unbranded object.

Canonical locale-list conversion precedes `CoerceOptionsToObject`. Undefined
creates null-prototype options; null rejects; other primitives are boxed in the
called builtin's Realm. `localeMatcher` is read next, and the provider resolves
a locale with an empty relevant-extension-key set. The constructor then reads
`type`, `notation` and `compactDisplay`, including `compactDisplay` for
noncompact notation, before the shared digit operation with defaults 0 and 3.
No `numberingSystem` or formatting-style option is read.

The shared `builtins/intl_number` owner is consumed by both NumberFormat and
PluralRules. It retains the four raw fraction/significant digit values until
`roundingIncrement`, `roundingMode`, `roundingPriority` and `trailingZeroDisplay`
have been observed. Only the selected bounds are converted. Prepared result
slots have no policy accessors; the factory returns the only digit handle
accepted after the ordered reads, conditional conversions and increment/
precision checks. NumberFormat retains its original style/currency defaults
and its later `compactDisplay` observation. No parallel digit algorithm or
NumberFormat-shaped placeholder configuration is used for PluralRules.

Methods validate the intrinsic PluralRules brand before observing numeric
arguments. Proxies and lookalike ordinary objects do not forward that brand.
`select` performs the shared `ToIntlMathematicalValue` observation: exact
UTF-16 strings, including lone surrogates, intrinsic BigInt decimal text,
intrinsic shortest Number text and separate negative zero. It does not narrow
strings or BigInts to an IEEE Number.

`selectRange` first rejects either undefined endpoint, then coerces start and
end in order. Both observations complete before the native NaN decision, so a
NaN start cannot suppress a throwing end hook. Descending, negative and infinite
endpoints are admitted by the service. Scalar NaN/infinities return `other`;
range NaN becomes a called-Realm `RangeError`. A typed checked mathematical
value is required at the shared numeric wire field boundary.

The pure native rounding path implements signed `FormatNumericToString`.
Selection uses its unsigned bare visible digits. Range equality compares that
bare string before consulting category-range data; identical rounded strings
return the start category, while differing strings use the matrix even when
both categories are the same. The pinned cardinal/ordinal data and explicit
ILD range/compact policies are owned by
[`intl-pluralrules-provider.md`](../intl-pluralrules-provider.md).

`resolvedOptions` creates a fresh object in the called method's Realm. It
reports table order, selected precision fields, compact display only for
compact notation, and a fresh category array ordered zero, one, two, few,
many, other. The property values and arrays use ordinary writable, enumerable,
configurable data properties. Errors from conversions retain the original
thrown value; intrinsic receiver/option/range errors use the called function's
Realm. No public `Intl` method is used as a semantic delegate.

## Registration and private state

The IR catalogue defines five distinct function IDs and ordinals, builtin
lengths 0/1/0/1/2, and the constructor-only constructable flag. The service is an
actual Intl namespace member rooted with all five bodies. Shared entry/created
Realm installers consume one property table, install the constructor and
three prototype methods, and retain the distinct PluralRules default prototype
in the Realm intrinsic record. Prototype and method descriptors follow the
ordinary Intl constructor table; the toStringTag is `Intl.PluralRules`.

The 120-byte internal record owns two traced text references (resolved and data
locale), one untraced selected category mask and twelve closed configuration
words. `PluralConfigurationWord` is the record and native codec ordering
owner. A private brand-producing receiver factory grants the only record view
admitted by the PluralRules configuration wire field. Reserved constructor
objects and initialized publishable objects are distinct move-only states.
Temporary output destinations are reserved below retained constructor objects
and scratch locals are released in reverse reservation order.

Intl ABI 7 extends the future Temporal ABI 6 with operations 17 through 20:
locale resolution, supported locales, scalar selection and range selection.
The NumberFormat operation numbers and wire-v1 messages remain unchanged.
Plural wire version 1 uses a 16-byte little-endian u64 header followed by
length-prefixed text/numeric spans and closed u64 configuration words. Its
locale request has the canonical list before matcher. Selection carries the
resolved/data locale association, twelve configuration words and one or two
lossless numeric fields. Its result is a category code, not a JS string object.

The shared `builtins/intl_provider_wire.rs` owner now has eleven closed request
variants for NumberFormat, PluralRules and ListFormat. They determine the
operation and complete field order;
callers cannot supply an unrelated opcode beside a field list. The encoder
mints a private request handle and the capacity/header factory mints the only
response handle accepted by the reader. Its borrowed response lifetime prevents
releasing that handle before finishing the bounded cursor. The common response
reader checks every word/span/count against the returned extent, validates
header version/operation/direction and rejects trailing bytes. Constructor
masks admit only six category bits and require `other`. Selection admits only
the six category codes and requires membership in the receiver's retained
locale/type category mask. The two-call capacity protocol
requires the exact declared output length. Only a typed NaN-range rejection
becomes a public RangeError; malformed private messages, corrupt data or
resource failures stay host failures.

## Source authority and verification

The observation/rounding authority is immutable ECMA-402 main revision
`e463f3c8b62e5c67f4846cd05eec40d8d5947ed0`, captured 2026-09-30:
[PluralRules objects](https://tc39.es/ecma402/#pluralrules-objects),
[SetNumberFormatDigitOptions](https://tc39.es/ecma402/#sec-setnumberformatdigitoptions)
and [ToIntlMathematicalValue](https://tc39.es/ecma402/#sec-tointlmathematicalvalue).
The design receipt binds the captured primary bytes and pinned CLDR sources.

The Engine controls authored for this batch cover exact numeric provenance,
cardinal/ordinal data association, all signed rounding modes, delayed/ignored
option coercions, category-range diagonal versus bare-string equality,
constructor/options order, Realm/brand/descriptors and real JS runtime errors.
Their 14 tests plan 28 mode observations. The exact pinned PluralRules inventory
contains 52 files and 104 canonical mode IDs. Adjacent NumberFormat replay
contains 253 files and 506 modes. These are inventories and unexecuted controls,
not PASS receipts. Compilation, focused native/Engine/CLI controls, the exact
pinned replay and broad checkpoints remain required before publication.
