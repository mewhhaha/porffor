# PlainDate calendar rounding parity

`Temporal.PlainDate.prototype.until` and `since` round year, month and week
differences using two actual calendar-date endpoints. Day distances determine
which endpoint is nearer. At an exact half-even tie, parity belongs to the
truncated selected calendar-unit coefficient divided by `roundingIncrement`.
It does not belong to the fractional day distance or to retained larger fields.

The arithmetic authority is the immutable Temporal Stage4 integration used by
the [contextual-window contract](temporal-selected-rounding-window.md),
`ptomato/ecma262` commit `3d4a6e7124a6878cb5af3132af7e01e01a88317f`:
[NudgeToCalendarUnit](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/duration.emu#L1238)
retains the inner unit bound when computing the total, and
[ApplyUnsignedRoundingMode](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/abstractops.emu#L2626)
uses `(lowerBound / (upperBound - lowerBound)) modulo 2` at a half-even tie.
This separate Temporal integration authority is not presented as part of the
repository's current ECMA-262 `5345883164` pin.

The PlainDate nudge retains that unit quotient while dating both endpoints.
It passes the signed travel direction and exact magnitude day distances to the
existing Duration rounding decision shared with PlainYearMonth. A destination
at the far endpoint selects it under every rounding mode. No division converts
the calendar progress to floating point, and no nanosecond scaling is needed.
The valid PlainDate span keeps twice the magnitude day distance well inside
`i64`. Both candidate calendar additions still perform their original range
checks before selection, including an unselected far endpoint.

The entry owner still acquires and validates options, truncates the selected
unit, drops smaller fields, rounds days directly, bubbles retained calendar
fields, and negates the rounding mode and result for `since` in their existing
order. Calendar-independent quotient parity applies to every calendar accepted
by that owner; it has no fixture, path or calendar-name condition.

The focused Engine controls cover all nine modes at exact odd/even year and
month midpoints in both directions, increment quotients greater than one,
retained larger fields, week/day dispatch, neighboring non-ties, exact dates,
and unselected endpoint range errors. They require real Wasm AOT execution.
The original Test262 `roundingmode-half-boundary.js` fixtures for both methods
remain unchanged and require all four sloppy/strict executions to pass.

This repair was prepared against `eb1ca63c9dbf09148703ccfbd0ca35d9629753e7`
during a frozen native suite. Source review, exact-rational preparation controls
and Rust formatting checks do not establish native acceptance. Rust compilation,
the focused Engine controls, unchanged pinned fixtures and broader retention
remain pending integration and execution by the native owner.
