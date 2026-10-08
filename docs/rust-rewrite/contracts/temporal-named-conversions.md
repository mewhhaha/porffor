# Temporal zoned conversion ownership

Prepared source contract; runtime verification is pending integration of the
complete named-zone consumer graph. This document establishes no named-zone
admission or conformance result.

PlainDate/PDT construction bounds retain approved proposal head
`e8cc03fc970a65a3359e8870e3b35e687ac94e55`, spec/plaindatetime.html:
absolute ISO epoch days at most 100000001 followed by the exclusive
Instant±day nanosecond interval; PlainDate evaluates that operation at noon.
The unmerged PR3966 first day check drops the extra day and conflicts with
its own non-throwing creation after a valid Instant is projected through any
zone. An integration transcription defect is the supported inference, not
a confirmed upstream ruling. Existing positive boundary controls are
preserved. Provider inverse ValidateISODaysRange has its distinct inclusive
±100000000 bound at prescribed inverse steps; it does not replace creation
bounds or add an early check to general contextual projection.

Instant.toZonedDateTimeISO checks the receiver brand before zone conversion,
then carries its exact epoch into the new ZonedDateTime with ISO calendar.
Identifier resolution retains observable identifier separately from primary
identity. It does not attach an offset that could be reused at another epoch.

PlainDate.toZonedDateTime has one argument. An object first supplies timeZone;
zone conversion completes before plainTime is read. Missing or undefined time
uses the distinct GetStartOfDay operation. Explicit time completes
ToTemporalTime and ISO date/time validation, then uses compatible inverse
selection. No second options argument is read. The receiver's actual calendar
slot survives both allocation branches.

PlainDateTime.toZonedDateTime checks its brand, resolves the zone, completes
GetOptionsObject and its sole Disambiguation read, and retains that policy
through inverse selection. It never reads offset or overflow options. The
complete candidate list passes Instant validation before policy chooses an
epoch; option reads precede this algorithmic validation. Allocation borrows
proved epoch, resolved zone and actual calendar handles.

The shared foundation owns brand/date/epoch/calendar factories, inverse policy,
provider response checks and the allocator. Its private gap topology
certificate must establish the current nearest nonempty local endpoint
semantics before data-only gap coordinates can support disambiguation or
GetStartOfDay. All named consumers and capability dependencies must migrate
together before removing the explicit named semantic gap.

The authored native target aot_temporal_named_conversions checks exact negative
fractional epochs and slot suppression; historical midnight and whole-date
jumps; PlainDate second-argument suppression and getter order; fold/gap policy
selection and actual calendar; option/range/abrupt precedence and foreign error
realm; half-hour jumps and Instant boundaries. Six controls are planned in
sloppy and strict execution. None has been executed at this preparation stage.

After complete integration, run the native target and unchanged pinned
conversion cases, then the shared broad checkpoint:

```sh
cargo test -p lila-engine --test aot_temporal_named_conversions -- --test-threads=1
```

Primary algorithms:
[Instant.toZonedDateTimeISO](https://tc39.es/proposal-temporal/#sec-temporal.instant.prototype.tozoneddatetimeiso),
[PlainDate.toZonedDateTime](https://tc39.es/proposal-temporal/#sec-temporal.plaindate.prototype.tozoneddatetime),
[PlainDateTime.toZonedDateTime](https://tc39.es/proposal-temporal/#sec-temporal.plaindatetime.prototype.tozoneddatetime),
[GetEpochNanosecondsFor](https://tc39.es/proposal-temporal/#sec-temporal-getepochnanosecondsfor)
and [GetStartOfDay](https://tc39.es/proposal-temporal/#sec-temporal-getstartofday).
