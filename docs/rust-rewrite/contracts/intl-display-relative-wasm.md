# DisplayNames and RelativeTimeFormat compiled consumers

The isolated service batch adds nine real builtin bodies to the JavaScript →
spec IR → lowering IR → Wasm path. DisplayNames supplies construction,
`supportedLocalesOf`, `resolvedOptions` and `of`; RelativeTimeFormat supplies
construction, `supportedLocalesOf`, `resolvedOptions`, `format` and
`formatToParts`. Both families are installed in the initial Realm and every
created Realm, with registry-owned names, lengths, descriptors and intrinsics.
No user source enters the primitive provider.

## Observation and private state

Constructors require NewTarget and resolve its prototype before observing
locales and options. DisplayNames applies strict GetOptionsObject, reads
localeMatcher, style, the required type, fallback and languageDisplay in order,
and observes languageDisplay for all types. A missing type throws TypeError;
an invalid provided type throws RangeError. Only a language record retains
languageDisplay. RelativeTimeFormat boxes primitive options as required by the
pinned constructor, and reads localeMatcher, numberingSystem, style and numeric
in order. The supported-locale methods use their specified primitive option
boxing and produce fresh called-function-Realm Arrays.

Each constructor reserves a tagged result before observable operations and
publishes its private brand only after successful checked native resolution.
GetPrototypeFromConstructor uses the actual required Realm intrinsic when
NewTarget's prototype is primitive. Exhaustive intrinsic and fallback enums,
the closed global registry and the Realm record layout include both families.
The Realm record is 608 bytes; the new prototype fields follow the existing
fields at offsets 592 and 600. Both service records are 40 bytes and their
primitive string pointers participate in the existing traced heap model.

DisplayNames.of checks the private brand before ToString(code). A proof owner
exists only after that observable conversion, and the primitive request copies
the original UTF-16 units. The identity-matched native kernel validates and
canonicalizes codes. Invalid codes produce RangeError in the called method's
Realm; valid unknown codes produce the canonical whole-code fallback or
undefined according to the captured option. The IR and lowering shape retain
String | Undefined through lexical binding and subsequent reads. A captured String
intrinsic, equality check or concatenation supplies its own result kind.
Observable calls may invalidate a later mutable global String lookup; the IR
control captures the intrinsic before those calls. No String-only assumption
is attached to of.

RelativeTimeFormat methods check the brand, perform ToNumber(value), then
ToString(unit), then reject nonfinite values and invalid units. The completed
input proof owns those ordered conversions, finite original binary64 bits and
the closed singular unit. Plural unit aliases normalize only after ToString.
Negative zero retains its bits through the wire and selects the past numeric
pattern. Native formatting shares checked NumberFormat rounding and exact
cardinal PluralRules selection. Numeric-auto literals and genuine Arabic
numeric patterns without a placeholder remain complete literal-only output.

Both RelativeTimeFormat output methods call the same primitive parts operation.
Format concatenates returned text in Wasm. FormatToParts allocates fresh Arrays
and objects in the called method's Realm, retains part order, and creates unit
only when the response explicitly supplies it. Numeric parts retain the input's
singular unit, including NumberFormat bidi literals; pattern literals omit it.
ResolvedOptions uses exact service-specific key order and exposes only the
captured configuration and checked selected locale/numbering identity.

## Primitive boundary and artifact identity

Host ABI 10 appends operations 27–32 to the existing closed domain. Typed wire
builders accept branded receiver owners and completed input owners, rather
than unvalidated local numbers. Both native and Wasm frames retain the existing
version-one, two-u64 header and operation-times-two/direction convention. The
host copies and decodes the complete request before any response write, checks
buffer capacity and bounds, and rejects malformed frames as host faults.
DisplayNames code rejection has a separate path to the specified JavaScript
RangeError. Runtime faults cannot masquerade as a normal missing name.

The [provider integration](../intl-display-relative-provider.md) binds consumed
source, locked dependencies, data inputs and the actual host consumer into the
kernel and composite identities. Catalogue admission checks the existing 307
reachable currency codes through actual DisplayNames lookups with fallback
none, and checks every one of the 78 positional numbering systems through
actual RelativeTimeFormat resolution and formatting. These are production
admission requirements and precede immutable public lists.

The successor [RelativeTime data image](intl-relative-time-data-image.md)
carries the actual unchanged CLDR47 fourteen-locale template JSON. Admission
checks exact pinned bytes and the selected Locale/Number foundations before
decoding the complete native catalogue. The retained catalogue owns its
NumberFormat/PluralRules consumer. Selected formatting rejects a foreign
template locale owner before numeric-auto literals, including when the
foreign catalogue uses the very same Number owner. Primitive decoding uses
the installed catalogue; the default borrowed accessor shares its admitted
image cache. The complete twelve-owner provider has Embedded placement and a
full pinned Conformance producer. This source batch is uncompiled and untested.

## Verification scope

The authored Engine target has fourteen DisplayNames tests and eight
RelativeTimeFormat tests, each running strict and sloppy scripts: 22 Rust
controls and 44 planned modes. The native additions declare ten DisplayNames,
twenty-one RelativeTimeFormat and two shared provider controls. Six host tests
cover successful copy/capacity/overlap, semantic rejection or genuine
literal-only output, and malformed-frame faults before writes. Two IR controls
exercise DisplayNames's optional result. Source review and generation have
completed. The complete preceding source passes workspace/all-target
compilation. Its first native run records 314 passes and two failures across
316 controls, with no ignores: a Dialect/Standard fixture mismatch and an
extreme-finite spelling mismatch. This successor preserves both DisplayNames
expectations and uses the already pinned ECMAScript Ryu formatter at the finite
relative-number boundary. The existing numeric validator and genuine profiles
remain unchanged. The repaired source passes fresh workspace/all-target compilation, all 316
native controls, three public calendar controls, six host controls and the
artifact identity gate, with no ignores. The next IR checkpoint records one
pass and one failure because its new fixture assumed a mutable global String
lookup retained a static target after an observable call. This successor
captures String before those calls and passes both IR controls and all 11
catalogue controls after a fresh all-target compile. The next backend checkpoint
passes 71 of 72 heap controls and exposes an RTF temporary release out of stack
order. This successor reserves the two returned temporaries before its scratch
locals, preserving all emitted observations and the allocator assertion. Fresh
compilation then passes, followed by all 72 heap controls. The module
checkpoint passes four of five controls and reveals its stale last-global
expectation still names Collator. The complete Segmenter successor updates
that bounded registry fixture to its actual highest entry while preserving
uniqueness and density checks. Fresh affected backend/Engine verification
remains pending. MAIN and pinned admission remain pending.
The original native failure and the later IR failure are retained in
`target/continuation-intl-display-relative-focused-attempt2/005-ir-optional-value/ACTUAL.json` and
`target/continuation-intl-display-relative-focused-attempt1/001-native/ACTUAL.json`.

The complete pinned DisplayNames subtree contains 57 files/114 modes;
RelativeTimeFormat contains 80 files/160 modes. Two existing supported-values
consumer files add four DisplayNames observations in the enumeration scope.
Every selected mode remains included and any failure retains its owner and
reason. MAIN admission, broad workspace verification and full ECMA-402
conformance remain open. Segmenter and DurationFormat retain T23 ownership.

The behavioral owners follow [DisplayNames](https://tc39.es/ecma402/#sec-displaynames-objects)
and [RelativeTimeFormat](https://tc39.es/ecma402/#sec-relativetimeformat-objects),
with the actual Test262 content pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b` bound separately from source receipts.
