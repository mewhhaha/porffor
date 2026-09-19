# Pinned IANA2026a named time zones

The identifier catalogue and transition snapshots use one IANA release,
2026a. `tzdata2026a.tar.gz` is the exact IANA data release; selected files under
`source/` retain their original bytes and public-domain license. The pinned
`jiff-tzdb`0.1.6 crate contains all 598 names and the rearguard slim TZif archive.
Its archive hash is the existing Cargo.lock checksum. No host filesystem,
system timezone setting, network lookup, finite-year offset table or ambient
clock participates in runtime resolution.

`catalogue.tsv` is generated from every Zone and Link in the regional files,
`etcetera`, `backward`, and `factory`. Primary identifiers follow ECMA-402's
country-preserving `zone.tab`/`backzone` rules. UTC aliases retain their own
normalized Identifier and resolve their PrimaryIdentifier to UTC. A row in
`zone.tab` remains primary even when tzdb's default build shares another
country's transition bytes. Byte equality is only storage sharing, never an
alias or naming identity rule. There is no pending rename waiting period in
this release: Kyiv's replacement predates the two-year recommendation.

There is one pinned source/spec edge: Pacific/Johnston is a default Link to
Honolulu (US), while its own country is UM with two `zone.tab` entries, and
`backzone` declares Johnston as a **Zone**, not the Link expected by the
abstract algorithm's last country-preserving branch. The generator retains a
cross-country identifier as its own primary only when an explicit `backzone`
Zone proves that identity. This general rule currently affects that one row;
the exact names/count are recorded in the manifest. It never chooses an
arbitrary zone in a different country, and does not claim different transition
bytes from the pinned jiff archive.

IANA's explicit `#= TARGET1` annotations retain the intended target of a Link
that was flattened only for old parsers. They supply geographic ownership for
legacy aliases absent from `zone.tab`; each annotated chain is checked for
cycles, known targets and agreement with the ordinary terminal Zone. This
corrects `Iceland` to the country of `Atlantic/Reykjavik` without changing its
transition bytes. The manifest records all nine annotated targets and the
single country override in this release. `zone.tab` remains authoritative for
its own names. Countries are never inferred from arbitrary `backzone` targets:
the cross-country `Atlantic/Jan_Mayen` link still belongs to SJ.

The supplemental ICU77.1 `zoneinfo64` input is used **only for its explicit
Names and Regions country arrays**, as fallback after those IANA authorities.
Its `Iceland` row carries the flattened target's CI country, demonstrating why
it cannot override IANA's explicit alias ownership. No identifier, alias
target, UTC offset or transition from that file is admitted. Every overlap
agrees with IANA2026a `zone.tab`, which supplies newer names such as
America/Coyhaique directly.
This is necessary because CLDR short IDs are not reliable country prefixes,
and CLDR alias groups can cross countries (South Pole/Auckland). The ICU
release commit, original URL and Unicode license are retained alongside the
source. Its older timezone version is not the identifier/transition authority.

The provider validates every catalogue identity, terminal primary, exact
name/archive coverage and per-record SHA-256 before parsing immutable TZif
records. It checks selector indices and future rules before publication.
Unknown user names produce UnknownTimeZone; corrupt bundled records and
inconsistent previously-resolved identities produce InvalidTimeZoneData.
Historical second offsets, type-zero initial records and arbitrary-year POSIX
tails remain exact throughout Date/Temporal's accepted range. Negative
subsecond inputs are floored before requesting epoch seconds.

The exact-version `timezone_provider` vendor patch adds the selected DST bit
without changing its resolver arithmetic. It also proves standard-time
stability across the complete inclusive ±184-day LDML47 generic-name window.
Both explicit and POSIX boundaries participate; equal endpoints do not imply
stability. Only a proven Standard result can construct the shared stable
snapshot. Names consume that same immutable offset/variant/proof snapshot.

From the repository root:

```sh
python3 scripts/generate-intl-named-time-zones.py
python3 scripts/generate-intl-named-time-zones.py --check
python3 -m unittest discover -s scripts/tests -p test_generate_intl_named_time_zones.py -v
cargo test -p lila-intl provider::named_time_zones
cargo test -p timezone_provider --features tzif snapshot
```

The generator uses only the Python standard library and checked-in pinned
inputs. Check mode writes nothing. `manifest.json` records archive/source
sizes and hashes, generated output hashes, catalogue counts and the complete
digest recipe. The provider digest covers the IANA/transition/upstream crate
archives, country metadata, catalogue, generator, selector validation source
and all vendored selector files. The parent provider combines it with the
locale and zone-name component identities before artifact admission.

Sources: [IANA2026a](https://data.iana.org/time-zones/releases/tzdata2026a.tar.gz),
[ECMA-402 time-zone rules](https://tc39.es/ecma402/#sec-use-of-iana-time-zone-database),
[jiff-tzdb0.1.6](https://docs.rs/jiff-tzdb/0.1.6/jiff_tzdb/),
[ICU77.1 country metadata](https://github.com/unicode-org/icu/blob/457157a92aa053e632cc7fcfd0e12f8a943b2d11/icu4c/source/data/misc/zoneinfo64.txt),
and [LDML47 type fallback](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-dates.md#type-fallback).
