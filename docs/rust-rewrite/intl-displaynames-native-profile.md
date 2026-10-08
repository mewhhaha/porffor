# DisplayNames native source foundation

This source packet implements the finite pure DisplayNames service, a genuine
CLDR47 profile producer, checked code/configuration/profile owners, and lossless
primitive codecs. Product constructor registration, JavaScript observations,
Realm behavior, Engine/AOT dispatch and the final provider identity join are
the original packet's separate pending work. The complete isolated successor
now connects those consumers and the shared provider/host domain; its
[Wasm contract](contracts/intl-display-relative-wasm.md) and
[provider integration](intl-display-relative-provider.md) record the joined
source and pending execution gates. No DisplayNames product or pinned-suite
pass is claimed.

The input is the already captured CLDR47 source tree at
`crates/lila-intl/data/datetime-cldr-47`, commit
`2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`. The profile selects exactly `ar`,
`ar-EG`, `de`, `en`, `en-US`, `fr`, `hi`, `it`, `ja`, `ko`, `zh`, `zh-Hans` and
`zh-Hans-CN`; its default is `en-US`. These are proposed native data associations,
not a broader product locale claim. The existing LDML resolver checks every
input hash, declared parent and subtree alias. The complete pinned BCP47 input
manifest additionally binds the calendar identifier-to-LDML mapping. The
inherited Unicode license is retained in those source directories.

All six public types have long, short and narrow records. The name pool interns
only equal canonical JSON records of the same domain. There are 105 records and
312 locale/style/domain associations; the payload is 1,617,788 bytes. Removing
references reconstructs every original selected name exactly. Provenance binds
19,183 actual primary XML leaves. Each record is validated once before the
private closed pool enum exists; each association checks the pool index and
domain before sharing an Arc. Duplicate, unordered, unused or wrong-domain pools,
invalid names, unresolved inheritance markers, an incomplete dateTimeField set,
changed source identities and an altered catalogue reject admission. Pointer
identity never establishes canonical reference ownership.

The producer uses supplied narrow, short and default name forms in that order
for narrow; short and default for short; and default for long. It never truncates
or translates text. Independent script names prefer actual stand-alone forms;
language composition uses the regular script form. Currency names come from the
count-less currency `displayName`, never a symbol or a plural label. Calendar
names use actual BCP47 canonical/deprecated identifiers and their registered LDML
source keys; this domain is independent of DateTimeFormat's admitted 16 set.
The twelve closed dateTimeField keys use actual CLDR `week`, `dayperiod` and
`zone` field records for `weekOfYear`, `dayPeriod` and `timeZoneName`.

Language input first passes the Unicode language-id grammar and duplicate-variant
check, then reuses the complete existing native CanonicalizeLocale operation.
That preserves its pinned aliases and reserved five-to-eight-letter language
handling. Deprecated debugging translations remain visible in source provenance
but cannot become a field for a different canonical language identifier. Dialect
selects the longest actual language-subtag match and composes any remaining
qualifiers with the captured localePattern and localeSeparator. Standard uses the
base language plus those genuine qualifier names. Composition supplies a name
only when all required names exist. A missing field returns the complete
case-regularized/canonical code for fallback `code`, or None for fallback `none`;
None means JavaScript undefined. No partial guessed name is added.

Region, script and currency perform their prescribed ASCII case regularization.
Calendar accepts unknown well-formed three-to-eight-character alphanumeric
hyphen-separated subtags and only lowercases them. dateTimeField is case-sensitive.
UTF-16 code units remain unchanged on the request wire, including isolated
surrogates; invalid codes reject only when the native code constructor sees them.
The languageDisplay slot exists only in the closed Language selection. Eventual
JS code must still observe and validate that option for every constructor type
before discarding it for nonlanguage selections.

The three closed wire operations reserve global tag 27 ResolveDisplayNamesLocale,
28 SupportedDisplayNamesLocales and 29 DisplayName. They use the current version 1
header and operation-times-two/request-or-response convention. Decoders reject
wrong operations/directions, trailing or truncated bytes, unknown domain codes,
invalid lengths, unadmitted resolved locales and nonlanguage languageDisplay
slots. Supported locale responses preserve supported original requested tags,
while resolved locale owners contain an actual admitted data locale without
irrelevant Unicode extensions. The final shared operation table must map these
tags exhaustively; the original source packet does not edit that table. The
isolated coherent successor supplies those exhaustive mappings under ABI 10.

Actual source verification: the producer reproduction check exits 0, all eight
Python controls pass, and Rust source formatting passes. The Python controls
independently compare every consumed value to its pinned primary XML text and
verify lossless pooling, complete domains, actual source vectors and tamper
rejection. The first reproduction attempt failed only because rustfmt split two
generated identity declarations; the retained failure is followed by the corrected
producer and passing check. No Cargo command or native test ran in this lane.

Ten native controls are declared and remain unexecuted. They cover complete
admission, all six types/styles, actual widths/scripts/currencies, dialect and
standard composition, aliases and unknown valid codes, pinned grammar vectors,
fallback None/code distinction, resolution, malformed profiles and lossless/wrong
wire frames. A consumer control checks all 26^3 currency codes against the actual
NumberFormat reachable 307 union, and every required DateTimeFormat calendar in
all three styles. These are acceptance gates, not source-derived runtime results.

Refresh the source artifacts with
`python3 -B scripts/generate-intl-displaynames-profile.py` and reproduce them with
the same command plus `--check`. Run the eight source controls using
`python3 -B -m unittest discover -s scripts/tests -p test_generate_intl_displaynames_profile.py -v`.
The owner-run isolated native foundation must register both new modules and run
`cargo test --offline --locked -p lila-intl --lib display_names::tests:: -- --test-threads=2`
before claiming native admission. The final whole product batch must complete
its compiler/Engine/AOT gates and all pinned DisplayNames modes, including the
calendar and currency enumeration consumers.

Normative code and slot requirements are
[ECMA-402 DisplayNames](https://tc39.es/ecma402/#sec-displaynames-objects), with
name composition grounded in
[Unicode LDML locale display names](https://unicode.org/reports/tr35/tr35-general.html#Locale_Display_Name_Algorithm).
The current vendored Test262 content pin is
`aa55200d1310384c5cf69ea95b2a2ecba457007b`; its exact consumed fixture bytes and
execution mode inventory belong to the source packet metadata.
