# Hebrew calendar ownership

This source batch joins Hebrew field resolution, projection, partial dates and arithmetic on the existing Temporal ISO carriers. It adds canonical `hebrew` with signed AM years. It does not add another object representation, a host calendar service, or a provider formatting-code fallback. The ref87 combined whole-workspace/all-target Rust type check passed, including the shared MonthDay admission correction. Emitted-Wasm execution and pinned conformance verification remain pending.

The current [Stage 4 era/monthCode draft, March 19, 2026](https://tc39.es/proposal-intl-era-monthcode/) defines the calendar and MonthCode policy. The pinned ICU calendar source at `vendor/icu_calendar-2.0.6/src/cal/hebrew.rs` separates standard codes from formatting codes; its Adar II formatting alias M06L is not a Temporal code. The native arithmetic lane uses the pinned integer Hebrew algorithm and signed Euclidean operations, with actual adjacent new-year lengths rather than a common-year-plus-one approximation.

## One code domain and one completed month

`TemporalCalendarMonthCode` owns canonical spellings and their internal encodings. The actual calendar policy selects suitable entries; data pooling, incoming-code matching and output formatting consume that same domain. Hebrew accepts regular M01 through M12 and M05L. M13, M06L and every other leap spelling reject. Regular codes from M06 onward shift one ordinal in a leap year, while M05L occupies ordinal6.

The four property-bag decoders consume `emit_temporal_resolve_calendar_month`: PlainDate, PlainYearMonth, PlainMonthDay and ZonedDateTime. PlainDateTime uses the PlainDate resolver. Its private non-Copy `TemporalResolvedCalendarMonth` owns the resolved-year proof, the actual ordinal and the original code/presence. The closed diagnostic context selects the actual caller's existing intrinsic Realm messages. The acquired String payload becomes an encoded original code only after suitability succeeds; no user-visible field or String is re-coerced.

Agreement and overflow are distinct phases. The [field-resolution algorithm](https://tc39.es/proposal-intl-era-monthcode/#sec-nonisoresolvefields) derives a constrained supplied-year ordinal for comparison while retaining the original code. The actual date and MonthDay conversion functions consume the completed month owner and apply requested overflow later. This prevents a conversion from omitting original-code validation or independently choosing another year/ordinal pair.

ToMonthCode keeps its earlier ordering: ToPrimitive with a String hint, actual String requirement, then shape validation at field acquisition. Calendar suitability follows later field and option reads. Missing required fields, incomplete era pairs and the existing MonthDay numeric-month/year rule retain their TypeError precedence. User-thrown values return through the existing abrupt route before generated errors can replace them.

## Projection and receiver defaults

The retained `TemporalCalendarDateLocals` contains calendar, native year, ordinal, month/day lengths and leap information. Its new months-in-year field comes from the same native year. Month and monthsInYear accessors use these completed values; they do not derive Hebrew month count from the stored ISO year or a fixed calendar count.

`emit_temporal_calendar_month_code_payload` accepts a borrowed completed projection. The Hebrew leaf selects its canonical encoded code from that projection's native year and ordinal, and the closed enum maps it to the pooled String. A caller cannot supply only a bare ordinal. PlainDate, PlainDateTime, PlainYearMonth, PlainMonthDay and ZonedDateTime getters all use this route. ZonedDateTime retains its actual record calendar and complete local ISO date rather than formatting a numeric month result after discarding year and leap information.

All five `.with` paths snapshot the receiver's canonical String code before user field effects and projection release. The snapshot has its own reserved local because field acquisition may overwrite an absent input-code output. It merges only when the input supplies neither month nor monthCode. Explicit month/code exclusion, year/era replacement and field/option order remain with the existing owners. A year change therefore preserves M12 or M05L, then resolves the correct ordinal in the replacement year.

Snapshots and formatter scratch obey the existing reverse local lifetime. Projection locals remain live through formatting, and each receiver snapshot is released at its matching position in the caller's reverse scratch list. No persistent temporary crosses the consumed month handoff or partial-reference allocation.

## MonthDay and partial carriers

A supplied MonthDay year controls agreement, overflow and admission before reference selection. Its ordinal must agree in that supplied year. Hebrew year admission checks whether the native year intersects the full ISO carrier range, using its actual start and next-year start against the shared ISO bounds; the requested month/day need not itself fall inside that range. After requested overflow succeeds, conversion derives the regulated year's accepted canonical code before selecting the deterministic reference year and resolving a new ordinal there. An ordinal from a leap year cannot be copied into a common reference year. The successor shared admission owner applies that same whole-year rule to every earlier range-checked calendar after its conservative arithmetic envelope, before month resolution. ISO retains its OverflowOnly policy. The obsolete chosen-date ISO-year gate is retired; fixed-calendar limit controls retain the behavioral debt until execution.

A yearless MonthDay does not invent a supplied year. It keeps a suitable canonical code, regulates day against that code's maximum possible month length, then selects the latest fully eligible reference date. Variable M02/M03 day30 and M05L consequently select earlier eligible years when required. The actual absent-year regulator and both final reference factories consume the native policy. No host clock is read.

MonthDay.toPlainDate snapshots the reference projection's canonical code before reading the argument year/era pair. It resolves that code for the target year with the same completed month factory and existing constrain policy. Its year-only argument must not reuse the reference ordinal.

YearMonth still uses calendar day1 for its derived reference, and explicit constructor reference ISO fields remain intact. All five carriers continue to store their existing ISO fields and retained calendar slot; arithmetic and relative contexts operate through the shared calendar owner.

## Arithmetic and verification scope

The shared arithmetic lane uses exact Hebrew month serials for balance, Add and month differences. The year step retains canonical code before applying the month step. Difference preserves the actual virtual year/month anchors before day clamp; rounding and bubbling use the next real calendar-year anchor rather than dividing by one year's 12/13 count. Projection derives year length from adjacent native new years, including all six Hebrew year lengths.

The finite Engine cohorts exercise projection, fields, arithmetic, partial dates, relative durations, limits and borrowed Realms through the existing WasmAOT route in strict and sloppy modes. They include ordinary/leap ordinal shifts, sole M05L acceptance, original-code overflow, all receiver merges, reference-year re-resolution, variable month-day references, real rounding anchors, source ordering and original abrupt identity. These are authored semantic controls, not execution evidence.

Native errors keep the called builtin's defining Realm through existing current-function helpers. Public constructor or error-global mutation does not select a replacement Realm. The batch does not claim Intl Hebrew formatting data, all Temporal calendars, whole T22, Apia policy changes, semantic GC migration or refreshed Test262 counts. The passing combined type checkpoint supplies no semantic claim; focused regressions and broad verification still own those claims.
