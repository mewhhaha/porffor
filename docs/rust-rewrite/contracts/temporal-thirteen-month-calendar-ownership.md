# Thirteen-month calendar ownership

The closed Temporal calendar domain admits `coptic`, `ethiopic` and `ethioaa`,
including canonicalization of `ethiopic-amete-alem` to `ethioaa`. They use the
existing ISO-backed carriers and completed date/partial-reference owners.
Annotated strings retain ISO coordinates; property bags resolve calendar
coordinates before conversion. This is Temporal calendar arithmetic support;
it does not add Intl calendar formatting data or complete T22.

`TemporalThirteenMonthCalendar` selects the epoch and reference years together.
The private `temporal_calendar_arithmetic/thirteen_month.rs` leaf follows the
pinned [ICU4X Coptic conversion](https://github.com/unicode-org/icu4x/blob/c9fac4e625ccb2c6a7aa35079fff9709db4385ac/utils/calendrical_calculations/src/coptic.rs)
and [Ethiopian epoch shift](https://github.com/unicode-org/icu4x/blob/c9fac4e625ccb2c6a7aa35079fff9709db4385ac/utils/calendrical_calculations/src/ethiopian.rs).
Unix epochs are -615558, -716367 and -2725242 respectively. Signed division
is Euclidean; months 1–12 have 30 days and month 13 has 5 days, or 6 when
`year mod 4` is 3. Forward conversion is
`epoch + 365*(year-1) + floor(year/4) + 30*(month-1) + day-1`.
Ethioaa uses its own arithmetic year and epoch, with the same leap phase.

Four parent-consumed leaf methods compute month length, fill the existing
`TemporalCalendarDateLocals`, convert a regulated triple and choose a MonthDay
reference. They allocate no Temporal object and release temporary locals in
reverse reservation order. The parent bounds calendar years to
`[-280000, 285000]` before multiplication; completed carriers retain their
distinct converted ISO limits. Full-date boundary days and YearMonth reference
months remain separate from supplied MonthDay native-year admission.

The [March 2026 era/month-code proposal](https://tc39.es/proposal-intl-era-monthcode/)
supplies the current calendar, era and month-code contract. Coptic `am` and
ethioaa `aa` accept signed era years directly. Ethiopian `am` resolves directly;
`aa` resolves to era year minus 5500. Reporting selects `am` for a positive
Ethiopian arithmetic year and `aa` otherwise, adding 5500. Reporting intervals
do not reject signed input era years. Obsolete `incar`/`mundi` aliases and
mixed-case era spellings remain rejected. These input rules deliberately follow
the current proposal and pinned Test262 era fixtures rather than older provider
era-input restrictions.

`TemporalCalendarArithmetic::month_arithmetic()` selects the closed Thirteen
policy for these domains. Native-year counts, shared arithmetic and complete
projections consume that authority; year/month rounding uses dated anchors.
All four field decoders consume the shared resolved-month constructor: `M13`
is suitable only in these three domains, and leap codes such as `M13L` remain
unsuitable. Generic
MonthCode syntax validation still occurs at the original field read, before
later field effects; suitability and field agreement retain their later order.

YearMonth uses calendar day 1. MonthDay selects the latest valid match on or
before ISO 1972-12-31: reference years 1689/1965/7465, the preceding year for
later candidates, and two years earlier for `M13` day 6. A missing year uses
that leap reference for regulation. Before month information, the shared month
factory requires a supplied native year to intersect the full ISO carrier
range, using the actual start and next-year start. The requested month/day may
be outside that range or in an adjacent ISO year; its overflow regulation and
reference choice follow year admission. This follows the
[whole-year admission rule](https://tc39.es/proposal-intl-era-monthcode/#sec-nonisomonthdaytoisoreferencedate).
Existing completed partial owners remain mandatory at publication.

`aot_temporal_thirteen_month_calendars` contains six finite cohorts covering
projection, signed eras and field order, 13-month arithmetic and rounded carry,
partial references, relative durations, limits and borrowed intrinsic Realms.
Literal pinned dates include the leap epagomenal reference 1971-09-11 and the
late month-4 reference 1972-01-02. The Engine target requires Wasm AOT in both
strictness modes, exact Normal completion 262 and one exact print event.
The complete production and control targets passed the ref82 combined
all-target Rust type check on 2026-10-04. Emitted Wasm, focused runtime regressions
and broad verification remain pending.
No published conformance count changes, GC migration or Apia issue #3310 claim
follows from this batch.

The shared whole-native-year admission correction passed the ref87 combined
whole-workspace/all-target Rust type check on 2026-10-04. Emitted-Wasm and
runtime verification remain pending; the earlier calendar checkpoint retains
its original source scope.
