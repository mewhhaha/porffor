# DurationFormat native foundation

This is a source-only native/data proposal. It adds no module registration,
provider capability, global opcode, compiler intrinsic, Engine host, CLI result,
or product admission. Operations 36–38 are reserved only in the local typed
request domain: locale resolution, supported locales and partitioned formatting.
All 15 native regression declarations remain unexecuted. T23 remains open.

The primary rule authority is ECMA-402, 12th edition (June 2025), section 13:
https://tc39.es/ecma402/2025/#durationformat-objects.
The captured CLDR 47 commit is 2ef784e3a4168bc2a43cd1b5b9839b6636f5899c.
The existing source archive is f85496f1e76ca0a12a0d7603c5f3243a94e2506926176f6ec7fe178327ffcebf;
its exact member authority is 582f922168284a76255554a4b74c6200f0e954853ef2eaf49bceb393a2d48b49.
Separate payload licensing lives in data/duration-cldr-47/LICENSE.

The service-local source selection is ar, ar-EG, de, en, en-US, es, fr, hi, it,
ja, ko, sr, zh, zh-Hans and zh-Hans-CN, with en-US as DefaultLocale.
This does not broaden another service's locale catalogue. Every locale must
resolve to its exact NumberFormat profile. Its actual checked ListFormat owner
must be that locale or a genuine prefix ancestor; es and sr require their own
named owners. Admission compares every unit-list style against actual 2/3/4
ListFormat partitions before publishing a Duration locale proof. The retained
owner is reused by every configuration. These admission checks are authored,
not executed in this packet.

The producer records 2,925 exact primary leaf observations: 2,700 unit/plural
patterns, 180 unit-list templates and 45 digital patterns. All 2,700 unit patterns
are independently reconstructed from the already-admitted NumberFormat canonical
tables and required to match before export. Plural lateral fallback uses the
requested count and then other within each source locale before its parent;
case defaults to genuine nominative. Alias targets, inherited markers, source
locale/path, qualifiers and full strings are retained. No output oracle or
synthetic locale label is used. All 78 numbering systems remain the existing
NumberFormat domain, including its exact CLDR 48/UCD 17 ToLS supplement.

Digital hm/hms/ms patterns are parsed into checked field widths and separators.
The Serbian source is h.mm.ss. Its dots remain dots; generic number timeSeparator
is not substituted into a duration pattern. LDML 47 defines no replacement
pattern character for that symbol. Unsupported fields, quotes, endpoints or
inconsistent separator contexts reject source admission rather than approximating.

DurationRecord is the only native formatting input. It accepts completed IEEE
Number fields, checks finite integrality and uniform signs, ignores negative-zero
fields when finding DurationSign, requires absolute years/months/weeks below 2^32,
and checks the exact normalized day/time sum below 2^53 seconds. Day contributes
86,400×10^9 nanoseconds. Integral IEEE magnitudes are reconstructed from their
actual significand/exponent; no decimal-shortest approximation or f64 summation
is used. Checked multiplication/addition rejects mathematical bound overflow.
The admitted magnitudes and sign become immutable, so subsecond absorption fits
u128 and creates the existing canonical ExactDecimal owner without reparsing.

CheckedDurationConfiguration owns all ten effective unit styles/displays, the
numeric/fractional continuation, adjacent padding and 0–9 fractional precision.
Invalid textual/numeric transitions, forbidden fractional always-display and
unsupported unit styles reject construction. Exact decimals go through the real
shared NumberFormat partitioner, which also owns cardinal selection and rounding.
Fractional combinations use truncation. Only the first displayed group owns the
sign, including a leading displayed negative zero. Digital zero-minute bridges
and auto suppression use exact unrounded values. Genuine checked ListFormat
partitions retain element order; number-pattern parts carry their singular unit,
while list and digital separator literals have no unit. Part/text limits are
validated before the complete native partition is returned.

Future AOT work must complete ToDurationRecord observations before crossing this
primitive boundary. Duration-like bag reads remain alphabetic as specified,
including partial-record/error precedence. Reuse the actual T22 completed branded
record/string-parser seam; do not invoke getters on branded Temporal.Duration
objects or export raw heap arrays. Constructor/newTarget/realm ownership,
format/formatToParts/supportedLocalesOf/resolvedOptions, global dispatch, byte
codecs, compiled consumers, provider identity and all 111 pinned files/222 modes
are mandatory later whole-batch work. No prior CLI or MAIN receipt transfers
across that future source change.

The component recipe binds the new production code, exact captured profile and
producer closure, actual NumberFormat/PluralRules/ListFormat algorithms and
identities, and the real locked transitive package identities. Tests and its own
generated identity file are excluded. Source tamper controls detect arithmetic,
list, raw-source and dependency changes; native controls cover unsafe IEEE
integers, limits, signs, option transitions, padding, fractional truncation,
Serbian literals, part ownership, all fifteen service associations and all 78
numbering systems. Source checks do not establish native or product success.

The complete native/host successor activates the existing DurationFormat service,
appends global operations36–38, adds the DurationPatterns capability, and binds
the seven-component provider ABI12 composite to actual source identities. It
reuses one immutable cached Duration/NF/List owner before provider publication;
the fifteen service-local locale associations and all78 numbering systems are
validated through actual provider operations at catalogue admission. Seventeen
Duration algorithm controls, twelve wire controls, two provider controls and four
host controls are declared; all remain unexecuted in this source lane. The
independent AOT/compiler callers and all111 pinned files/222modes remain required
before product admission. Original source44/wire7 and Source2/Source3 failure
receipts remain immutable and are not converted into passing claims.
