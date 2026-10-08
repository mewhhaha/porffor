# Chinese and Dangi calendar ownership

This complete source batch joins projection, fields, partial references, calendar
arithmetic and relative operations through the existing Temporal ISO carriers.
The canonical calendar identifiers use related Gregorian arithmetic years and
have no eras. The preceding Hebrew/shared MonthDay source passed the ref87
all-target workspace type checkpoint. This newer source passed the successor
ref93 combined all-target Rust type check. Emitted-Wasm validation, native catalog
construction, focused semantic execution and grouped broad checks remain pending. Authored controls are not execution evidence.

The normative authority is the [Stage 4 era/monthCode draft, March 19, 2026](https://tc39.es/proposal-intl-era-monthcode/).
It requires the published modern Chinese and Dangi intervals and permits an
approximation outside them. The pinned native authority is ICU calendar2.0.6,
including its vendored `cal/chinese_based/{construction,proleptic}.rs`, locked
calendrical_calculations0.2.4 and ICU calendar data2.0.0. Standard month codes,
related ISO years and exact integer moments are used; cyclic sixty-year labels
and provider formatting aliases cannot become Temporal fields.

## One checked catalog and actual image consumer

The private data leaf owns `TemporalEastAsianCalendar`, the immutable row layout
and a `OnceLock<Result<Catalog, CatalogError>>`. Only `Catalog::build` constructs
its two named calendar catalogs through the pinned public Date API. It walks
all8357 retained related years per kind, from−3653 through4703, and complete
preceding/following model years. It validates year/day identity,12/13 counts,
29/30 lengths, regular/leap code order, the New Year offset18..52, July1
containment, declared totals and exact next starts. Cumulative month prefixes
are checked as they are produced. Both exact day joins and adjoining model
month-period boundaries/counts must agree before either catalog is published.

The constructor uses `cyclic_year().related_iso`, `month().standard_code`, and
actual Rata Die dates. The retained day joins are RD−1334565 and1717776. A
single capture-free unwind boundary converts a pinned-provider assertion
failure into an explicit catalog error; native errors never select another
model. The complete result, including a failure, is cached. There is no host
calendar service or runtime provider call.

The existing `uses_temporal_calendar` emission predicate reaches
`StringPool::collect`. It gates checked-image construction and appends the
calendar rows after all existing pooled bytes. Its carrier-name arms also
conservatively cover five typed Duration methods: compare, add, subtract, round
and total. Compare, round and total can reach calendar operations in the first
emission pass; add/subtract reject calendar units and ignore extra options, as
required by [AddDurations](https://tc39.es/proposal-temporal/#sec-temporal-adddurations).
Namespace
bootstrap roots are computed after the compiled partition and this predicate;
subsequent fixpoint discovery cannot supply an image to an already compiling
body. The same predicate therefore owns both initial Duration reachability
and the calendar helper/image decision. Other modules do not construct or
append the catalog. Collection now returns `Result`, and the sole product
caller propagates its emission error. The alreadylocked ICU calendar dependency
is used with compiled data; no new service or dependency resolution is needed.

Each eight-byte row stores packed month lengths/leap ordinal/New Year offset
and a signed cumulative month prefix. A closed row-slot enum supplies actual
four-byte load offsets. Private image headers provide the checked row address,
range, month total and separate lower/upper serial offsets. The
native lunar/solar period and observer-anchor constants feed those headers and
the emitted model; no duplicate lookup or second catalog census can drift.

## Completed emitted year and wide exact moments

One private non-Copy completed year owner contains kind, related year, adjacent
starts, month mask/count, leap ordinal and first month serial. Its sole factory
selects a retained row or the exact mean model before doing year calculation.
All projection, conversion, days/count, code/ordinal, serial and inverse helpers
obtain or borrow that owner. Consumers cannot independently pair another kind,
year, model or serial authority with its month metadata.

The proleptic leaf translates the pinned signed Euclidean mean-moon/solar-term
algorithm. Local moments are a canonical pair of Unix epoch day and milliseconds
within that day. Period division splits the day into quotient/remainder; period
multiplication splits the period index before multiplying. Thus a genuine large
virtual anchor does not require global-millisecond multiplication or emitted
i128. The maximum residue product is below signed i64; the wide±2^34 year
proof bounds all remaining day/serial arithmetic. Historical observers are
calendar-kind policy, separate from the user's IANA zone. Only the ancient and
modern observer arms are reachable outside the retained interval.

The field month factory rejects an acquired year outside this wide envelope
before native month information. Duration date components and rounding
increments bound genuine virtual candidates inside that same domain. Private
working-year normalization preserves the original input; final public carrier
limits remain their existing distinct owners. Full native-year admission is
not replaced with an approximate ISO-year or nineteen-year-cycle check.

Month serials use retained cumulative prefixes and exact selected moon periods
outside the catalog, with independent offsets at the two joins. Inside the image,
inverse serial searches retained row prefixes. Outside it, the same exact mean
lunar period projects the selected moon day, then a native-year adjustment
resolves the actual month. A Metonic-cycle estimate cannot substitute for this
selected authority.

## Shared pure calendar helpers — 2026-10-06

Calendar projection and calendar-to-ISO conversion now call shared Wasm bodies.
Those bodies consume the same Chinese, Dangi and Umm al-Qura year lookups as
other calendar operations. This prevents Duration round, total and compare,
and ZonedDateTime until and since, from copying the complete calendar branches
and year algorithms into each arithmetic step.

The closed runtime-helper registry fixes all six signatures. Every input and
result below is an `i64`; results appear in Wasm return order.

| Helper | Inputs | Ordered results |
| --- | --- | --- |
| `TemporalChineseYear` | Related year | Year, start, next start, month mask, leap ordinal, month count, first month serial |
| `TemporalDangiYear` | Related year | Year, start, next start, month mask, leap ordinal, month count, first month serial |
| `TemporalUmmAlQuraYear` | Table year | Packed year row |
| `TemporalUmmAlQuraEpoch` | Epoch day within the table interval | Calendar year, packed year row |
| `TemporalCalendarProjectDate` | Calendar ID, ISO year, month, day | Calendar year, month ordinal, day, days in month, day of year, days in year, leap flag, months in year |
| `TemporalCalendarFieldsToIso` | Calendar ID, regulated calendar year, month ordinal, day | ISO year, month, day |

`EastAsianYearLocals` and `TemporalCalendarDateLocals` retain their existing
ownership. Each record's private `result_locals` order supplies both the helper
return sequence and the caller's reverse stack stores. The projection borrows
the caller's calendar local; that local is neither returned nor released with
the eight owned result locals. Dedicated result tokens restrict complete year,
projection and ISO-triplet publication to their calendar owners. Conversion
writes the complete returned ISO triplet into the caller's original fields.

The original private algorithms still own retained/model year selection, exact
table membership, signed arithmetic and civil fallback. Callers retain field
regulation, range checks and JavaScript observation order. These six helpers
carry scalar arithmetic results and require no Realm, JavaScript completion or
new GC record.

All six declarations are registered when the heap is enabled. During a forced
builtin discovery pass with `uses_temporal_calendar == false`, their bodies
contain `unreachable` with their actual registered result signatures. Stub
emission returns before reading a calendar image. The same existing planning
predicate admits the retained data and the real helper bodies together once
calendar consumers are compiled.

The authored `temporal_calendar_shared_size` control prints sizes before
validation, validates the emitted Wasm, bounds each helper and selected
consumer, checks direct consumer-to-conversion calls and checks that those
conversions actually call the four year helpers. Existing Chinese/Dangi and
Umm al-Qura semantic controls remain in place. Combined Rust checking, emitted
validation, focused runtime execution and measured size acceptance for this
six-helper batch are pending; the source change establishes no conformance
count or completed T22 claim.

### Shared date arithmetic — 2026-10-07

The next source batch also shares the existing complete month-length,
year/month balancing and date-difference algorithms. The cached native attempt
still had 2,360,810-byte Duration.round, 1,915,792-byte total and 1,388,121-byte
compare bodies after the six helpers above. These sizes are from the preceding
artifact; this successor has not yet been compiled or measured.

| Helper | Inputs (`i64`) | Ordered results (`i64`) |
| --- | --- | --- |
| `TemporalCalendarDaysInMonth` | Calendar ID, calendar year, month ordinal | Days in month |
| `TemporalCalendarBalanceYearMonth` | Calendar ID, calendar year, possibly unbalanced month ordinal | Balanced calendar year, month ordinal |
| `TemporalCalendarDifferenceDate` | Calendar ID, left ISO year/month/day, right ISO year/month/day, largest unit | Years, months, weeks, days |

The original private bodies retain their algorithms. Complete difference still
creates both projections through the original `TemporalCalendarDateLocals`
factory, preserves both Surpasses phases, and constrains the midpoint only after
the virtual position comparison. Dedicated non-discardable result tokens publish
both balanced fields or all four date-duration fields together. The helper
compiler loads result order forward; the caller stores the stack in reverse.
No Realm, completion or JavaScript value enters these scalar signatures.
Addition and regulation keep their existing error paths in their callers.

All three use the same `uses_temporal_calendar` planning predicate and typed
`unreachable` stubs as the preceding helpers, so initial discovery cannot read
an unplanned calendar image. The existing size control now requires nine real
bounded helper bodies, direct consumer calls to both addition helpers, and
round/total/Zoned difference calls to the shared complete difference. It also
requires that difference itself consumes the shared projection, ISO conversion
and month-length helpers. Compilation, emitted validation and native execution
for this successor remain pending.

### Shared complete conversions — 2026-10-07

The next measured artifact still contained 1,861,681-byte Duration.round,
1,513,907-byte total and 1,311,800-byte compare bodies after shared object-header
projection. Each relativeTo dispatch expanded two complete ZonedDateTime
conversions and three PlainDate conversions. Those complete operations now
have one emitted body each, in addition to the nine scalar calendar helpers.

`TemporalZonedDateTimeConvert` receives the input and options Values, the actual
called function's non-null `FunctionContext`, and the caller Environment.
`TemporalPlainDateConvert` receives the input, an optional overflow-options
Value, the same actual context and the Environment. The optional operand keeps
presence separate from the Value tag: `Read(undefined)` retains its option-read
operation, while `Omit` exposes no value to the guarded reader. All five original
PlainDate option-read positions consume that private parameter.

The helper entry retains an owned copy only when its typed declaration includes
`caller_function_context`. It does not manufacture `BodyEntryLocals` or derive
the function Realm from the active Realm or Environment. Both intrinsic
prototype selection and native error allocation read the original immutable
`FunctionContext.REALM`. The Environment remains a separate operand for existing
property-read, conversion and invocation helpers. The retained context remains
live through completion publication and is cleared in the normal epilogue;
an early return releases its Wasm frame. Existing helpers without a transported
context retain their original Environment/active-Realm error route.

Both helpers return the existing complete JavaScript Completion: Value tag,
scalar, reference, completion kind and target. ZonedDateTime returns the same
genuine result record allocated by its original kernel. PlainDate transports
its three ISO coordinates and calendar through a temporary genuine
`TemporalPlainDateObject`, using the called Realm's intrinsic prototype. This
adds one temporary PlainDate object on successful conversion. It invokes no
public constructor or prototype getter. The facade projects the original slots
only after Normal completion and promptly releases the temporary carrier;
abrupt values and caller throw routing remain intact. Parsing, field reads,
option reads, calendar regulation and zone-provider order remain in their
original private algorithms.

The ZonedDateTime property-bag path obtains its calendar through the existing
`ToTemporalCalendarIdentifier` factory after reading `calendar` and before the
first date field. It accepts the operation's date-shaped calendar strings. The
constructor retains its separate `CanonicalizeCalendar` operation and diagnostic
context. Retargeting the old raw-slot guard exposed and corrected a pre-existing
bag call to bare canonicalization; the existing called-Realm bag fixture now
uses a date-shaped ISO calendar string to cover this distinction.

The existing calendar planning predicate admits both real conversion bodies
with their retained data. When that predicate is false, their registered
whole-Completion signatures receive typed `unreachable` stubs before any image
or input work. The size control now requires direct converter calls from the
Duration consumers and direct ZonedDateTime conversion calls from until/since,
with a 1 MiB bound for each selected consumer and converter. Existing created-Realm
fixtures cover successful branded/string/bag conversion, nested option hooks,
intrinsic error Realm, abrupt read prefixes and thrown object identity. These
source changes are awaiting the joined type, artifact and runtime checkpoint.

## Closed code policy and field phases

`TemporalCalendarMonthCode` owns canonical spellings, encodings, numeric month,
leap flag and comparison rank. Existing encodings, including M05L14, retain
their identity; all remaining leap codes extend that same declaration. Chinese
and Dangi accept M01..M12 and their leap forms; M13 and M13L remain unsuitable.
The source pool, incoming matching and borrowed projection formatter consume
that domain. Const validation binds each native kind to its calendar ID and
each spelling to its numeric/leap projection. It rejects duplicate logical
codes or a missing regular counterpart before reference publication can
compile. Calendar, month-count and year-length policies are exhaustive; the
year-length policy carries no unused kind payload.

The sole non-Copy resolved-month factory checks suitability only after the
usual property/option acquisition and required-field guards. For a supplied
year it computes the constrained code ordinal for agreement while retaining
the original code through requested overflow. Missing leap Constrain skips
back to the corresponding regular month. A yearless lunisolar MonthDay does
not invent an ordinal from year0. The borrowed completed projection formatter
uses its actual year and month; all existing getters and receiver snapshots
keep that ownership until the canonical String has been produced.

Year addition retains the original code before serial month addition. Difference
keeps both original-code and balanced-ordinal Surpasses phases before clamping
the day. The code rank comes from the closed numeric/leap projection, never
from the internal encoding. Date/YearMonth rounding dates both brackets from
the actual same origin and retains separate years and months. Plain/zoned
relative operations consume the existing retained calendar context.

## Complete MonthDay reference policy

Both partial reference factories consume the [explicit non-ISO reference table](https://tc39.es/proposal-intl-era-monthcode/#sec-nonisomonthdaytoisoreferencedate).
A supplied native year must contain some date inside the full supported ISO
carrier; actual adjacent starts prove this before month resolution. Requested
overflow regulates in that supplied year, then the accepted canonical code is
resolved again for the reference year. The requested native month/day itself
may precede or follow the carrier while the year remains admissible.

Yearless regulation uses30 days for suitable Chinese/Dangi codes before table
availability is applied. The table includes M09L2014, M10L1984 and M11L2033
for days1..10 versus2034 for days11..29. M01L/M12L and specified leap day30
combinations have no historical reference; Reject throws, while Constrain
uses the corresponding regular code and its reference. The Chinese/Dangi M03
day30 references remain distinct. There is no generic1900..1972-only search.
The latest actual eligible native candidate in the prescribed ISO reference
year is converted with the same completed-year authority and accepted code.

YearMonth derived references still use native day1. Explicit ISO constructor
references and full-date/YearMonth final limits retain their own authority.
MonthDay.toPlainDate retains its canonical code and resolves it in the argument
year through the same month proof, with its separate full-date carrier check.

## Controls and remaining acceptance

Six finite Engine cohorts cover projection, fields, arithmetic, partial dates,
relative duration, limits and borrowed Realms. The wrapper requires WasmAot in
strict and sloppy modes, exact Normal Number262 and a sole expected print event.
Literal pinned modern pairs, signed years, retained/model joins, code/ordinal
shifts, absent references, requested overflow, virtual anchors, rounding and
original abrupt identities exercise actual semantics. No numeric oracle, native
catalog run, fixture parser or runtime execution generated these sources.

Errors use the existing current-function intrinsic Realm helpers. Mutable public
constructor/error globals do not select another Realm. This source supplies no
fresh Test262 counts, full T22 completion, Apia issue #3310 resolution, Intl
formatting expansion, GC migration or unsupported runtime fallback. The grouped
semantic and conformance checkpoints still own those claims.
