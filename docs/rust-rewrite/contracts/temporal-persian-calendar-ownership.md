# Persian calendar ownership

The seventh admitted calendar uses the existing ISO-backed Temporal carriers
and completed calendar-field/reference owners. Constructors and annotated
strings keep ISO coordinates; field bags resolve signed `ap` years and Persian
month/day coordinates before conversion. Projection, regulation, addition,
difference, partial references and relative-duration consumers share this domain.

The private `temporal_calendar_arithmetic/persian.rs` leaf emits the pinned
[ICU4X fast Persian rule](https://github.com/unicode-org/icu4x/blob/c9fac4e625ccb2c6a7aa35079fff9709db4385ac/utils/calendrical_calculations/src/persian.rs):
the 33-year cycle and its 78 correction years govern forward conversion,
inverse projection and leap status together. Signed division is Euclidean.
Months 1–6 have 31 days, months 7–11 have 30, and month 12 has 29 or 30.
The parent bounds arithmetic years conservatively to `[-273000, 276000]`
before multiplication; each completed carrier then applies its own ISO limit.
The unused 2820-year comparison rule is not a product authority.

Four parent-consumed methods fill the existing projection locals, compute month
length, convert a regulated triple, and choose a MonthDay reference. Their
temporary locals release in reverse reservation order. The leaf publishes no
Temporal object. Existing completed date/reference owners remain mandatory at
all allocators, after options, regulation and prescribed limit checks finish.

YearMonth references use Persian day 1. MonthDay references use the latest valid
match on or before ISO 1972-12-31: common year 1351, or leap year 1350 when the
candidate is later or is month 12 day 30. Before month information, a supplied
native year must contain some date inside the full ISO carrier range. The
shared month factory requires this admission: the actual starts of that year
and its successor bracket the carrier bounds. The requested month/day can lie
outside those bounds or in an adjacent ISO year; overflow then regulates it
before the independent reference selection. PlainDate's full-date and
YearMonth's reference-month limits remain separate. These are the current
[non-ISO MonthDay reference rules](https://tc39.es/proposal-intl-era-monthcode/#sec-nonisomonthdaytoisoreferencedate).

The era domain accepts exact `ap`, including zero and negative era years;
`sh`, `hs` and mixed-case era aliases remain rejected. Calendar identifiers
continue to canonicalize through the existing closed calendar resolver. The
[calendar and era tables](https://tc39.es/proposal-intl-era-monthcode/#sec-temporal-calendarsupportsera)
require the published Persian leap behavior for years 1206–1498 AP and allow
implementation-defined approximations outside that interval. This compiler
keeps the pinned ICU rule across its supported carrier range.

The finite Engine target `aot_temporal_persian_calendar` covers independently
pinned Nowruz dates, signed years, correction years 1502/1503, observable field
and option order, virtual month anchors, partial references, plain/zoned
relative arithmetic, carrier limits and borrowed intrinsic Realms. It requires
Wasm AOT in both strictness modes with exact completion and output. The ref80
`cargo xc --locked --offline` whole-workspace all-target Rust type check passed,
including these Rust test targets. Emitted Wasm, focused runtime regressions and
broad verification remain required; type checking does not establish runtime
acceptance. Other calendars, full T22 and Apia issue #3310 remain open.

The shared whole-native-year admission correction passed the ref87 combined
whole-workspace/all-target Rust type check on 2026-10-04. Emitted-Wasm and
runtime verification remain pending; the earlier calendar checkpoint retains
its original source scope.
