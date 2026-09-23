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
catalogue and `Tzif::from_bytes` on hash-checked jiff-tzdb0.1.6 bytes. The patch
does not change transition selection arithmetic or the existing Temporal API.
