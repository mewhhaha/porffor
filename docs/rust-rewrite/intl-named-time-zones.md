# Named zones in the Wasm DateTimeFormat path

`Intl.DateTimeFormat` and the three Date locale methods resolve the complete
pinned IANA 2026a catalogue through the pure typed Intl provider. JavaScript
option reads, coercion order, realm errors, Temporal field masks, date arithmetic,
parts and range formatting remain compiled Wasm. The host receives only validated
identifiers, exact epoch seconds, a closed name style and a resolved locale. It
has no parser, JavaScript objects, source recognition, ambient zone database or
JavaScript interpreter.

## Identities, data and capability claims

`LookupNamedTimeZone` returns the normalized Identifier and PrimaryIdentifier as
separate fields. The immutable catalogue owns case folding, complete Zone/Link
membership and country/backzone primary resolution; transition bytes come from
jiff-tzdb 0.1.6, also IANA 2026a. It verifies every catalogue spelling and TZif hash
before publishing the provider. Names use pinned CLDR47 data, exact UTC metazone
periods and narrow pinned ICU77.1 country metadata. Their generators document
source hashes, licenses, output extents and the source-derived geographic cases.

The DateTimeFormat record preserves Identifier, as required by the repository's
pinned `canonical-tz` Test262 contract (`timezone-not-canonicalized.js`). This is
an explicit version boundary: live ECMA-402 2027 CreateDateTimeFormat stores
PrimaryIdentifier. The typed lookup carries both identities so upgrading that
suite/spec contract requires changing the projection, not reconstructing lost
identity or changing transition data.

`EmbeddedIntlProvider` replaces the locale-only name. Its composite digest is
computed from locale, transition/catalogue and name generation digests. ABI 3
replaces the old unused canonical-zone operation with named lookup and adds the
snapshot operation. The existing artifact identity gate rejects stale host/data
identities before Wasmtime instantiation, including cached artifacts.

The profile adds data requirements through sealed operation types. It advertises
locale transforms, transitions and zone names without advertising complete
DateTimeFormat pattern/calendar data. Name requests accept only the formatter's
actual resolved `en` / `en-US` domain and its `ca`/`hc`/`nu` extensions. A direct
provider request for another name locale fails explicitly. DateTimeFormat's
existing locale negotiation can still resolve an unsupported request to en-US;
that resolved locale is observable in `resolvedOptions()`.

## Snapshot and lifetime boundary

A formatter stores its identifier, a closed Named/FixedOffset kind, and signed
seconds for fixed offsets. Named records carry no cached offset. The kind and
fixed-second slots are scalar GC metadata; the previous cached GMT-name payload
is removed. The six timeZoneName option codes and spellings are derived from
`lila-intl::TimeZoneNameStyle`, shared with the provider.

Every exact input obtains offset and optional name from one immutable transition
snapshot. Date milliseconds and Temporal.Instant nanoseconds are reduced with
floor at second boundaries, including negative subseconds. Selection covers the
full Date/Instant domain, initial records, historical second offsets and POSIX
future rules. The narrow timezone_provider 0.1.2 patch exposes the selected TZif
`is_dst`; names never infer season or daylight identity from offset magnitude.

For generic standard-name fallback, the provider examines every explicit and
POSIX transition in the inclusive ±184-day window. Only an unchanged Standard
snapshot can carry `StandardTimeStability::Stable`; two equal endpoint offsets
alone cannot prove this. Daylight snapshots carry no standard-time proof.
The pinned LDML47 selector applies direct-name, metazone, type and location/offset
fallback rules. Its metazone qualification follows the published CLDR47 rules
for generic and specific names, rather than reproducing ICU77's separate
same-wall-time optimization. Missing metadata for a newer IANA name uses the
specified localized offset fallback. These display choices are implementation
dependent; offsets and transition boundaries are not approximated.

The shared required-capacity host protocol copies request bytes before any
response write. Query and retry are pure and cannot repeat JS coercions. Unknown
constructor names reject with the current constructor realm's RangeError;
malformed snapshot requests and inconsistent responses are ABI/provider faults.
The AOT decoder validates result sizes and the requested-name shape before
publishing packed strings.

Each range endpoint retains its own name with its own broken-down fields. One
component projection copies the selected endpoint, and one reverse release owns
all ten locals. Practical equality follows the ordered range-field table, which
excludes timeZoneName: equal repeated wall times can collapse to the start's
output even when the daylight name differs. Plain Temporal values bypass
transition lookup and preserve their wall-clock fields across gaps and overlaps.
The existing digit renderer localizes the provider's Latin decimal skeleton once.

## Coverage and remaining domains

The native integration target covers identifier aliases and casing, one-time
coercion and realm errors, millisecond/nanosecond transition neighbors, historical
seconds, half-hour DST, dateline jumps, future rules and Date/Instant limits,
separate range snapshots, repeated-time collapse, Plain Temporal values, all six
fixed-offset styles, digit localization and Date-only host import roots. Direct
host tests cover capacity queries, overlapping spans, rejection and malformed
memory requests. Provider tests cover catalogue/data corruption, exact-window
stability and pinned name selection. Source structure tests retain constructor,
range-local and GC ownership checks across the migration.

Verification commands for the integrated batch:

```sh
python3 scripts/generate-intl-named-time-zones.py --check
python3 scripts/generate-intl-time-zone-names.py --check
cargo check --workspace --all-targets
cargo test -p lila-intl -- --test-threads=1
cargo test -p lila-engine intl_time_zone_host::tests -- --test-threads=1
cargo test -p lila-engine --test aot_intl_named_time_zones -- --test-threads=1
cargo test -p lila-engine --test aot_date_locale -- --test-threads=1
cargo test -p lila-aot-wasm --test intl_dtf_time_zone_authority_privacy_structure --test intl_dtf_time_zone_name_style_privacy_structure --test intl_date_time_format_heap_slot_structure -- --test-threads=1
```

The implementation stage was source-reviewed and formatted; product compilation,
native controls and the exact Intl Test262 replay remain the integration owner's
verification step. This does not claim new suite totals. Arabic date patterns,
Chinese calendar fields, broader Intl services and named-zone Temporal.ZonedDateTime
operations remain outside this DateTimeFormat batch.
