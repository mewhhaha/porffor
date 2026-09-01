# Temporal PlainDateTime component projection

Status: implemented for `Temporal.PlainDateTime.prototype.toPlainDate` and
`.toPlainTime`.

## Boundary

The shared component emitter accepts only the non-copyable, non-derived
`TemporalPlainDateTimeComponent::{PlainDate, PlainTime}` domain. Both producers
first use the common receiver check and field extraction. One exhaustive match
then owns the divergent allocation:

- `PlainDate` loads `%Temporal.PlainDate.prototype%`, transfers the receiver's
  calendar and date fields, and allocates a PlainDate.
- `PlainTime` projects the six time locals and allocates a PlainTime without a
  calendar or PlainDate prototype local.

There is no Boolean, default or wildcard projection. Adding a component must
therefore define its complete allocation behavior at the compiler boundary.

## Durable evidence

`temporal_plain_date_time_component_structure.rs` pins the exact two-variant
domain and lack of convenience capabilities, receiver extraction before the
single exhaustive match, the arm-specific prototype/calendar/time ownership,
the explicit standard-module import and the exact two-producer census.

The existing pinned
`built-ins/Temporal/PlainDateTime/prototype/toPlainDate/basic.js` and
`built-ins/Temporal/PlainDateTime/prototype/toPlainTime/basic.js` leaves are the
behavioral witnesses. This source-equivalent invariant batch adds no new
fixture.

## Verification

```sh
cargo test -p lila-aot-wasm --test temporal_plain_date_time_component_structure
./target/debug/lila --jobs 1 test262 run built-ins/Temporal/PlainDateTime/prototype/toPlainDate/basic.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000
./target/debug/lila --jobs 1 test262 run built-ins/Temporal/PlainDateTime/prototype/toPlainTime/basic.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 60000
cargo xc
cargo fmt --all -- --check
git diff --check
```

The bounded structure target passes all `3/3` tests, and both pinned basic
leaves pass both variants (`4/4`) with every failure bucket at zero. `cargo xc`,
workspace formatting and the diff check are green. The following shared
684-dump semantic golden passes `2/2` in 681.86 seconds, adds only the adjacent
PlainDateTime field-read witness and removes none. All 683 retained
non-accounting summaries are equal; 51 retained dumps differ only in compiler
accounting, each with 294 fewer emitted code bytes. No broad Test262 run or
published-status refresh was performed for this batch.

## Deferrals

This contract does not complete either returned Temporal type, add custom
calendar/time-zone protocols, or complete PlainDateTime or T22.
