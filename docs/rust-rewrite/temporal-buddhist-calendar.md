# Temporal Gregorian and Buddhist calendar follow-up

The completed-baseline replay on merged main rejected
`intl402/Temporal/PlainDate/prototype/daysInYear/basic-buddhist.js` with
`Invalid Temporal.PlainTime string`: the shared calendar-string parser did not
recognize `buddhist` and eventually tried its time-string grammar. A separate
main probe showed Gregorian PlainDate and PlainDateTime returning numeric
`weekOfYear` and `yearOfWeek`; ZonedDateTime already returned `undefined`.

The AOT compiler now admits the Buddhist calendar through the same canonical
calendar identifier table used by constructors, property bags, annotated strings
and calendar slots. Its arithmetic is proleptic Gregorian, with calendar years
543 greater than stored ISO years. The closed `TemporalCalendarArithmetic`
domain makes this relationship explicit. Constructors and strings retain ISO
coordinates. Property bags resolve era/year agreement in calendar years before
producing a `TemporalResolvedIsoYear`; absent years retain an existing receiver's
ISO year or the MonthDay ISO reference year. That prevents partial `with` calls
from subtracting the offset twice.

Calendar year and era getters cover PlainDate, PlainDateTime, PlainYearMonth
and ZonedDateTime. The single Buddhist `be` era preserves zero and negative era
years. Month/day fields, leap rules, date addition, differences and rounding
continue to operate on the stored ISO dates. MonthDay converts a supplied year
before applying overflow and then stores its ISO 1972 reference date. Only ISO
provides week numbering; all three date carriers now apply the same rule.

This follows the [Intl era and monthCode proposal](https://tc39.es/proposal-intl-era-monthcode/),
its Buddhist proleptic-calendar and era tables, and the pinned Buddhist fixtures.
The installed ICU implementation in
`vendor/icu_calendar-2.0.6/src/cal/buddhist.rs` independently uses the same year
offset and delegates month/day arithmetic to ISO. No new host service or
runtime dependency is required for this calendar.

`Intl.DateTimeFormat` still has no Buddhist formatter data. Its calendar
canonicalization assertion now checks spellings supported by both services;
Temporal arithmetic support does not imply formatter patterns or era labels.
Other calendars still need their own complete arithmetic implementation.
Existing missing Temporal methods also remain separate work, including
ZonedDateTime.prototype.with and calendar-relative Duration operations. This
change does not claim that all Buddhist fixtures or all Temporal tests pass.

The dedicated native regression target is
`cargo test -p lila-engine --test aot_temporal_buddhist_calendar -- --test-threads=1`.
It covers constructors versus bags, annotation round trips, eras and extreme
supported years, leap rules, partial-field merging, arithmetic and rounding,
MonthDay reference years, and exception/read order. Verification and pinned
replay results belong to the coordinating batch report; no new full-suite count
is asserted here.
