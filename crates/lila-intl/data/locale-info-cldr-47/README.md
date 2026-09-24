# Pinned CLDR47 Locale information

These inputs back ECMA-402's Locale information operations (15.5.6-15.5.17:
`getCalendars`, `getCollations`, `getHourCycles`, `getNumberingSystems`,
`getTimeZones`, `getTextInfo` and `getWeekInfo`) and the collation list of
`Intl.supportedValuesOf` (8.3.2).

`common/collation/*.xml` (all 133 files) and
`common/properties/scriptMetadata.txt` are the unmodified upstream bytes of
Unicode CLDR47, release commit `2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`, the
same release as every other lila-intl provider. `manifest.json` records each
file's upstream URL, byte length, SHA-256 and git blob SHA-1; the Unicode
license is retained in `LICENSE`. Already-pinned inputs are referenced, not
copied, under `shared_inputs` with their own SHA-256:
`supplementalData.xml` (from `datetime-cldr-47`), the BCP47 `ca` and `co`
registries (from `cldr-47-bcp47`), IANA2026a `zone.tab` and the IANA primary
identifier catalogue (from `iana-tzdb-2026a`).

From the repository root:

```sh
python3 scripts/generate-intl-locale-info.py
python3 scripts/generate-intl-locale-info.py --check
python3 -m unittest discover -s scripts/tests -p test_generate_intl_locale_info.py -v
cargo test -p lila-intl --lib locale_info
```

The standard-library generator verifies every hash before parsing and writes
`src/provider/locale_info/generated.rs` and `generated-report.json`. Check mode
writes nothing. The provider digest is SHA-256 of
`lila-intl-locale-info-v1\0`, the manifest's SHA-256 and the SHA-256 of the
canonical JSON of every generated row; the outer Intl provider digest includes
it, so changed data rejects previously compiled artifacts.

Data rules:

- **Calendar preferences** come from `calendarPreferenceData`, canonicalized
  through the BCP47 `ca` aliases (`gregorian` → `gregory`). CLDR47 keys these
  by region only; a language-region row fails generation instead of being
  ignored. At runtime CalendarsOfLocale keeps only AvailableCalendars, the
  calendars `Intl.DateTimeFormat` formats, and falls back to `gregory`.
- **Hour cycles** come from `timeData`, keyed by region or by
  `language-region` (`fr_CA` → `fr-CA`). The preferred symbol comes first,
  followed by the allowed symbols, mapped by UTS35 Part 4: `h`/`hb`/`hB` →
  `h12`, `H`/`Hb`/`HB` → `h23`, `K` → `h11`, `k` → `h24`, without duplicates.
- **Week data**: a region has week data when any non-`alt` `firstDay`,
  `minDays`, `weekendStart` or `weekendEnd` element names it. Each field falls
  back to the `001` value. The weekend is the inclusive start-to-end range,
  sorted by ISO weekday number.
- **Text direction** uses the `RTL` column of `scriptMetadata.txt`; `YES` is
  `rtl`, `NO` is `ltr`, and `UNKNOWN` scripts (such as `Zyyy`) yield
  `undefined`.
- **Collations**: `%Intl.Collator%.[[AvailableLocales]]` is the set of CLDR
  collation locales except root. `en_US_POSIX` is excluded because UTS35
  converts its legacy `POSIX` variant into `-u-va-posix`, and an Available
  Locales element cannot carry a Unicode extension. Every remaining element's
  less narrow prefixes are present and `en-US` (the default locale) is
  included, as ECMA-402 9.1 requires. A locale's `[[co]]` list is the union of
  the collation types along its collation parent chain (the `collations`
  component `parentLocales`, which UTS35 treats as stand-alone, otherwise
  truncation, ending at root), mapped through the BCP47 `co` aliases
  (`phonebook` → `phonebk`, `traditional` → `trad`). `standard` and `search`
  (ECMA-402 10.2.3) and CLDR-internal `private-*` types are removed. Every draft
  level is retained, as in the NumberFormat profile. `digits-after` (cs) is not
  a registered BCP47 collation type and is excluded; the report lists it. The
  generator checks that root's list equals ECMA-402's `« "emoji", "eor" »`
  fallback.
- **Time zones**: TimeZonesOfLocale returns the IANA `zone.tab` identifiers
  for the region, each checked to be an ECMA-402 primary identifier in the
  pinned catalogue, sorted in code unit order.

Numbering systems, currencies, units, calendars and primary time zones for
`Intl.supportedValuesOf` come from the existing NumberFormat, DateTimeFormat
and IANA providers at runtime, so each list has exactly one authority.

Sources: [CLDR47 collation](https://github.com/unicode-org/cldr/tree/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/common/collation),
[scriptMetadata.txt](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/common/properties/scriptMetadata.txt),
[UTS35 Part 4 Time Data and Week Data](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-dates.md),
and [ECMA-402 Locale objects](https://tc39.es/ecma402/#locale-objects).
