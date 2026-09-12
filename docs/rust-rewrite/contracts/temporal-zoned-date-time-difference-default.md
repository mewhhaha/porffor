# ZonedDateTime difference default largest unit

Status: current direct-arithmetic architecture; native and broad checkpoint
verified on 2026-09-12.

`Temporal.ZonedDateTime.prototype.until` and `since` use
`DifferenceTemporalZonedDateTime`. Its `GetDifferenceSettings` call has
`"hour"` as the fallback largest unit and `"nanosecond"` as the fallback
smallest unit. `Temporal.PlainDateTime` uses the same unit domain and rounding
rules, but its fallback largest unit is `"day"`.

The pinned specification behavior is mirrored by the vendored
`temporal_rs` kernels:

- `ZonedDateTime::diff_internal_with_provider` resolves settings with
  `Unit::Hour` and `Unit::Nanosecond`; and
- `PlainDateTime::diff` resolves them with `Unit::Day` and
  `Unit::Nanosecond`.

The pinned Test262 cases make the distinction observable:

- `built-ins/Temporal/ZonedDateTime/prototype/{until,since}/defaults-to-returning-hours.js`;
- `built-ins/Temporal/ZonedDateTime/prototype/{until,since}/largestunit-undefined.js`; and
- `built-ins/Temporal/ZonedDateTime/prototype/{until,since}/largestunit-default.js`.

An omitted, `undefined` or `"auto"` `largestUnit` therefore resolves to the
larger of `hour` and the resolved `smallestUnit`. A one-year fixed-offset
difference is expressed as 8784 hours, not 366 days. If `smallestUnit` is a
date unit, that larger unit remains the default.

## Closed settings plan and direct arithmetic

One shared settings producer owns the four observable reads in order:

1. `largestUnit`;
2. `roundingIncrement`;
3. `roundingMode`; and
4. `smallestUnit`.

`TemporalDateTimeDifferenceSettingsPlan` has four exhaustive states:

| Plan | Fallback largest unit | Rounding mode |
| --- | --- | --- |
| `PlainUntil` | `day` | Requested mode |
| `PlainSince` | `day` | Negated mode |
| `ZonedUntil` | `hour` | Requested mode |
| `ZonedSince` | `hour` | Negated mode |

The producer returns the builtin-scoped, `#[must_use]`, non-`Copy`
`ResolvedTemporalDateTimeDifferenceSettings` witness containing all four
resolved locals. The entry emitter owns their lifetime and lends the witness
directly to arithmetic. The settings reader negates a `since` rounding mode
once; arithmetic negates the final duration once. There is no serializer,
internal options object, second option read, or PlainDateTime difference
builtin call in the ZonedDateTime path.

`emit_temporal_difference_date_time` in `builtins/temporal_difference.rs` owns
the shared field arithmetic, calendar rounding and expansion into larger
units. PlainDateTime enters it with `TemporalDifferenceContext::Plain`.
ZonedDateTime date-unit differences enter it with
`TemporalDifferenceContext::Zoned { offset_seconds_local }`. The closed
context selects the range authority for calendar candidates: the ISO date
bounds used by `CalendarDateAdd` on the Plain path, or the corresponding
instant after applying the fixed offset on the Zoned path. The Plain path
converts a candidate directly to epoch nanoseconds; it does not impose the
additional lower-midnight restriction of constructing a PlainDateTime.
Operation direction remains the separate
`TemporalPlainDifferenceOperation::{Until, Since}` domain.

ZonedDateTime still obtains trusted local date-time fields through the existing
conversion path for date-unit arithmetic. Sharing these converted fields does
not delegate option interpretation or rounding to a public PlainDateTime
method. Whole seconds and subseconds remain separate in time-duration
arithmetic; the complete difference is not multiplied into an overflowing i64
nanosecond total.

## Observable order and time-zone boundary

The ZonedDateTime entry brands the receiver, converts `other`, and checks
calendar equality before reading options. After resolving settings, a largest
unit of `day` or larger requires equal time-zone identifiers. For a largest
unit smaller than `day`, the emitter subtracts exact epoch seconds/subseconds
directly and may compare operands with different supported zones. It rounds
that pair with `emit_temporal_round_difference_time` and balances into the
resolved largest unit. These branch and ordering requirements follow
[DifferenceTemporalZonedDateTime](https://tc39.es/proposal-temporal/#sec-temporal-differencetemporalzoneddatetime).

The current local-calendar implementation supports UTC and fixed numeric
offsets. The context carries that known offset into candidate validation; it
does not introduce named-zone transition or DST arithmetic.

When time-unit rounding retains calendar days, the Plain path includes those
days in the quantity being rounded. The Zoned path rounds the time remainder
and validates both adjacent day endpoints before selecting a result. Even
fixed 24-hour days can therefore produce different half-even results: 28 hours
rounded to an eight-hour increment yields `P1DT8H` for PlainDateTime and `P1D`
for a UTC ZonedDateTime. The nanosecond/increment-one shortcut returns the
unrounded difference before that relative rounding step.

## Observable regression

The durable Wasm fixture checks both `until` and `since` with omitted options,
an empty object, explicit `undefined`, `"auto"` and explicit `"hours"`. The
25-hour vector retains its minute and subsecond tail while reporting zero days.
A `smallestUnit: "day"` vector proves that the default is the larger of hour
and the smallest unit rather than an unconditional hour.

Side-effecting option values record the complete read-and-conversion order and
prove that every user property is consumed once. This prevents a later
shortcut from restoring the correct numeric answer by double-reading the
original options bag.

A focused source-structure test must pin the four-state plan, exhaustive
fallback and rounding ownership, the non-copyable witness, direct consumers,
closed range context, and absence of the retired options transport. Runtime
controls must also exercise wide exact differences, `since` sign ownership,
calendar-unit rounding across boundaries, and option reads before the
date-unit time-zone guard.

## Historical composition and evidence

The original implementation called the compiled PlainDateTime `until` or
`since` body with the user's options. That preserved property access but
incorrectly selected PlainDateTime's `day` fallback.

The 2026-08-13 repair introduced three settings plans (`PlainUntil`,
`PlainSince`, `ZonedDelegate`). ZonedDateTime resolved user options once, then
materialized explicit primitive settings in an internal null-prototype object
for the selected PlainDateTime builtin to read. `ZonedDelegate` kept the mode
unnegated because that delegate owned the direction. This repaired the hour
default without repeating user effects. The 2026-09-10 implementation removes
that transport and moves arithmetic behind the direct shared boundary above.

The completed 2026-08-13 current-pin Wasm-AOT Date-family snapshots were
produced by an older binary. They are ownership evidence only: the aggregate
Date leaf passed 75 of 78 cases with exactly the three now-landed constructor
realm-prototype failures, while `Date/UTC`, `Date/now`, `Date/parse` and
`Date/prototype` passed 17/17, 6/6, 8/8 and 485/485 respectively. At that point,
no completed current-pin Temporal Wasm leaf established the seam's runtime
result.

Those older snapshots and the later operation-domain checkpoint recorded in
[the plain difference contract](temporal-plain-difference-operation.md) retain
their original scope. They do not verify the current direct-arithmetic batch.

## Current verification requirements

The 2026-09-10 batch requires focused native regressions followed by the
coordinated broad checkpoint. Relevant retained checks include:

```sh
cargo test -p lila-aot-wasm --test temporal_zoned_date_time_difference_defaults_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_uses_zoned_date_time_hour_difference_default -- --exact
./target/debug/lila test262 run built-ins/Temporal/ZonedDateTime/prototype/until/defaults-to-returning-hours.js --execution-backend wasm --timeout-ms 240000 --threads 1
./target/debug/lila test262 run built-ins/Temporal/ZonedDateTime/prototype/since/defaults-to-returning-hours.js --execution-backend wasm --timeout-ms 240000 --threads 1
./target/debug/lila test262 run built-ins/Temporal/ZonedDateTime/prototype/until/largestunit-undefined.js --execution-backend wasm --timeout-ms 240000 --threads 1
./target/debug/lila test262 run built-ins/Temporal/ZonedDateTime/prototype/since/largestunit-undefined.js --execution-backend wasm --timeout-ms 240000 --threads 1
./target/debug/lila test262 run built-ins/Temporal/ZonedDateTime/prototype/until/largestunit-default.js --execution-backend wasm --timeout-ms 240000 --threads 1
./target/debug/lila test262 run built-ins/Temporal/ZonedDateTime/prototype/since/largestunit-default.js --execution-backend wasm --timeout-ms 240000 --threads 1
```

The final current-SHA closure remains the complete T22 Date/Temporal ladder and
the low-RAM current-pin publication path.

## Non-claims

This seam does not implement named time zones, DST-sensitive
`DifferenceZonedDateTime`, or a default-zone provider. Shared arithmetic changes
also affect PlainDateTime and require its neighboring controls. The historical
Date evidence is not a new Date verification result, and the current source
does not establish that the complete Temporal tree is green or publish new
snapshots and README status.

## Direct-arithmetic checkpoint completed 2026-09-12

The [ZonedDateTime baseline follow-up](../zoned-date-time-baseline-follow-up.md)
records 408/408 pinned executions, 49/49 focused Wasmtime regressions (including
all ten difference tests), 1,122 IR tests, 428 backend tests, 102 Temporal
structural tests, the workspace check, and 191/191 fake-fixture executions.
The pinned replay repairs 136 reproduced main failures and retains the other
46 observed passes. These results do not publish a new full-suite baseline.
