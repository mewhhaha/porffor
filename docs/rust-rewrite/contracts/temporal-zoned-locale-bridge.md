# Prepared ZonedDateTime locale bridge

Status: staged, uncompiled and unexecuted, 2026-09-30. This belongs to the complete
T22 named-zone consumer batch. Production continues to reject named construction
until that whole graph, its provider certificate and verification are integrated.
No generated README status or conformance count changes are established here.

The ZonedDateTime formatting leaf is owned by the generator lane. It checks the
receiver brand, recovers exact Instant/zone/actual-calendar proof handles, loads
the original locales and options values, and calls the private Intl bridge.
Public receiver properties and PlainDateTime conversion do not provide its data.

The bridge uses the existing DateTimeFormat initializer and parts renderer. Its
closed creation input permits a forced zone only for this Instant locale recipe;
ordinary Intl/Date/other Temporal callers retain their existing argument path.
The borrowed proof handles come from `temporal_zone_provider`:

```rust
emit_intl_dtf_format_temporal_zoned(
    &NormalizedTemporalInstantLocals,
    &ResolvedTemporalZoneLocals,
    &TemporalCalendarSlotLocals,
    locales_payload, locales_tag, options_payload, options_tag, function,
) -> Result<(), EmitError>
```

Locale canonicalization, options boxing, all GetOption conversions, abrupt
completions and ordering remain compiled Wasm semantics. At the normal timeZone
observation point the initializer Gets once; any non-undefined user value throws
TypeError before ToString, even if it equals the receiver's zone. The stored
PrimaryIdentifier is already validated. Temporal UTC/named kinds become named
Intl data; only the fixed-offset branch reads the validated numeric seconds.
No user coercion or host system-zone default supplies this record zone.

After all format getters and ordinary style/component conflicts, the no-style
branch implements the zoned-date-time Instant default: when every required date
or time component is absent, an absent timeZoneName becomes short. Explicit era
and timeZoneName do not suppress date/time defaults; fractionalSecondDigits does.
Styles and any explicit required component preserve their existing selection.
The fresh formatter exposes no other recipe to user code, so this compiled
adjustment can use the existing Any/All provider request domain. Locale/pattern
selection and rendering remain data services, not JS coercion or policy hosts.

After formatter initialization, an ISO receiver accepts the resolved locale
calendar. Other receiver calendars must equal it or throw RangeError in the
builtin's defining Realm. The primitive rendering input uses Instant kind,
Euclidean floor seconds and a positive remainder floored to milliseconds. Thus
-1ns formats at -1ms rather than zero; no Number or TimeClip truncation intervenes.
Intl supports at most three fractional digits, and pinned zone transitions have
integral-second coordinates. The original exact receiver epoch is unchanged.
The renderer resolves the receiver zone at that formatted epoch, including DST.
Ordinary DateTimeFormat.format/parts/range calls still reject ZonedDateTime.

This follows the current Temporal ECMA-402 integration algorithms:
[CreateDateTimeFormat](https://tc39.es/proposal-temporal/#sec-createdatetimeformat),
[GetDateTimeFormat](https://tc39.es/proposal-temporal/#sec-getdatetimeformat), and
[ZonedDateTime.toLocaleString](https://tc39.es/proposal-temporal/#sec-temporal.zoneddatetime.prototype.tolocalestring).
The downloaded primary spec is retained in the conversion lane's `spec/intl.html`.
Some older rendered references use different timeZone error wording; the current
algorithm and pinned cases require TypeError for any supplied option value.

The staged engine target authors four meaningful fixtures for fresh sloppy and
strict observations (eight planned executions): negative fractional epochs and
DST with public receiver getters suppressed; full option order, forbidden-value
coercion suppression and a foreign borrowed builtin; actual-calendar compatibility
with later getter precedence; and defaults, styles, numeric zones and aliases.
Node checked syntax in both modes only. No fixture runtime or Cargo ran.
The exact pinned inventory contains 27 physical files and 54 planned modes,
including all 23 ZonedDateTime locale cases and four ordinary Intl rejection
controls, with source hashes and actual metadata against content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. These inventories are not pass results.

The generator's leaf attachment, provider proof factories, creation-time day-start
validation, remaining consumer graph, allocation census, capability closure and
shared guards must integrate together. Root owns the single compile, focused
native/pinned controls and final broad verification checkpoint.
