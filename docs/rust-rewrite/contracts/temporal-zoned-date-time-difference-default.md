# ZonedDateTime difference default largest unit

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

## Existing composition and the defect

The Wasm-AOT ZonedDateTime implementation converts both operands to trusted
PlainDateTime values and directly calls the compiled PlainDateTime `until` or
`since` body. Passing the user's options object through that call preserves
observable property access, but also selects PlainDateTime's hard-coded `day`
fallback. The result has the right sign and shape while balancing into the
wrong largest unit.

Reading `largestUnit` in the ZonedDateTime wrapper and then passing the same
object to the delegate is not a valid repair: getters, proxies and conversion
hooks would be observed twice. Duplicating the complete PlainDateTime
difference emitter would avoid the second read but would create a second
arithmetic authority and substantially increase emitted Wasm.

## Closed settings plan and linear witness

One shared settings producer owns the four observable reads in order:

1. `largestUnit`;
2. `roundingIncrement`;
3. `roundingMode`; and
4. `smallestUnit`.

Its closed compile-time plan has exactly three states:

- PlainDateTime `until`: `day` fallback, rounding mode consumed directly;
- PlainDateTime `since`: `day` fallback, rounding mode negated before use; and
- ZonedDateTime delegation: `hour` fallback, unnegated rounding mode retained
  because the selected PlainDateTime delegate still owns the operation
  direction.

The producer returns a private, `#[must_use]`, non-`Copy` resolved-settings
witness. PlainDateTime consumes the witness directly in its existing
arithmetic. ZonedDateTime consumes it through one materializer that builds an
unreachable null-prototype options object containing explicit primitive values
for all four settings. The existing PlainDateTime delegate may read and
validate that internal object again, but no user getter or conversion hook is
repeated. The user options object is never passed to the delegate.

The internal object is a transport representation, not a new JavaScript API or
general Temporal options abstraction. It exists only to keep one arithmetic
body while carrying already resolved settings across the existing builtin-call
boundary.

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

A focused source-structure test pins the three-state plan, its exhaustive
fallback and rounding ownership, the non-copyable witness, both consumers, and
the rule that ZonedDateTime passes the normalized options locals rather than
the user's locals to the PlainDateTime delegate.

## Baseline and deferred gates

The completed 2026-08-13 current-pin Wasm-AOT Date-family snapshots were
produced by an older binary. They are ownership evidence only: the aggregate
Date leaf passed 75 of 78 cases with exactly the three now-landed constructor
realm-prototype failures, while `Date/UTC`, `Date/now`, `Date/parse` and
`Date/prototype` passed 17/17, 6/6, 8/8 and 485/485 respectively. No completed
current-pin Temporal Wasm leaf establishes this seam's runtime result.

While that baseline owns Cargo and Test262 resources, this batch performs only
source inspection and cheap static checks. After release, verification must
include:

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

This seam does not implement named time zones or DST-sensitive
`DifferenceZonedDateTime`. It does not make time-unit differences legal across
different zones, move `GetDifferenceSettings` before the existing time-zone
guard, add a default-zone provider, or change another Temporal class. It does
not change Date, claim the current Date fixes have executed successfully, claim
the complete Temporal tree is green, or refresh snapshots and README status.
