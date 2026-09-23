# Zoned date-time field replacement

The completed-baseline replay exposed the missing
`Temporal.ZonedDateTime.prototype.with` method. A separate frozen-main witness
showed an offset object whose `toString` returned a number producing a
`RangeError`; `ToOffsetString` requires a `TypeError` because the primitive is
not a string. The Buddhist native regressions also exposed a missing
`ZonedDateTime.prototype.toPlainDate` method.

`with` now compiles through the normal builtin registry and Wasm emitter. It
checks the receiver and partial object, reads `calendar` and `timeZone`, and
prepares fields in alphabetical order. The shared closed date/time read mode
enables the offset row only for zoned replacement. Offset conversion uses
`ToPrimitive` with a string hint, checks the primitive type, and validates the
offset grammar before reading the next field. A throwing conversion retains its
original completion. This boundary is shared with `ZonedDateTime.from` property
bags; ordinary calendar-era string conversion remains separate.

The From/With options context selects `reject` or `prefer` as the offset
default. Both operations read disambiguation, offset and overflow in that
order. Calendar merging resolves era/year agreement before month/monthCode
agreement and preserves receiver fields omitted by the caller. ISO, Gregorian
and Buddhist calendars use the existing typed ISO-year resolver, so an absent
Buddhist year is not converted twice.

After regulation, both operations consume the same fixed-zone epoch kernel.
It preserves exact seconds and subsecond offset values through BigInt epoch
construction. `prefer` and `reject` validate the original ISO day range; `use`
and `ignore` validate the balanced epoch. The returned object has the intrinsic
prototype and retains the receiver's calendar and time zone.

`toPlainDate` and `toPlainDateTime` share one closed allocation target after
the existing local-component conversion. They preserve the calendar and use
the local ISO date, including dates adjacent to the minimum instant. No public
method calls or extra property reads implement either conversion.

This follows the specification algorithms for
[`with`](https://tc39.es/proposal-temporal/#sec-temporal.zoneddatetime.prototype.with),
[`ToOffsetString`](https://tc39.es/proposal-temporal/#sec-temporal-tooffsetstring),
and [`toPlainDate`](https://tc39.es/proposal-temporal/#sec-temporal.zoneddatetime.prototype.toplaindate).
UTC and fixed-offset time zones remain the supported zone domain. Other
calendars, named-zone transitions, and calendar-relative Duration operations
remain separate gaps; the change does not claim that every Temporal fixture
passes.

The focused native target is
`cargo test -p lila-engine --test aot_temporal_zoned_with -- --test-threads=1`.
It covers metadata and branding, exact field and option order, abrupt values,
all numeric fields, calendar merges, offset modes, nanoseconds, and epoch
limits. The existing Buddhist target also checks conversion and arithmetic
across all implemented date carriers. Calendar `since` results are compared
with ISO `since`; reversing `until` is not an equivalent oracle at month-end
boundaries.

The pinned `built-ins/Temporal/ZonedDateTime/prototype/with` cohort contains
41 files and 82 execution modes. Eleven additional `intl402` files cover the
implemented Gregorian and Buddhist calendars with UTC or fixed offsets.
The resulting focused cohort is 52 files and 104 execution modes. Remaining
internationalized fixtures require other calendar or named-zone capabilities.
Execution results and full-suite counts belong to the coordinating batch
report.
