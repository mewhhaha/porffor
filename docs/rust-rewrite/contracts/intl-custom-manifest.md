# Strict Custom projection manifests

The SDK `CustomIntlProfile::from_manifest_json` and CLI `--intl-manifest PATH`
consume the same versioned input and construct the existing private-field
`CustomIntlProfile`. This is an input route to the eight physical locale
projections. Version 2 also selects localized currency data from the actual
Number and DisplayNames owners. These versions use the same provider and profile.
Version 3 adds localized DateTime calendar data and composes the same selections.
Version 4 adds paired localized Number and DateTime numbering-system data.
Version 5 adds selected named-zone transition data and its real name consumers.
Version 6 adds actual public service and frame selection.

```json
{
  "schema_version": 1,
  "custom_id": "my-profile",
  "locale_filters": {
    "list": ["es", "he"],
    "relative_time": ["fr", "pl"]
  }
}
```

The closed optional keys are `list`, `relative_time`, `display_names`, `duration`,
`number_plural`, `date_time`, `collator` and `segmenter`. NumberFormat and PluralRules
share the same physical selection. An absent key retains that component's full
pinned domain. A present key must be a nonempty list of structurally valid unique
locale IDs; null is rejected. At least one filter is required. The existing
constructor sorts the structural locale IDs; the selected native producers then
validate canonical aliases and actual pinned rows, add mandatory en-US fallback
and recompute their physical dependency closure. The manifest never bypasses
those original image, digest or same-owner admission gates.

Input is UTF-8 and bounded to 256 KiB in both SDK and file reading. Unknown or
duplicate fields, unsupported versions, malformed Custom IDs and unsupported
dimensions reject explicitly. Version 6 service selection derives and emits only
the actual required component closure. The complete Locale, normalization, input
segmentation rules/models, calendar kernels and IANA/country authorities retain
their existing contracts.

Version 2 requires a root `currency_codes` list and allows `locale_filters` to be
omitted when currency data is the only projection:

```json
{
  "schema_version": 2,
  "custom_id": "currency-data",
  "currency_codes": ["EUR", "JPY"],
  "locale_filters": { "number_plural": ["fr"], "display_names": ["fr"] }
}
```

Codes use the original CurrencyCode parser, are sorted and canonicalized, and
must be distinct and have actual pinned data. Empty, null, syntactic-only codes
and unknown dimensions reject. Version 1 continues to reject currency fields.
The SDK `for_currency_codes` constructs a currency-only profile;
`with_currency_codes` composes the selection with any admitted locale filters.
The Number and DisplayNames producers remove real localized labels and their
reachable pool entries. Their exact descriptors rederive selected bytes, and
provider admission requires the two currency domains to agree. Omitted codes
retain NumberFormat's code fallback and DisplayNames' code/none fallback.
Global fraction digits, all numbering systems and other data authorities remain
complete. See the [currency projection contract](intl-currency-projection.md).

Version 3 requires root `date_time_calendars` and may also include `currency_codes`
and `locale_filters`. It selects localized DateTime calendar data. Every retained
locale keeps its original default calendar as well as the requested canonical
calendar types; global kernels, LocaleInfo preferences and supported calendar
values remain complete. Empty, null, duplicate and unknown calendar types reject
before user source loading. Versions 1 and 2 retain their original closed keys.
The SDK `for_date_time_calendars` and `with_date_time_calendars` construct this
same private selection. See the
[localized calendar contract](intl-calendar-projection.md).

Version 4 requires root `numbering_systems` and may include the existing
`date_time_calendars`, `currency_codes` and `locale_filters` fields:

```json
{
  "schema_version": 4,
  "custom_id": "numbering-data",
  "numbering_systems": ["deva"],
  "date_time_calendars": ["chinese"],
  "currency_codes": ["EUR"],
  "locale_filters": { "number_plural": ["fr", "ar-EG"], "date_time": ["fr", "ar-EG"] }
}
```

The original `NumberingSystemOption` parser canonicalizes each Unicode nu name;
SDK admission requires actual pinned decimal data. Empty, null, canonical
duplicates, malformed and unavailable names reject before source loading.
`for_numbering_systems` constructs a numbering-only selection and
`with_numbering_systems` composes it with other checked fields. Both real native
factories receive the same typed list. Number's heavy associations and DateTime's
localized symbol maps retain requested systems plus each actual locale default,
including private Number rows used by RelativeTime and Duration. The full digit
kernels and Locale default-numbering authority remain on the same owners.
Supported numbering values derive from those admitted global kernels, rather
than requiring every projected service to format every global option. Export
admission rejects unequal Number and DateTime requested domains. Versions 1–3
keep their original keys and meanings. See the
[numbering projection contract](intl-numbering-projection.md).

Version 5 requires `named_time_zones`, optionally composed with all version-4
data fields and locale filters. `for_named_time_zones` and
`with_named_time_zones` use actual complete IANA lookup to normalize casing and
aliases to primary `TimeZoneId` values. Canonical alias duplicates, empty/null
lists, unknown names and fixed offsets in this named-only list reject before
source loading. Versions 1–4 retain their closed fields.

The actual Named producer retains selected primaries, mandatory UTC and all their
original alias TZif records. The same admitted Named owner supplies selected
DateTime and internal timezone-name data; every dependent descriptor rederives
the original names, metazones and pool references and binds the actual selected
foundation. IANA identities, country membership and global supported primary
names remain complete. Known omitted zones still resolve to their genuine
identity, but accessing their omitted transition data reports an explicit Custom
data-unavailable error. They never become unknown names or silently use UTC.
Fixed-offset arithmetic remains independent. See the
[named-zone contract](intl-named-zone-projection.md).

Version 6 requires a nonempty `services` list and permits the preceding optional
data fields and locale filters. Exact names are `Locale`, `Collator`,
`NumberFormat`, `DateTimeFormat`, `PluralRules`, `RelativeTimeFormat`, `ListFormat`,
`DisplayNames`, `Segmenter` and `DurationFormat`. Unknown/duplicate names, null,
empty lists and dimensions targeting omitted frames reject before source loading.
RelativeTime-only retains Locale, List, Number and RelativeTime frames but grants
no public NumberFormat, PluralRules or ListFormat operations. Actual sparse
providers, dynamic Wasm frames and version-two exports consume the same checked
service closure. Older manifest versions retain complete service permissions.
See the [service contract](intl-service-projection.md).

The CLI admits a manifest before reading the user program and rejects combining
it with `--intl-profile` or any component locale flag. The Custom ID comes from
the manifest. Existing Wasm-only command gates remain. Cache keys consume the
same checked Custom fields as SDK construction and locale flags, including the
currency, calendar, numbering and named-zone presence markers and sorted values
and requested-service bitset in cache v18; neither manifest
filename nor whitespace/order is an additional semantic input. Actual emitted
identity and every required component frame still come from one selected provider.

Authored controls cover all eight physical SDK frames, equality with the direct
constructor, unknown dimensions, null/duplicates/empty filters, version/identity
refusal, bounded input, ambiguous CLI flags before file IO, actual CLI/SDK artifact
equality, script/module execution and rejection before source loading. They remain
unrun. Version-2 controls additionally cover paired physical currency closure,
fallbacks, exported re-admission, cache identity and CLI/SDK equality. This closes
external locale, currency, calendar, numbering and named-zone manifest input,
and actual Custom service/frame selection. Full pinned acceptance remains open. The selected bundle now has a
[canonical export and re-admission route](intl-bundle-export.md); its actual
cross-host reproducibility controls remain unrun.
