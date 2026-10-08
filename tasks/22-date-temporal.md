# T22 — Date and Temporal

## Partial-date reference formatting — 2026-10-07 dry repair

The current-source audit found that PlainMonthDay and PlainYearMonth incorrectly
suppressed their reference ISO year/day with `calendarName: "never"` for non-ISO
calendars. The shared reference-field predicate now retains those fields for
every non-ISO calendar; annotation suppression stays private to its separate
consumer. Two finite strict/sloppy controls cover all sixteen calendars, all
four modes, explicit and expanded reference dates, JSON behavior, option order,
abrupt identity and both borrowed Realm directions. The
[formatting contract](../docs/rust-rewrite/contracts/temporal-partial-date-reference-format.md)
records the specification and deferred verification. No compilation or runtime
tests were run for this dry batch.

The 2026-10-07 upstream recheck still finds Apia issue #3310 open and proposal
#3318 draft; the integration algorithm retains the affected assertions.
The owned contextual-window rejection remains pending an accepted result
policy. Historical “remaining calendars” and “configured default-zone” gaps
below are superseded by the implemented sixteen-calendar and configured-zone
batches; complete T22 conformance and independent-host acceptance remain open.

The 2026-10-07 native collection passes all four Duration controls, paired East
Asian field controls and paired Umm al-Qura arithmetic, limits, projection and
relative-Duration controls. Three partial-date/limit controls expose missing
Intl imports for shared converters; the import plan now consumes the same
calendar-helper gate as emission. Three East Asian runtime failures identify
Gregorian fallthrough: the arithmetic gate omitted both lunisolar domains.
The source repair exhaustively classifies every arithmetic domain, including
East Asian and Hebrew. Umm al-Qura's remaining field failure is a fixture
accessor copied by Object.assign before the observed call; that fixture and its
tabular Islamic/thirteen-month counterparts now pass the original bag directly.
Their assertions and deadlines remain. The joined repair is written; native
verification of that repair and full T22 acceptance remain open. Earlier results
below retain their original source scope.

The joined repair passes the 2026-10-07 all-feature/all-target workspace check
in 49.14 seconds and the converter artifact regression. Paired native controls
now pass East Asian limits, partial dates and projection; Umm al-Qura fields
and partial dates; tabular Islamic and thirteen-month fields; and Hebrew
partial dates and relative Duration. The remaining arithmetic fixtures
incorrectly equate swapped-endpoint differences across leap months; `since`
negates the same receiver's difference. The remaining Duration fixture
incorrectly expects add/subtract to permit calendar units with a second
argument, which those methods ignore. All three corrected fixtures now pass
both strict and sloppy modes in `tasks-calendars-resource-repairs1`: East Asian
arithmetic in 247.05 seconds, East Asian Duration in 229.16 seconds and Hebrew
arithmetic in 205.66 seconds. Full T22 acceptance remains open.

The ref87 `cargo xc --locked --offline` whole-workspace all-target Rust type
check passed on 2026-10-04 in 30 seconds of watched wall time, covering Hebrew
and the shared MonthDay native-year admission correction, including Umm al-Qura
table and reference-policy const validation. It covers the complete Islamic civil/tabular,
Coptic/Ethiopic/Ethioaa, Persian and Indian calendar batches, configured-zone
and selected-window consumers, and their Rust test targets. Formatting, module inventory and
task-plan checks passed. Emitted Wasm, focused runtime
regressions, broad suites and full T22 acceptance remain pending. Historical
execution evidence below retains its original source scope.

The first newer combined check at ref91 stopped on one exhaustive match in
the shared era accessor: Chinese and Dangi were omitted. Attempt13 retains
exit101 after 30 seconds of watched wall time. The correction explicitly joins
both to the no-era undefined-result arm; the closed domain and authored
projection controls already specify that behavior. The unused image-type
re-export is also removed. No other diagnostic failed compilation. The corrected
source subsequently passed the ref93 combined all-target Rust type check in
30 seconds of watched wall time (Cargo22.73 seconds). Emitted-Wasm/runtime
verification remains pending; the failed attempt13 supplies no passing proof.

## Chinese and Dangi calendars — 2026-10-04 dry implementation

The closed calendar domain admits both names with related Gregorian arithmetic
years and no eras. All twelve regular/leap MonthCodes share one spelling,
number, leap and ordering authority. Suitability precedes supplied-year
agreement, which retains the original code until requested overflow. Yearless
MonthDay postpones lunisolar ordinals until reference selection. The sole month
factory rejects arbitrary years outside the wide arithmetic envelope before
any native month information; actual full/partial carrier limits remain distinct.

The private compiler-native catalog uses the pinned ICU2.0.6 public Date API.
Its only constructor validates all retained years, adjacent model years,
month order/length/count, New Year offsets, July1 containment, exact next starts
and both day/month-serial joins. One checked immutable image is appended only
by the existing Temporal emission predicate. Its five typed Duration consumers
are included before first-pass body emission; later bootstrap/fixpoint discovery
cannot repair an image absent from an already compiling body. A provider invariant failure is
an explicit emission error. The emitted non-Copy year owner selects retained
rows or the exact proleptic mean model before calculation. Paired integer
moments avoid global-millisecond overflow for genuine wide virtual anchors;
serial inverse uses bounded search against the actual selected year boundaries.

Shared Add/difference/rounding retain original-code and balanced-ordinal
Surpasses phases before day clamp. Both partial reference factories consume
MonthDay's complete table, including M09L2014, M10L1984, split M11L2033/2034,
absent M01L/M12L and unavailable leap day30 combinations. Supplied native years
use whole-year intersection before regulation; accepted codes are derived after
requested overflow and re-resolved in the reference year. Plain/zoned relative
contexts keep the actual calendar owner.

Six finite strict/sloppy Engine cohorts are authored. The complete newer batch
passed the ref93 combined all-target Rust type checkpoint. Emitted-Wasm
validation and semantic execution remain pending; the ref87 proof retains its
predecessor scope. Apia issue #3310, complete
T22 and full conformance remain open. See the
[Chinese/Dangi contract](../docs/rust-rewrite/contracts/temporal-east-asian-calendar-ownership.md).

## MonthDay native-year admission — 2026-10-04 dry implementation

All range-checked calendars admit a supplied native year exactly when any date
in that year intersects the full supported ISO carrier. The shared owner first
checks a conservative arithmetic envelope, then compares the actual native
new-year and next-year starts against the shared full-date bounds, before the
sole month factory resolves agreement or overflow. The requested month/day
need not itself fall inside that carrier. ISO retains its specified
OverflowOnly exemption. The partial-reference factory no longer rejects a
chosen date merely because its corresponding ISO year lies outside the
carrier. Five existing finite limit/Realm cohorts own the genuine lower/upper
boundary regressions.

This batch also repairs the single exhaustive ROC arithmetic arm caught by
ref86's first Hebrew combined check, retained as failed attempt11. The ref87
combined whole-workspace/all-target check passed as attempt12; emitted-Wasm
and runtime acceptance remain pending. No published conformance result changes.

## Hebrew calendar — 2026-10-04 dry implementation

The fourteenth canonical entry admits `hebrew` with signed AM years. One
private integer leaf owns native new years, six actual year lengths, month
serials and complete conversion. Closed month/year policies replace calendar-only
counts and a uniform common-base footer. The exact signed serial inverse is
proved against both elapsed-month year boundaries; projection corrects its
estimate against actual starts. Conservative field envelopes and completed
full-date/partial owners retain their distinct limits.

The sole non-Copy month constructor consumes the resolved-year owner after
required-field checks and acquisition. It validates calendar suitability,
compares the supplied ordinal against a constrained code, and retains the
encoded original code for later requested-overflow validation. Canonical M06
moves from ordinal 6 to 7 in a leap year; M05L keeps its identity across year
changes until constrain/reject. Getter output and receiver defaults borrow the
actual calendar/year/month projection. MonthDay re-resolves its canonical code
when choosing the reference year and when converting to a supplied full year.

Hebrew MonthDay admits a supplied native year when any date in that year
intersects the full ISO carrier. It checks exact adjacent new-year starts
before month resolution and preserves the accepted code after overflow when
selecting the reference. The successor shared admission batch applies the
same whole-native-year rule to the earlier range-checked calendars and retires
their chosen-date ISO-year gate; those finite controls still require execution.

Year addition preserves the code before month addition, and virtual difference
compares before day clamping. Date and YearMonth rounding retain years and
months separately and date both brackets from the same origin. Their existing
emitters now have private difference owners, allowing field and rounding work
in independent files. Relative plain/zoned consumers keep the shared private
calendar context. Six finite strict/sloppy Engine cohorts cover the native
boundaries, fields/order, partials, arithmetic/rounding, relative durations and
borrowed Realms. The complete source and Rust control targets passed the ref87
combined all-target type check; emitted-Wasm and runtime acceptance remain
pending. Remaining calendars, Apia issue
#3310 and full T22 remain open. See the
[Hebrew contract](../docs/rust-rewrite/contracts/temporal-hebrew-calendar-ownership.md).

## Umm al-Qura calendar — 2026-10-04 dry implementation

The thirteenth canonical entry admits only `islamic-umalqura`. Separate
UmmAlQura arithmetic consumes the required AH 1300..1600 literal year table;
outside the exact table boundaries, real civil emission supplies all fields.
The Type-II Civil/Tbla arithmetic kind stays closed. A consumed typed Hijri
era selector shares AH/BH membership and the signed `1-eraYear` involution
without choosing a Type-II month pattern for the table calendar.

The native data constructor validates each ISO literal, twelve month flags,
354/355-day total, exact year census, contiguous year starts and both civil
joins. The original Unicode permission notice and exact source attribution
remain with the transformed rows. Balanced emitted year/epoch lookups use
the same validated records and exact boundaries. The actual projection fills
year, month, day, ordinal, month length and 355-day flag; the consumed parent
footer adds that flag to the exhaustive common-year authority 354. Count-aware
add, difference, YearMonth rounding and plain/zoned relative operations remain
joined through the retained private calendar and completed ISO carriers.

MonthDay has one absent-year regulator and two final reference consumers.
All twelve day30 reference years are const-derived from actual table rows,
choosing the latest fully converted candidate in ISO 1900..1972. The absent
year chooses that month-specific year before overflow regulation, preserving
valid day30 and constrained day31 even when no single year has every long
month. Final day1..29 references use 1392 or 1391 at the cutoff; day30 uses the
derived policy. Supplied years regulate with their actual month length first,
then retain the converted ISO-year-only check. Full-date and YearMonth month
boundaries keep their distinct authority.

Six finite strict/sloppy Engine cohorts cover projection and actual table
lengths, all four field routes and original abrupt identities, both-sign
arithmetic/virtual anchors/rounding, all latest day30 references and both
partial factories, relative contexts, carrier limits and borrowed intrinsic
Realms. Both required table edges, AH1600 and both adjacent civil joins are
explicit. The prior Islamic cohort now rejects only bare `islamic` in its
remaining unsupported-ID control. This complete batch passed the ref84
combined all-target Rust type and const checks. Emitted-Wasm and runtime
acceptance remain pending; remaining calendars, Apia
issue #3310 and full T22 remain open. See the
[Umm al-Qura contract](../docs/rust-rewrite/contracts/temporal-umalqura-calendar-ownership.md).

## Islamic civil and tabular calendars — 2026-10-04 dry implementation

The eleventh and twelfth canonical entries admit `islamic-civil` and
`islamic-tbla`; `islamicc` canonicalizes to civil. One closed kind binds each
Unix epoch and MonthDay reference policy. The private emitted leaf uses exact
Type-II thirty-year arithmetic with signed Euclidean division. Its inverse
is proved against the actual forward year starts; it does not inherit the
approximate year-only helper as a completed conversion.

All native projection, month lengths, complete calendar-field conversion,
regulation, addition, difference and both partial-reference routes join the
existing ISO carriers. The consumed arithmetic domain states both its fixed
twelve-month count and common-year length 354. The projection footer adds the
actual leap flag; existing solar domains state 365. Existing count-aware
MonthCode decoders, YearMonth rounding and plain/zoned relative contexts retain
the same canonical calendar, original conversion order and completed owners.

AH maps input eraYear directly and BH maps 1-eraYear. Reporting uses AH for a
positive arithmetic year and BH otherwise. Non-positive input era years
remap rather than being rejected; aliases and case-folding remain specific
to calendar IDs, not era spellings. YearMonth converts calendar day 1 before
its ISO month limits. MonthDay regulates an absent year with leap year 1390,
then selects the latest match before the 1972 cutoff: common reference 1392,
1391 for later common candidates and 1390 for M12 day30. A supplied year first
regulates in its actual calendar year and checks only the fully converted
ISO year, independently of full-date boundary days and YearMonth months.

Six finite strict/sloppy Engine cohorts cover literal projection and signed
era/new-year boundaries, all four field routes, original hooks and errors,
both-sign arithmetic and virtual anchors, partial references, relative
durations, complete carrier limits and both borrowed intrinsic Realm
directions. This complete source batch passed the ref83 combined all-target
Rust type check; emitted-Wasm and runtime acceptance remain pending.
Remaining calendars, Apia issue #3310 and full
T22 acceptance stay open. See the
[Islamic tabular contract](../docs/rust-rewrite/contracts/temporal-islamic-tabular-calendar-ownership.md).

## Thirteen-month calendars — 2026-10-04 dry implementation

Coptic, Ethiopic and Ethioaa complete the eighth through tenth canonical
calendar entries, with `ethiopic-amete-alem` canonicalized to Ethioaa. One
closed kind binds each epoch and reference policy. The private emitted leaf
uses pinned Euclidean Coptic/Ethiopian integer arithmetic, including negative
years and the universal four-year leap rule. Complete projection, field
regulation, addition, difference and both partial-reference factories consume
that leaf through the existing ISO carriers and completed owners.

The arithmetic domain is the authority for fixed month counts. All four actual
MonthCode suitability decoders admit M13 only for the thirteen-month calendars;
syntax coercion, field acquisition and caller-specific errors retain their order.
Getters, month balance, PlainDate difference and rounding carry, and all
YearMonth total/quantum/bubble/output operations use the retained count.
ISO constructors and range checks retain ISO month limits. Relative plain,
zoned and Duration operations retain the same calendar through their existing
context owners.

Exact Coptic `am` and Ethioaa `aa` era years are signed. Ethiopian `am` maps by
identity and `aa` maps by eraYear minus 5500; reported positive years use `am`,
and nonpositive years use `aa`. Era input is not rejected merely because it
crosses the reported era boundary. MonthDay missing-year regulation uses a
leap reference; final selection chooses the latest valid match before the
1972 cutoff. Supplied years check the fully converted ISO year independently
of PlainDate boundary days and YearMonth boundary months.

Six finite strict/sloppy Engine cohorts cover pinned projection and leap
boundaries, observable fields/options, both-sign arithmetic and virtual
anchors, partial references, relative rounding, full carrier limits and
borrowed intrinsic Realms. The complete batch passed the ref82 combined
all-target Rust type check; emitted Wasm and runtime acceptance remain pending.
Other calendars, Apia issue #3310
and full T22 acceptance remain open. See the
[thirteen-month contract](../docs/rust-rewrite/contracts/temporal-thirteen-month-calendar-ownership.md).

## Persian calendar — 2026-10-04 dry implementation

The seventh admitted calendar has one closed PersianSolar arithmetic domain
and signed `ap` era, including zero. Projection, field resolution, regulation,
addition, difference anchors and partial-date references join together through
the existing retained calendar and completed field owners. The private emitted
leaf follows pinned calendrical_calculations 0.2.4's actual ICU 33-year integer
policy, sharing its correction years across forward, inverse and leap paths.
No runtime evaluator or second date representation is introduced.

Euclidean arithmetic preserves negative years. A conservative Persian envelope
covers the full ISO carrier before integer arithmetic, followed by each
carrier's actual final range check. A supplied MonthDay year now checks its
whole native-year interval against the full ISO carrier before month
resolution. Missing-year regulation uses a leap-capable Persian
year; the completed reference selects the latest valid match before the 1972
cutoff. YearMonth uses calendar day one converted to ISO; explicit constructors
retain explicit ISO references. Plain, zoned and relative Duration consumers
use the same projected calendar through the already required context owners.

Finite strict/sloppy Engine cohorts cover projection and correction boundaries,
signed eras, hook/option order, both-sign arithmetic and virtual anchors,
partial references, relative plain/zoned operations, full carrier limits and
borrowed intrinsic Realms. This whole batch passed the ref80 combined all-target
Rust type check; emitted Wasm and runtime acceptance remain pending. Other calendars,
the upstream Apia rounding-window issue3310 and full T22 acceptance stay open.
See the [Persian contract](../docs/rust-rewrite/contracts/temporal-persian-calendar-ownership.md).

## Indian calendar — 2026-10-04 dry implementation

The complete additional-calendar batch admits `indian` with full-date
projection, calendar-coordinate field resolution, arithmetic and partial-date
references. Calendar-sensitive plain, zoned and relative Duration consumers
require retained canonical calendar slots. Indian's March year boundary and
month lengths are emitted integer arithmetic; no identifier-only admission or
new host evaluator is used. YearMonth and MonthDay allocations consume their
completed calendar reference dates rather than ISO day 1/year 1972 substitutes.

Six finite Engine cohorts cover twelve sloppy/strict observations, including
field order, virtual difference anchors, original abrupt identity, carrier
limits and both called intrinsic Realm directions. Authoring and review do not
establish a passing execution. The combined all-target Rust type check passed;
focused regressions and the T22 conformance checkpoint remain pending. See the
[Indian calendar contract](../docs/rust-rewrite/temporal-indian-calendar.md).

## Configured default-zone source — 2026-10-04 status reconciliation

The working tree already contains the complete authored configured-system-zone
Date batch. ConfiguredSystemTimeZone validates one immutable UTC/fixed/named
choice, RealmBuilder consumes it, and the real host primitive and private
response decoder feed Date local operations, omitted-zone Temporal.Now and
Intl.DateTimeFormat. Realm clones, workers and created Realms retain that choice.
The UTC default and injected HostClock remain explicit independent policies.

Actual source and regression controls passed the combined all-target Rust type
check; execution remains pending. Earlier named-zone-only statements below describe their
original checkpoint, rather than the current source surface. Additional calendar
algorithms, Apia contextual-window ownership and full Date/Temporal/Intl
conformance remain open. See the
[configured-zone contract](../docs/rust-rewrite/contracts/date-system-time-zone.md).

## Selected rounding-window authority — 2026-10-03 dry source

Contextual rounding now constructs provisional endpoints before selecting the
prescribed initial or recomputed window. Only final bracket/span validation
creates the private owner consumed by actual rounding and exact total
arithmetic. The shared day arm no longer applies the additional shift reserved
for year/month. Existing calendar, named-zone, inverse and Instant-check
authorities remain in use. The retained-field origin predicate follows the
immutable ECMA-262 integration algorithm linked in the contract.

Authored controls cover constrained month/year recomputation, retained
year/month/week/day fields, exact zone totals, endpoint range errors and signed
DST ties. The combined all-target Rust type check passed; execution remains
pending. Adjacent Apia
collapsed/unbracketed windows retain their T22 semantic gap and issue3310
ownership. See the
[selected-window contract](../docs/rust-rewrite/contracts/temporal-selected-rounding-window.md).

The 2026-10-05 primary-source audit found issue3310 still open and both its
[proposed algorithm change](https://github.com/tc39/proposal-temporal/pull/3318)
and [Test262 change](https://github.com/tc39/test262/pull/5044) still draft.
The current integration algorithms retain the assertions that the Apia case
violates; no accepted replacement behavior was found. Production and the six
owned Apia controls are unchanged. This audit closes no conformance debt and
runs no compilation or execution; the dated evidence is retained under
`.lila-task-work/recovery-20261005/t22-window`.

**Status:** In progress — complete named-zone consumer batch integrated; product verification, full APIs/data and conformance remain open

**Parallel group:** Feature lane; Date and Temporal can be separate sub-owners  
**Depends on:** T04, T05, T06, T10, T18, T20; locale formatting integrates with T23  
**Blocks:** Time-related T23/T26 closure

## Current repository state

The integrated complete consumer batch joins the pinned IANA provider's exact
offset, inverse-candidate and strict transition operations to real emitted
Temporal callers. Foundation proofs retain exact epochs, Identifier and
PrimaryIdentifier, calendar slots and completed options through ZonedDateTime,
relative Duration arithmetic, conversions and the shared Intl formatter.
All candidates undergo the prescribed Instant checks before policy selection;
GetStartOfDay remains distinct from compatible midnight resolution.

The integrated batch replaces the broad available-name lookup rejection. Its remaining
`RuntimeSemanticGap::TemporalZonedRoundingWindow` is T22 wire 8: a zero,
wrong-direction or finally unbracketed contextual nudge window after the
prescribed recomputation. It is an uncatchable compiler semantic rejection,
not a JavaScript exception, runtime crash or passing negative test. The legal
Apia controls are owned evidence for unresolved upstream issue 3310; normal
named-zone operations are not blocked by this diagnostic.

The named-zone predecessor admitted the five closed calendar identifiers
`iso8601`, `gregory`, `buddhist`, `roc` and `japanese`. The Indian source batch
above adds its own complete arithmetic domain; its verification is pending.
T23 owns pinned locale/data services; wider Intl calendar profiles do not
establish wider Temporal arithmetic.
The earlier named-zone-only checkpoint kept SystemTimeZoneIdentifier at UTC.
The newer configured default-zone source above now supplies actual Date, Now
and omitted-zone Intl consumers; its verification remains pending. The existing
HostClock remains the clock authority. Additional calendars and full
Date/Temporal/Intl conformance remain open.

Verification of the newer source is pending. At the earlier named-zone
checkpoint, its integrated revision passed all-target checking and 74 native
tests: 37 timezone-provider, 25
named-provider, six exact-domain and six Engine-host tests. Eight compiler
structure controls and the arithmetic string-pool regression pass. All nine
Temporal Engine targets pass their 50 tests in attempt9, including six
created-Realm controls and seven named conversions. The repaired catalogue
retains all original brand, descriptor and borrowed-accessor assertions for
PlainMonthDay/PlainYearMonth getter metadata.

The named-zone Instant string repair retains the resolved zone and projects
the rounded exact Instant before formatting. Historical offset seconds affect
civil time while the suffix uses the prescribed nearest-minute rounding.
Its option reader observes timeZone before unit validation and converts the
zone after validation, preserving RangeError precedence over a non-string
zone. All seven named conversion controls pass (14 sloppy/strict observations),
including DST boundaries, negative nanoseconds, Instant endpoints and
called-function Realm errors. Both previously failed pinned modes of
`intl402/Temporal/Instant/prototype/toString/timezone-offset.js` now pass.

Attempt9 passes its first 133 pinned physical files (266 modes), then fails on
`intl402/Temporal/ZonedDateTime/prototype/getTimeZoneTransition/transition-at-instant-boundaries.js`
because `Intl.supportedValuesOf` is missing before entering the transition
loop. Its 171 command receipts and failed terminal result remain retained.
The 2026-10-01 enumeration checkpoint passes the full transition-boundary loop
in both modes across all 447 listed primary zones. Its six paired Engine controls
also pass. Its complete API and consumer cohorts record 54/78 passes, with all
24 Runtime Bugs owned; five Chinese Temporal calendar-mismatch files account
for ten failed modes before the expected mismatch assertion. The complete T22 replay and checkpoint remain open. This historical
three-calendar result precedes the completed Buddhist checkpoint below.
Neither bounded result establishes an aggregate conformance milestone.

The completed 2026-10-01 Buddhist predecessor checkpoint records 518/558
real modes across 279 physical files, with all 40 Runtime Bugs retained and no
exclusions. The five exact Temporal calendar-mismatch files pass 10/10 modes,
and the complete transition-boundary loop passes 2/2 across 447 primary zones.
The whole DateTimeFormat cohort records 468/496 passes (248 physical files);
the whole enumeration cohort records 38/50 passes (25 physical files). The
separate smoke suite passes 191/191 executions from 190 physical files.
Calendar and locale coverage, Chinese Temporal construction, range formatting
and missing Intl consumers retain T22/T23 ownership. These results apply to the
Buddhist predecessor, before the joined PluralRules, ListFormat, Collator,
system-zone and Tolong Siki source. That composition requires its own product
verification; the complete T22 checkpoint and full conformance remain open. See the
[enumeration contract](../docs/rust-rewrite/intl-supported-values.md) and
[complete consumer contract](../docs/rust-rewrite/temporal-named-zone-consumer-batch.md).
The following checkpoints retain their original evidence and capability
boundaries.

A further authority repair removes the parser branch that trusted a numeric
string offset for any slash-containing annotation and replaced its named zone
with a fixed offset. The shared pinned IANA catalogue now decides name
availability before any Temporal object is constructed: unknown identifiers
throw an intrinsic `RangeError`; available named identifiers report the typed
out-of-band `RuntimeSemanticGap::TemporalNamedTimeZone`, owned by T22. This is
missing compiler inverse resolution and consumer wiring, not missing tzdata or
a missing Wasmtime capability. UTC and numeric zones retain their existing
semantics; parse goals that ignore an annotation retain that behavior.

The `aot_temporal_zone_authority` target passes seven tests and 30 fresh sloppy
and strict Wasm-AOT observations, with an additional Test262 runner control for
four catch/runtime-negative cases, also passing. All-target checking and the
full IR suite pass on 2026-09-30; broad verification and pinned replay are
pending. Neither the typed gap nor the deferred
named-zone spec fixture counts as conformance. See the
[named-zone authority contract](../docs/rust-rewrite/contracts/temporal-named-zone-authority.md)
for the wire domain, pinned controls and remaining T22 acceptance.

The ownership map assigns `intl402/Temporal` consumer cases to T22 by the
longest-prefix rule. Pure Intl service cases remain T23, and provider defects
remain shared T23 repair dependencies. Parser, lowering, harness and
dynamic-source failure precedence is unchanged; ownership is not inferred by
parsing a diagnostic string.

The active Duration batch now routes `round`, `total` and `compare` through
real relative-date arithmetic in the dedicated
`builtins/temporal_duration_relative.rs` codegen owner. It adds calendar-unit
windows, balancing and exact split nanosecond spans for plain relative dates
and the existing UTC/fixed-offset zoned values. Two follow-up corrections keep
that path coherent: calendar rounding reuses the origin only for an entirely
empty start duration, preserving larger year/month fields when the rounded
month/week count is zero; `relativeTo` strings with only key/value annotations
such as `[u-ca=iso8601]` take the plain-date parser rather than requiring a
time-zone annotation. The ISO parser still owns annotation validation.

The new `lila-engine` target `aot_temporal_duration_relative` exercises the
product Wasm-AOT path with four independent fixtures. They cover positive and
negative retained calendar fields, leap/end-of-month arithmetic, calendar-only
and critical annotations, plain-date-time slot conversion, fixed-offset
strings, exact nanosecond and wide totals, observable option order, branded
getter suppression, abrupt identity and both epoch bounds. On 2026-09-29
workspace/all-target checking passes, and the engine target passes all four
tests. Two initial fixture expectations were corrected against
[ISODateSurpasses](https://tc39.es/proposal-temporal/#sec-temporal-isodatesurpasses):
the raw February 31 candidate surpasses February 29, so the January 31 to
February 29 difference balances to zero months and 29 days. Broader integration
remains pending. All five exact rounding-window and relativeTo-string files
pass their ten sloppy/strict executions on 2026-09-30. The neighboring calendar
read-order, month-code and fixed-offset parsing controls pass all 12 executions.
No new Test262 aggregate or
T22 closure is claimed. The default time-zone provider, complete
calendar coverage and full pinned Date/Temporal trees remain open.

Refresh the focused Duration evidence with:

```sh
cargo test -p lila-engine --test aot_temporal_duration_relative -- --test-threads=2
```

The general Date parsing follow-up replaces the two epoch-only display string
branches with bounded runtime parsing of the existing UTC display formats. It
also adds reduced ISO date-time forms and validates end-of-day rollover before
offset adjustment and TimeClip. Twelve explicit Wasm-AOT engine regressions are
wired into CI; their results are recorded at the tested PR commit rather than
being counted as a new Test262 aggregate. See
[the parser contract and verification scope](../docs/rust-rewrite/aot-date-parsing.md).
This does not close T22 or add the missing default time-zone provider.

Date has a substantial dedicated backend implementation and focused complete
leaves. Temporal now has a dedicated builtin module with Instant and
ZonedDateTime work and focused snapshots. A realm-owned `HostClock` is the one
product boundary for JavaScript-visible wall-clock and monotonic reads:
`UtcEpochMilliseconds` validates the shared Date/Temporal range, monotonic
instants and durations are separate types, tests may inject a deterministic
clock, and the production `SystemHostClock` is shared by realm clones and agent
workers. `Date.now`, `Temporal.Now` clock reads, the Wasm monotonic-clock import
and the Test262 agent monotonic operation all consume that boundary without
changing their Wasm ABI. Engine compilation, timeout and sleep machinery still
uses real execution-control timers and is deliberately not virtualized.

Date's remaining current-time consumers now share that boundary too. A private,
exhaustive `DateTimeValueSource` distinguishes a branded receiver slot from the
realm host clock; `Date()`, zero-argument `new Date()` and `Date.now()` all take
the clock arm, while prototype string methods take the receiver arm. The Date
constructor's catalog entry carries the load-bearing wall-clock capability, and
an injected nonzero clock regression prevents the function and constructor from
agreeing only because both substituted the Unix epoch. This does not change the
UTC/fixed-offset limitation of local Date formatting. The private source domain
now derives no cloning or copying capability: five exact construction sites
move through two typed boundaries into the sole exhaustive consumer, and the
recursive bounded structure target pins the complete ten-mention ownership
census, both arm bodies and their instruction order. The complete lifecycle now
lives in the private `builtins/date/local_string.rs` child: its exhaustive
consumer, sole clock-import access and all five raw producers moved with the
domain, while the parent and dispatcher retain only semantic calls. The exact
four-line domain and 41-line consumer/current-time-wrapper selections retain
SHA-256
`71ba6e635d162f63abcd7a35eb6cf7e66ae2e53b02eb43d038ec79febc0d3492`
and
`4ff02aca4eca4f2bc447c380079b9c9a6e01182b072de4366364dd9947dfa6dc`;
their combined 45-line hash is
`4c570b074e898f3a4b9930d42b12cb0694ce3c64e3469266c78d4acf7c6afe61`.
The resulting 1,675-line parent and 339-line child have SHA-256
`afe18d7006f8d8ffde380e8d667c56837cdef6d19ebe0e845c9434b41e0609c0`
and
`ae6d62f5ac5586704695a77839582e3a2fe8dc3fdb0b95e50b3be5157f4ec435`;
the retained `Date.now()` wrapper and standard constructor call retain
SHA-256
`026e7208baa5402ac0bc15e098caf04c72b69f8cffb480d058c6759287666e63`
and
`c15bb6f7dbebf9fd006c56abfd49faf9b78e6bdc660dcc1efa81283d2bd03afa`.
The earlier Batch Y structure/runtime/workspace checkpoint remains green. At
the 2026-08-28 Batch Z checkpoint, `cargo xc` is green, the recursive structure
target passes `7/7`, both exact backend source-oracle tests pass `2/2`, and the
exact injected-clock engine and locale-string CLI witnesses each pass `1/1`.
This move is source-equivalent; emitted-Wasm golden verification remains
deferred.

Date local-string selection now uses a private, non-derived
`DateLocalStringFormat` domain. Its complete policy owner now lives in the
private `builtins/date/local_string.rs` child: the sole consumer exhaustively
projects the date and time halves together for `Date`, `Time` and
`DateAndTime`, and four exact producers cover the Date function call and the
three prototype string methods. The parent and dispatcher retain only their
existing semantic calls. The frozen five-line domain, ten-line `Date()`
producer and 270-line formatter selection retain SHA-256
`8189b9bba6c4e3c5dbb6f771fdbf23aa7a4f4e96d3898bec520380ac9e7d7916`,
`7736637485e2f8118b32ca75f7eb5e7ca5fc8cf12c826d729dfa6966d14d7cff`
and
`59c1fe7398cca3b5066118019ee541285520fe4a4305ddab4a5a932420ad64b2`;
their combined 285-line hash is
`a155dd53ada727aadd829f02894112843b4187f47c9316710ef932234992493a`.
The final method delimiter moves outside that semantic selection; the 271-line
physical formatter and complete 286-line physical move have SHA-256
`455adc77784d562e552aadb0ae299a73b532d77e5b9bfab4af5d97d359cd49ec`
and
`45898a09089b25568c753b7990b7fba581fa19222ab8d41810fab80466f8d069`.
The 292-line child has SHA-256
`a75df03ae28a2524c322fa66b028ca50d28d2dcda642334fd0ce0ad73dfce143`
and reduces the current parent from 2,010 to 1,722 lines. The recursive bounded
structure target pins zero parent policy names, the exact nine/five child
censuses, complete mapping and unchanged one/two/two/two dispatcher calls. The
unchanged Date-function call line and 25-line prototype dispatch slice retain
SHA-256
`45e55826e4ee110a73b8067f64c1555e01b2d418a74fbae785a7d2c73cd597d4`
and
`d3ea28e34b1c66dec63dbdb38d299a5e1d93f2477457bbfa2d0fe6e76be567ee`.
This is source-equivalent and expected to leave emitted Wasm byte-identical;
at the 2026-08-28 Batch Y checkpoint, the combined structure target passes
`7/7`, and the exact injected-clock engine and existing three-format CLI
witnesses each pass `1/1`. The shared `cargo xc`, formatting, diff, module-
boundary and task-plan checks are green. Emitted-Wasm goldens and broad
conformance suites remain deferred.

At this earlier checkpoint the configurable default-zone boundary was still
missing. The current authored configured-system-zone batch now connects the
injected immutable Realm setting to actual Date, Now and omitted-zone Intl
consumers, as recorded above; those changes passed the combined Rust type check
and remain unexecuted.
The complete Temporal class surface, broader calendar coverage and full pinned
Date/Temporal trees remain open.

The fourteen Date component-setter entries now select one private seven-case
`DateComponentSetterOperation` at the standard dispatcher boundary. Five
direct exhaustive projections own argument count, invalid-date initialization,
both invalid-date execution gates and the replaced component tuple. The shared
emitter no longer accepts the unrestricted builtin catalog, contains no
`is_full_year` Boolean and has no wildcard compiler panic. The focused
[Date component-setter operation contract](../docs/rust-rewrite/contracts/date-component-setter-operation.md)
pins all seven local/UTC producer pairs and the separate read-only builtin
length matrix. The structure target passes `4/4`, the exact existing CLI
fixture passes `1/1`, and the focused `setUTCMinutes` Test262 leaf passes both
ordinary executions `2/2`. This is a source-equivalent type closure; it does
not change the documented UTC/fixed-offset limitation or claim a
default-time-zone boundary. The shared 678-dump semantic golden passes `2/2` in
722.99 seconds; this closure adds no fixture, and all 674 retained dumps are
equal after accounting normalization.

Batch AU makes the raw setter family a private `DateComponentSetterOperation`
with no derived capabilities and exposes only seven fixed Date setter entries
to the fourteen local/UTC catalog IDs. The frozen 306-line domain/emitter
selection has SHA-256
`53813c73ebb92bdaa9541b57c83694c11c4f3dcc214c8cc27f056eb980d44240`;
restoring only the former derive and visibility reproduces that source exactly.
At the 2026-08-28 Batch AU checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, the exact setter CLI fixture passes `1/1`, and
the focused `setUTCMinutes` leaf passes both sloppy/strict Wasm-AOT executions
`2/2` with every failure bucket at zero. This source-equivalent boundary claims
no new Date behavior, local-time/default-time-zone support, broader conformance
or published conformance-count change.

Artifacts that use the existing Intl locale host operation now carry the full
canonical `IntlDataIdentity`, and the engine matches it to the shared provider
before Wasmtime compilation or instantiation. That closes the
artifact/provider identity seam shared with T23, but it does not add a
time-zone provider, transition data, or a default-zone consumer. In particular,
the carried identity's pinned tzdb field is metadata for the selected complete
data line; the current Locale-only external provider still does not claim that
capability.

Temporal time-string parsing now carries an exhaustive calendar-consumer
policy. `PlainTime` consumes the `Ignore` arm required by its grammar, while
`ToTemporalCalendarIdentifier` consumes `Resolve`, so a time string's
`[u-ca=...]` value is canonicalized exactly when it denotes a calendar. This
keeps `PlainTime.from("T11:30[u-ca=unknown]")` valid while making
`withCalendar("T11:30[u-ca=notacal]")` throw instead of silently defaulting to
`iso8601`. Complete calendar coverage and the full Temporal tree remain open.

The private validated-epoch proof used by the two implemented
`Temporal.Instant.fromEpoch*` builtins is now linear. `EpochNanoseconds` is
non-`Copy` and derives no incidental capabilities; the sole
`emit_alloc_validated_temporal_instant` consumer exhaustively destructures its
named unvalidated pair before allocation. A Rust-lexical guard pins the exact 5
source mentions, range-check-before-proof construction and both
validate-before-allocate builtin paths. This source-equivalent hardening is
recorded in
`docs/rust-rewrite/contracts/temporal-instant-epoch-proof.md`; no Temporal
behavior or conformance result changes. The focused structure target passes all
5 tests after compiling the backend; no broad Cargo or Test262 suite was run.

`Temporal.ZonedDateTime` calendar coercion now has a private closed policy
instead of a `parse_iso_strings` Boolean. The property-bag producer selects
`ToTemporalCalendarIdentifier`, the constructor selects
`CanonicalizeCalendar`, and the sole consumer projects those variants with an
exhaustive match. A focused three-test source contract pins the two-variant
domain, both producer mappings, the exact two-producer/one-consumer census and
the absence of a Boolean or wildcard fallback. Behavioral closure is still
open: the existing ZonedDateTime property-bag fixture currently throws before
it can observe an ISO-derived calendar, and the pinned
`built-ins/Temporal/ZonedDateTime/calendar-invalid-iso-string.js` constructor
leaf remains 0/2 because the thrown value has the wrong error constructor.
Those pre-existing semantic failures were not hidden behind a weakened fixture
or an expected-failure declaration. The focused source contract passes `3/3`,
`cargo xc` is green and the shared 645-artifact pre/post Wasm golden has an
empty recursive diff.

The shared `Temporal.PlainYearMonth` / `Temporal.PlainMonthDay` receiver check
now accepts only the existing closed `TemporalPartialDateType`. Exhaustive
projections bind each type to both its internal brand and its exact receiver
diagnostic, so the two wrappers can no longer pair one partial-date brand with
the other type's error. The bounded structure target passes `3/3`; `cargo xc`
is green, and the 647-artifact Wasm golden has an empty recursive pre/post diff.
The adjacent valid/invalid receiver Test262 leaves were not rerun in this typed
boundary checkpoint. No broader Temporal branding or conformance change is
claimed.

Date construction now has a closed, required realm-prototype fallback. The
realm-intrinsics record owns `%Date.prototype%`, entry and created realms both
publish that slot, and all zero-, one- and multiple-argument Date construction
select it through the same typed `GetFunctionRealm` policy when
`NewTarget.prototype` is primitive. Missing resolved-realm bootstrap state
traps rather than falling back to the entry global, while object-valued custom
prototypes and the existing branded Date allocation remain on the same
specification path. The prototype read occurs after zero-argument clock acquisition or argument
coercion, and object-valued Object, Function and Array prototypes retain their
payload tag through Date allocation. This targets the three exact
`built-ins/Date/proto-from-ctor-realm-{zero,one,two}.js`
failures from the 2026-08-13 current-pin Wasm-AOT baseline; focused current-SHA
execution remains deferred until that low-RAM matrix releases Cargo/Test262.
The invariant and deferred gates are recorded in
`docs/rust-rewrite/contracts/date-constructor-realm-prototype.md`.

ZonedDateTime differences now have a closed default-largest-unit plan. The
shared PlainDateTime settings reader distinguishes PlainDateTime `until`,
PlainDateTime `since` and ZonedDateTime delegation; the first two resolve an
unset or `"auto"` `largestUnit` from `day`, while the delegate resolves it from
`hour`. A non-copyable resolved-settings witness is consumed directly by the
PlainDateTime arithmetic or materialized as an unreachable normalized options
bag for the existing ZonedDateTime-to-PlainDateTime call. User getters and
conversion hooks are therefore observed once, while the arithmetic body stays
single-sourced. This targets the pinned ZonedDateTime
`defaults-to-returning-hours`, `largestunit-undefined` and
`largestunit-default` cases for both `until` and `since`. Current-SHA execution
remains deferred while the low-RAM matrix owns Cargo/Test262; the invariant and
deferred gates are recorded in
`docs/rust-rewrite/contracts/temporal-zoned-date-time-difference-default.md`.

Shared Temporal unit-option reads now accept one closed
`TemporalUnitOptionProperty` instead of an independent property-name string and
`allow_auto` Boolean. Exhaustive projections bind `largestUnit` to accepting
`"auto"` and bind `smallestUnit` and `unit` to rejecting it. All 16 current
producers use the named property variants, so arbitrary property names and
inconsistent auto policies are absent from the caller boundary. The bounded
`temporal_unit_option_property_domain_structure` target pins the projections,
typed reader and exact 5/10/1 producer distribution. The structure target
passes `3/3`, `cargo xc` is green, and the 647-artifact Wasm golden has an empty
recursive pre/post diff. After closing the namespace-rooting defect below, the
focused ZonedDateTime CLI witness passes `1/1`; the `largestUnit: "auto"`,
invalid `smallestUnit`, disallowed/invalid `total` unit and PlainYearMonth
option-order Test262 witnesses each pass `2/2`. No behavior change is claimed
by this source-equivalent type closure.

The shared PlainDate/PlainMonthDay property-bag field sweep now accepts one
closed `TemporalDateFieldReadMode` instead of positional `read_calendar` and
`strict_month_code` Booleans. Four variants name full PlainDate conversion,
PlainDate `with`, full PlainMonthDay conversion and PlainMonthDay `with`; two
direct exhaustive matches bind each mode to its calendar-read and month-code
validation policy without projecting either decision back to a Boolean. A
bounded source target pins the exact four producers and passes `3/3`. The
focused CLI witness uses Proxy field bags and invalid month codes to distinguish
calendar-read counts and whether syntax rejection occurs before the later
overflow-option read, and passes `1/1`. This is a source-equivalent type closure;
other Temporal option policies, field readers and complete calendar semantics
remain open. The shared semantic golden passes `2/2` in 707.34 seconds and adds
only this fixture to the preceding 663-dump checkpoint; no fixture is removed.

The PlainDateTime property-bag field sweep now accepts the private
`TemporalPlainDateTimeFieldReadMode::{Conversion, With}` domain instead of a
raw `read_calendar` Boolean. One exhaustive match makes conversion the sole
calendar Get/canonicalization path and makes `with` emit neither operation;
the exactly two producers name those modes directly. `with` also now performs
the required observable `Get` operations for `calendar` and `timeZone` before
the shared field sweep, rejecting any non-`undefined` result without reading
`calendar` a second time. A focused Proxy fixture pins both access orders and
the forbidden calendar getter. PlainYearMonth retains its separate reader; its
typed closure is recorded below. The checkpoint passes its bounded structure
target at `4/4` and its exact CLI witness at `1/1`; fixture syntax, formatting
and the diff check are also green. The pinned `from` order, `with` order and
forbidden-calendar leaves pass all `6/6` variants with every failure bucket at
zero; the `with` order leaf moved from `0/2` runtime bugs before the ordinary-
Get repair to `2/2`. It does not complete PlainDateTime, calendars, time zones
or T22.

The separate PlainYearMonth property-bag field sweep now accepts the private,
non-copyable `TemporalPlainYearMonthFieldReadMode::{Conversion, With}` domain
instead of its remaining raw `read_calendar` Boolean. One borrowed exhaustive
match keeps conversion's calendar Get/canonicalization before the shared field
sweep and makes `with` emit neither operation there; the exactly two producers
name those rows. `with` retains its earlier observable calendar/timeZone
rejection reads, so it cannot read calendar a second time through the shared
reader. The focused
[contract](../docs/rust-rewrite/contracts/temporal-plain-year-month-field-read-mode.md)
and bounded recursive-census guard record this source-equivalent closure.
The structure target passes `3/3`, the neighboring PlainDateTime guard passes
`4/4`, and the exact `from` and `with` order leaves pass both variants (`4/4`)
with every failure bucket at zero. `cargo xc` is green. No new fixture or
broader Temporal conformance claim is added.

The internal ZonedDateTime field-delivery answer is now the one-shot,
non-derived `ZdtFieldResult::{NumberOnStack, WrittenByCallee}` domain. Each
twelve-arm field dispatch creates one value and the immediately following
exhaustive match consumes it before local release, so a second by-value result
publication no longer compiles. The Rust-lexical guard pins the exact 15-use
ownership census, 10/4 qualified routes, two `delivery` identifiers, all twelve
complete field bodies and the final consumer. This is derive-only,
source-equivalent hardening; it adds no Temporal behavior or conformance claim.
See
[`temporal-zoned-date-time-field-result.md`](../docs/rust-rewrite/contracts/temporal-zoned-date-time-field-result.md).
The dedicated and neighboring structure targets pass `3/3` each, the exact
ZonedDateTime era/component CLI witness passes `1/1`, and formatting plus the
owned diff check are green.

The `Temporal.ZonedDateTime.from` property-bag path now reads and string-coerces
`disambiguation`, `offset` and `overflow` after the final `year` field
conversion but before calendar era resolution and the required `year` and `day`
checks. The required `timeZone` check and conversion now occur at its sorted
field position, before `year` and the option phase. The existing typed option
reader remains the sole owner
of that three-option order. Its five scratch locals are released before the
linear era witness reaches its consuming resolver. The focused structure guard
pins those call and lifetime boundaries. The existing ZonedDateTime era fixture
now records all nine option Get, toString-getter and toString-call events for
three later failures: an invalid gregory era, neither a year nor an
`era`/`eraYear` pair, and an absent day. Missing and invalid present time-zone
cases observe neither the later `year` getter nor any option getter. See the
[`property-bag option-order contract`](../docs/rust-rewrite/contracts/temporal-zoned-date-time-property-bag-option-order.md).
The structure target passes `3/3` and the existing ZonedDateTime era fixture
passes its focused Wasm-AOT CLI test `1/1`; fixture syntax, formatting and
`cargo check -p lila-aot-wasm` are green. This change does not reorder the
earlier property-bag field reads, add custom calendar or time-zone protocols,
broaden the string or branded-object paths, close T22, or change a published
conformance count.

`Temporal.PlainDateTime.prototype.toPlainDate` and `.toPlainTime` now select the
non-copyable `TemporalPlainDateTimeComponent::{PlainDate, PlainTime}` domain
instead of a raw `time` Boolean. Receiver field extraction remains shared and
precedes one exhaustive match: the PlainDate arm alone owns prototype loading,
calendar transfer and date allocation, while the PlainTime arm alone projects
time locals and allocates the time result. The existing pinned basic leaves
pass both variants (`4/4`) with every failure bucket at zero; this
source-equivalent invariant checkpoint adds no fixture. Its bounded structure
target passes all `3/3` tests, and `cargo xc`, workspace formatting and the diff
check are green. The following shared 684-dump semantic golden passes `2/2` in
681.86 seconds, adds only the field-read witness and removes none. All 683
retained non-accounting summaries are equal; 51 retained dumps differ only in
compiler accounting, each with 294 fewer emitted code bytes. It does not
complete PlainDateTime or T22.

The IR Temporal shape and Wasm bootstrap now consume the same two ordered
member lists: all eight advertised constructors and the three advertised
`Temporal.Now` functions. Planning a bare `Temporal` reference roots both
levels, then publishes a private typed namespace witness; bootstrap cannot be
called with an incomplete member set and no longer silently skips unrooted
properties. The previously ignored bare-namespace regression is active, and a
reflective engine witness checks every advertised property through a
`var namespace = Temporal` alias. Rust formatting and bounded source checks are
green; the structure target passes `3/3`, and the active planner regression,
reflective engine witness and previously blocked ZonedDateTime CLI fixture each
pass `1/1`. `cargo xc` is green. The 647-artifact semantic golden changes 22
fixture dumps plus the manifest summaries; every changed fixture already
carried a Temporal root before this repair, and no other dump changed. The four
focused Test262 files pass all `8/8` variants. This repairs namespace
materialization for the existing shape but does not claim the still missing
complete Temporal API or time-zone/calendar semantics.

Temporal calendar canonicalization now accepts one closed context instead of
independent TypeError and RangeError strings. Exhaustive projections bind the
shared PlainDate-family path and the ZonedDateTime constructor path to their
existing diagnostic pairs, so cross-paired or arbitrary messages no longer
compile at the canonicalization boundary. The bounded structure target pins
the two variants, both projections, the typed helper and the exact two-producer
census and passes `3/3`; the existing ZonedDateTime-era CLI witness passes
`1/1`. `cargo xc` and every repository policy gate pass in the shared
checkpoint. Its golden changes are confined to the separately owned
mixed-BigInt equality repair and fixture; this source-equivalent type closure
adds no Temporal artifact delta. Broader Temporal conformance is not claimed.

`Temporal.Duration.prototype.negated` and `.abs` now reach their shared field
transform through argument-free named emitters and a private closed
`TemporalDurationFieldTransform`. The shared emitter exhaustively distinguishes
negation from absolute value, so the standard dispatcher can no longer compile
with an opaque or transposed Boolean. The bounded structure target passes
`3/3`. The pinned `negated/basic.js`, `abs/basic.js` and `abs/new-object.js`
Wasm-AOT leaves pass all `6/6` strict/non-strict variants with run artifacts
confined to `target/`. `cargo xc` and every repository policy gate pass in the
shared checkpoint. Its golden changes are confined to the separately owned
mixed-BigInt equality repair and fixture; this typed dispatch adds no Temporal
artifact delta. No broad Temporal or snapshot refresh is claimed.

`Temporal.Duration.prototype.add` and `.subtract` now use argument-free named
emitters over a private closed `TemporalDurationArithmeticOperation`. The
shared emitter coerces the right-hand duration first, then exhaustively chooses
whether to negate its fields before the common normalized addition, preserving
observable coercion and validation order while removing the transposable
Boolean from standard dispatch. The bounded structure target passes `3/3`.
The pinned `add/basic.js` and `subtract/basic.js` Wasm-AOT leaves pass all `4/4`
strict/non-strict variants with run artifacts confined to `target/`. `cargo
xc` and every repository policy gate pass. The 648-artifact golden changes are
confined to the separately owned mixed-BigInt equality repair and fixture;
this typed dispatch adds no Temporal artifact delta. No broad Temporal or
snapshot refresh is claimed.

The four plain Temporal receiver families now share one closed
`TemporalPlainArithmeticOperation` at their `add` / `subtract` boundary.
PlainDate, PlainYearMonth, PlainTime and PlainDateTime each consume the domain
with a direct exhaustive match before their common arithmetic, while all eight
standard-builtin producers must name `Add` or `Subtract` instead of supplying a
transposable Boolean. A bounded source target owns the two variants, four
consumers and exact four-plus-four producer census. The focused CLI witness
moves each receiver forward and backward by known fields, so every named
producer has an observable result independent of formatting. The source target
passes `3/3`, and the exact CLI witness passes `1/1`. Rust formatting, the
module-boundary policy, `cargo xc` and the diff check are green. The following
669-dump semantic golden passes `2/2` in 771.49 seconds, adds only this witness,
removes none and leaves 667 of 668 retained dumps equal after accounting
normalization; the sole retained structural change is the independent Promise
callback Realm witness. No Test262 tree was run. This is a source-equivalent
type closure; broader Temporal arithmetic, calendar and time-zone conformance
remain open. The boundary and verification commands are recorded in
`docs/rust-rewrite/contracts/temporal-plain-arithmetic-operation.md`.

The same four plain Temporal receiver families now share one closed
`TemporalPlainDifferenceOperation` at their `until` / `since` boundary. All
eight standard-builtin producers name `Until` or `Since`; every emitter matches
that operation exhaustively once for rounding-mode ownership and once for final
duration sign. The existing ZonedDateTime delegate remains a separate settings
plan so its normalized rounding mode reaches the selected PlainDateTime
builtin unnegated. A bounded source target owns the two variants, four
consumers, eight exhaustive decisions and exact four-plus-four producer census.
The focused CLI witness uses asymmetric ceiling vectors for all eight producers
so a transposition changes both magnitude and sign. This is a source-equivalent
type closure. The new structure target passes `3/3`, the preserved ZonedDateTime
and arithmetic targets pass `5/5` and `3/3`, and the exact CLI witness passes
`1/1`; module boundaries, scoped formatting and the scoped diff check are green.
The following shared 671-dump semantic golden passes `2/2` in 697.36 seconds,
adds this witness and the independent `Array.fromAsync` Promise-Realm witness,
removes none and leaves all 669 retained dumps equal after accounting
normalization. No Test262 tree was run. Broader difference arithmetic, calendar
and time-zone conformance remain open. The boundary and verification commands
are recorded in
`docs/rust-rewrite/contracts/temporal-plain-difference-operation.md`.

The five plain `ToTemporal*` converters now accept one data-bearing closed
`TemporalConversionOverflowOptions` instead of an options payload local, tag
local and `read_options` Boolean. Each public `from` producer constructs
`Read { payload_local, tag_local }`; the 15 internal compare, equality,
difference and plain-composition producers construct `Omit` and no longer
allocate dummy undefined locals. Sixteen direct exhaustive matches preserve
the existing overflow read point across the five conversion paths. The bounded
source target owns the exact variants, five consumers, five-plus-15 producers
and absence of raw read controls or dummy lifecycles. The focused CLI witness
executes all 20 producers, observes exactly one overflow getter read from each
`from`, and attaches throwing overflow getters to every internally converted
branded argument. The structure target passes `3/3`, the exact CLI witness
passes `1/1`, and workspace Rust formatting and the scoped diff check are
green. This is a source-equivalent type closure; its boundary is recorded in
`docs/rust-rewrite/contracts/temporal-conversion-overflow-options.md`. The
shared 674-dump semantic golden passes `2/2` in 717.58 seconds, adds this
witness plus the independent Promise combinator Realm and GroupBy result-kind
witnesses, removes none and leaves all 671 retained dumps equal after
accounting normalization. Broad Date/Temporal and Test262 trees remain
deferred.

PlainTime's existing closed `TemporalTimeUnit` domain is now the sole core
field authority for declaration index, record offset, valid maximum,
nanosecond scale and prototype accessor selection. Allocation, loading,
rejection, constraint and scalar conversion select locals through the same
unit authority instead of pairing independent positional arrays. The six
standard-builtin accessor producers now pass a named unit into a restricted
emitter, removing its catch-all over the full builtin catalog. Adding a
wall-clock unit therefore requires every PlainTime core policy to be selected
before the compiler builds.
The focused ownership law is recorded in
[`temporal-plain-time-field-authority.md`](../docs/rust-rewrite/contracts/temporal-plain-time-field-authority.md).
The separate alphabetical table still owns observable property-bag read order.
This does not change emitted Wasm or complete PlainTime or T22.

The four ZonedDateTime arithmetic/difference catalog cases now cross fixed
`add`, `subtract`, `until` and `since` family entries. The raw emitters and the
private, non-derived `ZonedDateTimeArithmetic::{Add, Subtract}` and
`ZonedDateTimeDifference::{Until, Since}` domains no longer escape through the
builtin module or shared dispatcher. The exact former 36-line policy selection
retains reconstructed SHA-256
`82f3f206759543894d9ec36a278938c4a17e3f0db2602df13f9c9e7c1f1756a0`;
the visibility-normalized 122-line arithmetic and 217-line difference emitters
retain SHA-256
`0df4c7b1b768c8520b30f505c8d5c5f6e18d1a8dbee0dff7b08149f2aa3bbde2`
and
`8c95229bd602e45445a7c6ad5e2a89b3d120b903be74b73ac185782859d73cdf`.
The focused
[`direction-dispatch contract`](../docs/rust-rewrite/contracts/temporal-zoned-date-time-direction-dispatch.md)
target passes `3/3`, four neighboring structure targets pass `15/15`, and the
exact arithmetic/era and difference-default CLI controls each pass `1/1`.
This is source-equivalent hardening with no new Temporal behavior, no closure
of the documented DST or ordering gaps, and no T22 closure.

The five branded types with a `[[Calendar]]` slot now remain under one
owner-private `TemporalCalendarCarrier`. Its complete list and exhaustive brand
and record-offset projections feed a private raw fast path with exactly the two
existing semantic callers. The focused
[`calendar-carrier privacy contract`](../docs/rust-rewrite/contracts/temporal-calendar-carrier-privacy.md)
records the exact original SHA-256 witnesses
`1726881c45223f008814169edef8a3066c23b8733d86714d63570535ba3dd831`
and
`a74006922ea5018cd1d001421de4f83b70c23db9b73924ab24627415c642765c`.
The focused structure target passes `3/3`; the exact five-carrier
getter-suppression leaf passes both Wasm-AOT executions with every failure
bucket at zero, and `cargo xc` is green.
This source-equivalent hardening has no new Temporal behavior or conformance
claim.

The owner-private `TemporalParsedMonthDayYear` and raw parser now stay inside
the PlainMonthDay string path in `temporal_plain_month_day.rs`. The only parser
call still precedes the overflow option read, and the private reference-year
step remains the only consuming projection. The focused
[`parsed-year privacy contract`](../docs/rust-rewrite/contracts/temporal-plain-month-day-parsed-year-privacy.md)
records the exact original SHA-256 witnesses
`edd8d04d5cf6ec69edd44225d78506a09d49e857a028ad52071a39d78417a4be`
and
`a6f4eeae8728f7f922afac564ea96b845164c0115682f0821fabdb76d0cac6ff`.
The focused structure target passes `3/3`; the exact valid/invalid string plus
reference-year leaf passes both Wasm-AOT executions with every failure bucket
at zero, and `cargo xc` is green.
This source-equivalent hardening has no new Temporal behavior or conformance
claim.

The active Duration and PlainDateTime declaration-order offset tables now stay
inside their respective codegen owners. The owner-private `TEMPORAL_DURATION_FIELD_OFFSETS`
retains two allocation/load consumers. The owner-private `TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS`
does the same without coupling active local order to the separate passive T05
layout metadata. The focused
[`field-offset table privacy contract`](../docs/rust-rewrite/contracts/temporal-field-offset-table-privacy.md)
records the exact original SHA-256 witnesses
`b47f9d79e4e1dc65b91a4ac7a2663a20b54cb5b6aea099266b381e6380e06ab1`
and
`f7047424c3fe0d3837f3d5db310d41d2c7a61740badcb97ec606c89c65746123`.
The focused structure target passes `3/3`; the exact Duration ten-field plus
PlainDateTime nine-field constructor leaves pass all four Wasm-AOT executions
with every failure bucket at zero, and `cargo xc` is green.
This source-equivalent hardening has no new Temporal behavior or conformance
claim.

The owner-private `TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE`
now stays with its sole `fromEpochMilliseconds` RangeError consumer, while the
owner-private `TEMPORAL_INSTANT_VALUE_OF_MESSAGE` stays with its sole `valueOf`
TypeError consumer. The focused
[`Temporal.Instant diagnostic privacy contract`](../docs/rust-rewrite/contracts/temporal-instant-diagnostic-privacy.md)
records the exact original SHA-256 witnesses
`783be630ab0b186ca6e47d703313d37314540e71454c1d1ec5f994b93f4a249d`
and
`e50fae0dab7f68f5d12df521f40cea34c2d47ddf0e71078e02871ef30b754b11`.
The focused structure target passes `3/3`; the exact non-integral plus
implicit-conversion leaves pass all four Wasm-AOT executions with every failure
bucket at zero, and `cargo xc` is green.
This source-equivalent hardening has no new Temporal behavior or conformance
claim.

The sole T22-owned exact Test262 materializer is gone.
`built-ins/Date/prototype/setUTCMonth/arg-coercion-order.js` now materializes its
unchanged pinned source with exactly the merged `assert.js` and vendored
`compareArray.js` preludes. A focused provenance test pins those origins and
the complete concatenated bytes. Both raw sloppy/strict Wasm-AOT executions
pass `2/2` with every failure bucket at zero. The shortcut inventory now
contains 404 entries, including 256 semantic shortcuts, and no entry has T22
removal ownership. This satisfies the materialization-removal acceptance item
below; the full Date and Temporal trees, default time-zone boundary and
remaining API work still keep T22 in progress.

## Objective

Implement exact Date semantics and the complete Temporal API for the pinned revisions using deterministic clock, calendar and time-zone interfaces. Reuse vendored `temporal_rs` where appropriate, but preserve JavaScript-observable coercion, property access, branding, realm and descriptor behavior in Lila's own runtime/compiler layers.

## Host time contract

Define typed host services for:

- current UTC epoch time used by `Date.now` and Temporal's clock-facing operations;
- default time-zone identifier and offset-transition queries;
- pinned time-zone database/version metadata;
- monotonic time for Test262 agent APIs, kept distinct from wall-clock time.

Tests must inject deterministic clocks/zones. Production defaults may use the host, but behavior must not depend on process locale or undocumented OS parsing.

## Date

Implement all Date constructor/function and prototype semantics:

- call vs construct behavior, zero/one/multiple arguments and custom new target;
- `TimeClip`, MakeTime/MakeDay/MakeDate, year 0-99 adjustment and invalid dates;
- ISO date-time string parsing required by ECMAScript, legacy implementation-defined forms only where explicitly supported/documented;
- local/UTC getters and setters, overflow normalization and argument coercion order;
- DST gaps/folds and historical offset handling through the pinned zone provider;
- `Date.parse`, `Date.UTC`, `Date.now`;
- `toISOString`, `toJSON`, `toString`, `toUTCString`, date/time/localized variants, `valueOf`, `getTime` and `@@toPrimitive`;
- exact descriptors, branding, realm errors and subclass/custom-prototype behavior.

Do not use Rust/OS date parsers for ECMAScript ISO parsing unless wrapped by exhaustive compatibility tests.

## Temporal integration architecture

Compile JavaScript-observable Temporal algorithms to Wasm. Vendored Rust calendar/time-zone code may supply deterministic data and pure algorithms; the emitted adapter must own:

- ordered option/property access and conversion through T04;
- Lila object branding/internal slots and prototype dispatch;
- realm-specific constructors/prototypes/errors;
- conversion to/from ECMAScript strings, Numbers, BigInts and objects;
- current Stage4 calendar/time-zone identifier conversion, available-name validation and branded Temporal slot recovery where the algorithm permits it;
- ordinary property-bag and option reads, Proxy/getter effects, coercions and arbitrary abrupt completions;
- iterable/record construction and property descriptors.

Do not expose Rust library structs directly as JavaScript objects or let the library bypass proxies/getters. Current Stage4 uses supported calendar/time-zone identifiers; it does not require the retired user-defined Calendar/TimeZone method protocols. Branded carriers and ordinary bags still retain their specified conversion, branding and Realm behavior.

## Temporal API scope

Implement every class/function in the pinned suite, including as applicable:

- `Temporal.Instant`;
- `PlainDate`, `PlainTime`, `PlainDateTime`, `PlainYearMonth`, `PlainMonthDay`;
- `ZonedDateTime`;
- `Duration`;
- `Now` operations;
- known supported calendars, available IANA identifiers, numeric fixed-offset identifiers and distinct observable/primary zone identity under the current Stage4 algorithms;
- parsing/formatting, arithmetic, comparison, rounding, balancing, total, since/until and field preparation;
- offset options, disambiguation, overflow, smallest/largest unit, rounding increment/mode and relative-to behavior;
- branded Temporal slot carriers wherever admitted, ordinary bags, Proxy/getter observations and exact abrupt-completion propagation.

Use generated tables/enums for option names and units to prevent inconsistent validation among methods.

## Correctness and data requirements

- Pin calendar, Unicode and IANA time-zone data versions in reproducible build metadata.
- Add vectors around leap years, negative epoch values, extreme valid ranges, nanosecond boundaries, DST transitions and calendar eras.
- Preserve arbitrary-precision nanosecond arithmetic where required; do not pass through f64.
- Avoid host-dependent formatting except through the explicitly pinned Intl layer.
- Distinguish invalid input errors and range boundaries exactly.

## Acceptance criteria

- Full pinned `built-ins/Date`, `built-ins/Temporal` and T22-owned `intl402/Temporal` consumer trees are green through Wasm-AOT.
- Runs are deterministic under an injected clock/time zone and reproducible across supported hosts.
- Date parsing, setters and formatting pass DST/extreme-range/coercion-order tests.
- Temporal option/property access order works with proxies/getters and abrupt completions.
- All Temporal classes enforce branding, descriptors, subclassing and cross-realm error behavior.
- Time-zone data version is pinned and surfaced in developer diagnostics; every named consumer retains zone identity through exact projection and inverse selection.
- Every conformance failure, including unresolved contextual rounding windows, retains an explicit owner and reason; a semantic gap never satisfies a negative JavaScript test.
- No exact Test262 date/time materialization remains.

## Required tests

Complete the coordinated compile, focused native controls and broad checkpoint
from the batch workflow before the canonical release build and pinned refreshes.
Keep that source/binary identity fixed through publication.

```sh
cargo test -p lila-runtime time_ --quiet
cargo test -p lila-aot-wasm date_ --quiet
cargo test -p lila-aot-wasm temporal_ --quiet
cargo test -p lila-cli wasm_date --quiet
cargo build --release --locked -j 2 -p lila-cli
./target/release/lila --jobs 1 test262 run built-ins/Date --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 2 --snapshot-dir target/test262-scratch/t22 --snapshot-name t22-date
./target/release/lila --jobs 1 test262 run built-ins/Temporal --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 240000 --threads 2 --snapshot-dir target/test262-scratch/t22 --snapshot-name t22-temporal
./target/release/lila --jobs 1 test262 run intl402/Temporal --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 240000 --threads 2 --snapshot-dir target/test262-scratch/t22 --snapshot-name t22-intl-consumers
```

After complete integration, follow the [batch verification ladder](../docs/rust-rewrite/batch-workflow.md), the focused consumer commands and guarded publication command in the [batch contract](../docs/rust-rewrite/temporal-named-zone-consumer-batch.md). These refresh commands are not verification results. Use deterministic injected clocks and explicit UTC, numeric and IANA zone arguments, including DST gaps/folds. The authored configurable default-zone service requires its own combined verification. Spec-exec/reference comparisons are diagnostics only; Wasm-AOT executions provide product evidence.

### Created-Realm Temporal completion follow-up

The named conversion and locale controls exposed a concrete created-Realm omission: only Instant and Duration were published. The integrated completion shares all eight implemented family member lists, publishes the rooted Now namespace, and connects both NewTarget fallback and returned values to immutable realm prototypes. Its six live GC slots extend the realm record to 568 bytes. All-target checking and both compiler structure controls pass. All six paired `aot_temporal_created_realm` controls now pass in attempt8, including the repaired getter metadata. The Instant string failure was repaired in attempt9; the later missing enumeration keeps full verification open; see the [source contract](../docs/rust-rewrite/contracts/temporal-created-realm-completion.md). The prior Instant-only receipt remains historical.

The integrated created-Realm constructor follow-up preserves object-valued NewTarget prototype tags through one opaque checked pair for all eight families. Every actual Temporal allocation caller selects the closed constructor/intrinsic source. All-target checking and the paired Function/Array/Proxy prototype controls pass in attempt8. The Instant string repair now passes its Engine and focused pinned controls in attempt9; full conformance remains open.

## Buddhist formatter assertion follow-up (2026-10-01)

The completed broad workspace command records an actual failure in the
seven-test Buddhist target: six pass, and the field-error/read-order test
finishes with false because its final Intl assertion still expects Gregorian
after explicitly requesting the now-admitted Buddhist formatter. The isolated
successor changes only that final expected identifier to `buddhist`; all
preceding error/read-order controls and sibling tests are preserved. Fresh
target execution and final broad verification remain pending. This is not
a wider Temporal calendar admission or a conformance milestone.


## Duration localized formatting consumer — 2026-10-01

Temporal.Duration.prototype.toLocaleString now has a dedicated compiled
consumer with Intl host and synchronous user-code effects. It reads the
receiver's branded ten stored fields, initializes a private DurationFormat
record through the same locale/option algorithm as the constructor and
assembles the native partition as a string. Planning roots the real
DurationFormat initializer dependencies. ISO toString and toJSON retain
their existing path; localized formatting no longer dispatches to that ISO
renderer. Source is complete and the coordinated Duration compilation,
Engine controls and exact pinned locale-method cases remain unexecuted.
T22's full Temporal conformance gate remains open.
