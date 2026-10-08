# Immutable Intl component images

The Locale/List/Collator foundation passed the all-feature, all-target Rust type
checkpoint at MAIN142 on 2026-10-05. Its build exported the actual pinned ICU
payloads; runtime and reproducibility acceptance remain pending. The successor
complete twelve-component source batch is uncompiled and unexecuted.

## Actual ownership and consumption

`lila-intl/build.rs` exports the exact locked ICU data before execution through
`BlobExporter`; fixed marker order, exact sorted identifier enumeration and the
exporter's final sorted resources avoid hash-map iteration in emitted blobs.
It writes Locale, List, Collator and Segmenter binary payloads into Cargo's
output directory. Segmenter includes its exact pinned locale descriptor;
Collator carries its own exact 1,082-name native locale inventory.
Calendar and IANA exporters provide actual singleton calendar payloads and
all 598 pinned timezone transition records.
There is no parser, JavaScript VM, host-locale query or OS data lookup in this
provider. Source-owned Collator generated data remains the genuine existing
nine-marker export; no generated payload or pinned input is replaced.

The private closed component registry has twelve implemented cases.
A frame carries schema, component, selected profile, fixed canonical default,
exact upstream versions, sorted complete marker inventory, actual payload bytes
and SHA-256. Manifest and image extents are bounded before allocation or typed
payload loading. Noncanonical manifests, unknown fields, trailing bytes,
unsupported versions, changed inventories and corrupt digests are rejected.
The framing type is private: only a family constructor that loads real data
can publish its admitted immutable owner.

Locale retains five actual singleton marker payloads: aliases; common,
extended and script/region likely subtags; and locale parents. Its canonicalizer,
expander and fallbacker use the deserializing blob provider. Reserved-language
rules retain the same admitted alias payload. The image also owns its native
CLDR47 keyword payload: 60 Unicode and five transform alias rows. Runtime
canonicalization uses this retained authority; the source-generated rows are
build inputs and independent test evidence. Native Locale information and
country membership have separate selected owners described below.

List and Collator use that admitted fallbacker and canonicalizer. They retain
its digest, fully admit their actual formatter/profile catalogues and reject
configuration proofs from a different image owner. The selected provider checks
all component profiles and Locale foundation digests before publication. Host
request decoding receives that same selected kernel, so a decoded request
cannot silently acquire a global default proof. Collator's exact cached
singleton/indexed inventories remain its only data model. List binary
conditional DFA decoding requires a little-endian host; unsupported big-endian
admission explicitly fails instead of losing conditional formatting patterns.

The NumberProfiles image carries the genuine 6,091,829-byte LNF47 schema2
binary and its exact payload descriptor. Its native schema marker is
`lila/number/profiles/v2`; it is not an ICU marker. Descriptor admission binds the
CLDR47 data plus the separately recorded tols numbering supplement from
CLDR48/UCD17. NumberFormat and PluralRules consume one retained decoded owner,
including cardinal/ordinal rules, range matrices and compact exponents. Dynamic
admission owns the decoded text instead of leaking it into static storage.
Resolved configurations retain the full owner and reject foreign proofs before
numeric normalization or table lookup. RelativeTimeFormat and DurationFormat
retain that same selected numeric owner in their own admitted template images;
RelativeTimeFormat checks template and Number ownership before a numeric:auto
literal can return.

Segmenter carries seven genuine ICU families, all exact identifiers and the
existing checked 17-locale descriptor. Admission requires all base tables,
four LSTM model prefixes, cjdict and the Finnish/Swedish word and Greek sentence
overrides. Grapheme, word and sentence constructors consume the same image and
Locale fallback authority. The retained grapheme owner replaces the remaining
baked constructor in UTF16 partitioning; foreign configurations fail before
partitioning. Current native and wire behavior is preserved.

DisplayNames carries its actual 1,617,788-byte native JSON, admitting 13 locales
and 105 typed name pools with retained Locale canonicalization data. Foreign
template proofs fail before code parsing or name lookup. Reserved-language
alias evidence is derived once per image admission. Its pure Locale owner also
retains the actual selected keyword aliases.

RelativeTime carries the actual 259,890-byte native CLDR47 template JSON and
retains the selected Number/Plural owner. Duration carries its actual
136,434-byte native CLDR47 template JSON and retains selected Number and List
owners, including resolved List rows captured during admission. Its complete
catalogue operation rejects foreign template proofs before empty, numeric or
text branches and performs final composition through the selected List owner.
The provider checks these actual Arc associations alongside their content
identities before publication; identical frame bytes cannot authorize mixing
independently decoded dependent owners. Each frame names its genuine native
schema rather than an unrelated ICU marker.

NamedTimeZones carries the complete IANA2026a catalogue and actual TZif records,
the original 418-row zone.tab and checked 247-country projection. Admission
retains the existing transition, POSIX-tail, topology and source validators.
Country membership holds that same named-zone Arc. Runtime does not query
jiff-tzdb or an OS database; the exact pinned package is a build/test input.

DateTime carries the actual 9,879,674-byte native profile JSON and four genuine
calendar singleton markers: Chinese, Dangi, JapaneseModern and the vendored
Umm al-Qura payload. Its algorithms consume the admitted calendar data and
retained Locale and IANA owners. Opaque LILADTF2 plans bind all three content
identities before scalar/range conversion or rendering. Identical-byte plans
remain portable across independent admissions; foreign content is refused.

TimeZoneNames carries the actual 160,640-byte native CLDR47 profile: 446 zones,
600 aliases, 190 metazones and 669 UTC periods. Its complete resolve operation
owns the selected IANA lookup, transition/stability query and its own name
formatter. NativeLocaleInformation owns the four existing calendar, hour-cycle,
week and text tables, retains the selected DateTime calendar inventory and
Locale keyword authority, and dispatches all four real native operations.
Provider publication requires these actual dependent owners to agree; equal
digests alone cannot authorize mixing independent decoded foundations.

## Artifact and cache boundary

A Wasm module with an actual Intl or system-zone host import carries one complete
provider identity and the actual twelve component image sections:

- `lila.intl-locale-data.v1`
- `lila.intl-list-data.v1`
- `lila.intl-collator-data.v1`
- `lila.intl-number-profiles.v1`
- `lila.intl-segmenter-data.v1`
- `lila.intl-display-names-data.v1`
- `lila.intl-relative-time.v1`
- `lila.intl-duration-data.v1`
- `lila.intl-named-time-zone-data.v1`
- `lila.intl-datetime-data.v1`
- `lila.intl-time-zone-names-data.v1`
- `lila.intl-locale-information.v1`

The provider digest binds their framed digests and all existing host data
identities. The Engine scans and admits the component group while the artifact
is inert, then validates its complete identity once. This runs before native
compilation, module-cache use and start code. Exact default bytes reuse the
already admitted cached default kernel; other allowed frames construct an
actual selected provider. Missing, duplicate, unexpected or incompatible image
sections use the existing typed artifact rejection/eviction boundary. No image
failure falls back to baked host data.

Static-only supported-values names are actual compiled catalogue data. Their
producer identities are checked at compilation; the recursive compiler-source
fingerprint includes the Intl source, exact Cargo lock and emitter. Those
artifacts contain the same selected twelve-frame provider group. Program bytes and native
module cache keys include actual image sections whenever the host is used.

## Profile truth and remaining acceptance

Minimal and Custom-named frames currently require byte equality with each actual
build-exported or native pinned component payload before typed admission. A self-described
version or recomputed digest cannot turn arbitrary data into pinned data.
A matching twelve-component Custom ID selects that same named Custom profile
in the provider plan and canonical artifact identity. Its selected service and
transitive operation capabilities remain the full current pinned closure;
Minimal components retain the Minimal plan. Mixed component labels or actual
foundations reject before provider publication. Changing only a Custom artifact's
profile field to Minimal rejects the otherwise unchanged artifact identity.
Unfiltered Custom IDs name the complete pinned data. Eight actual locale
projections are available through one checked Custom selection; currency,
calendar, numbering and zone manifest dimensions remain separate work.
The explicit Conformance producer requires the complete unprojected twelve-owner
group. Its private proof checks all component profiles and full physical source
payloads before obtaining the complete capability plan; the consumed
supported-values catalogue validates before provider publication. Every current operation consumes its actual selected image
owner, so the finite provider declares Embedded placement. This describes data
ownership; it does not establish full service coverage or Test262 conformance.

Configured named system zones retain complete provider identity binding and
reject Custom-named component groups. UTC and fixed offsets accept selected
component identities because they need no named-zone provider proof. SDK and
CLI select Minimal, Conformance or named Custom data; locale projection flags
require Custom. Broader custom image selection, size measurement
and cross-host stable-byte evidence remain acceptance work.

## Meaningful authored controls

Three private frame controls cover canonical component/profile/inventory
binding, corruption/truncation/trailing/version rejection and explicit
incomplete typed Conformance/unknown-field refusal. Complete selection controls
consume actual frames/catalogues and reject checksum-valid projected data
relabeled Conformance. Three Locale controls consume genuine alias,
likely-subtag and parent data, retain immutable cached ownership, bind Custom
identity without changing data, and reject a foreign payload even with a valid
manifest/checksum. Provider controls reject mixed components and independently decoded native
foundations before publication. List and Collator own semantic admission and foreign-owner controls
in their adapter contracts.

Three Engine controls cover exact default-kernel reuse; missing, duplicate,
corrupt, unexpected and wrong-identity sections; actual Custom-named selected
kernel consumption, canonical Custom profile metadata and rejection of an
otherwise unchanged artifact relabeled Minimal; and real JS-to-Wasm execution of Locale/List/Collator,
Number/Plural scalar and range, RelativeTimeFormat numeric and auto templates,
DisplayNames names, Duration/List composition and Segmenter through selected
host decoders with the default UTC Realm. The real Wasm control also exercises
keyword aliases, DateTime formatting in UTC and America/New_York, all four
native Locale queries and actual country timezone membership. A provider
control rejects independently decoded Locale/IANA/DateTime/name/information
foundations even when all framed content digests match. Existing Intl AOT and GC host
controls remain the default-image regression surface. These controls are
written and unexecuted. A capped whole-batch compilation and focused regressions
must precede any completion claim.
