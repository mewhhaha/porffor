# Islamic civil and tabular calendar ownership

The closed Temporal calendar domain admits `islamic-civil` and `islamic-tbla`,
with `islamicc` canonicalized to the civil calendar. `TemporalIslamicCalendar`
selects the Unix epoch and partial-reference policy together. Existing ISO-backed
carriers retain ISO constructor/string coordinates; bags resolve calendar fields
through the existing completed date and partial-reference owners. Bare `islamic`
remains unsupported. The separately admitted Umm al-Qura calendar consumes its
required table through distinct arithmetic; its shared AH/BH selector does not
widen the Type-II Civil/Tbla arithmetic kind. Temporal support here does
not add Intl formatting data or complete T22.

The [March 2026 era/month-code proposal](https://tc39.es/proposal-intl-era-monthcode/)
specifies the Type-II cycle and Friday/Thursday epochs, ISO 0622-07-19 and
0622-07-18. Both have twelve months. `ah` resolves directly and `bh` resolves to
`1-eraYear`; nonpositive inputs remap on reporting. A positive arithmetic year
reports `ah`, otherwise `bh` with `1-year`. Reporting intervals are not an input
positivity restriction. Exact lowercase era spellings are required. Month codes
M01–M12 are suitable; M13 and leap-month codes are not. These current rules and
pinned Test262 era-boundary controls supersede older provider era-input limits.

The private `temporal_calendar_arithmetic/islamic.rs` leaf emits integer
arithmetic. Its forward rule matches committed `calendrical_calculations` 0.2.4
and the [pinned ICU4X tabular calculation](https://github.com/unicode-org/icu4x/blob/c9fac4e625ccb2c6a7aa35079fff9709db4385ac/utils/calendrical_calculations/src/islamic.rs):
`E-1 + 354*(y-1) + floor((3+11*y)/30) + 29*(m-1) + floor(m/2) + d`,
with Unix E -492148 or -492149. Odd months have 30 days, even months 29, and
M12 has an extra day when Euclidean `rem(14+11*y,30)<11`.

Projection uses the [original authors' exact inverse](https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L2041):
`floor((30*(epochDay-E)+10646)/10631)`. At the first day of year y its numerator
is `10631*y+29-r`; at the last it is `10631*(y+1)-1-r'`, for residues in 0–29.
This establishes the year across both endpoints. The registry's mean-year
estimate is not used; the workspace's vendored ICU calendar corrects that
estimate, and those two provider paths must not be described as identical.
Division and leap residues remain Euclidean for signed years.

Four actual parent-consumed leaf methods compute month length, fill the existing
`TemporalCalendarDateLocals`, convert regulated fields to ISO and choose a
MonthDay reference. They allocate no Temporal object and release temporary
locals in reverse order. The parent bounds calendar years to `[-300000,300000]`
before arithmetic, then retains each completed carrier's own converted ISO
limit. YearMonth checks the converted day-one reference's ISO month. Before
month information, the shared month factory requires a supplied MonthDay
native year to intersect the full ISO carrier range. Actual year-start and
next-year-start conversion supply that predicate; the requested month/day may
be outside the full-date range or in an adjacent ISO year. This follows the
[current whole-year rule](https://tc39.es/proposal-intl-era-monthcode/#sec-nonisomonthdaytoisoreferencedate).

`TemporalCalendarArithmetic::year_length()` selects LunarCommonPlusLeap for
this domain. The actual projection footer uses base 354 and its projected leap
flag; solar domains select their own policy. The consumed month_arithmetic()
selects Twelve for native-year counts and shared arithmetic, while rounding
uses actual year/month anchors. All four field routes consume the shared
resolved-month constructor and retain syntax, agreement and option order. Relative-duration plain/zoned consumers keep
the private calendar slot; virtual month anchors remain compared before day
clamping.

MonthDay starts with reference year 1392. M12 day30 selects leap year 1390 before
conversion; other candidates past ISO 1972-12-31 select 1391. An absent year
also regulates against 1390. Exact civil/tabular pairs are M11 day27
1972-01-14/1972-01-13, M12 day29 1972-02-15/1972-02-14, and M12 day30
1971-02-26/1971-02-25. Supplied years pass native-year admission before month
resolution, then regulate before reference completion and publication.

`aot_temporal_islamic_tabular_calendars` has six finite cohorts for projection,
fields, arithmetic, partial dates, relative duration and limits/Realms. It covers
signed epochs and AH2's boundary, lunar 354/355 lengths, all four field decoders,
virtual anchors, 12-month carry, latest references, private relative slots,
carrier-specific limits, both borrowed defining-Realm directions and original
throws after public globals are poisoned. Each cohort requires actual Wasm AOT,
strict and sloppy modes, Normal completion 262 and one exact print event.
This complete batch passed the ref83 whole-workspace all-target Rust type
check. Emitted Wasm, runtime regressions and broad acceptance remain pending.
No published count, full T22,
GC migration or Apia issue #3310 claim follows from these sources.

The shared whole-native-year admission correction passed the ref87 combined
whole-workspace/all-target Rust type check on 2026-10-04. Emitted-Wasm and
runtime verification remain pending; the earlier calendar checkpoint retains
its original source scope.
