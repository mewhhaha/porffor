# Temporal PlainYearMonth field-read mode

Status: implemented for `ToTemporalYearMonth` property-bag conversion and
`Temporal.PlainYearMonth.prototype.with`; focused verification is complete.

## Boundary

The shared PlainYearMonth field reader accepts only the private, non-copyable
`TemporalPlainYearMonthFieldReadMode::{Conversion, With}` domain. One borrowed
exhaustive match owns the sole mode difference:

| Mode | Producer | Calendar behavior in the shared reader |
| --- | --- | --- |
| `Conversion` | `ToTemporalYearMonth` property-bag path | performs `Get(calendar)` and canonicalizes it before the remaining field sweep |
| `With` | `Temporal.PlainYearMonth.prototype.with` | emits no calendar read or canonicalization |

The `with` algorithm retains its earlier `RejectObjectWithCalendarOrTimeZone`
reads of `calendar` and `timeZone`. Its typed `With` row prevents the shared
reader from performing a second calendar read. Era, eraYear, month, monthCode
and year reading, day exclusion and overflow ordering remain shared and
unchanged.

## Durable evidence

`temporal_plain_year_month_field_read_mode_structure.rs` pins the exact
capability-free two-row domain, the sole borrowed exhaustive match, the
calendar body and its position before the shared sweep, the exact two producer
mappings, the retained `with` rejection reads and recursive source censuses.
The neighboring PlainDateTime guard no longer permits this reader to regress
to its former Boolean.

## Focused witnesses

The direct behavioral witnesses are:

- `built-ins/Temporal/PlainYearMonth/from/order-of-operations.js`;
- `built-ins/Temporal/PlainYearMonth/prototype/with/order-of-operations.js`.

They distinguish conversion's calendar-first read from `with`'s single
calendar/timeZone rejection reads. No new fixture is required for this
source-equivalent type closure.

The bounded structure target passes `3/3`, the neighboring PlainDateTime guard
passes `4/4`, and both exact pinned leaves pass both variants (`4/4`) with every
failure and unsupported bucket at zero. `cargo xc` is green. No semantic golden
was run because the mode changes only Rust-time instruction selection.

## Deferrals

This contract does not add custom calendar protocols, change field or option
semantics, unify the distinct PlainDate/PlainDateTime readers, or complete
PlainYearMonth, Temporal or T22.
