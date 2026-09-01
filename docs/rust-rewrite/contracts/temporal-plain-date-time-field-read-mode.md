# Temporal PlainDateTime field-read mode

Status: focused-verified for `ToTemporalDateTime` property-bag conversion and
`Temporal.PlainDateTime.prototype.with`.

## Boundary

The shared PlainDateTime field reader accepts only the private
`TemporalPlainDateTimeFieldReadMode::{Conversion, With}` domain. The type is
neither derived nor copyable. One direct exhaustive match owns the sole mode
difference:

| Mode | Producer | Calendar behavior |
| --- | --- | --- |
| `Conversion` | `ToTemporalDateTime` property-bag path | performs `Get(calendar)` and canonicalizes the result before the date/time field sweep |
| `With` | `Temporal.PlainDateTime.prototype.with` | emits no calendar read or canonicalization in the shared sweep |

`with` now performs the required observable `Get` operations for `calendar` and
`timeZone` in `RejectTemporalLikeObject`. A non-`undefined` result is rejected;
an otherwise valid partial bag proceeds to the ordinary alphabetical field
sweep without reading `calendar` a second time. PlainYearMonth owns a distinct
typed field-reader boundary recorded in
[`temporal-plain-year-month-field-read-mode.md`](temporal-plain-year-month-field-read-mode.md);
the two method families are not coupled through a shared policy type.

## Durable evidence

`temporal_plain_date_time_field_read_mode_structure.rs` pins the exact
capability-free two-variant domain, the single exhaustive match, the sole
calendar Get/canonicalization body, the exact two producer mappings, and the
absence of a Boolean at this reader boundary.

`wasm_temporal_plain_date_time_field_read_mode.js` observes the conversion and
`with` property-read order through Proxy bags. It requires conversion to read
`calendar` before the alphabetical date/time fields, requires `with` to read
`calendar` and `timeZone` exactly once before its field sweep, and requires a
forbidden calendar getter to be called before its value is rejected. A second
getter throws a sentinel object and requires that exact abrupt completion to
propagate.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test temporal_plain_date_time_field_read_mode_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_preserves_plain_date_time_field_read_modes -- --exact --test-threads=1
node --check crates/lila-cli/tests/fixtures/wasm_temporal_plain_date_time_field_read_mode.js
./target/debug/lila --jobs 1 test262 run built-ins/Temporal/PlainDateTime/from/order-of-operations.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 180000
./target/debug/lila --jobs 1 test262 run built-ins/Temporal/PlainDateTime/prototype/with/order-of-operations.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 180000
./target/debug/lila --jobs 1 test262 run built-ins/Temporal/PlainDateTime/prototype/with/calendar-throws.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --threads 1 --timeout-ms 180000
cargo fmt --all -- --check
git diff --check
```

The structure target passes `4/4`, the exact CLI witness passes `1/1`, and the
three pinned Test262 leaves pass both variants (`6/6`) with every failure bucket
at zero. The `with` order leaf changed from `0/2` runtime bugs before the
ordinary-Get repair to `2/2` after it. Fixture syntax, formatting and diff
checks are green. The following shared 684-dump semantic golden passes `2/2`
in 681.86 seconds, adds only this fixture and removes none. All 683 retained
non-accounting summaries are equal; 51 retained dumps differ only in compiler
accounting, each with 294 fewer emitted code bytes. No published-status refresh
was run.

## Deferrals

This contract does not unify the distinct PlainYearMonth reader, add custom
calendar or time-zone protocols, close the remaining PlainDateTime methods, or
complete T22.
