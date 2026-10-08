//! `Duration` `relativeTo` math: the plain-path algorithms behind `round`,
//! `total` and `compare` (`DifferencePlainDateTimeWithRounding`,
//! `RoundRelativeDuration`, `DifferencePlainDateTimeWithTotal`,
//! `DateDurationDays`). Retained zoned proofs use temporal_zoned_arithmetic.
//!
//! Three representations recur below:
//!
//! * A date duration is four `i64` locals (years, months, weeks, days).
//! * A time duration is an `i64` `(seconds, subsecond)` pair in the same
//!   truncated convention `normalize_seconds` produces: the subsecond part
//!   stays strictly inside a second and carries the pair's sign (or zero).
//! * An epoch-nanosecond stamp is an `i64` `(days, nanos)` span: whole epoch
//!   days plus a time-of-day nanosecond count in `[0, 86_400 * 10^9)`.
//!   Stamps never multiply out to a single nanosecond count, which would
//!   overflow `i64` for year-wide windows; comparisons run days-major.

use super::super::*;
use super::temporal_duration::TemporalDurationFields;
use super::temporal_duration_methods::{
    CompletedTemporalDurationRoundOptionsLocals, TemporalDurationTotalUnitLocals,
};
use super::temporal_options::{TemporalOverflow, TemporalRoundingMode, TemporalUnit};
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use super::temporal_zone_provider::TemporalPlainRelativeContextLocals;
use crate::gc_types::*;

/// Nanoseconds in a civil day. Every span normalizes its nanosecond leg into
/// `[0, NANOSECONDS_PER_DAY)`.
const NANOSECONDS_PER_DAY: i64 = 86_400_000_000_000;

impl<'a> FunctionBuilder<'a> {
    /// `ToInternalDurationRecordWith24HourDays`, split across the module's
    /// date/time representation: the calendar fields project to `i64`, the
    /// time fields normalize to `(seconds, subsecond)`.
    pub(super) fn emit_temporal_relative_internal_from_fields(
        &mut self,
        fields: &TemporalDurationFields,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        function: &mut Function,
    ) {
        for (unit, destination) in [
            (TemporalUnit::Year, date[0]),
            (TemporalUnit::Month, date[1]),
            (TemporalUnit::Week, date[2]),
            (TemporalUnit::Day, date[3]),
        ] {
            (fields.number_bits(unit)).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            (destination).store(function);
        }
        self.emit_temporal_duration_normalize_seconds(
            fields,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
    }

    /// `AddTime(MidnightTimeRecord(), timeDuration)`: fold a `(seconds,
    /// subsecond)` pair into whole `days` plus a time of day. The day count
    /// rounds toward negative infinity (Euclidean division) so the leftover
    /// seconds and subseconds are always nonnegative.
    pub(super) fn emit_temporal_relative_split_time(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        days_local: I64Local,
        day_seconds_local: I64Local,
        day_subsecond_local: I64Local,
        function: &mut Function,
    ) {
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (day_subsecond_local).store(function);
        (seconds_local).load(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (day_seconds_local).store(function);
        // A truncated division leaves the remainder counter-signed for
        // negative inputs; shift one second over to land in `[0, 10^9)`.
        (day_subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (day_subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Add);
        (day_subsecond_local).store(function);
        (day_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (day_seconds_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (day_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (days_local).store(function);
        (day_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        (day_seconds_local).store(function);
        (day_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (day_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Add);
        (day_seconds_local).store(function);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (days_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// `GetUTCEpochNanoseconds` as a `(days, nanos)` span. The day leg is the
    /// epoch-day count; the nanosecond leg folds the time of day and stays
    /// below one civil day.
    pub(super) fn emit_temporal_relative_epoch_span(
        &mut self,
        year_local: I64Local,
        month_local: I64Local,
        day_local: I64Local,
        day_seconds_local: I64Local,
        day_subsecond_local: I64Local,
        span_days_local: I64Local,
        span_nanos_local: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            span_days_local,
            function,
        );
        (day_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (day_subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (span_nanos_local).store(function);
    }

    /// RangeError unless a `(days, nanos)` span satisfies
    /// `ISODateTimeWithinLimits`: `|days|` within 10^8 + 1, and the exact
    /// instant edges (`ns ≤ min − day`, `ns ≥ max + day`). With the nanos
    /// leg in `[0, NANOSECONDS_PER_DAY)` the low edge bites only at
    /// exactly (−100000001, 0) and the high edge at any (100000001, _).
    pub(super) fn emit_temporal_relative_reject_bad_datetime_span(
        &mut self,
        days_local: I64Local,
        nanos_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(100_000_001));
        function.instruction(&Instruction::I64GeS);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(-100_000_001));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(-100_000_001));
        function.instruction(&Instruction::I64Eq);
        (nanos_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::TEMPORAL_DURATION_RELATIVETO_IS_OUTSIDE_THE_REPRESENTABLE_DATE_TIME_RANGE, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Subtract two normalized spans. Both inputs carry their nanosecond leg
    /// in `[0, NANOSECONDS_PER_DAY)`, so one fixup in either direction
    /// renormalizes the result.
    pub(super) fn emit_temporal_relative_span_sub(
        &mut self,
        left_days_local: I64Local,
        left_nanos_local: I64Local,
        right_days_local: I64Local,
        right_nanos_local: I64Local,
        out_days_local: I64Local,
        out_nanos_local: I64Local,
        function: &mut Function,
    ) {
        (left_days_local).load(function);
        (right_days_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (out_days_local).store(function);
        (left_nanos_local).load(function);
        (right_nanos_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (out_nanos_local).store(function);
        (out_nanos_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (out_nanos_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_DAY));
        function.instruction(&Instruction::I64Add);
        (out_nanos_local).store(function);
        (out_days_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (out_days_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// Compare two normalized spans days-major: -1, 0 or 1 in `out_local`.
    pub(super) fn emit_temporal_relative_span_cmp(
        &mut self,
        left_days_local: I64Local,
        left_nanos_local: I64Local,
        right_days_local: I64Local,
        right_nanos_local: I64Local,
        out_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (out_local).store(function);
        (left_days_local).load(function);
        (right_days_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (left_days_local).load(function);
        (right_days_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (left_nanos_local).load(function);
        (right_nanos_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (left_nanos_local).load(function);
        (right_nanos_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// `AddTimeDurationToEpochNanoseconds`: shift a normalized span by a
    /// signed `(seconds, subsecond)` pair. The seconds split into whole days
    /// plus a sub-day remainder before scaling, so the nanosecond leg never
    /// overflows.
    pub(super) fn emit_temporal_relative_span_add_duration(
        &mut self,
        span_days_local: I64Local,
        span_nanos_local: I64Local,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        function: &mut Function,
    ) {
        let shift_days_local = self.runtime_schema().reserve_i64_local(function);
        let shift_nanos_local = self.runtime_schema().reserve_i64_local(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (shift_days_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (shift_nanos_local).store(function);
        (span_days_local).load(function);
        (shift_days_local).load(function);
        function.instruction(&Instruction::I64Add);
        (span_days_local).store(function);
        (span_nanos_local).load(function);
        (shift_nanos_local).load(function);
        function.instruction(&Instruction::I64Add);
        (span_nanos_local).store(function);
        // The shift's nanosecond leg fits in one day either side of zero, so
        // at most one carry runs in each direction.
        (span_nanos_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_DAY));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (span_nanos_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_DAY));
        function.instruction(&Instruction::I64Sub);
        (span_nanos_local).store(function);
        (span_days_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (span_days_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (span_nanos_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (span_nanos_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_DAY));
        function.instruction(&Instruction::I64Add);
        (span_nanos_local).store(function);
        (span_days_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (span_days_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(shift_nanos_local, function);
        self.runtime_schema()
            .release_i64_local(shift_days_local, function);
    }

    /// `TimeDurationSign` of a `(seconds, subsecond)` pair: the seconds win
    /// unless they are zero.
    pub(super) fn emit_temporal_relative_time_sign(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        out_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (out_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// Raw AddDaysToISODate civil balance. The owning algorithm performs its
    /// own prescribed creation/local/Instant checks, without an extra date
    /// allocation check in this mathematical probe.
    pub(super) fn emit_temporal_relative_add_days(
        &mut self,
        date: [I64Local; 3],
        days_local: I64Local,
        function: &mut Function,
    ) {
        let epoch_local = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_plain_date_epoch_days(date[0], date[1], date[2], epoch_local, function);
        (epoch_local).load(function);
        (days_local).load(function);
        function.instruction(&Instruction::I64Add);
        (epoch_local).store(function);
        self.emit_temporal_civil_from_days(epoch_local, date[0], date[1], date[2], function);
        self.runtime_schema()
            .release_i64_local(epoch_local, function);
    }

    /// `DifferenceISODateTime` for the ISO calendar. Both inputs are within
    /// limits; the date/time pairs arrive as `(year, month, day)` triples
    /// plus `(day_seconds, day_subsecond)` time-of-day pairs. The result is
    /// always single-signed: when the time and date comparisons disagree in
    /// value sign, a day moves across to make them agree.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_relative_difference_iso_date_time(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        first: [I64Local; 3],
        first_seconds_local: I64Local,
        first_subsecond_local: I64Local,
        second: [I64Local; 3],
        second_seconds_local: I64Local,
        second_subsecond_local: I64Local,
        largest_unit_local: I64Local,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_sign_local = self.runtime_schema().reserve_i64_local(function);
        let date_sign_local = self.runtime_schema().reserve_i64_local(function);
        let date_largest_local = self.runtime_schema().reserve_i64_local(function);
        let adjusted = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];

        (second_seconds_local).load(function);
        (first_seconds_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (seconds_local).store(function);
        (second_subsecond_local).load(function);
        (first_subsecond_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (subsecond_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        (subsecond_local).store(function);
        self.emit_temporal_relative_time_sign(
            seconds_local,
            subsecond_local,
            time_sign_local,
            function,
        );
        self.emit_temporal_compare_iso_date(first, second, date_sign_local, function);
        for (index, local) in adjusted.iter().enumerate() {
            (second[index]).load(function);
            (*local).store(function);
        }
        (time_sign_local).load(function);
        (date_sign_local).load(function);
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_add_days(adjusted, time_sign_local, function);
        (seconds_local).load(function);
        (time_sign_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (seconds_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // `dateLargestUnit` is the larger of day and largestUnit: a
        // year/month/week largestUnit survives, anything day-or-smaller
        // becomes day.
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (largest_unit_local).load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::End);
        (date_largest_local).store(function);
        self.emit_temporal_difference_calendar_date(
            calendar,
            first,
            adjusted,
            date_largest_local,
            date[0],
            date[1],
            date[2],
            date[3],
            function,
        );
        // A time-category largestUnit folds the day leg into the time.
        (largest_unit_local).load(function);
        (date_largest_local).load(function);
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        (seconds_local).load(function);
        (date[3]).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (date[3]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(adjusted[2], function);
        self.runtime_schema()
            .release_i64_local(adjusted[1], function);
        self.runtime_schema()
            .release_i64_local(adjusted[0], function);
        self.runtime_schema()
            .release_i64_local(date_largest_local, function);
        self.runtime_schema()
            .release_i64_local(date_sign_local, function);
        self.runtime_schema()
            .release_i64_local(time_sign_local, function);
        Ok(())
    }

    /// `NudgeToDayOrTime`: round a day-or-smaller unit without a time zone.
    /// The day leg folds into the time, the pair rounds to the quantum, and a
    /// date-category largestUnit unfolds whole days back out. Truncated days
    /// are plain truncating divisions: the subsecond leg stays strictly
    /// inside one second with the pair's sign, so it can never push the day
    /// count across an integer boundary.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_relative_nudge_to_day_or_time(
        &mut self,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        dest_days_local: I64Local,
        dest_nanos_local: I64Local,
        largest_unit_local: I64Local,
        increment_local: I64Local,
        smallest_unit_local: I64Local,
        mode_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        did_expand_local: I64Local,
        function: &mut Function,
    ) {
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let folded_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let folded_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let diff_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let diff_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let whole_days_local = self.runtime_schema().reserve_i64_local(function);
        let rounded_whole_days_local = self.runtime_schema().reserve_i64_local(function);
        let delta_sign_local = self.runtime_schema().reserve_i64_local(function);
        let time_sign_local = self.runtime_schema().reserve_i64_local(function);

        (seconds_local).load(function);
        (date[3]).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        (seconds_local).load(function);
        (folded_seconds_local).store(function);
        (subsecond_local).load(function);
        (folded_subsecond_local).store(function);
        self.emit_temporal_duration_unit_quantum(
            smallest_unit_local,
            increment_local,
            quantum_local,
            function,
        );
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_round_seconds(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_round_subsecond(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (seconds_local).load(function);
        (folded_seconds_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (diff_seconds_local).store(function);
        (subsecond_local).load(function);
        (folded_subsecond_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (diff_subsecond_local).store(function);
        (folded_seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (whole_days_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (rounded_whole_days_local).store(function);
        self.emit_temporal_relative_sign_of_difference(
            rounded_whole_days_local,
            whole_days_local,
            delta_sign_local,
            function,
        );
        self.emit_temporal_relative_time_sign(
            folded_seconds_local,
            folded_subsecond_local,
            time_sign_local,
            function,
        );
        (delta_sign_local).load(function);
        (time_sign_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        (did_expand_local).store(function);
        (dest_days_local).load(function);
        (nudged_days_local).store(function);
        (dest_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        self.emit_temporal_relative_span_add_duration(
            nudged_days_local,
            nudged_nanos_local,
            diff_seconds_local,
            diff_subsecond_local,
            function,
        );
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        (rounded_whole_days_local).load(function);
        (date[3]).store(function);
        (seconds_local).load(function);
        (rounded_whole_days_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (seconds_local).store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        (date[3]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            time_sign_local,
            delta_sign_local,
            rounded_whole_days_local,
            whole_days_local,
            diff_subsecond_local,
            diff_seconds_local,
            folded_subsecond_local,
            folded_seconds_local,
            quantum_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// The sign of `left - right` (-1, 0 or 1) without computing the
    /// difference itself.
    pub(super) fn emit_temporal_relative_sign_of_difference(
        &mut self,
        left_local: I64Local,
        right_local: I64Local,
        out_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (out_local).store(function);
        (left_local).load(function);
        (right_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (left_local).load(function);
        (right_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// `ComputeNudgeWindow` without a time zone: truncate the duration's
    /// smallest-unit leg to the increment, lay the `[r1, r2]` window over the
    /// ISO date, and stamp both ends. `additional_shift` slides a nonzero
    /// window one increment outward when the destination fell outside it.
    #[allow(clippy::too_many_arguments)]
    /// Pure ComputeNudgeWindow calendar durations. The caller interprets its
    /// endpoints in either a plain coordinate or the actual retained zone.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_relative_compute_nudge_date_durations(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        date: [I64Local; 4],
        sign_local: I64Local,
        iso: [I64Local; 3],
        increment_local: I64Local,
        unit_local: I64Local,
        additional_shift_local: I64Local,
        r1_local: I64Local,
        r2_local: I64Local,
        start_date: [I64Local; 4],
        end_date: [I64Local; 4],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let trunc_mode_local = self.runtime_schema().reserve_i64_local(function);
        let base_local = self.runtime_schema().reserve_i64_local(function);
        let step_local = self.runtime_schema().reserve_i64_local(function);
        let zero_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let week_unit_local = self.runtime_schema().reserve_i64_local(function);
        let weeks_start = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let weeks_end = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let until_out = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];

        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        (trunc_mode_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (zero_local).store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        (increment_local).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (step_local).store(function);

        // Year: the window replaces the whole date duration.
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (date[0]).load(function);
        (base_local).store(function);
        self.emit_temporal_plain_time_round_nanoseconds(
            base_local,
            increment_local,
            trunc_mode_local,
            function,
        );
        self.emit_temporal_relative_shifted_base(
            base_local,
            step_local,
            additional_shift_local,
            r1_local,
            function,
        );
        (r1_local).load(function);
        (step_local).load(function);
        function.instruction(&Instruction::I64Add);
        (r2_local).store(function);
        (r1_local).load(function);
        (start_date[0]).store(function);
        (r2_local).load(function);
        (end_date[0]).store(function);
        for local in [
            start_date[1],
            start_date[2],
            start_date[3],
            end_date[1],
            end_date[2],
            end_date[3],
        ] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Month: the window keeps the years and replaces months and below.
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Month.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (date[1]).load(function);
        (base_local).store(function);
        self.emit_temporal_plain_time_round_nanoseconds(
            base_local,
            increment_local,
            trunc_mode_local,
            function,
        );
        self.emit_temporal_relative_shifted_base(
            base_local,
            step_local,
            additional_shift_local,
            r1_local,
            function,
        );
        (r1_local).load(function);
        (step_local).load(function);
        function.instruction(&Instruction::I64Add);
        (r2_local).store(function);
        (date[0]).load(function);
        (start_date[0]).store(function);
        (date[0]).load(function);
        (end_date[0]).store(function);
        (r1_local).load(function);
        (start_date[1]).store(function);
        (r2_local).load(function);
        (end_date[1]).store(function);
        for local in [start_date[2], start_date[3], end_date[2], end_date[3]] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Week: leftover days join the week count through a week-unit
        // difference before truncating. The week window never shifts.
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        for (index, local) in weeks_start.iter().enumerate() {
            (iso[index]).load(function);
            (*local).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            weeks_start[0],
            weeks_start[1],
            weeks_start[2],
            date[0],
            date[1],
            zero_local,
            zero_local,
            overflow_local,
            function,
        )?;
        for (index, local) in weeks_end.iter().enumerate() {
            (weeks_start[index]).load(function);
            (*local).store(function);
        }
        self.emit_temporal_relative_add_days(weeks_end, date[3], function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        (week_unit_local).store(function);
        self.emit_temporal_difference_calendar_date(
            calendar,
            weeks_start,
            weeks_end,
            week_unit_local,
            until_out[0],
            until_out[1],
            until_out[2],
            until_out[3],
            function,
        );
        (date[2]).load(function);
        (until_out[2]).load(function);
        function.instruction(&Instruction::I64Add);
        (base_local).store(function);
        self.emit_temporal_plain_time_round_nanoseconds(
            base_local,
            increment_local,
            trunc_mode_local,
            function,
        );
        (base_local).load(function);
        (r1_local).store(function);
        (base_local).load(function);
        (step_local).load(function);
        function.instruction(&Instruction::I64Add);
        (r2_local).store(function);
        for (source, start, end) in [
            (date[0], start_date[0], end_date[0]),
            (date[1], start_date[1], end_date[1]),
        ] {
            (source).load(function);
            (start).store(function);
            (source).load(function);
            (end).store(function);
        }
        (r1_local).load(function);
        (start_date[2]).store(function);
        (r2_local).load(function);
        (end_date[2]).store(function);
        function.instruction(&Instruction::I64Const(0));
        (start_date[3]).store(function);
        function.instruction(&Instruction::I64Const(0));
        (end_date[3]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Day: the window keeps years through weeks and replaces the days.
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (date[3]).load(function);
        (base_local).store(function);
        self.emit_temporal_plain_time_round_nanoseconds(
            base_local,
            increment_local,
            trunc_mode_local,
            function,
        );
        // ComputeNudgeWindow shifts only year/month windows. Day, like week,
        // retains the same coefficients on the prescribed recomputation.
        (base_local).load(function);
        (r1_local).store(function);
        (r1_local).load(function);
        (step_local).load(function);
        function.instruction(&Instruction::I64Add);
        (r2_local).store(function);
        for (source, start, end) in [
            (date[0], start_date[0], end_date[0]),
            (date[1], start_date[1], end_date[1]),
            (date[2], start_date[2], end_date[2]),
        ] {
            (source).load(function);
            (start).store(function);
            (source).load(function);
            (end).store(function);
        }
        (r1_local).load(function);
        (start_date[3]).store(function);
        (r2_local).load(function);
        (end_date[3]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            until_out[3],
            until_out[2],
            until_out[1],
            until_out[0],
            weeks_end[2],
            weeks_end[1],
            weeks_end[0],
            weeks_start[2],
            weeks_start[1],
            weeks_start[0],
            week_unit_local,
            overflow_local,
            zero_local,
            step_local,
            base_local,
            trunc_mode_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_relative_compute_nudge_window(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        date: [I64Local; 4],
        sign_local: I64Local,
        origin_days_local: I64Local,
        origin_nanos_local: I64Local,
        iso: [I64Local; 3],
        iso_seconds_local: I64Local,
        iso_subsecond_local: I64Local,
        increment_local: I64Local,
        unit_local: I64Local,
        additional_shift_local: I64Local,
        r1_local: I64Local,
        r2_local: I64Local,
        start_days_local: I64Local,
        start_nanos_local: I64Local,
        end_days_local: I64Local,
        end_nanos_local: I64Local,
        start_date: [I64Local; 4],
        end_date: [I64Local; 4],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let zero_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let end_date_scratch = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        function.instruction(&Instruction::I64Const(0));
        (zero_local).store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        self.emit_temporal_relative_compute_nudge_date_durations(
            calendar,
            date,
            sign_local,
            iso,
            increment_local,
            unit_local,
            additional_shift_local,
            r1_local,
            r2_local,
            start_date,
            end_date,
            function,
        )?;
        // PR3966 ComputeNudgeWindow at 3d4a6e7124a6878cb5af3132af7e01e01a88317f
        // tests the entire start DateDuration's sign (issue3316), rather than
        // the historical proposal's rounded-field predicate r1 = 0. Larger
        // retained fields must reach CalendarDateAdd before measuring progress.
        (start_date[0]).load(function);
        for local in &start_date[1..] {
            (*local).load(function);
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (origin_days_local).load(function);
        (start_days_local).store(function);
        (origin_nanos_local).load(function);
        (start_nanos_local).store(function);
        function.instruction(&Instruction::Else);
        for (index, local) in end_date_scratch.iter().enumerate() {
            (iso[index]).load(function);
            (*local).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            end_date_scratch[0],
            end_date_scratch[1],
            end_date_scratch[2],
            start_date[0],
            start_date[1],
            start_date[2],
            start_date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_relative_epoch_span(
            end_date_scratch[0],
            end_date_scratch[1],
            end_date_scratch[2],
            iso_seconds_local,
            iso_subsecond_local,
            start_days_local,
            start_nanos_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (index, local) in end_date_scratch.iter().enumerate() {
            (iso[index]).load(function);
            (*local).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            end_date_scratch[0],
            end_date_scratch[1],
            end_date_scratch[2],
            end_date[0],
            end_date[1],
            end_date[2],
            end_date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_relative_epoch_span(
            end_date_scratch[0],
            end_date_scratch[1],
            end_date_scratch[2],
            iso_seconds_local,
            iso_subsecond_local,
            end_days_local,
            end_nanos_local,
            function,
        );

        for local in [
            end_date_scratch[2],
            end_date_scratch[1],
            end_date_scratch[0],
            overflow_local,
            zero_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// `NudgeToCalendarUnit` without a time zone. The fractional progress
    /// `(dest - start) / (end - start)` never materializes as a quotient for
    /// rounding: directed modes compare the destination against the window
    /// ends, half modes compare the two absolute distances, and ties break
    /// by the unsigned mode. All comparisons run on exact integer spans, so
    /// no floating-point error can tip a rounding decision. The `[[Total]]`
    /// projection accumulates the complete integer rational before one
    /// nearest-even division, including large calendar increments.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_relative_nudge_to_calendar_unit(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        sign_local: I64Local,
        date: [I64Local; 4],
        origin_days_local: I64Local,
        origin_nanos_local: I64Local,
        dest_days_local: I64Local,
        dest_nanos_local: I64Local,
        iso: [I64Local; 3],
        iso_seconds_local: I64Local,
        iso_subsecond_local: I64Local,
        increment_local: I64Local,
        unit_local: I64Local,
        mode_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        did_expand_local: I64Local,
        total_bits_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let shift_local = self.runtime_schema().reserve_i64_local(function);
        let r1_local = self.runtime_schema().reserve_i64_local(function);
        let r2_local = self.runtime_schema().reserve_i64_local(function);
        let start_days_local = self.runtime_schema().reserve_i64_local(function);
        let start_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let end_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let start_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let end_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let cmp_local = self.runtime_schema().reserve_i64_local(function);
        let umode_local = self.runtime_schema().reserve_i64_local(function);
        let dist_a_days_local = self.runtime_schema().reserve_i64_local(function);
        let dist_a_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let dist_b_days_local = self.runtime_schema().reserve_i64_local(function);
        let dist_b_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let pick_end_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(0));
        (did_expand_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (shift_local).store(function);
        self.emit_temporal_relative_compute_nudge_window(
            calendar,
            date,
            sign_local,
            origin_days_local,
            origin_nanos_local,
            iso,
            iso_seconds_local,
            iso_subsecond_local,
            increment_local,
            unit_local,
            shift_local,
            r1_local,
            r2_local,
            start_days_local,
            start_nanos_local,
            end_days_local,
            end_nanos_local,
            start_date,
            end_date,
            function,
        )?;
        // The destination must sit inside the window; otherwise one
        // additional shift expands it outward.
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_cmp(
            start_days_local,
            start_nanos_local,
            dest_days_local,
            dest_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            end_days_local,
            end_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_cmp(
            end_days_local,
            end_nanos_local,
            dest_days_local,
            dest_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (shift_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_compute_nudge_window(
            calendar,
            date,
            sign_local,
            origin_days_local,
            origin_nanos_local,
            iso,
            iso_seconds_local,
            iso_subsecond_local,
            increment_local,
            unit_local,
            shift_local,
            r1_local,
            r2_local,
            start_days_local,
            start_nanos_local,
            end_days_local,
            end_nanos_local,
            start_date,
            end_date,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        (did_expand_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // `GetUnsignedRoundingMode`: zero and infinity decide between the
        // window ends, the half modes compare distances first.
        for (mode, positive, negative) in [
            (TemporalRoundingMode::Ceil, 1_i64, 0_i64),
            (TemporalRoundingMode::Floor, 0, 1),
            (TemporalRoundingMode::Expand, 1, 1),
            (TemporalRoundingMode::Trunc, 0, 0),
            (TemporalRoundingMode::HalfCeil, 3, 2),
            (TemporalRoundingMode::HalfFloor, 2, 3),
            (TemporalRoundingMode::HalfExpand, 3, 3),
            (TemporalRoundingMode::HalfTrunc, 2, 2),
            (TemporalRoundingMode::HalfEven, 4, 4),
        ] {
            (mode_local).load(function);
            function.instruction(&Instruction::I64Const(mode.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(positive));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(negative));
            function.instruction(&Instruction::End);
            (umode_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        (pick_end_local).store(function);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            end_days_local,
            end_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // `progress = 1`: the destination reached the far end.
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // Off the near end, `infinity` takes the far one.
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The half modes weigh the absolute distances to each end.
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_sub(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            end_days_local,
            end_nanos_local,
            dest_days_local,
            dest_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_sub(
            start_days_local,
            start_nanos_local,
            dest_days_local,
            dest_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            dest_days_local,
            dest_nanos_local,
            end_days_local,
            end_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_cmp(
            dist_a_days_local,
            dist_a_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        function.instruction(&Instruction::Else);
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Half-even tests the ordinal bucket r1/(r2-r1), including an
        // increment greater than one.
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (r1_local).load(function);
        (increment_local).load(function);
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        (pick_end_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        for (index, local) in end_date.iter().enumerate() {
            (*local).load(function);
            (date[index]).store(function);
        }
        (end_days_local).load(function);
        (nudged_days_local).store(function);
        (end_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (did_expand_local).store(function);
        function.instruction(&Instruction::Else);
        for (index, local) in start_date.iter().enumerate() {
            (*local).load(function);
            (date[index]).store(function);
        }
        (start_days_local).load(function);
        (nudged_days_local).store(function);
        (start_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Exact distances for the once-rounded total rational.
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_sub(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            end_days_local,
            end_nanos_local,
            start_days_local,
            start_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_sub(
            start_days_local,
            start_nanos_local,
            dest_days_local,
            dest_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            start_days_local,
            start_nanos_local,
            end_days_local,
            end_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Accumulate the exact rational before one nearest-even division.
        self.emit_temporal_relative_nudge_total_exact(
            r1_local,
            dist_a_days_local,
            dist_a_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            increment_local,
            sign_local,
            total_bits_local,
            function,
        );

        for local in [
            pick_end_local,
            dist_b_nanos_local,
            dist_b_days_local,
            dist_a_nanos_local,
            dist_a_days_local,
            umode_local,
            cmp_local,
            end_date[3],
            end_date[2],
            end_date[1],
            end_date[0],
            start_date[3],
            start_date[2],
            start_date[1],
            start_date[0],
            end_nanos_local,
            end_days_local,
            start_nanos_local,
            start_days_local,
            r2_local,
            r1_local,
            shift_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// `BubbleRelativeDuration` without a time zone: after a calendar-unit
    /// nudge expands, larger units absorb the result while the nudged stamp
    /// still reaches them. Only year, month and week bubble; each adopted
    /// end zeroes the time legs. `start_unit` is the larger of the rounded
    /// unit and day.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_relative_bubble(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        sign_local: I64Local,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        iso: [I64Local; 3],
        iso_seconds_local: I64Local,
        iso_subsecond_local: I64Local,
        largest_unit_local: I64Local,
        start_unit_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let done_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let end_years_local = self.runtime_schema().reserve_i64_local(function);
        let end_months_local = self.runtime_schema().reserve_i64_local(function);
        let end_weeks_local = self.runtime_schema().reserve_i64_local(function);
        let end_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let end_span_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_span_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_days_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_sign_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(0));
        (done_local).store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        // Units bubble from just below `start_unit` down to `largest_unit`;
        // with three calendar units the cascade is explicit, descending.
        for unit in [TemporalUnit::Week, TemporalUnit::Month, TemporalUnit::Year] {
            (done_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            (start_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            // Week only bubbles toward a week largestUnit.
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
            function.instruction(&Instruction::I64Ne);
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            (date[0]).load(function);
            (end_years_local).store(function);
            (date[1]).load(function);
            (end_months_local).store(function);
            (date[2]).load(function);
            (end_weeks_local).store(function);
            function.instruction(&Instruction::I64Const(0));
            (end_days_local).store(function);
            // The bubbling unit increments by the sign; larger legs stay,
            // smaller legs clear.
            match unit {
                TemporalUnit::Year => {
                    (end_years_local).load(function);
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                    (end_years_local).store(function);
                    function.instruction(&Instruction::I64Const(0));
                    (end_months_local).store(function);
                    function.instruction(&Instruction::I64Const(0));
                    (end_weeks_local).store(function);
                }
                TemporalUnit::Month => {
                    (end_months_local).load(function);
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                    (end_months_local).store(function);
                    function.instruction(&Instruction::I64Const(0));
                    (end_weeks_local).store(function);
                }
                _ => {
                    (end_weeks_local).load(function);
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                    (end_weeks_local).store(function);
                }
            }
            for (index, local) in end_date.iter().enumerate() {
                (iso[index]).load(function);
                (*local).store(function);
            }
            self.emit_temporal_add_calendar_date(
                calendar,
                end_date[0],
                end_date[1],
                end_date[2],
                end_years_local,
                end_months_local,
                end_weeks_local,
                end_days_local,
                overflow_local,
                function,
            )?;
            self.emit_temporal_relative_epoch_span(
                end_date[0],
                end_date[1],
                end_date[2],
                iso_seconds_local,
                iso_subsecond_local,
                end_span_days_local,
                end_span_nanos_local,
                function,
            );
            self.emit_temporal_relative_span_sub(
                nudged_days_local,
                nudged_nanos_local,
                end_span_days_local,
                end_span_nanos_local,
                beyond_days_local,
                beyond_nanos_local,
                function,
            );
            // The difference normalizes its nanosecond leg nonnegative, so
            // the days decide the sign unless they are zero.
            function.instruction(&Instruction::I64Const(0));
            (beyond_sign_local).store(function);
            (beyond_days_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(-1));
            (beyond_sign_local).store(function);
            function.instruction(&Instruction::Else);
            (beyond_days_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            (beyond_days_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            (beyond_nanos_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(1));
            (beyond_sign_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(0));
            (sign_local).load(function);
            function.instruction(&Instruction::I64Sub);
            (beyond_sign_local).load(function);
            function.instruction(&Instruction::I64Ne);
            self.open_frame(ControlFrameKind::If, function);
            (end_years_local).load(function);
            (date[0]).store(function);
            (end_months_local).load(function);
            (date[1]).store(function);
            (end_weeks_local).load(function);
            (date[2]).store(function);
            (end_days_local).load(function);
            (date[3]).store(function);
            function.instruction(&Instruction::I64Const(0));
            (seconds_local).store(function);
            function.instruction(&Instruction::I64Const(0));
            (subsecond_local).store(function);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            (done_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        for local in [
            beyond_sign_local,
            beyond_nanos_local,
            beyond_days_local,
            end_span_nanos_local,
            end_span_days_local,
            end_date[2],
            end_date[1],
            end_date[0],
            end_days_local,
            end_weeks_local,
            end_months_local,
            end_years_local,
            overflow_local,
            done_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// Exact total for every completed calendar window:
    /// (|r1|*W + D*increment)/W, negated for a negative sign. Both distances
    /// and the complete numerator retain two limbs until one final division.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_relative_nudge_total_exact(
        &mut self,
        r1_local: I64Local,
        dist_a_days_local: I64Local,
        dist_a_nanos_local: I64Local,
        dist_b_days_local: I64Local,
        dist_b_nanos_local: I64Local,
        increment_local: I64Local,
        sign_local: I64Local,
        total_bits_local: I64Local,
        function: &mut Function,
    ) {
        // Completed window endpoints lie within the contextual ISO/Instant
        // domain, less than 200,000,004 days apart: W and D are < 2^74 ns.
        // Valid integral date coefficients are < 2^37 and increment < 2^30.
        // |r1|*W + D*increment is therefore < 2^112. Neither limb product
        // loses bits, and twice the division remainder is below 2^75.
        let d_hi = self.runtime_schema().reserve_i64_local(function);
        let d_lo = self.runtime_schema().reserve_i64_local(function);
        let w_hi = self.runtime_schema().reserve_i64_local(function);
        let w_lo = self.runtime_schema().reserve_i64_local(function);
        let coefficient = self.runtime_schema().reserve_i64_local(function);
        let a_hi = self.runtime_schema().reserve_i64_local(function);
        let a_lo = self.runtime_schema().reserve_i64_local(function);
        let b_hi = self.runtime_schema().reserve_i64_local(function);
        let b_lo = self.runtime_schema().reserve_i64_local(function);
        let n_hi = self.runtime_schema().reserve_i64_local(function);
        let n_lo = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_relative_span_to_u128(
            dist_a_days_local,
            dist_a_nanos_local,
            d_hi,
            d_lo,
            function,
        );
        self.emit_temporal_relative_span_to_u128(
            dist_b_days_local,
            dist_b_nanos_local,
            w_hi,
            w_lo,
            function,
        );
        (r1_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        (r1_local).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        (r1_local).load(function);
        function.instruction(&Instruction::End);
        (coefficient).store(function);
        self.emit_temporal_relative_u128_mul_coefficient(
            w_hi,
            w_lo,
            coefficient,
            a_hi,
            a_lo,
            function,
        );
        self.emit_temporal_relative_u128_mul_coefficient(
            d_hi,
            d_lo,
            increment_local,
            b_hi,
            b_lo,
            function,
        );
        self.emit_temporal_relative_u128_add(a_hi, a_lo, b_hi, b_lo, n_hi, n_lo, function);
        self.emit_temporal_relative_window_ratio(
            n_hi,
            n_lo,
            w_hi,
            w_lo,
            total_bits_local,
            function,
        );
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (total_bits_local).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        (total_bits_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [
            n_lo,
            n_hi,
            b_lo,
            b_hi,
            a_lo,
            a_hi,
            coefficient,
            w_lo,
            w_hi,
            d_lo,
            d_hi,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// A proved nonnegative calendar-window distance, without scaling its
    /// complete nanosecond count into a single i64.
    fn emit_temporal_relative_span_to_u128(
        &mut self,
        days: I64Local,
        nanos: I64Local,
        high: I64Local,
        low: I64Local,
        function: &mut Function,
    ) {
        let scale = self.runtime_schema().reserve_i64_local(function);
        let original_low = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_DAY));
        (scale).store(function);
        self.emit_temporal_relative_u64_mul_to_u128(days, scale, high, low, function);
        (low).load(function);
        (original_low).store(function);
        (low).load(function);
        (nanos).load(function);
        function.instruction(&Instruction::I64Add);
        (low).store(function);
        (high).load(function);
        (low).load(function);
        (original_low).load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        (high).store(function);
        self.runtime_schema()
            .release_i64_local(original_low, function);
        self.runtime_schema().release_i64_local(scale, function);
    }

    // The completed bounds above ensure the product fits both output limbs.
    fn emit_temporal_relative_u128_mul_coefficient(
        &mut self,
        high: I64Local,
        low: I64Local,
        coefficient: I64Local,
        out_high: I64Local,
        out_low: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_relative_u64_mul_to_u128(low, coefficient, out_high, out_low, function);
        (high).load(function);
        (coefficient).load(function);
        function.instruction(&Instruction::I64Mul);
        (out_high).load(function);
        function.instruction(&Instruction::I64Add);
        (out_high).store(function);
    }

    fn emit_temporal_relative_window_ratio(
        &mut self,
        high: I64Local,
        low: I64Local,
        divisor_high: I64Local,
        divisor_low: I64Local,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        let bit_index = self.runtime_schema().reserve_i64_local(function);
        let significand = self.runtime_schema().reserve_i64_local(function);
        let remainder_high = self.runtime_schema().reserve_i64_local(function);
        let remainder_low = self.runtime_schema().reserve_i64_local(function);
        let borrow = self.runtime_schema().reserve_i64_local(function);
        let bit = self.runtime_schema().reserve_i64_local(function);
        let bit_count = self.runtime_schema().reserve_i64_local(function);
        let exponent = self.runtime_schema().reserve_i64_local(function);
        (high).load(function);
        (low).load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::Else);
        for local in [
            significand,
            remainder_high,
            remainder_low,
            bit_count,
            exponent,
        ] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        function.instruction(&Instruction::I64Const(127));
        (bit_index).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (high).load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        (bit).store(function);
        (high).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (low).load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Or);
        (high).store(function);
        (low).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (low).store(function);
        (remainder_high).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (remainder_low).load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Or);
        (remainder_high).store(function);
        (remainder_low).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (bit).load(function);
        function.instruction(&Instruction::I64Or);
        (remainder_low).store(function);
        (remainder_high).load(function);
        (divisor_high).load(function);
        function.instruction(&Instruction::I64GtU);
        (remainder_high).load(function);
        (divisor_high).load(function);
        function.instruction(&Instruction::I64Eq);
        (remainder_low).load(function);
        (divisor_low).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        (bit).store(function);
        (bit).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (remainder_low).load(function);
        (divisor_low).load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        (borrow).store(function);
        (remainder_low).load(function);
        (divisor_low).load(function);
        function.instruction(&Instruction::I64Sub);
        (remainder_low).store(function);
        (remainder_high).load(function);
        (divisor_high).load(function);
        function.instruction(&Instruction::I64Sub);
        (borrow).load(function);
        function.instruction(&Instruction::I64Sub);
        (remainder_high).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (bit_count).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (bit).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (bit_index).load(function);
        (exponent).store(function);
        function.instruction(&Instruction::I64Const(1));
        (bit_count).store(function);
        function.instruction(&Instruction::I64Const(1));
        (significand).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (bit).load(function);
        function.instruction(&Instruction::I64Or);
        (significand).store(function);
        (bit_count).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (bit_count).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (bit_count).load(function);
        function.instruction(&Instruction::I64Const(54));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        (bit_index).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (bit_index).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        (bit).store(function);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        (significand).store(function);
        (bit).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        (remainder_low).load(function);
        (remainder_high).load(function);
        function.instruction(&Instruction::I64Or);
        (high).load(function);
        function.instruction(&Instruction::I64Or);
        (low).load(function);
        function.instruction(&Instruction::I64Or);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (significand).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (significand).load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        (exponent).load(function);
        function.instruction(&Instruction::I64Const(971)); // 1023 - 52
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ReinterpretF64);
        (output_bits).store(function);
        for local in [
            exponent,
            bit_count,
            bit,
            borrow,
            remainder_low,
            remainder_high,
            significand,
            bit_index,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Unsigned 64-by-64-bit multiplication to a 128-bit `(high, low)` pair.
    /// The 32-bit split keeps every partial product in `u64`.
    fn emit_temporal_relative_u64_mul_to_u128(
        &mut self,
        left_local: I64Local,
        right_local: I64Local,
        high_local: I64Local,
        low_local: I64Local,
        function: &mut Function,
    ) {
        let left_low_local = self.runtime_schema().reserve_i64_local(function);
        let left_high_local = self.runtime_schema().reserve_i64_local(function);
        let right_low_local = self.runtime_schema().reserve_i64_local(function);
        let right_high_local = self.runtime_schema().reserve_i64_local(function);
        let middle_local = self.runtime_schema().reserve_i64_local(function);
        (left_local).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        (left_low_local).store(function);
        (left_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        (left_high_local).store(function);
        (right_local).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        (right_low_local).store(function);
        (right_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        (right_high_local).store(function);
        // `low` is the low product plus the middle products shifted up.
        (left_low_local).load(function);
        (right_low_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (low_local).store(function);
        (left_low_local).load(function);
        (right_high_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (left_high_local).load(function);
        (right_low_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (middle_local).store(function);
        (low_local).load(function);
        (middle_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        (low_local).store(function);
        // `high` is the high product plus the middle carry plus both
        // overflow carries. Each carry test compares against one addend.
        (left_high_local).load(function);
        (right_high_local).load(function);
        function.instruction(&Instruction::I64Mul);
        (middle_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        (high_local).store(function);
        (middle_local).load(function);
        (left_low_local).load(function);
        (right_high_local).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        (high_local).load(function);
        function.instruction(&Instruction::I64Add);
        (high_local).store(function);
        (low_local).load(function);
        (middle_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        (high_local).load(function);
        function.instruction(&Instruction::I64Add);
        (high_local).store(function);
        for local in [
            middle_local,
            right_high_local,
            right_low_local,
            left_high_local,
            left_low_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Unsigned 128-bit addition. The outputs must not alias the inputs:
    /// the carry test reads the original low legs after the sum lands.
    fn emit_temporal_relative_u128_add(
        &mut self,
        left_high_local: I64Local,
        left_low_local: I64Local,
        right_high_local: I64Local,
        right_low_local: I64Local,
        high_local: I64Local,
        low_local: I64Local,
        function: &mut Function,
    ) {
        (left_low_local).load(function);
        (right_low_local).load(function);
        function.instruction(&Instruction::I64Add);
        (low_local).store(function);
        (left_high_local).load(function);
        (right_high_local).load(function);
        function.instruction(&Instruction::I64Add);
        (low_local).load(function);
        (left_low_local).load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        (high_local).store(function);
    }

    /// `additionalShift ? base + step : base` for the nudge window's `r1`.
    fn emit_temporal_relative_shifted_base(
        &mut self,
        base_local: I64Local,
        step_local: I64Local,
        additional_shift_local: I64Local,
        out_local: I64Local,
        function: &mut Function,
    ) {
        (additional_shift_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (base_local).load(function);
        function.instruction(&Instruction::Else);
        (base_local).load(function);
        (step_local).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        (out_local).store(function);
    }

    /// `TemporalDurationFromInternal`: balance the time legs under the
    /// largestUnit, add the date duration's days onto the day field, pin the
    /// year/month/week fields, and create the validated result.
    pub(super) fn emit_temporal_relative_from_internal(
        &mut self,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        largest_unit_local: I64Local,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_relative_balance_internal(
            date,
            seconds_local,
            subsecond_local,
            largest_unit_local,
            fields,
            function,
        )?;
        self.emit_create_temporal_duration(fields, function)?;
        Ok(())
    }

    /// The field half of `TemporalDurationFromInternal`: balance and combine
    /// without creating the result, for callers (like `toString`) that print
    /// the fields instead of returning a duration.
    pub(super) fn emit_temporal_relative_balance_internal(
        &mut self,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        largest_unit_local: I64Local,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // `BalanceTimeDuration` takes the date duration's days as its day
        // leg, so a largestUnit below days folds them into hours, minutes
        // and seconds instead of stranding them on the day field. The days
        // leg stays within the validated day bound (about 1.04e11), so the
        // `i64` scaling cannot overflow. This clobbers `seconds_local`;
        // both callers are done with it.
        (date[3]).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (seconds_local).store(function);
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            largest_unit_local,
            fields,
            function,
        )?;
        for (source, unit) in [
            (date[0], TemporalUnit::Year),
            (date[1], TemporalUnit::Month),
            (date[2], TemporalUnit::Week),
        ] {
            self.emit_temporal_duration_set_integer_field(fields, unit, source, function);
        }
        Ok(())
    }

    /// `DateDurationDays`: resolve a date duration against a plain
    /// `relativeTo` to whole days. A years/months/weeks leg of zero skips
    /// the calendar add; otherwise the epoch-day shift plus the day leg is
    /// the answer.
    pub(super) fn emit_temporal_relative_date_duration_days(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        date: [I64Local; 4],
        iso: [I64Local; 3],
        out_days_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let later = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let zero_local = self.runtime_schema().reserve_i64_local(function);
        let epoch_days_local = self.runtime_schema().reserve_i64_local(function);
        let later_epoch_days_local = self.runtime_schema().reserve_i64_local(function);

        (date[0]).load(function);
        (date[1]).load(function);
        function.instruction(&Instruction::I64Or);
        (date[2]).load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (date[3]).load(function);
        (out_days_local).store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (zero_local).store(function);
        for (index, local) in later.iter().enumerate() {
            (iso[index]).load(function);
            (*local).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            later[0],
            later[1],
            later[2],
            date[0],
            date[1],
            date[2],
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            iso[0],
            iso[1],
            iso[2],
            epoch_days_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            later[0],
            later[1],
            later[2],
            later_epoch_days_local,
            function,
        );
        (later_epoch_days_local).load(function);
        (epoch_days_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (date[3]).load(function);
        function.instruction(&Instruction::I64Add);
        (out_days_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            later_epoch_days_local,
            epoch_days_local,
            zero_local,
            overflow_local,
            later[2],
            later[1],
            later[0],
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// `Duration.prototype.round` with a present `relativeTo`: add the
    /// duration to the relativeTo instant on the wall clock, difference the
    /// two datetimes with rounding, and build the result. Zoned and plain
    /// `relativeTo` values share this path; the calendar-aware math only ever
    /// sees the resolved wall clock.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_duration_round_plain_math(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        fields: &TemporalDurationFields,
        relative_year_local: I64Local,
        relative_month_local: I64Local,
        relative_day_local: I64Local,
        relative_seconds_local: I64Local,
        relative_subsecond_local: I64Local,
        smallest_local: I64Local,
        largest_local: I64Local,
        increment_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let rel_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let rel_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_days_local = self.runtime_schema().reserve_i64_local(function);
        let target_days_local = self.runtime_schema().reserve_i64_local(function);
        let target_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let target_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let target = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let origin_days_local = self.runtime_schema().reserve_i64_local(function);
        let origin_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let dest_days_local = self.runtime_schema().reserve_i64_local(function);
        let dest_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_days_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let did_expand_local = self.runtime_schema().reserve_i64_local(function);
        let total_ignored_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let cmp_local = self.runtime_schema().reserve_i64_local(function);
        let day_const_local = self.runtime_schema().reserve_i64_local(function);
        let iso = [
            relative_year_local,
            relative_month_local,
            relative_day_local,
        ];

        self.emit_temporal_relative_internal_from_fields(
            fields,
            date,
            rel_seconds_local,
            rel_subsecond_local,
            function,
        );
        // The target time-of-day starts from the relativeTo wall time
        // (midnight for plain dates), so a blank duration lands exactly on
        // the relativeTo instant.
        (rel_seconds_local).load(function);
        (relative_seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (rel_seconds_local).store(function);
        (rel_subsecond_local).load(function);
        (relative_subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (rel_subsecond_local).store(function);
        self.emit_temporal_relative_split_time(
            rel_seconds_local,
            rel_subsecond_local,
            overflow_days_local,
            target_seconds_local,
            target_subsecond_local,
            function,
        );
        (date[3]).load(function);
        (overflow_days_local).load(function);
        function.instruction(&Instruction::I64Add);
        (target_days_local).store(function);
        for (index, local) in target.iter().enumerate() {
            (iso[index]).load(function);
            (*local).store(function);
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        self.emit_temporal_add_calendar_date(
            calendar,
            target[0],
            target[1],
            target[2],
            date[0],
            date[1],
            date[2],
            target_days_local,
            overflow_local,
            function,
        )?;
        // An empty difference rounds to the zero duration without consulting
        // the rounding machinery.
        self.emit_temporal_compare_iso_date(iso, target, cmp_local, function);
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (relative_seconds_local).load(function);
        (target_seconds_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        (relative_subsecond_local).load(function);
        (target_subsecond_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        for local in [
            date[0],
            date[1],
            date[2],
            date[3],
            rel_seconds_local,
            rel_subsecond_local,
        ] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_difference_iso_date_time(
            calendar,
            iso,
            relative_seconds_local,
            relative_subsecond_local,
            target,
            target_seconds_local,
            target_subsecond_local,
            largest_local,
            date,
            rel_seconds_local,
            rel_subsecond_local,
            function,
        )?;
        self.emit_temporal_relative_epoch_span(
            relative_year_local,
            relative_month_local,
            relative_day_local,
            relative_seconds_local,
            relative_subsecond_local,
            origin_days_local,
            origin_nanos_local,
            function,
        );
        self.emit_temporal_relative_epoch_span(
            target[0],
            target[1],
            target[2],
            target_seconds_local,
            target_subsecond_local,
            dest_days_local,
            dest_nanos_local,
            function,
        );
        // `DifferencePlainDateTimeWithRounding` range-checks both
        // datetimes; the zoned twin validates the target epoch instead
        // (`AddZonedDateTime`). Both sit after the empty shortcut above,
        // which is the early return the limit tests pin.
        self.emit_temporal_relative_reject_bad_datetime_span(
            origin_days_local,
            origin_nanos_local,
            function,
        )?;
        self.emit_temporal_relative_reject_bad_datetime_span(
            dest_days_local,
            dest_nanos_local,
            function,
        )?;
        // Nanosecond rounding at increment 1 returns the difference as is.
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Eq);
        (increment_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_internal_sign(
            date,
            rel_seconds_local,
            rel_subsecond_local,
            sign_local,
            function,
        );
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_nudge_to_calendar_unit(
            calendar,
            sign_local,
            date,
            origin_days_local,
            origin_nanos_local,
            dest_days_local,
            dest_nanos_local,
            iso,
            relative_seconds_local,
            relative_subsecond_local,
            increment_local,
            smallest_local,
            mode_local,
            nudged_days_local,
            nudged_nanos_local,
            did_expand_local,
            total_ignored_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        (rel_seconds_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (rel_subsecond_local).store(function);
        // An expanded calendar nudge bubbles toward the largestUnit; a week
        // smallestUnit never bubbles further.
        (did_expand_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_bubble(
            calendar,
            sign_local,
            date,
            rel_seconds_local,
            rel_subsecond_local,
            nudged_days_local,
            nudged_nanos_local,
            iso,
            relative_seconds_local,
            relative_subsecond_local,
            largest_local,
            smallest_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_nudge_to_day_or_time(
            date,
            rel_seconds_local,
            rel_subsecond_local,
            dest_days_local,
            dest_nanos_local,
            largest_local,
            increment_local,
            smallest_local,
            mode_local,
            nudged_days_local,
            nudged_nanos_local,
            did_expand_local,
            function,
        );
        (did_expand_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        (smallest_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        (day_const_local).store(function);
        self.emit_temporal_relative_bubble(
            calendar,
            sign_local,
            date,
            rel_seconds_local,
            rel_subsecond_local,
            nudged_days_local,
            nudged_nanos_local,
            iso,
            relative_seconds_local,
            relative_subsecond_local,
            largest_local,
            day_const_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_from_internal(
            date,
            rel_seconds_local,
            rel_subsecond_local,
            largest_local,
            fields,
            function,
        )?;

        for local in [
            day_const_local,
            cmp_local,
            overflow_local,
            total_ignored_local,
            did_expand_local,
            sign_local,
            nudged_nanos_local,
            nudged_days_local,
            dest_nanos_local,
            dest_days_local,
            origin_nanos_local,
            origin_days_local,
            target[2],
            target[1],
            target[0],
            target_subsecond_local,
            target_seconds_local,
            target_days_local,
            overflow_days_local,
            rel_subsecond_local,
            rel_seconds_local,
            date[3],
            date[2],
            date[1],
            date[0],
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// `Duration.prototype.total` with a present `relativeTo`:
    /// `DifferencePlainDateTimeWithTotal` between the relativeTo instant and
    /// the duration's target, projected to one `f64`. Calendar units nudge
    /// with truncation; time units divide the exact nanosecond count.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_duration_total_plain_math(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        fields: &TemporalDurationFields,
        relative_year_local: I64Local,
        relative_month_local: I64Local,
        relative_day_local: I64Local,
        relative_seconds_local: I64Local,
        relative_subsecond_local: I64Local,
        unit_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let time_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let time_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let overflow_days_local = self.runtime_schema().reserve_i64_local(function);
        let target_days_local = self.runtime_schema().reserve_i64_local(function);
        let target_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let target_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let target = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let overflow_local = self.runtime_schema().reserve_i64_local(function);
        let cmp_local = self.runtime_schema().reserve_i64_local(function);
        let origin_days_local = self.runtime_schema().reserve_i64_local(function);
        let origin_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let dest_days_local = self.runtime_schema().reserve_i64_local(function);
        let dest_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_days_local = self.runtime_schema().reserve_i64_local(function);
        let nudged_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let sign_local = self.runtime_schema().reserve_i64_local(function);
        let did_expand_local = self.runtime_schema().reserve_i64_local(function);
        let total_bits_local = self.runtime_schema().reserve_i64_local(function);
        let trunc_local = self.runtime_schema().reserve_i64_local(function);
        let one_local = self.runtime_schema().reserve_i64_local(function);
        let negative_local = self.runtime_schema().reserve_i64_local(function);
        let num_high_local = self.runtime_schema().reserve_i64_local(function);
        let num_low_local = self.runtime_schema().reserve_i64_local(function);
        let divisor_local = self.runtime_schema().reserve_i64_local(function);
        let iso = [
            relative_year_local,
            relative_month_local,
            relative_day_local,
        ];

        self.emit_temporal_relative_internal_from_fields(
            fields,
            date,
            time_seconds_local,
            time_subsecond_local,
            function,
        );
        // The target time-of-day starts from the relativeTo wall time
        // (midnight for plain dates), so a blank duration lands exactly on
        // the relativeTo instant.
        (time_seconds_local).load(function);
        (relative_seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (time_seconds_local).store(function);
        (time_subsecond_local).load(function);
        (relative_subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (time_subsecond_local).store(function);
        self.emit_temporal_relative_split_time(
            time_seconds_local,
            time_subsecond_local,
            overflow_days_local,
            target_seconds_local,
            target_subsecond_local,
            function,
        );
        (date[3]).load(function);
        (overflow_days_local).load(function);
        function.instruction(&Instruction::I64Add);
        (target_days_local).store(function);
        for (index, local) in target.iter().enumerate() {
            (iso[index]).load(function);
            (*local).store(function);
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow_local).store(function);
        self.emit_temporal_add_calendar_date(
            calendar,
            target[0],
            target[1],
            target[2],
            date[0],
            date[1],
            date[2],
            target_days_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_compare_iso_date(iso, target, cmp_local, function);
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (relative_seconds_local).load(function);
        (target_seconds_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        (relative_subsecond_local).load(function);
        (target_subsecond_local).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        let published = self.runtime_schema().reserve_i64_local(function);
        published.store(function);
        self.completion().value().set_number(published, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        self.runtime_schema().release_i64_local(published, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_difference_iso_date_time(
            calendar,
            iso,
            relative_seconds_local,
            relative_subsecond_local,
            target,
            target_seconds_local,
            target_subsecond_local,
            unit_local,
            date,
            time_seconds_local,
            time_subsecond_local,
            function,
        )?;
        self.emit_temporal_relative_epoch_span(
            relative_year_local,
            relative_month_local,
            relative_day_local,
            relative_seconds_local,
            relative_subsecond_local,
            origin_days_local,
            origin_nanos_local,
            function,
        );
        self.emit_temporal_relative_epoch_span(
            target[0],
            target[1],
            target[2],
            target_seconds_local,
            target_subsecond_local,
            dest_days_local,
            dest_nanos_local,
            function,
        );
        // `DifferencePlainDateTimeWithTotal` range-checks both datetimes;
        // the zoned twin validates the target epoch instead
        // (`AddZonedDateTime`). Both sit after the empty shortcut above.
        self.emit_temporal_relative_reject_bad_datetime_span(
            origin_days_local,
            origin_nanos_local,
            function,
        )?;
        self.emit_temporal_relative_reject_bad_datetime_span(
            dest_days_local,
            dest_nanos_local,
            function,
        )?;
        // A nanosecond total is the time legs' exact nanosecond count.
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_abs_time(
            time_seconds_local,
            time_subsecond_local,
            negative_local,
            function,
        );
        self.emit_temporal_relative_seconds_to_u128(
            time_seconds_local,
            time_subsecond_local,
            num_high_local,
            num_low_local,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        (divisor_local).store(function);
        self.emit_u128_div_to_f64(
            num_high_local,
            num_low_local,
            divisor_local,
            total_bits_local,
            function,
        );
        (negative_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        (total_bits_local).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        (total_bits_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        // Calendar units nudge with truncation at increment 1.
        (unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_internal_sign(
            date,
            time_seconds_local,
            time_subsecond_local,
            sign_local,
            function,
        );
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        (trunc_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (one_local).store(function);
        self.emit_temporal_relative_nudge_to_calendar_unit(
            calendar,
            sign_local,
            date,
            origin_days_local,
            origin_nanos_local,
            dest_days_local,
            dest_nanos_local,
            iso,
            relative_seconds_local,
            relative_subsecond_local,
            one_local,
            unit_local,
            trunc_local,
            nudged_days_local,
            nudged_nanos_local,
            did_expand_local,
            total_bits_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        // Day and time units fold the day leg into the time and divide.
        (time_seconds_local).load(function);
        (date[3]).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (time_seconds_local).store(function);
        self.emit_temporal_relative_abs_time(
            time_seconds_local,
            time_subsecond_local,
            negative_local,
            function,
        );
        self.emit_temporal_relative_seconds_to_u128(
            time_seconds_local,
            time_subsecond_local,
            num_high_local,
            num_low_local,
            function,
        );
        for (unit, length) in [
            (TemporalUnit::Day, NANOSECONDS_PER_DAY),
            (TemporalUnit::Hour, 3_600_000_000_000),
            (TemporalUnit::Minute, 60_000_000_000),
            (TemporalUnit::Second, 1_000_000_000),
            (TemporalUnit::Millisecond, 1_000_000),
            (TemporalUnit::Microsecond, 1_000),
        ] {
            (unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(length));
            (divisor_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_u128_div_to_f64(
            num_high_local,
            num_low_local,
            divisor_local,
            total_bits_local,
            function,
        );
        (negative_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        (total_bits_local).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        (total_bits_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (total_bits_local).load(function);
        let published = self.runtime_schema().reserve_i64_local(function);
        published.store(function);
        self.completion().value().set_number(published, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        self.runtime_schema().release_i64_local(published, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            divisor_local,
            num_low_local,
            num_high_local,
            negative_local,
            one_local,
            trunc_local,
            total_bits_local,
            did_expand_local,
            sign_local,
            nudged_nanos_local,
            nudged_days_local,
            dest_nanos_local,
            dest_days_local,
            origin_nanos_local,
            origin_days_local,
            cmp_local,
            overflow_local,
            target[2],
            target[1],
            target[0],
            target_subsecond_local,
            target_seconds_local,
            target_days_local,
            overflow_days_local,
            time_subsecond_local,
            time_seconds_local,
            date[3],
            date[2],
            date[1],
            date[0],
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// Negate a `(seconds, subsecond)` pair in place when it is negative,
    /// reporting the original sign in `negative_local` (0 or 1).
    pub(super) fn emit_temporal_relative_abs_time(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        negative_local: I64Local,
        function: &mut Function,
    ) {
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        (negative_local).store(function);
        (negative_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        for local in [seconds_local, subsecond_local] {
            function.instruction(&Instruction::I64Const(0));
            (local).load(function);
            function.instruction(&Instruction::I64Sub);
            (local).store(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// A nonnegative `(seconds, subsecond)` pair to a 128-bit nanosecond
    /// count. The 32-bit split keeps every partial product in `u64`.
    pub(super) fn emit_temporal_relative_seconds_to_u128(
        &mut self,
        seconds_local: I64Local,
        subsecond_local: I64Local,
        high_local: I64Local,
        low_local: I64Local,
        function: &mut Function,
    ) {
        let hi32_local = self.runtime_schema().reserve_i64_local(function);
        let lo32_local = self.runtime_schema().reserve_i64_local(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (hi32_local).store(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (lo32_local).store(function);
        (hi32_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        (lo32_local).load(function);
        function.instruction(&Instruction::I64Add);
        (low_local).store(function);
        (low_local).load(function);
        (hi32_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        (hi32_local).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        (high_local).store(function);
        self.runtime_schema()
            .release_i64_local(lo32_local, function);
        self.runtime_schema()
            .release_i64_local(hi32_local, function);
    }

    /// `Duration.compare` with calendar units and a present `relativeTo`:
    /// each date duration resolves to whole days against the relativeTo
    /// date, the day counts fold into the times at 24 hours, and the two
    /// `(seconds, subsecond)` pairs compare seconds-major.
    pub(super) fn emit_temporal_duration_compare_plain_math(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        one_fields: &TemporalDurationFields,
        two_fields: &TemporalDurationFields,
        relative_year_local: I64Local,
        relative_month_local: I64Local,
        relative_day_local: I64Local,
        out_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let one_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let two_date = [
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
            self.runtime_schema().reserve_i64_local(function),
        ];
        let one_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let one_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let two_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let two_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let one_days_local = self.runtime_schema().reserve_i64_local(function);
        let two_days_local = self.runtime_schema().reserve_i64_local(function);
        let iso = [
            relative_year_local,
            relative_month_local,
            relative_day_local,
        ];

        self.emit_temporal_relative_internal_from_fields(
            one_fields,
            one_date,
            one_seconds_local,
            one_subsecond_local,
            function,
        );
        self.emit_temporal_relative_internal_from_fields(
            two_fields,
            two_date,
            two_seconds_local,
            two_subsecond_local,
            function,
        );
        self.emit_temporal_relative_date_duration_days(
            calendar,
            one_date,
            iso,
            one_days_local,
            function,
        )?;
        self.emit_temporal_relative_date_duration_days(
            calendar,
            two_date,
            iso,
            two_days_local,
            function,
        )?;
        (one_seconds_local).load(function);
        (one_days_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (one_seconds_local).store(function);
        (two_seconds_local).load(function);
        (two_days_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (two_seconds_local).store(function);
        // Unbalanced totals obey the same range as constructed durations:
        // a day count past about 1.04e11 pushes the seconds total past
        // 2^53 (`TEMPORAL_DURATION_MAXIMUM_SECONDS`), and the comparison
        // throws instead of ordering the overflow.
        for seconds in [one_seconds_local, two_seconds_local] {
            (seconds).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            (seconds).load(function);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::Else);
            (seconds).load(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(9_007_199_254_740_992));
            function.instruction(&Instruction::I64GeS);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError, RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_FIELDS_MUST_NOT_EXCEED_THE_SUPPORTED_RANGE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(0));
        (out_local).store(function);
        (one_seconds_local).load(function);
        (two_seconds_local).load(function);
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (one_subsecond_local).load(function);
        (two_subsecond_local).load(function);
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(-1));
        (out_local).store(function);
        function.instruction(&Instruction::Else);
        (one_subsecond_local).load(function);
        (two_subsecond_local).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (one_seconds_local).load(function);
        (two_seconds_local).load(function);
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        (out_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        for local in [
            two_days_local,
            one_days_local,
            two_subsecond_local,
            two_seconds_local,
            one_subsecond_local,
            one_seconds_local,
            two_date[3],
            two_date[2],
            two_date[1],
            two_date[0],
            one_date[3],
            one_date[2],
            one_date[1],
            one_date[0],
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// The rounding sign: -1 for a negative internal duration, +1
    /// otherwise (a zero duration rounds upward, matching the spec's
    /// `InternalDurationSign < 0 ? -1 : 1`).
    pub(super) fn emit_temporal_relative_internal_sign(
        &mut self,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        out_local: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_relative_time_sign(seconds_local, subsecond_local, out_local, function);
        for local in [date[3], date[2], date[1], date[0]] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(-1));
            (out_local).store(function);
            function.instruction(&Instruction::Else);
            (local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(1));
            (out_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (out_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (out_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    // Plain relativeTo has a retained branded ISO date and actual calendar.
    // Current admitted calendar arithmetic is exhaustively Gregorian in the
    // shared zoned authority; no JS getters or fixed-offset zone are replayed.
    pub(super) fn emit_temporal_duration_round_plain_relative(
        &mut self,
        fields: &TemporalDurationFields,
        relative: &TemporalPlainRelativeContextLocals,
        options: &CompletedTemporalDurationRoundOptionsLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let zero = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        (zero).store(function);
        let date = relative.date().fields();
        self.emit_temporal_duration_round_plain_math(
            relative.calendar(),
            fields,
            date[0],
            date[1],
            date[2],
            zero,
            zero,
            options.smallest_unit(),
            options.largest_unit(),
            options.rounding_increment(),
            options.rounding_mode(),
            function,
        )?;
        self.runtime_schema().release_i64_local(zero, function);
        Ok(())
    }
    pub(super) fn emit_temporal_duration_total_plain_relative(
        &mut self,
        fields: &TemporalDurationFields,
        relative: &TemporalPlainRelativeContextLocals,
        unit: &TemporalDurationTotalUnitLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let zero = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        (zero).store(function);
        let date = relative.date().fields();
        self.emit_temporal_duration_total_plain_math(
            relative.calendar(),
            fields,
            date[0],
            date[1],
            date[2],
            zero,
            zero,
            unit.local(),
            function,
        )?;
        self.runtime_schema().release_i64_local(zero, function);
        Ok(())
    }
    pub(super) fn emit_temporal_duration_compare_plain_relative(
        &mut self,
        one: &TemporalDurationFields,
        two: &TemporalDurationFields,
        relative: &TemporalPlainRelativeContextLocals,
        out: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let date = relative.date().fields();
        self.emit_temporal_duration_compare_plain_math(
            relative.calendar(),
            one,
            two,
            date[0],
            date[1],
            date[2],
            out,
            function,
        )?;
        Ok(())
    }
}
