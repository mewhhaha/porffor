# ListFormat provider and Wasm consumers

This prepared batch connects `Intl.ListFormat` construction, locale support,
resolved options, string formatting and parts to a primitive Rust list
provider. JavaScript observation, object identity and exceptions stay in the
AOT Wasm compiler. The host receives checked primitive data and does not call
user code. Compilation and product verification of this batch are pending;
the authored controls and input inventory are not conformance results.

## Observable lifecycle

The [immutable ECMA-402 ListFormat
algorithms](https://github.com/tc39/ecma402/blob/e463f3c8b62e5c67f4846cd05eec40d8d5947ed0/spec/listformat.html)
require NewTarget for construction and perform
`OrdinaryCreateFromConstructor` before locale/options observation. Locale
canonicalization precedes strict `GetOptionsObject`, then localeMatcher, type
and style are read and converted in their specified order. Constructor
primitive options are rejected. `supportedLocalesOf` first canonicalizes its
locale list and then uses primitive option boxing, as prescribed by the shared
locale filtering operation. These two option policies have separate callers.

The private GC record is initialized before publication. The prototype stays
ordinary and unbranded; public lookalike properties and a Proxy around an
instance cannot supply the private brand. Function-, Array- and Proxy-valued
NewTarget prototypes keep their actual representation tags. Primitive
prototype fallback uses the actual NewTarget Realm, including bound and
nested Proxy constructors. Methods use their own defining Realm for intrinsic
errors and newly created result arrays/objects, including borrowed calls and
nested hooks. `resolvedOptions` returns a fresh object with locale, type and
style in that order.

Both formatting methods check the receiver brand before any iterator Get.
Undefined is an empty list; other values use the real sync iterator protocol.
The iterator method and next method are read once, next is called with the
iterator receiver and no arguments, and each result's done/value is observed
in order. Iterator acquisition and next/done/value abrupt completions
propagate without closing. A yielded value must already be a primitive String:
there is no ToString of arbitrary elements. A non-string creates a TypeError
in the called method's Realm and closes once; a return getter/call failure or
primitive return result cannot replace that original throw.

## Native data and UTF-16 partition

The provider consumes the existing pinned `icu_list` 2.0.1, `icu_list_data`
2.0.0 and `writeable` 0.6.3 kernels/data. The image source successor exports all
three List markers at build time, then consumes the actual immutable blob through
typed deserialization and the selected Locale image's canonicalizer/fallbacker.
Runtime baked List/fallback providers are retired. Complete nine-way admission
publishes the image owner, and selected formatting requires its retained profile
identity. Whole Intl placement remains External; this component does not claim
full Conformance data. See the [image contract](contracts/intl-list-data-image.md).
Available locales come from the actual list marker identifiers, with checked
fallback loads for conjunction, disjunction and unit at long, short and narrow
widths. A locale proof owns
the associated nine profiles; an arbitrary locale or independent profile ID
cannot enter a checked configuration. The actual admitted catalogue/load
count must come from native verification, rather than a NumberFormat
catalogue or a pooled-template count.

ICU4X data commit `5e404744dd6c9dd7f86aac82586e3fa98ea75f7f` supplies contextual
Spanish y/e and o/u and Hebrew vav/U+2011 joins. Selecting a contextual template
decodes valid UTF-16 pairs and uses U+FFFD for an isolated surrogate only in the
condition stream. This is the chosen implementation-defined template policy.
The original elements never pass through that lossy projection for output.

A checked partition contains separate nonempty literal UTF-16 tokens and
indices into the original observed primitive-string list. Indices occur
exactly once in input order, including duplicate strings and empty elements.
The empty list has no tokens; a singleton empty string has one empty element.
The parts writer records ICU element scopes rather than deriving parts from
nonempty rendered ranges. String formatting concatenates the same partition
that parts formatting publishes as fresh type/value objects. No public Array
Get or additional coercion occurs while substituting retained elements.

## Primitive protocol and identity

Future Intl host ABI 8 adds ResolveListLocale (21), SupportedListLocales (22)
and FormatListParts (23), preserving prior wires. A message begins with two
little-endian u64 words: version 1 and operation*2 plus request 0/response 1.
Locale requests carry the canonical list and closed matcher; resolution returns
one admitted locale without a separately mixable profile/data-locale field.
Formatting carries that retained locale, closed type/style, item count and each
string's counted raw little-endian u16 code units. A response carries counted
tokens: Literal 0 with UTF-16 units, or Element 1 with an original item index.

The native operation validates headers, domains, extents, exact exhaustion and
the locale/profile association. The Wasm response proof borrows the observed
list and verifies its complete ordered index domain before substitution.
Engine host dispatch copies input before capacity checks or overlapping output
writes; insufficient capacity writes nothing. Malformed protocol, impossible
part indices, data faults and resource failures remain host/runtime faults;
they are not catchable option errors or successful semantic-gap results. The
actual list data identity participates in the composite artifact/provider gate.

## Authored verification and refresh

The new Engine target has 16 tests and 32 planned strict/sloppy observations.
Fourteen fixtures check metadata, order, abrupt stopping, primitive-boxing
differences, iterator lifecycle/closing, branding, all nine English template
configurations, contextual Spanish/Hebrew joins, exact UTF-16, fresh parts,
Malayalam/Māori final literal suffixes, called Realms and tagged NewTarget
prototypes. Two uncaught controls require
actual JavaScript TypeError/RangeError classifications with no semantic-gap,
parse or IR diagnostic. They cannot pass as Unsupported. One compilation job
and the existing 120000ms cold-fixture budget bound each Engine invocation.

The untouched pinned Test262 content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b` has exactly 81 ListFormat files and
162 planned modes. The bounded constructor/options/iterator and locale-example source review
found no concrete conflict with the captured current algorithms; every mode
still needs execution. Static created-Realm controls do not replace the
pinned foreign Function/NewTarget case or authorize excluding it.

After the complete service batch is composed and admitted, root runs its
source/generator checks, one full compile, native/protocol controls and:

```sh
python3 scripts/limited_verification.py -- cargo test --locked --offline -p lila-engine --test aot_intl_list_format -- --test-threads=1
python3 scripts/limited_verification.py -- ./target/debug/lila --jobs 1 test262 run intl402/ListFormat --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 240000 --snapshot-dir target/listformat-snapshots --snapshot-name listformat-full
```

The canonical CLI must first be captured with its source/binary identity.
Selection receipts must prove all 81 sources and 162 actual modes, original
selectors and no exclusions. Shared wire/locale/iterator changes also require
their existing compiler guards, the full adjacent NumberFormat/PluralRules
cohorts, whole workspace, full fake suite and broad Intl observations with every
actual failure retained and owned. Refresh aggregate artifacts and the README
numeric block only through the canonical publisher after verified completion.
These planned commands and authored checks make no runtime or full ECMA-402
claim; T23 remains open.
