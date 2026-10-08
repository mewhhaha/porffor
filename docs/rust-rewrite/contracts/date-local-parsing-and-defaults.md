# Date parsing and immutable UTF-16 input

Status: source-authored for atomic T05 on 2026-10-04. No new parsing, compilation
or runtime check has run; the whole batch must precede expensive verification.

The actual ISO and owned-display parsers borrow a nonnull immutable StringValue.
Their cursor reads its immutable CodeUnitArray, checks the index before every
read, and advances over exact UTF-16 units. ASCII grammar matching does not pack
strings as byte addresses. Pure calendar fields and cursor positions use typed
locals. Parser result publication applies the shared TimeClip producer.

Date-only ISO forms use UTC. Date-time forms without a zone use the immutable
system zone and Date-owned compatible inverse selection. Explicit offsets remain
independent of that zone. The complete owned local-display grammar preserves
historical whole-second offsets and rejects malformed suffixes; unrelated
implementation-defined Date.parse forms are not claimed as new support.

The raw-layout spelling guard is retired. Existing `aot_date_parsing` and
`aot_date_system_time_zone` controls cover grammar, calendar validation, negative
and extended years, default-zone selection and display round trips. Their
execution and pinned conformance remain pending.
