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

## Temporal time zones

Temporal resolves every zone through the same catalogue and TZif snapshots.
`lila-intl::TemporalTimeZone` is closed over `Offset(TemporalOffsetMinutes)`
and `Named(TimeZoneId)`; there is no third variant and no path that reads a
named zone as UTC. ABI 7 adds one operation, `QueryTemporalTimeZone` (code 16),
whose request carries the stored identifier and one closed
`TemporalTimeZoneQuery`:

- `OffsetAt` is `GetOffsetNanosecondsFor`.
- `EpochFor` is `GetPossibleEpochNanoseconds` followed by
  `DisambiguatePossibleEpochNanoseconds`, including the gap rule that moves
  `compatible`/`later` forward and `earlier` back by the transition's offset
  change.
- `EpochForOffset` is `InterpretISODateTimeOffset` for `offset: "prefer"` and
  `"reject"`, with exact or minute-rounded matching.
- `StartOfDay` is `GetStartOfDay`, including days whose midnight is skipped.
- `Transition` is `GetNamedTimeZoneNextTransition` or
  `GetNamedTimeZonePreviousTransition` within the representable instants.

All pinned transitions and offsets are whole seconds, so queries and answers
exchange seconds and the compiled caller keeps the sub-second remainder. The
provider's `ZoneRules` reads named zones through the same `Tzif::get` selector
as `Intl.DateTimeFormat`, and it defines a transition as an instant where that
selector's offset changes. A transition search therefore cannot disagree with
an offset lookup. Range failures
are returned as a closed `TemporalTimeZoneRangeError`; the compiled code turns
them into RangeErrors in the current realm. A malformed request is an ABI fault.

On the compiled side, `ToTemporalTimeZoneIdentifier` resolves names with the
same case-insensitive `LookupNamedTimeZone` as `Intl.DateTimeFormat`. A
ZonedDateTime stores the normalized identifier, not the primary one, and
`TimeZoneEquals` compares primary identifiers. The zoned operations all issue
these queries, including civil accessors, `offset`, `hoursInDay`,
`startOfDay`, `round`, `with`, `withPlainTime`, `withTimeZone`,
`getTimeZoneTransition`, `add`/`subtract`, `until`/`since`, `toString`/`toJSON`,
conversions from PlainDate and PlainDateTime, `Instant` strings with a
`timeZone`, `Temporal.Now`, and Duration `relativeTo`. ZonedDateTime `until`/`since`
and Duration `round`/`total`/`compare` with a zoned `relativeTo` share one
implementation of `DifferenceZonedDateTime`, the nudge and bubble steps of
`RoundRelativeDuration` and `TotalRelativeDuration`, and `AddZonedDateTime`.
Their day lengths come from the zone. A date-only zoned string starts at
`GetStartOfDay`. `ZonedDateTime.prototype.toLocaleString` builds a formatter
with the receiver's zone as `toLocaleStringTimeZone` and formats an Instant
with `~zoned-date-time~` defaults.

One behaviour follows the reference implementation and Test262 rather than
the printed text. When a backward transition crosses midnight, the pieces of
two dates interleave. `ZonedDateTime.prototype.round` to `day` treats an
instant at or after the next start of day as that day's last nanosecond. The
printed algorithm asserts that this case cannot occur. Test262 covers it in
`round/same-date-starts-twice.js`.

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
Chinese calendar fields and broader Intl services remain outside this
DateTimeFormat batch. Named-zone Temporal operations are covered by the section
above.
