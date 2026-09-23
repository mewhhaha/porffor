# Intl.DateTimeFormat time-zone authority

`lila-intl::FixedTimeZoneOffset` validates minute-aligned offsets strictly
below 24 hours. `NamedTimeZoneIdentity` pairs a normalized identifier with its
primary identifier, using the complete pinned IANA catalogue. The provider
selects an offset and localized name for each exact endpoint; a named zone's
offset is never cached as a constant constructor result.

The backend-private `DtfCanonicalTimeZone` and `DtfResolvedTimeZone` retain a
move-only lifecycle. Reserving owns three unwritten locals; only the option
reader consumes that state and returns a resolved state. Only the resolved
state can publish the identifier, fixed-offset seconds and zone-kind slots
and release those locals. Fixed seconds and kind are scalar heap fields.

The former backend-local offset type and constant-offset named-zone table
are removed. The `intl_dtf_time_zone_authority_privacy_structure` target and
`scripts/check-module-boundaries.sh` enforce the shared validation authority
and private constructor lifecycle. Provider and native transition tests cover
the endpoint behavior; see the
[named-zone implementation](../intl-named-time-zones.md).
