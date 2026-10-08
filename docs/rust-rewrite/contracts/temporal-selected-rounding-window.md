# Selected contextual Temporal rounding windows

This 2026-10-03 source change moves zoned contextual-window validation to the
point where NudgeToCalendarUnit has selected its initial or recomputed window.
ComputeNudgeWindow supplies provisional endpoints. The final factory checks
that the destination is bracketed and that the span is nonzero with the
retained duration sign. The actual rounding and exact total consumer requires
that completed owner by value, including its validated sign and destination.
The endpoint/date scratch locals are released after consumption.

The shared calendar-duration helper applies additionalShift only to year and
month. Week and day keep their original coefficients during the prescribed
recomputation. No other retry or offset policy is introduced. Both initial and
recomputed endpoint resolution retain the existing calendar, pinned named-zone
provider, compatible inverse and prescribed Instant checks. Even an unselected
far endpoint can therefore produce its required RangeError.

The arithmetic authority remains the repository's immutable Stage4 integration
head, `ptomato/ecma262` commit `3d4a6e7124a6878cb5af3132af7e01e01a88317f`:
[ComputeNudgeWindow](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/duration.emu#L1217)
and [NudgeToCalendarUnit](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/duration.emu#L1260).
Its origin-reuse condition tests the whole start DateDuration's sign. Larger
calendar fields therefore remain in the near endpoint even when the rounded
month, week or day coefficient is zero. An entirely empty start duration
preserves the exact origin occurrence, including a later fold.

The rendered proposal and proposal repository still show the older `r1 = 0`
condition. [Issue3316](https://github.com/tc39/proposal-temporal/issues/3316#issuecomment-4595396348)
records the maintainer's explanation that the ECMA-262 integration PR is the
source of truth. The whole-duration predicate follows that corrected primary
algorithm; it is not an inferred extension or an unresolved semantic gap.
[Issue3310](https://github.com/tc39/proposal-temporal/issues/3310) remains open:
its adjacent Apia day windows can collapse or fail to bracket after the
prescribed computation. Such final windows still report the uncatchable
RuntimeSemanticGap::TemporalZonedRoundingWindow owned by T22. This compiler
semantic rejection cannot satisfy a negative JavaScript test.

The 2026-10-05 primary-source audit found no accepted skipped-day resolution.
Issue3310 is still open. The proposed
[extra-nudge change](https://github.com/tc39/proposal-temporal/pull/3318) and
[Test262 cases](https://github.com/tc39/test262/pull/5044) are both draft; the
discussion has not settled the expected total for the negative Apia day.
The [current integration source](https://raw.githubusercontent.com/ptomato/ecma262/temporal-stage-4/temporal/duration.emu)
still leaves week/day coefficients unchanged during the single recomputation,
asserts distinct selected endpoints before calendar-unit division, and asserts
the day-span sign in NudgeToZonedTime. This evidence supports retaining the
owned rejection, rather than adding an unaccepted retry or result policy.
The production algorithm and existing Apia ownership controls are unchanged;
T22 remains open for this case.

The authored Engine controls cover constrained month/year second windows,
exact UTC/fixed-offset/named-zone totals, pre-epoch leap dates, unselected
endpoint range checks, DST day spans, signed half ties, retained larger
calendar fields and exact fold occurrence reuse. Existing Apia gap
controls remain explicit. The combined source has not been compiled or run;
this contract records implementation, not verified conformance. Full Date and
Temporal acceptance, wider calendars and configurable system defaults remain
open.
