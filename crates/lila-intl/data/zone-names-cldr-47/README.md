# Pinned time-zone name data

The name provider uses CLDR 47.0.0 at commit
`2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`. Every upstream file URL, byte
length and SHA-256 is recorded in `manifest.json`; the Unicode license is
retained in `LICENSE`. The supported DateTimeFormat locale inventory is `en`
and `en-US`, with field inheritance through `root` → `en` → `en_US`. The host
accepts these bases and the formatter's resolved `ca`, `hc` and `nu` Unicode
keys. Other name locales return `UnsupportedNameLocale`.

`icu-77-1-zoneinfo64.icu` is the unmodified ICU77.1 resource from commit
`457157a92aa053e632cc7fcfd0e12f8a943b2d11`. Only its parallel `Names` and
`Regions:array` fields supply country metadata. Its transition records are
never read. Its source provenance and license are retained beside it. Accepted
identifiers, country-preserving primary identity, offsets, DST variants and
transition-window proofs belong to the independent IANA2026a provider.

The generated tables contain 600 CLDR aliases, 446 canonical zones, 190
referenced metazones, and 669 half-open UTC metazone periods. An absent start
or end is unbounded. Gaps stay gaps. Metadata selection uses the input's exact
UTC second and cannot confuse repeated wall times or clamp to ICU convenience
timestamp bounds. An IANA identity newer than the CLDR catalogue receives
the appropriate localized offset fallback.

The six styles follow the pinned [LDML47 time-zone formatting rules](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-dates.md#Using_Time_Zone_Names).
Direct names precede metazones. Effective zone and metazone daylight forms,
across both widths, govern type fallback. A missing generic name may use a
standard name with the transition provider's complete standard-time stability
proof for the closed ±184-day interval. Without a name, generic styles use
country/city location then localized offset; specific styles use localized
offset. Country short names are used for primary/single-zone location names;
metazone qualifiers use the ordinary country name.

LDML47 §4.3 qualifies a nonpreferred metazone with its country or city for
generic **and specific** styles. This is a deliberate pinned policy, unlike
ICU77's specific-name and same-wall-time generic optimizations. For example,
English names include `Pacific Standard Time (Canada)` for Vancouver and
`Mountain Time (Phoenix)` for Phoenix. Seasonal London's missing generic name
uses `UK Time`, from CLDR's short country name. Tests use these pinned facts;
the host's installed ICU is not an oracle.

Names and offsets use the same immutable IANA transition snapshot. Names never
perform a second offset lookup. Numeric names retain historical seconds and
the complete ±23:59 fixed-offset domain. Returned digits are Latin skeletons;
the AOT formatter substitutes its selected numbering system once. Named UTC
keeps its direct UTC names, while fixed zero uses GMT offset names.

Regenerate and verify without network access:

```sh
python3 scripts/generate-intl-time-zone-names.py --report crates/lila-intl/data/zone-names-cldr-47/generated-report.json
python3 scripts/generate-intl-time-zone-names.py --check --report crates/lila-intl/data/zone-names-cldr-47/generated-report.json
python3 -m unittest discover -s scripts/tests -p test_generate_intl_time_zone_names.py
cargo test --locked -p lila-intl --lib time_zone_names
```

The generator rejects mismatched source hashes, unsupported pattern changes,
overlapping periods, missing golden/reverse mappings, unresolved locale aliases
and inconsistent country arrays. `generated-report.json` records normalized
row and output hashes and sizes. The provider digest covers the source manifest,
normalized rows, the generator and `selector.json`; the outer Intl provider
includes this digest together with its locale and IANA data identities.
