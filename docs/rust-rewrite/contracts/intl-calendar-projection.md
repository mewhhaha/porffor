# Localized DateTime calendar projection

`DateTimeDataImage::for_custom_data_projection` consumes a Custom ID, an optional
locale selector, checked `DateTimeCalendar` values, and the actual Locale and IANA
foundation images. An omitted locale selector retains every pinned locale row.
The calendar selection is nonempty, duplicate-free and canonically sorted in the
descriptor. The selected public locales include the original en-US fallback.

The producer first admits the complete pinned native profile through its existing
source recipe. It then retains each selected locale's requested calendar
associations and that locale's original default. This rule also applies to en-US;
its unrequested calendars are removed. Original calendar preferences remain intact
and determine the same default. Gregorian and ISO associations share physical data
only when both are retained; sharing does not admit an excluded calculation tag.
Only reachable calendar pools survive, with their original names, patterns,
intervals and source identities. Every retained association is densely remapped.
Zone-name pools, all 78 positional kernels, algorithmic fields, era supplements,
geography and the exact complete calendar blob keep their original closure.
The independent numbering-system selection can additionally prune localized
symbol maps while retaining actual defaults and those global kernels; see the
[numbering contract](intl-numbering-projection.md).

The schema-two `LILADTP1` descriptor binds requested calendar types, selected
locales, source pool IDs, Custom ID, pinned native source and both actual foundation
digests. Admission rederives the whole canonical payload and requires byte equality
before minting its private catalogue. The decoder's recipe consumes that catalogue
to validate the exact selected service inventory and per-locale default closure.
The schema-one locale-only descriptor omits the new field and retains its existing
bytes. Full Minimal and unfiltered Custom images retain their exact source payload.

Full and projected profiles use one checked calendar-record representation. A
localized calendar is selected only when that locale owns its actual record.
Unsupported options and Unicode calendar extensions keep the locale's default;
an unsupported extension does not appear in the resolved locale. Plan admission
returns a private pair containing the selected locale and its exact calendar
record. Real plan selection consumes that pair, so a caller's forged calendar
tag cannot index an absent association. Portable plans continue to bind image,
Locale and IANA digests and rederive their selection before formatting.

Localized service availability is separate from global calendar availability.
`available_calendar_kernels` derives the full sixteen-calendar inventory from
the actual admitted calculation array. Native Locale information and
`Intl.supportedValuesOf('calendar')` consume this full authority on the same
selected DateTime owner. Localized projection therefore preserves independent
regional `Intl.Locale.calendars` results while narrowing DateTime name and pattern
data. It adds no alternate calendar engine or fallback data representation.

Three authored controls compare the physical selected references and pool bytes
with the complete source, retain actual defaults and global data, compare real
Chinese/Japanese formatting, distinguish unsupported service calendars from full
Locale/kernel availability, reject forged plans and changed descriptors/defaults,
and retain portable plans after input owners are dropped. SDK/manifest/cache/CLI
controls consume the same native producer. Compilation, formatting, data
generation, runtime and test execution remain deferred; no size or conformance
result is inferred from these source changes.
