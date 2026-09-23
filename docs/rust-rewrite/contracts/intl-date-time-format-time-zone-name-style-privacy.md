# Intl.DateTimeFormat time-zone-name style authority

`lila-intl::TimeZoneNameStyle` owns the six accepted `timeZoneName` styles,
their option spellings and their wire and heap codes. Its derived `OPTIONS`
table supplies the AOT constructor and `resolvedOptions`; the provider matches
the same enum exhaustively when selecting localized names. Invalid external
codes are rejected at the protocol boundary.

The backend has no second style enum or hand-maintained option table. This
shared domain replaces the former backend-private authority because emitted
formatters and the host provider must agree on each selection.

The `intl_dtf_time_zone_name_style_privacy_structure` target and
`scripts/check-module-boundaries.sh` enforce that ownership. Native style,
transition and range coverage is in `aot_intl_named_time_zones`. See the
[named-zone implementation](../intl-named-time-zones.md) for the pinned data
and supported locale boundary.
