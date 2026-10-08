# Lila exact transition snapshots

Upstream is the unmodified crates.io `timezone_provider` 0.1.2 archive,
SHA-256 `df9ba0000e9e73862f3e7ca1ff159e2ddf915c9d8bb11e38a7874760f445d993`.
The MIT and Apache-2.0 licenses and upstream source remain present.

This patch adds the selected `is_dst` flag to `TimeZoneTransitionInfo` at its
five construction paths: no-transition type zero, pre-first-transition type
zero, historical record, fixed POSIX footer, and seasonal POSIX footer. The
flag is copied from the selected record/rule; it is never inferred from offset
magnitude, hemisphere, or the current season. Jiff's pinned rearguard data
defines the variant used for localized time-zone names.

The companion `Tzif::offset_and_dst_are_constant` examines a closed, bounded
interval for localized generic-name fallback. It inspects every explicit
transition, the explicit-record/POSIX handoff, and POSIX rule transitions in
all intersecting years including neighboring-year spills. It reuses the
existing POSIX arithmetic and compares both offset and DST flag, so equal
endpoints and same-offset DST changes cannot establish false stability.
Its accepted domain includes Date/Temporal instants with two years of context.
Tests exercise each flag branch, equal seasonal endpoints, the upper boundary,
same-offset designation changes, and constant tails.

The existing compiled 2025b normalizer and filesystem provider are unchanged.
Lila's Intl path does not call them: it uses its generated IANA2026a identifier
catalogue and `Tzif::from_bytes` on hash-checked jiff-tzdb0.1.6 bytes. The public Temporal API remains unchanged.

A second exact-selector patch resolves POSIX rule boundaries in adjacent
nominal years before selecting the latest UTC transition. It recognizes
RFC9636 3.3.1 all-year daylight rules without consulting zone names, so the
unused standard type in the pinned Casablanca/El_Aaiun footer cannot become
an artificial annual offset. The selector uses the existing localized-name
context bound, checks epoch/calendar arithmetic, and carries the correct
pre-transition offset when calculating prior-year transition metadata.
Both private nanosecond conversions use checked Euclidean floor division,
including values strictly between -1second and 0.

Tests cover actual pinned all-year daylight records and stability, signed
start rules and end rules beyond 24hours crossing UTC calendar boundaries,
prior-year transition metadata, exact contextual limits, and negative
fractional instant/local inputs at a synthetic transition. The synthetic
same-year equal UTC start/end rule is ordered start then end, representing
an empty daylight interval. POSIX prose and glibc agree with this
interpretation, while current IANA tzcode treats that synthetic case as
perpetual DST; no actual pinned footer uses that unclassified same-year tie.


The staged Temporal data lane additionally exposes offset_change_boundaries and
posix_offset_change_cycle, both attached to Lila's typed native named-zone
operations. Boundary checks use the same get/POSIX snapshot selector; flag-only
changes are excluded. A complete Gregorian400-year cycle proves sparse leap
rules and empty perpetual tails. No approximate local-time inverse is exposed
through Lila: its exact inverse enumerates the validated offset catalogue and
round-trips every candidate through get. These changes require regenerating
Lila's provider identity before production integration.
