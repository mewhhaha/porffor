# Temporal partial-date reference formatting

The 2026-10-07 source audit found that PlainMonthDay and PlainYearMonth used
the calendar-annotation predicate to select their reference ISO year/day.
Consequently, `calendarName: "never"` removed both the annotation and the
reference field for non-ISO calendars. The pinned Test262 `calendarname-never`
cases for both types expose this mismatch.

`TemporalMonthDayToString` and `TemporalYearMonthToString` require the reference
field whenever the retained calendar is non-ISO, and for ISO when the requested
mode is `always` or `critical`. `FormatCalendarAnnotation` separately suppresses
the annotation for `never`. The shared partial-reference emitter now applies
the former rule to both types; the annotation predicate is private to its sole
annotation consumer. The distinction follows the integration algorithms for
[MonthDay](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/plainmonthday.emu#L334)
and [YearMonth](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/plainyearmonth.emu#L461).

The authored `aot_temporal_partial_date_format` controls cover all sixteen
calendar identifiers, all four annotation modes, explicit reference dates,
expanded ISO years, `toJSON` option non-observation, borrowed methods in both
Realm directions, branding before option reads, conversion order, and arbitrary
abrupt identity. They run in strict and sloppy mode in the later verification
batch. This source change has not been compiled or executed; it changes no
published conformance count.
