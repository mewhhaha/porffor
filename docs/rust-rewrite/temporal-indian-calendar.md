# Temporal Indian calendar batch

The AOT compiler adds `indian` alongside `iso8601`, `gregory`, `buddhist`, `roc`
and `japanese`. Indian years begin in March; its month lengths differ from ISO.
Calendar fields and arithmetic therefore consume a complete calendar-date
projection rather than deriving the calendar year from the ISO year alone.
Constructors and annotated strings continue to receive ISO coordinates.
Property bags receive coordinates in their selected calendar.

The private calendar arithmetic owner emits integer conversion in both
directions. Indian has twelve months and one signed `shaka` era, including year
zero. The Gregorian leap rule for calendar year plus 78 determines whether M01
has 30 or 31 days; M02 through M06 have 31, and M07 through M12 have 30. The
conversion uses an 80-day ordinal offset. These rules follow the pinned
[ICU Indian primitive](../../vendor/icu_calendar-2.0.6/src/cal/indian.rs) and cover
the existing ISO carrier range without a new host service or runtime ABI.

An unresolved calendar year cannot stand in for an ISO year. Field resolution
checks era/year agreement in calendar coordinates, regulates the complete
calendar month/day, then converts to ISO. Receiver defaults come from the same
complete projection. Existing field reads, month/monthCode and year/era
exclusion groups, overflow option ordering and abrupt propagation remain owned
by the existing field resolvers.

Date, DateTime, YearMonth, MonthDay and zoned accessors use that projection.
Epoch weekdays retain their existing arithmetic; non-ISO week fields remain
undefined. Calendar-sensitive add, difference and relative-duration helpers
require the retained canonical calendar slot at their actual function
boundaries. The fixed-twelve-month difference policy compares virtual,
unclamped calendar anchors before constraining the corresponding ISO date and
computing the remaining epoch days. Existing day-only, clock and timezone
operations keep their existing owners.

Partial dates use a completed reference-date owner. YearMonth converts calendar
day 1 and applies its own ISO year/month limits. MonthDay first regulates any
supplied calendar year, then chooses the latest valid matching month/day on or
before ISO 1972-12-31. Its Indian reference year varies by month and day; an ISO
1972 literal is insufficient. Strings, `with`, full-date conversions and
partial allocation consume the same reference factories.

The primary operations are
[CalendarYearMonthFromFields](https://tc39.es/proposal-temporal/#sec-temporal-calendaryearmonthfromfields),
[CalendarMonthDayToISOReferenceDate](https://tc39.es/proposal-temporal/#sec-temporal-calendarmonthdaytoisoreferencedate),
and [CalendarDateUntil](https://tc39.es/proposal-temporal/#sec-temporal-calendardateuntil).
Non-ISO difference and reference selection have implementation-defined parts;
the bounded policy above is explicit. It does not use the vendored generic
calendar factories or raw ICU field subtraction as a universal oracle.

The new `aot_temporal_indian_calendar` Engine target contains six finite
cohorts, each in sloppy and strict modes. They cover projection, fields,
arithmetic, partial references, retained relative durations, range boundaries
and called intrinsic Realms. They also retain Gregorian-family arithmetic
controls. Each requires actual Wasm-AOT execution, a normal Number 262 result
and its sole expected final output.

Refresh the focused evidence after the complete batch compiles:

```sh
cargo test -p lila-engine --test aot_temporal_indian_calendar -- --test-threads=2
cargo test -p lila-engine --test aot_temporal_buddhist_calendar -- --test-threads=2
```

Source authoring and review are not execution evidence. Compilation, these
controls and the full T22 checkpoint remain pending for this batch. The
existing Apia contextual-rounding-window gap retains its independent ownership.
Additional calendars and full Date/Temporal conformance remain open.
