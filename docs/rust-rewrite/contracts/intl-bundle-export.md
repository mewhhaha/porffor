# Canonical admitted Intl bundle export

`SelectedIntlDataBundle::export_bytes` publishes the same immutable
component frames emitted by the selected provider. Legacy version one has an eight-byte
magic/version, bounded canonical provider identity and a fixed twelve-component
inventory. Each component has its fixed index, zero reserved word and a bounded
little-endian length. There is no timestamp, filesystem path, machine locale or
input-manifest spelling in the bytes.

`SelectedIntlDataBundle::from_export_bytes` rejects unknown versions, incomplete
or reordered inventories, reserved words, oversized/truncated frames and trailing
bytes. It then uses every original typed `from_bytes` constructor. Locale and List
are admitted before Number when required; the same actual Number owner is passed to RelativeTime
and Duration, and the same actual List owner is passed to Duration. Named zones
precede DateTime and TimeZoneNames; LocaleInformation retains the same actual
DateTime owner. The original provider association, pinned source, selected payload,
capability and supported-values validators all run before publication. The
recomputed full provider identity must equal the stored canonical identity.

This route never accepts a checksum or a Custom name as a data proof. Import
rederives the original projected native payloads and remints their private
catalogues. Its returned bundle has the same opaque fields and frame/catalogue
methods as a newly selected bundle. Complete Conformance admission still requires
the full current pinned source closure and does not imply Test262 success.

The CLI `lila intl export --output PATH` consumes the existing global
`--intl-profile` or `--intl-manifest` input; omission selects Minimal. Selection and
export finish before an output transaction starts. A synced sibling temporary
file is published with an atomic hard link that refuses an existing destination,
then the temporary name is removed and the parent directory is synced on Unix.
Unsupported filesystem publication rejects clearly. `lila intl inspect --input
PATH` bounds its read and fully re-admits the bundle before printing its actual
canonical provider identity byte-for-byte, including its canonical trailing
newline. The 2026-10-10 continuation removes an extra formatting newline and
compares CLI stdout directly with the SDK identity bytes. It accepts no
independent selection flags.

SDK controls exercise selected roundtrips, identical re-export bytes, whole
Conformance data, damage/inventory/identity refusals, and a same-Custom-ID List
swap which violates the actual Number dependency digest. CLI controls compare two
separate-process exports with SDK bytes, exercise actual inspect and damaged-input
refusal, preserve an existing output and reject an unsupported manifest dimension
before output creation. These controls are authored and unrun; no cross-host
measurement or reproducibility result has been claimed.

Version two uses `LILAB002`, the same sixteen-byte header, and a checked nonzero
requested-service bitset at offset 14 in place of the version-one reserved word.
The component count at offset 12 must equal the exact dependency closure; each
present component retains its original stable ID, even when earlier IDs are
omitted. Sparse imports reject missing, extra and reordered frames before native
admission. Explicit ALL uses version two; legacy complete selections keep their
exact version-one bytes. Public service permissions remain separate from shared
dependency frames. Missing supported-values data is an explicit per-key error.

The package exports current admitted source data, including physical locale,
currency, calendar, numbering, named-zone and service selections. It does not
execute ICU datagen, import arbitrary upstream data or establish external AOT
placement. Cross-host reproducibility and runtime controls remain unrun.
