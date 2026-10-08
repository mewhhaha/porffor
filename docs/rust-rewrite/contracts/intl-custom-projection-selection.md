# Custom component projection selection

`IntlCompilationProfile::CustomProjection(CustomIntlProfile)` carries one checked
Custom ID and the List, RelativeTime, DisplayNames, Duration, Number, DateTime, Collator and Segmenter locale filters
whose native producers exist, plus localized currency, calendar, numbering,
named-zone data and a checked public service selection.
Its private constructor-owned fields reject an empty component filter,
duplicate structural locale IDs, malformed IDs, and a selection with no filters.
The locale inputs are sorted before becoming cache inputs. Selected Locale-backed
admission performs canonical alias and pinned-row validation later, when the
actual selected images are available.

The strict version-1 [external Custom manifest](intl-custom-manifest.md) constructs
this same owner through the SDK and `--intl-manifest PATH`. It admits only the
eight actual locale filters, rejects unknown dimensions and ambiguous CLI sources,
and passes through the same selected physical producer/consumer gates. Later
manifest versions add actual data dimensions; version 6 selects public services
and required frames through the same checked owner.

Version 2 additionally admits `currency_codes`. The existing nine-argument
locale constructor remains unchanged; `for_currency_codes` builds a currency-only
selection and `with_currency_codes` composes it with locale filters. Real Number
labels and overrides and DisplayNames Currency pools are physically projected,
with exact rederivation and a paired provider gate. Original omitted-code
fallbacks and full global fraction authority remain. See the
[currency contract](intl-currency-projection.md).

An absent filter retains that component's complete pinned Custom domain. A List
filter calls the physical List row projection; a RelativeTime filter calls the
physical RelativeTime JSON row projection. A DisplayNames filter calls the
physical native row-and-pool projection, retaining complete names for all six
domains and three widths while deleting unused typed pools and remapping their
references. A Duration filter projects complete native rows, including digital
patterns, unit patterns and list associations, and always retains the actual
en-US fallback row. A Number filter projects complete native rows and their
actually referenced pools for one coupled public NumberFormat and PluralRules
domain. The sixth constructor argument is this optional Number filter. The
seventh argument is the optional DateTime filter. It selects complete native
locale rows and the actual calendar and zone-name pool closure, with compact
references and the mandatory en-US fallback. Every retained locale keeps all
sixteen calendar associations; all 78 digit systems, both algorithmic field
rules, geography and the full calendar marker blob remain admitted. The eighth
argument is the optional Collator filter. It retains exact original Postcard
rows physically consumed by complete default-sort, search, all available sort
types, root emoji/eor and conditional numeric/shifted constructors. Its private
raw provider uses the same selected Locale fallback adapter and retains no full
blob fallback. The ninth argument is the optional Segmenter filter. It retains
all three global Unicode rules, all four LSTM models and the CJK dictionary,
while projecting actual locale associations and successful selected word and
sentence override rows. Its exact raw provider preserves both model and cjdict
attribute-prefix lookup beneath the original Locale fallback adapter. The
complete pinned source validates before the private selected catalogue is
admitted, and every projected byte is rederived before consumer publication.
All eight
retain the same selected Locale foundation.
RelativeTime keeps the exact selected Number owner; Duration keeps the exact
selected Number and List owners, including the private List dependency rows
required for Duration. A public List projection that excludes fr and sr must
still permit selected fr and sr Duration formatting through those private rows;
they must remain absent from public List support. Number's selected public rows
always include en-US. The same admitted Number owner retains the private rows
required by the selected RelativeTime and Duration catalogues, with independent
purpose checks. Those rows cannot become public Number or Plural locale or wire
proofs. Complete default numbering-system authority and all 78 digit systems
remain available to Locale and supported-values consumers. Number admission
binds the selected List digest and the exact dependent RelativeTime and Duration
catalogues. DateTime retains the selected Locale and IANA owners. Native Locale
information still captures that exact DateTime provider Arc and digest, and
derives available calendars through its real selected kernels and native pools.
DateTime filtering does not remove Locale calendar authority for an excluded
DateTime locale. All twelve images pass through the same provider identity, digest
and Arc association gates before publication.
Collator's public formatting catalogue is restricted to selected canonical rows
and en-US. The same owner retains the complete small Locale collation preference
catalogue; excluded formatting locales still answer `Locale.getCollations()`
through that checked authority, with actual selected root profiles for unmatched
locales. Neither these names nor a hidden locale mint public wire configurations.
Projection admission rederives the complete descriptor and row buffers from the
exact original source and selected Locale digest, then compares all bytes before
constructing actual typed consumers.

The SDK, Engine cache and CLI must consume this single selection. Cache keys
bind the Custom ID and the presence and ordered structural locale inputs of each
component independently in cache version eighteen. Number's presence and sorted
locale list are a fifth independently framed field; moving the same locale
bytes to another component cannot reuse the cached artifact. DateTime adds a
sixth field with the same presence, count and length framing. Collator adds a
seventh field with independent presence, count and length framing. Segmenter
adds an eighth field with the same independent framing. CLI filters
`--intl-list-locales`, `--intl-relative-time-locales`,
`--intl-displaynames-locales`, `--intl-duration-locales` and
`--intl-number-locales`, `--intl-datetime-locales`, `--intl-collator-locales` and
`--intl-segmenter-locales` are valid only with
`custom:ID`, and the
selected profile remains specific to the Wasm AOT path.
Currency selection adds its own presence, count and length-prefixed canonical
codes. The external manifest consumes the same checked fields; its filename,
JSON ordering and spelling do not become independent cache inputs.
Localized DateTime calendar selection adds an independent field with the same
framing. Manifest v3 and the SDK compose it with currencies and locale filters;
the original per-locale defaults remain physically present, while the global
kernel and LocaleInfo authorities remain complete. See the
[calendar contract](intl-calendar-projection.md).

Manifest v4 and the SDK numbering selection add another independent cache field.
The same canonical typed numbering names reach both real Number and DateTime
factories. Their localized associations retain requested systems and each actual
default; Number's private RelativeTime and Duration rows follow the same rule.
Complete default-nu lookup, digit kernels and algorithmic authorities remain on
the original owners. Provider/export admission requires equal requested domains;
global supported numbering values derive from the full admitted kernel catalogues.
See the [numbering contract](intl-numbering-projection.md).

Manifest v5 and the SDK named-zone selection bind an independent canonical
primary-ID field. The actual Named factory keeps selected primary and UTC records
and their genuine aliases; DateTime and internal timezone-name producers project
their physical name closure against that exact retained owner. Full IANA identity
and country authorities remain intact. A known omitted transition reports a
Custom data-unavailable error, preserving identity lookup without inventing data
or selecting UTC. Fixed offsets remain available. Provider/export admission binds
the actual requested domain, digest and retained Named Arc across the dependent
images. See the [named-zone contract](intl-named-zone-projection.md).

The ordinary library and CLI controls retain Duration-only and four-component
selections, compare actual twelve-frame artifacts and canonical identity with
the SDK selection, and execute selected formatting through strict/sloppy scripts
and modules. They exercise all four Duration styles, actual parts, Temporal's
intrinsic consumer, excluded-locale fallback, real Serbian digital separators
and private French/Serbian List composition. CLI parser controls retain early
rejection of duplicate, malformed, missing and unsupported-command selections.
The Number-only and five-component controls compare real compiler-produced
artifacts with the SDK graph, then exercise the coupled public locale domain,
parent lookup, decimal and BigInt formatting, parts and ranges, Polish plural
categories, private French RelativeTime and Serbian Duration formatting, and
Locale numbering authority outside the public Number domain. Negative admission
controls pair the actual Number frame with a different selected List or a
different dependent catalogue. No positive artifact rewrites substitute for the
normal producer.

DateTime-only and six-component controls compare ordinary compiler-produced
twelve-frame artifacts and canonical identity with the SDK graph. The selected
DateTime frame has fewer native rows and pools; all unfiltered component frames
remain exact. The actual library and CLI run strict/sloppy scripts and modules,
exercising selected Arabic digits, Chinese calendar fields, CLDR intervals,
parts, original New York transitions, all sixteen calendar associations and
Temporal's intrinsic DateTime consumer. Excluded-locale fallback, parent lookup,
opaque plan portability and foreign-image refusal remain explicit. The existing
provider Arc/digest association gates and inert Engine image reader are consumed
unchanged. Collator-only and seven-component controls compare real compiler and
CLI output with the SDK graph, including a hostless supported-values producer.
Strict/sloppy scripts and modules exercise genuine German default/search and
phonebook ordering, Swedish tailoring, canonical NFD equivalence, numeric shifted
comparison, original UTF-16 handling, selected String.localeCompare, private
Locale preferences and fresh supported-values arrays. Existing constructor and
control cohorts receive only an eighth `None` in that predecessor.

Segmenter-only and eight-component controls exercise the normal emitted graph,
strict/sloppy scripts, modules, retained UTF-16 partitions, Swedish word and
Greek sentence overrides, Thai and Burmese models and the CJK dictionary outside
the public locale domain. Native controls measure physical raw-row removal,
exact original prefix responses and all mandatory omission failures. The same
selected public catalogue governs wire reminting and foreign-profile refusal.
SDK/CLI artifact equality, cache field separation and validation before source
loading remain actual consumers. All existing constructor calls gain a ninth
`None`, except the actual CLI producer which passes its Segmenter selector.

This is a source-only successor to the seven-component product selector. Minimal and
the unfiltered named Custom selection retain their existing image producers.
The source recipe binds each real projection owner; generator
refresh, emitted size measurements, compilation and runtime verification remain
deferred until the complete source batch is stable. The eight selectors cover
the current service locale-row owners. Locale transforms, IANA zones and Locale
preferences remain complete authorities; the separate English timezone-name
frame has no independent per-locale row partition. Broader Custom manifest
dimensions remain open tasks. The separate `conformance` selection admits the
complete unprojected pinned group through a private source proof; Custom filters
cannot mint that proof. See the [Conformance data contract](intl-conformance-data-image.md).

`for_services` and `with_services` accept a nonempty unique list of exact service
names. A private checked plan derives the required frame closure. Data dimensions
aimed at absent components reject rather than becoming unused labels. Locale is
always retained as a foundation; shared Number/List and other dependencies never
grant their public formatter services. The builder creates only required frames,
and export, Wasm emission and inert admission consume that actual inventory.
Supported-values keys use retained owners independently; a valid missing key
throws a clear data-unavailable TypeError, while invalid key spelling remains
RangeError. See the [service contract](intl-service-projection.md).
