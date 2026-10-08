# RelativeTimeFormat immutable native data image

`RelativeTimeDataImage` carries the actual schema-one CLDR47 native template
catalogue consumed by RelativeTimeFormat. The payload is the existing
259,890-byte `relative_time_format/generated/profile.json`, SHA-256
`cf19084cea53a4ae2714e1062bf15b014f6460e24f53c6df50f2944f7690419d`.
The native inventory is `lila/relative-time/profiles/v1`, and the artifact
section is `lila.intl-relative-time.v1`. No ICU marker or exporter is claimed
for these Lila-native records.

## Admission and ownership

`for_profile` and `from_bytes` require the selected `LocaleDataImage` and
`NumberProfilesDataImage`. Admission checks frame integrity, the exact native
inventory, matching selected profiles and the Number image's Locale digest.
Minimal and named Custom producers preserve the same exact pinned data and
frame bytes. A separate `for_custom_projection` factory produces actual selected
template rows. Admission accepts either the original pinned payload or an exact
recomputed projection; an arbitrary replacement payload cannot claim the locked
source identity. Conformance remains unavailable.

The existing native decoder validates schema one, the exact CLDR47 commit,
all admitted locale associations, all 24 unit/style fields for each locale,
the complete six-category past/future patterns, placeholders and automatic
offsets. The template source, source capture and reports remain unchanged.
Literal-only genuine patterns retain their existing behavior.

Each admitted `RelativeProfiles` retains its actual `Arc<NumberProfiles>`.
Resolved relative locales can only be paired with that Number owner. Their
template locale records retain exact Arc identity in equality and admission.
Configuration construction continues pairing the resolved Number owner with
the same precision and cardinal-plural owner.

The selected `RelativeProfiles::format_parts` entry validates both Number and
template locale ownership before entering the formatter. This precedes the
numeric:auto literal return. Numeric formatting then uses the catalogue's
captured Number owner, preserving the existing ECMAScript finite spelling,
exact decimal rounding, plural selection, part order, unit tags and limits.

Root provider publication must match the image's Locale and Number digests
and require `uses_number_profiles` against the installed Number Arc. Digest
equality alone does not establish this ownership: independently admitted
Number images with identical bytes still have distinct catalogues. Primitive
wire decoding checks the selected catalogue/Number pair and reconstructs
configuration proof from those owners. The Engine host already calls the
selected kernel decoder. The default borrowed catalogue reads the one admitted
RelativeTime image cache rather than independently decoding static data.

## Actual native locale projection

`for_custom_projection` takes one selected Custom ID, structurally checked
requested locale identifiers and the actual selected Locale and Number images.
Its private catalogue constructor uses the selected Locale canonicalization
owner, requires every canonical request to name an exact row in the complete
admitted fourteen-locale source, and rejects empty, excessive, duplicate or
unavailable selections. The public catalogue always contains en-US once for
default resolution. The larger Number catalogue does not extend public
RelativeTime support. Normal prefix lookup, supported-locales behavior and
fallback use only the admitted selected template rows.

The factory first admits the complete pinned source through the existing
field/category/plural decoder. It then emits only the selected complete JSON
locale rows in sorted order. It canonicalizes JSON object key order so the
representation does not depend on serde_json's optional insertion-order feature.
No shared text pool or positional table needs remapping. The native marker and
primitive request/response wire schema remain unchanged.

The projected payload starts with `LILARTP1`, a little-endian u32 descriptor
length and u64 JSON length, then the descriptor and selected JSON. The descriptor
is bounded to 16 KiB and the JSON to the original pinned extent; all additions,
length conversion and total component extent are checked. The schema-one
descriptor binds the Custom ID, en-US default, full source SHA-256, selected
Locale and Number image digests, ordered public locale list and actual resolved
default numeric/numbering-system associations. The Number image remains complete
for Unicode `nu` negotiation and exact decimal/plural operations.

Admission parses only these bounded extents, derives the requested set from the
descriptor and recomputes the entire projected payload against the immutable
pins and supplied selected foundations. Byte equality is required before the
private catalogue reaches the selected-row decoder. Self-consistent modified
templates, omitted fields, foreign pins, altered numeric associations, incorrect
defaults, extra or reordered rows and changed labels cannot substitute data.
The decoder again validates every selected complete row and retains the supplied
exact Number Arc. Existing template Arc ownership gates precede automatic literal
formatting, and wire decoding remints constructor proof only for selected rows.

## Source controls and remaining work

Four new controls use actual admitted images. They exercise retained template
and Number owners after source/image handles drop; same-Number foreign
automatic-literal rejection and selected primitive decoding; self-consistent
altered-payload rejection; and mixed profile/foundation rejection with named
Custom consumption and foreign Number proof rejection. Existing semantic,
rounding, pattern, resource, wire and host controls are retained. The previous
foreign-Number automatic-literal control now targets selected RelativeTime
images as well.

Four additional projection controls cover actual selected payload rows/size,
all unit/style/category and automatic-literal formatting against the full owner,
selected public lookup/defaults with shared numbering negotiation, canonical
selection and invalid foundations, self-consistent payload/metadata corruption
and checked framing, selected primitive reminting, foreign automatic templates,
and exact Number lifetime/owner retention. Every earlier control and full native
source/output pin is retained.

The projection source is authored without compilation, data export, runtime or
test execution. Shared SDK/CLI composition and producer provenance belong to
Root's complete batch, with expensive verification deferred. The existing
twelve-component product source declares Embedded placement; this native lane
adds RelativeTime filtering only. General twelve-component filtering, full
service/data coverage, reproducible export verification and Conformance remain
open.
