# T22 — Date and Temporal

**Status:** In progress — Date and initial Temporal adapters exist; complete APIs/data remain open

**Parallel group:** Feature lane; Date and Temporal can be separate sub-owners  
**Depends on:** T04, T05, T06, T10, T18, T20; locale formatting integrates with T23  
**Blocks:** Time-related T23/T26 closure

## Current repository state

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

The default time-zone boundary is not implemented yet. Current AOT Date local
operations and `Temporal.Now` defaults remain UTC/fixed-offset behavior.
`lila-intl` owns the closed `CanonicalTimeZoneId` domain, but no canonical zone
provider or Wasm host ABI connects it to Date/Temporal yet; a realm setting with
no semantic consumer would not satisfy this task. The complete Temporal class
surface, custom calendar/time-zone protocols, pinned deterministic data, Intl
integration and materialization-free full Date/Temporal trees therefore remain
open.

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

Create a JavaScript-facing adapter around `temporal_rs` or the selected semantic kernel. The adapter must own:

- ordered option/property access and conversion through T04;
- Lila object branding/internal slots and prototype dispatch;
- realm-specific constructors/prototypes/errors;
- conversion to/from ECMAScript strings, Numbers, BigInts and objects;
- calendar/time-zone protocol calls, including user-defined objects and abrupt completion;
- iterable/record construction and property descriptors.

Do not expose Rust library structs directly as JavaScript objects or let the library bypass proxies/getters.

## Temporal API scope

Implement every class/function in the pinned suite, including as applicable:

- `Temporal.Instant`;
- `PlainDate`, `PlainTime`, `PlainDateTime`, `PlainYearMonth`, `PlainMonthDay`;
- `ZonedDateTime`;
- `Duration`;
- `Now` operations;
- calendar and time-zone handling required by the current proposal/revision;
- parsing/formatting, arithmetic, comparison, rounding, balancing, total, since/until and field preparation;
- offset options, disambiguation, overflow, smallest/largest unit, rounding increment/mode and relative-to behavior;
- protocol interactions with custom calendars/time zones if present in the pin.

Use generated tables/enums for option names and units to prevent inconsistent validation among methods.

## Correctness and data requirements

- Pin calendar, Unicode and IANA time-zone data versions in reproducible build metadata.
- Add vectors around leap years, negative epoch values, extreme valid ranges, nanosecond boundaries, DST transitions and calendar eras.
- Preserve arbitrary-precision nanosecond arithmetic where required; do not pass through f64.
- Avoid host-dependent formatting except through the explicitly pinned Intl layer.
- Distinguish invalid input errors and range boundaries exactly.

## Acceptance criteria

- Full pinned `built-ins/Date` and Temporal trees are green.
- Runs are deterministic under an injected clock/time zone and reproducible across supported hosts.
- Date parsing, setters and formatting pass DST/extreme-range/coercion-order tests.
- Temporal option/property access order works with proxies/getters and abrupt completions.
- All Temporal classes enforce branding, descriptors, subclassing and cross-realm error behavior.
- Time-zone data version is pinned and surfaced in developer diagnostics.
- No exact Test262 date/time materialization remains.

## Required tests

```sh
cargo test -p lila-runtime time_ --quiet
cargo test -p lila-aot-wasm date_ --quiet
cargo test -p lila-aot-wasm temporal_ --quiet
cargo test -p lila-cli wasm_date --quiet
./target/debug/lila test262 run built-ins/Date --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/Temporal --execution-backend wasm --timeout-ms 240000 --threads 4
```

Run with several injected zones—including UTC and zones with DST gaps/folds—and compare deterministic outputs against the spec-exec differential oracle (diagnostic comparison only; the Wasm-AOT results are the product evidence).
