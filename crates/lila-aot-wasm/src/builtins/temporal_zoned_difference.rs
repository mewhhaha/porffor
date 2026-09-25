//! Zoned arithmetic on exact times: `AddZonedDateTime`,
//! `DifferenceZonedDateTime` and the time-zone half of
//! `RoundRelativeDuration` / `TotalRelativeDuration`
//! (`ComputeNudgeWindow`, `NudgeToCalendarUnit`, `NudgeToZonedTime`,
//! `BubbleRelativeDuration`).
//!
//! Every wall-clock-to-exact step is `GetEpochNanosecondsFor(…, compatible)`
//! and every exact-to-wall-clock step is `GetISODateTimeFor`, both through the
//! time-zone kernel, so a day may be 23, 24 or 25 hours long and a month's
//! window follows the zone's transitions. The ISO calendar supplies the date
//! arithmetic, as it does for the plain types.
//!
//! Exact times are [`TemporalExactTime`] pairs normalized to a remainder in
//! `[0, 10^9)`; time durations are sign-consistent `(seconds, subsecond)`
//! pairs with `|subsecond| < 10^9`.

use super::super::*;
use super::temporal_duration::TemporalExactDivisor;
use super::temporal_options::{TemporalOverflow, TemporalRoundingMode, TemporalUnit};
use super::temporal_time_zone::TemporalDisambiguationSource;

/// An exact time: floored whole epoch seconds and a remainder in `[0, 10^9)`.
#[derive(Clone, Copy)]
pub(super) struct TemporalExactTime {
    pub(super) seconds: u32,
    pub(super) subsecond: u32,
}

/// An Internal Duration Record: the date part as `[years, months, weeks,
/// days]` and the time part as a sign-consistent pair.
#[derive(Clone, Copy)]
pub(super) struct TemporalInternalDuration {
    pub(super) date: [u32; 4],
    pub(super) time_seconds: u32,
    pub(super) time_subsecond: u32,
}

const CALENDAR_UNITS: [TemporalUnit; 4] = [
    TemporalUnit::Year,
    TemporalUnit::Month,
    TemporalUnit::Week,
    TemporalUnit::Day,
];

impl<'a> FunctionBuilder<'a> {
    pub(super) fn reserve_temporal_exact_time(&mut self) -> TemporalExactTime {
        TemporalExactTime {
            seconds: self.reserve_temp_local(),
            subsecond: self.reserve_temp_local(),
        }
    }

    pub(super) fn release_temporal_exact_time(&mut self, time: TemporalExactTime) {
        self.release_temp_local(time.subsecond);
        self.release_temp_local(time.seconds);
    }

    pub(super) fn reserve_temporal_internal_duration(&mut self) -> TemporalInternalDuration {
        TemporalInternalDuration {
            date: [
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
            ],
            time_seconds: self.reserve_temp_local(),
            time_subsecond: self.reserve_temp_local(),
        }
    }

    pub(super) fn release_temporal_internal_duration(&mut self, duration: TemporalInternalDuration) {
        self.release_temp_local(duration.time_subsecond);
        self.release_temp_local(duration.time_seconds);
        for local in duration.date.into_iter().rev() {
            self.release_temp_local(local);
        }
    }

    fn emit_copy_locals(&mut self, sources: &[u32], destinations: &[u32], function: &mut Function) {
        for (source, destination) in sources.iter().zip(destinations) {
            function.instruction(&Instruction::LocalGet(*source));
            function.instruction(&Instruction::LocalSet(*destination));
        }
    }

    /// `left - right` of two exact times as a sign-consistent pair.
    fn emit_temporal_exact_difference(
        &mut self,
        left: TemporalExactTime,
        right: TemporalExactTime,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        for (left, right, destination) in [
            (left.seconds, right.seconds, seconds_local),
            (left.subsecond, right.subsecond, subsecond_local),
        ] {
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(destination));
        }
        self.emit_temporal_duration_renormalize(seconds_local, subsecond_local, function);
    }

    /// -1, 0 or 1: the sign of a sign-consistent pair.
    fn emit_temporal_pair_sign(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        for (value, first) in [(seconds_local, true), (subsecond_local, false)] {
            if !first {
                function.instruction(&Instruction::LocalGet(output_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            function.instruction(&Instruction::LocalGet(value));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(value));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(output_local));
            if !first {
                function.instruction(&Instruction::End);
            }
        }
    }

    /// Leaves an `i32`: `left <= right` for two normalized exact times.
    fn emit_temporal_exact_at_most_i32(
        &mut self,
        left: TemporalExactTime,
        right: TemporalExactTime,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(left.seconds));
        function.instruction(&Instruction::LocalGet(right.seconds));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(left.seconds));
        function.instruction(&Instruction::LocalGet(right.seconds));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(left.subsecond));
        function.instruction(&Instruction::LocalGet(right.subsecond));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    /// Leaves an `i32`: whether a date duration is all zero.
    fn emit_temporal_date_duration_is_zero_i32(&mut self, date: &[u32; 4], function: &mut Function) {
        function.instruction(&Instruction::LocalGet(date[0]));
        for local in &date[1..] {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
    }

    /// `InternalDurationSign(duration) < 0 ? -1 : 1`.
    fn emit_temporal_internal_duration_direction(
        &mut self,
        duration: &TemporalInternalDuration,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(output_local));
        for local in duration
            .date
            .iter()
            .copied()
            .chain([duration.time_seconds, duration.time_subsecond])
        {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
    }

    /// `GetEpochNanosecondsFor(timeZone, CombineISODateAndTimeRecord(date,
    /// time of wall), compatible)`.
    fn emit_temporal_zoned_epoch_for_date_at_wall_time(
        &mut self,
        time_zone_payload_local: u32,
        date: [u32; 3],
        wall: &[u32; 9],
        output: TemporalExactTime,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = [
            date[0], date[1], date[2], wall[3], wall[4], wall[5], wall[6], wall[7], wall[8],
        ];
        self.emit_temporal_epoch_for_iso_date_time(
            time_zone_payload_local,
            &fields,
            TemporalDisambiguationSource::Compatible,
            output.seconds,
            output.subsecond,
            function,
        )
    }

    /// `AddZonedDateTime(epochNanoseconds, timeZone, calendar, duration,
    /// overflow)`: the date part moves the wall-clock date, the time part is
    /// added in exact time.
    pub(super) fn emit_temporal_add_zoned_date_time(
        &mut self,
        time_zone_payload_local: u32,
        start: TemporalExactTime,
        duration: &TemporalInternalDuration,
        overflow_local: u32,
        target: TemporalExactTime,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let wall = self.reserve_temporal_plain_date_time_field_locals();
        self.emit_temporal_date_duration_is_zero_i32(&duration.date, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `AddInstant(epochNanoseconds, duration.[[Time]])`.
        self.emit_copy_locals(
            &[start.seconds, start.subsecond],
            &[target.seconds, target.subsecond],
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_iso_date_time_for(
            time_zone_payload_local,
            start.seconds,
            start.subsecond,
            &wall,
            function,
        )?;
        self.emit_temporal_add_iso_date(
            wall[0],
            wall[1],
            wall[2],
            duration.date[0],
            duration.date[1],
            duration.date[2],
            duration.date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_require_iso_date_time_within_limits(&wall, function)?;
        self.emit_temporal_epoch_for_iso_date_time(
            time_zone_payload_local,
            &wall,
            TemporalDisambiguationSource::Compatible,
            target.seconds,
            target.subsecond,
            function,
        )?;
        function.instruction(&Instruction::End);
        for (target_local, time_local) in [
            (target.seconds, duration.time_seconds),
            (target.subsecond, duration.time_subsecond),
        ] {
            function.instruction(&Instruction::LocalGet(target_local));
            function.instruction(&Instruction::LocalGet(time_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(target_local));
        }
        self.emit_temporal_normalize_seconds_and_subseconds(
            target.seconds,
            target.subsecond,
            function,
        );
        self.emit_temporal_require_valid_epoch(target.seconds, target.subsecond, function)?;
        self.release_temporal_plain_date_time_field_locals(wall);
        Ok(())
    }

    /// `DifferenceZonedDateTime(ns1, ns2, timeZone, calendar, largestUnit)`,
    /// with `start_wall` holding `GetISODateTimeFor(timeZone, ns1)`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_difference_zoned_date_time(
        &mut self,
        time_zone_payload_local: u32,
        origin: TemporalExactTime,
        destination: TemporalExactTime,
        start_wall: &[u32; 9],
        largest_unit_local: u32,
        output: &TemporalInternalDuration,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let end_wall = self.reserve_temporal_plain_date_time_field_locals();
        let intermediate_date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let intermediate = self.reserve_temporal_exact_time();
        let compare_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let maximum_local = self.reserve_temp_local();
        let correction_local = self.reserve_temp_local();
        let success_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let start_time_local = self.reserve_temp_local();
        let end_time_local = self.reserve_temp_local();
        let time_sign_local = self.reserve_temp_local();
        let date_largest_local = self.reserve_temp_local();

        for local in output
            .date
            .iter()
            .copied()
            .chain([output.time_seconds, output.time_subsecond])
        {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        // Step 1: equal instants are the zero duration.
        function.instruction(&Instruction::LocalGet(origin.seconds));
        function.instruction(&Instruction::LocalGet(destination.seconds));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(origin.subsecond));
        function.instruction(&Instruction::LocalGet(destination.subsecond));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_iso_date_time_for(
            time_zone_payload_local,
            destination.seconds,
            destination.subsecond,
            &end_wall,
            function,
        )?;
        self.emit_temporal_compare_iso_date(
            [start_wall[0], start_wall[1], start_wall[2]],
            [end_wall[0], end_wall[1], end_wall[2]],
            compare_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(compare_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Step 4: the same wall-clock date is a pure time difference.
        self.emit_temporal_exact_difference(
            destination,
            origin,
            output.time_seconds,
            output.time_subsecond,
            function,
        );
        function.instruction(&Instruction::Else);
        // Steps 5-9: the direction the day correction moves.
        self.emit_temporal_exact_difference(
            destination,
            origin,
            output.time_seconds,
            output.time_subsecond,
            function,
        );
        self.emit_temporal_pair_sign(
            output.time_seconds,
            output.time_subsecond,
            time_sign_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(time_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(maximum_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(correction_local));
        // Step 10: `DifferenceTime(start time, end time)`.
        self.emit_temporal_plain_time_total_nanoseconds(
            &Self::temporal_plain_date_time_time_locals(start_wall),
            start_time_local,
            function,
        );
        self.emit_temporal_plain_time_total_nanoseconds(
            &Self::temporal_plain_date_time_time_locals(&end_wall),
            end_time_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(end_time_local));
        function.instruction(&Instruction::LocalGet(start_time_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(end_time_local));
        function.instruction(&Instruction::LocalGet(end_time_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(end_time_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(correction_local));
        function.instruction(&Instruction::End);
        // Steps 12-13: move the end date back until the remaining exact time
        // no longer runs against the direction.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(success_local));
        self.emit_temporal_plain_date_epoch_days(
            end_wall[0],
            end_wall[1],
            end_wall[2],
            days_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(correction_local));
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(success_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(correction_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compare_local));
        self.emit_temporal_civil_from_days(
            compare_local,
            intermediate_date[0],
            intermediate_date[1],
            intermediate_date[2],
            function,
        );
        self.emit_temporal_zoned_epoch_for_date_at_wall_time(
            time_zone_payload_local,
            intermediate_date,
            start_wall,
            intermediate,
            function,
        )?;
        self.emit_temporal_exact_difference(
            destination,
            intermediate,
            output.time_seconds,
            output.time_subsecond,
            function,
        );
        self.emit_temporal_pair_sign(
            output.time_seconds,
            output.time_subsecond,
            time_sign_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(time_sign_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(success_local));
        function.instruction(&Instruction::LocalGet(correction_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(correction_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Steps 15-16: `CalendarDateUntil(start date, intermediate date,
        // LargerOfTwoTemporalUnits(largestUnit, day))`.
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(date_largest_local));
        self.emit_temporal_difference_iso_date(
            [start_wall[0], start_wall[1], start_wall[2]],
            intermediate_date,
            date_largest_local,
            output.date[0],
            output.date[1],
            output.date[2],
            output.date[3],
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            date_largest_local,
            time_sign_local,
            end_time_local,
            start_time_local,
            days_local,
            success_local,
            correction_local,
            maximum_local,
            sign_local,
            compare_local,
        ] {
            self.release_temp_local(local);
        }
        self.release_temporal_exact_time(intermediate);
        for local in intermediate_date.into_iter().rev() {
            self.release_temp_local(local);
        }
        self.release_temporal_plain_date_time_field_locals(end_wall);
        Ok(())
    }

    /// `RoundRelativeDuration` with a time zone, rounding `duration` in place.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_round_relative_duration_zoned(
        &mut self,
        time_zone_payload_local: u32,
        duration: &TemporalInternalDuration,
        origin: TemporalExactTime,
        destination: TemporalExactTime,
        start_wall: &[u32; 9],
        largest_unit_local: u32,
        increment_local: u32,
        smallest_unit_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let expanded_local = self.reserve_temp_local();
        let start_unit_local = self.reserve_temp_local();
        let nudged = self.reserve_temporal_exact_time();
        self.emit_temporal_internal_duration_direction(duration, sign_local, function);
        // Calendar units, and `day` in a time zone, are irregular.
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_nudge_to_calendar_unit_zoned(
            time_zone_payload_local,
            sign_local,
            duration,
            origin,
            destination,
            start_wall,
            increment_local,
            smallest_unit_local,
            mode_local,
            nudged,
            expanded_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_nudge_to_zoned_time(
            time_zone_payload_local,
            sign_local,
            duration,
            start_wall,
            increment_local,
            smallest_unit_local,
            mode_local,
            nudged,
            expanded_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(expanded_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `LargerOfTwoTemporalUnits(smallestUnit, day)`.
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(start_unit_local));
        self.emit_temporal_bubble_relative_duration_zoned(
            time_zone_payload_local,
            sign_local,
            duration,
            nudged,
            start_wall,
            largest_unit_local,
            start_unit_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temporal_exact_time(nudged);
        for local in [start_unit_local, expanded_local, sign_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `TotalRelativeDuration` with a time zone and a calendar unit or `day`:
    /// `NudgeToCalendarUnit(…, 1, unit, trunc).[[Total]]` as f64 bits.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_total_relative_duration_zoned(
        &mut self,
        time_zone_payload_local: u32,
        duration: &TemporalInternalDuration,
        origin: TemporalExactTime,
        destination: TemporalExactTime,
        start_wall: &[u32; 9],
        unit_local: u32,
        output_bits_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let expanded_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let nudged = self.reserve_temporal_exact_time();
        self.emit_temporal_internal_duration_direction(duration, sign_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        function.instruction(&Instruction::LocalSet(mode_local));
        self.emit_temporal_nudge_to_calendar_unit_zoned(
            time_zone_payload_local,
            sign_local,
            duration,
            origin,
            destination,
            start_wall,
            increment_local,
            unit_local,
            mode_local,
            nudged,
            expanded_local,
            Some(output_bits_local),
            function,
        )?;
        self.release_temporal_exact_time(nudged);
        for local in [mode_local, increment_local, expanded_local, sign_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `ComputeNudgeWindow` with a time zone: `r1`/`r2`, the window's two
    /// date durations and their exact times at the origin's wall-clock time.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_compute_nudge_window_zoned(
        &mut self,
        time_zone_payload_local: u32,
        sign_local: u32,
        duration: &TemporalInternalDuration,
        origin: TemporalExactTime,
        start_wall: &[u32; 9],
        increment_local: u32,
        unit_local: u32,
        shift_local: u32,
        window: &TemporalNudgeWindow,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let step_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(step_local));
        for (index, unit) in CALENDAR_UNITS.into_iter().enumerate() {
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            // `RoundNumberToIncrement(field, increment, trunc)`; weeks count
            // the whole weeks in the days too (ISO weeks are seven days).
            function.instruction(&Instruction::LocalGet(duration.date[index]));
            if unit == TemporalUnit::Week {
                function.instruction(&Instruction::LocalGet(duration.date[3]));
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::I64DivS);
                function.instruction(&Instruction::I64Add);
            }
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalGet(increment_local));
            function.instruction(&Instruction::I64Mul);
            if matches!(unit, TemporalUnit::Year | TemporalUnit::Month) {
                // `additionalShift`.
                function.instruction(&Instruction::LocalGet(shift_local));
                function.instruction(&Instruction::LocalGet(step_local));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
            }
            function.instruction(&Instruction::LocalSet(window.r1));
            // The larger fields stay, the smaller ones are zero.
            for (slot, _) in CALENDAR_UNITS.into_iter().enumerate() {
                let source = if slot < index {
                    Some(duration.date[slot])
                } else {
                    None
                };
                for target in [window.start_date, window.end_date] {
                    match source {
                        Some(source) => {
                            function.instruction(&Instruction::LocalGet(source));
                        }
                        None => {
                            function.instruction(&Instruction::I64Const(0));
                        }
                    }
                    function.instruction(&Instruction::LocalSet(target[slot]));
                }
            }
            function.instruction(&Instruction::LocalGet(window.r1));
            function.instruction(&Instruction::LocalSet(window.start_date[index]));
            function.instruction(&Instruction::LocalGet(window.r1));
            function.instruction(&Instruction::LocalGet(step_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(window.end_date[index]));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        // A zero start duration starts at the origin itself, whose exact time
        // need not be `GetEpochNanosecondsFor` of its own wall-clock time in a
        // repeated hour. The printed text tests `r1 = 0`, which also fires
        // when a larger unit is non-zero (1 week and 0 days); the reference
        // implementation and Test262's `exact-multiple-of-larger-unit.js`
        // test the whole start duration's sign.
        function.instruction(&Instruction::LocalGet(window.start_date[0]));
        for field in &window.start_date[1..] {
            function.instruction(&Instruction::LocalGet(*field));
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_copy_locals(
            &[origin.seconds, origin.subsecond],
            &[window.start.seconds, window.start.subsecond],
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_zoned_window_bound(
            time_zone_payload_local,
            start_wall,
            &window.start_date,
            overflow_local,
            date,
            window.start,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_window_bound(
            time_zone_payload_local,
            start_wall,
            &window.end_date,
            overflow_local,
            date,
            window.end,
            function,
        )?;
        for local in date.into_iter().rev() {
            self.release_temp_local(local);
        }
        self.release_temp_local(overflow_local);
        self.release_temp_local(step_local);
        Ok(())
    }

    /// `GetEpochNanosecondsFor(timeZone, CalendarDateAdd(date of wall,
    /// duration, constrain) + time of wall, compatible)`.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_zoned_window_bound(
        &mut self,
        time_zone_payload_local: u32,
        start_wall: &[u32; 9],
        duration_date: &[u32; 4],
        overflow_local: u32,
        date: [u32; 3],
        output: TemporalExactTime,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_copy_locals(&start_wall[..3], &date, function);
        self.emit_temporal_add_iso_date(
            date[0],
            date[1],
            date[2],
            duration_date[0],
            duration_date[1],
            duration_date[2],
            duration_date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_zoned_epoch_for_date_at_wall_time(
            time_zone_payload_local,
            date,
            start_wall,
            output,
            function,
        )
    }

    /// `NudgeToCalendarUnit` with a time zone. Replaces the date part of
    /// `duration` with the chosen window end and zeroes its time part; with
    /// `total_bits` it also produces `[[Total]]`.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_nudge_to_calendar_unit_zoned(
        &mut self,
        time_zone_payload_local: u32,
        sign_local: u32,
        duration: &TemporalInternalDuration,
        origin: TemporalExactTime,
        destination: TemporalExactTime,
        start_wall: &[u32; 9],
        increment_local: u32,
        unit_local: u32,
        mode_local: u32,
        nudged: TemporalExactTime,
        expanded_local: u32,
        total_bits: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let window = self.reserve_temporal_nudge_window();
        let shift_local = self.reserve_temp_local();
        let inside_local = self.reserve_temp_local();
        let numerator = self.reserve_temporal_exact_time();
        let denominator = self.reserve_temporal_exact_time();
        let twice = self.reserve_temporal_exact_time();
        let encoded_local = self.reserve_temp_local();
        let four_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let take_end_local = self.reserve_temp_local();
        let compare_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(expanded_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(shift_local));
        // Steps 2-4: the window, retried once with `additionalShift` when the
        // destination falls outside it.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_temporal_compute_nudge_window_zoned(
            time_zone_payload_local,
            sign_local,
            duration,
            origin,
            start_wall,
            increment_local,
            unit_local,
            shift_local,
            &window,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_temporal_exact_at_most_i32(window.start, destination, function);
        self.emit_temporal_exact_at_most_i32(destination, window.end, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::Else);
        self.emit_temporal_exact_at_most_i32(window.end, destination, function);
        self.emit_temporal_exact_at_most_i32(destination, window.start, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(inside_local));
        function.instruction(&Instruction::LocalGet(inside_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(expanded_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // progress = (dest - start) / (end - start), both made non-negative by
        // the direction.
        self.emit_temporal_exact_difference(
            destination,
            window.start,
            numerator.seconds,
            numerator.subsecond,
            function,
        );
        self.emit_temporal_exact_difference(
            window.end,
            window.start,
            denominator.seconds,
            denominator.subsecond,
            function,
        );
        for local in [
            numerator.seconds,
            numerator.subsecond,
            denominator.seconds,
            denominator.subsecond,
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(local));
        }
        // Step 9: an empty window is a RangeError.
        function.instruction(&Instruction::LocalGet(denominator.seconds));
        function.instruction(&Instruction::LocalGet(denominator.subsecond));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal duration rounding window is empty",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if let Some(output_bits_local) = total_bits {
            self.emit_temporal_nudge_total_bits(
                &window,
                sign_local,
                numerator,
                denominator,
                output_bits_local,
                function,
            );
        }

        // Steps 12-17: which end the rounding mode picks. `encoded` places
        // the progress against one half: 0 at the start, 1 below, 2 at, 3
        // above; progress 1 always takes the end.
        function.instruction(&Instruction::LocalGet(numerator.seconds));
        function.instruction(&Instruction::LocalGet(denominator.seconds));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(numerator.subsecond));
        function.instruction(&Instruction::LocalGet(denominator.subsecond));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        for (target, source) in [
            (twice.seconds, numerator.seconds),
            (twice.subsecond, numerator.subsecond),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(target));
        }
        for (target, source) in [
            (twice.seconds, denominator.seconds),
            (twice.subsecond, denominator.subsecond),
        ] {
            function.instruction(&Instruction::LocalGet(target));
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(target));
        }
        self.emit_temporal_duration_renormalize(twice.seconds, twice.subsecond, function);
        self.emit_temporal_pair_sign(twice.seconds, twice.subsecond, compare_local, function);
        function.instruction(&Instruction::LocalGet(compare_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(encoded_local));
        function.instruction(&Instruction::LocalGet(numerator.seconds));
        function.instruction(&Instruction::LocalGet(numerator.subsecond));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(encoded_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(four_local));
        // `(abs(r1) / increment) modulo 2` breaks halfEven ties.
        function.instruction(&Instruction::LocalGet(window.r1));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(quotient_local));
        self.emit_temporal_duration_round_up_i32(
            encoded_local,
            four_local,
            quotient_local,
            sign_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(take_end_local));
        function.instruction(&Instruction::LocalGet(take_end_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(expanded_local));
        self.emit_copy_locals(&window.end_date, &duration.date, function);
        self.emit_copy_locals(
            &[window.end.seconds, window.end.subsecond],
            &[nudged.seconds, nudged.subsecond],
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_copy_locals(&window.start_date, &duration.date, function);
        self.emit_copy_locals(
            &[window.start.seconds, window.start.subsecond],
            &[nudged.seconds, nudged.subsecond],
            function,
        );
        function.instruction(&Instruction::End);
        for local in [duration.time_seconds, duration.time_subsecond] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        for local in [
            compare_local,
            take_end_local,
            quotient_local,
            four_local,
            encoded_local,
        ] {
            self.release_temp_local(local);
        }
        self.release_temporal_exact_time(twice);
        self.release_temporal_exact_time(denominator);
        self.release_temporal_exact_time(numerator);
        self.release_temp_local(inside_local);
        self.release_temp_local(shift_local);
        self.release_temporal_nudge_window(window);
        Ok(())
    }

    /// `total = r1 + progress × sign` (increment 1) as f64 bits, from one exact
    /// quotient: `(abs(r1) × den + num) / den`, signed by the direction. The
    /// window is one calendar unit, so both spans fit `i64` nanoseconds.
    fn emit_temporal_nudge_total_bits(
        &mut self,
        window: &TemporalNudgeWindow,
        sign_local: u32,
        numerator: TemporalExactTime,
        denominator: TemporalExactTime,
        output_bits_local: u32,
        function: &mut Function,
    ) {
        let numerator_local = self.reserve_temp_local();
        let denominator_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let high_local = self.reserve_temp_local();
        let low_local = self.reserve_temp_local();
        for (local, pair) in [(numerator_local, numerator), (denominator_local, denominator)] {
            function.instruction(&Instruction::LocalGet(pair.seconds));
            function.instruction(&Instruction::I64Const(1_000_000_000));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(pair.subsecond));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(window.r1));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(magnitude_local));
        self.emit_temporal_u64_product_plus(
            magnitude_local,
            denominator_local,
            numerator_local,
            high_local,
            low_local,
            function,
        );
        self.emit_temporal_exact_quotient_bits(
            high_local,
            low_local,
            TemporalExactDivisor::Local(denominator_local),
            output_bits_local,
            function,
        );
        // A zero total is +0 whatever the direction.
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(output_bits_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::LocalSet(output_bits_local));
        function.instruction(&Instruction::End);
        for local in [
            low_local,
            high_local,
            magnitude_local,
            denominator_local,
            numerator_local,
        ] {
            self.release_temp_local(local);
        }
    }

    /// `NudgeToZonedTime`: rounds the time part within the day that starts at
    /// the origin plus the date part, carrying into the next day when the
    /// rounded time reaches that day's exact length.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_nudge_to_zoned_time(
        &mut self,
        time_zone_payload_local: u32,
        sign_local: u32,
        duration: &TemporalInternalDuration,
        start_wall: &[u32; 9],
        increment_local: u32,
        unit_local: u32,
        mode_local: u32,
        nudged: TemporalExactTime,
        expanded_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let start = self.reserve_temporal_exact_time();
        let end = self.reserve_temporal_exact_time();
        let span = self.reserve_temporal_exact_time();
        let beyond = self.reserve_temporal_exact_time();
        let overflow_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let beyond_sign_local = self.reserve_temp_local();

        self.emit_copy_locals(&start_wall[..3], &date, function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        self.emit_temporal_add_iso_date(
            date[0],
            date[1],
            date[2],
            duration.date[0],
            duration.date[1],
            duration.date[2],
            duration.date[3],
            overflow_local,
            function,
        )?;
        self.emit_temporal_zoned_epoch_for_date_at_wall_time(
            time_zone_payload_local,
            date,
            start_wall,
            start,
            function,
        )?;
        // `AddDaysToISODate(start, sign)`.
        self.emit_temporal_plain_date_epoch_days(date[0], date[1], date[2], days_local, function);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(days_local));
        self.emit_temporal_civil_from_days(days_local, date[0], date[1], date[2], function);
        self.emit_temporal_zoned_epoch_for_date_at_wall_time(
            time_zone_payload_local,
            date,
            start_wall,
            end,
            function,
        )?;
        self.emit_temporal_exact_difference(end, start, span.seconds, span.subsecond, function);
        // `RoundTimeDurationToIncrement(duration.[[Time]], increment × unit)`.
        self.emit_temporal_round_difference_time(
            duration.time_seconds,
            duration.time_subsecond,
            unit_local,
            increment_local,
            mode_local,
            function,
        );
        for (target, left, right) in [
            (beyond.seconds, duration.time_seconds, span.seconds),
            (beyond.subsecond, duration.time_subsecond, span.subsecond),
        ] {
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(target));
        }
        self.emit_temporal_duration_renormalize(beyond.seconds, beyond.subsecond, function);
        self.emit_temporal_pair_sign(
            beyond.seconds,
            beyond.subsecond,
            beyond_sign_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(beyond_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        // The rounded time reaches the next day: round what lies beyond it.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(expanded_local));
        self.emit_copy_locals(
            &[beyond.seconds, beyond.subsecond],
            &[duration.time_seconds, duration.time_subsecond],
            function,
        );
        self.emit_temporal_round_difference_time(
            duration.time_seconds,
            duration.time_subsecond,
            unit_local,
            increment_local,
            mode_local,
            function,
        );
        self.emit_copy_locals(
            &[end.seconds, end.subsecond],
            &[nudged.seconds, nudged.subsecond],
            function,
        );
        function.instruction(&Instruction::LocalGet(duration.date[3]));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duration.date[3]));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(expanded_local));
        self.emit_copy_locals(
            &[start.seconds, start.subsecond],
            &[nudged.seconds, nudged.subsecond],
            function,
        );
        function.instruction(&Instruction::End);
        // `AddTimeDurationToEpochNanoseconds(roundedTime, …)`.
        for (target, time) in [
            (nudged.seconds, duration.time_seconds),
            (nudged.subsecond, duration.time_subsecond),
        ] {
            function.instruction(&Instruction::LocalGet(target));
            function.instruction(&Instruction::LocalGet(time));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(target));
        }
        self.emit_temporal_normalize_seconds_and_subseconds(
            nudged.seconds,
            nudged.subsecond,
            function,
        );

        for local in [beyond_sign_local, days_local, overflow_local] {
            self.release_temp_local(local);
        }
        self.release_temporal_exact_time(beyond);
        self.release_temporal_exact_time(span);
        self.release_temporal_exact_time(end);
        self.release_temporal_exact_time(start);
        for local in date.into_iter().rev() {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `BubbleRelativeDuration` with a time zone, from `start_unit_local`'s
    /// next larger unit up to `largest_unit_local`.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_bubble_relative_duration_zoned(
        &mut self,
        time_zone_payload_local: u32,
        sign_local: u32,
        duration: &TemporalInternalDuration,
        nudged: TemporalExactTime,
        start_wall: &[u32; 9],
        largest_unit_local: u32,
        start_unit_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let done_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let beyond_sign_local = self.reserve_temp_local();
        let end_duration = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let date = [
            self.reserve_temp_local(),
            self.reserve_temp_local(),
            self.reserve_temp_local(),
        ];
        let end = self.reserve_temporal_exact_time();
        let beyond = self.reserve_temporal_exact_time();

        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        // `smallestUnit is largestUnit` leaves the duration alone: no unit is
        // strictly between.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        for unit in [TemporalUnit::Week, TemporalUnit::Month, TemporalUnit::Year] {
            let index = unit.duration_field_index();
            // `unitIndex` runs from the start unit's index - 1 down to the
            // largest unit's; weeks count only when they are the largest unit.
            function.instruction(&Instruction::LocalGet(done_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::LocalGet(start_unit_local));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::LocalGet(largest_unit_local));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I32And);
            if unit == TemporalUnit::Week {
                function.instruction(&Instruction::LocalGet(largest_unit_local));
                function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            for slot in 0..4 {
                if slot < index {
                    function.instruction(&Instruction::LocalGet(duration.date[slot]));
                } else if slot == index {
                    function.instruction(&Instruction::LocalGet(duration.date[slot]));
                    function.instruction(&Instruction::LocalGet(sign_local));
                    function.instruction(&Instruction::I64Add);
                } else {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::LocalSet(end_duration[slot]));
            }
            self.emit_temporal_zoned_window_bound(
                time_zone_payload_local,
                start_wall,
                &end_duration,
                overflow_local,
                date,
                end,
                function,
            )?;
            self.emit_temporal_exact_difference(
                nudged,
                end,
                beyond.seconds,
                beyond.subsecond,
                function,
            );
            self.emit_temporal_pair_sign(
                beyond.seconds,
                beyond.subsecond,
                beyond_sign_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(beyond_sign_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalGet(sign_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_copy_locals(&end_duration, &duration.date, function);
            for local in [duration.time_seconds, duration.time_subsecond] {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(local));
            }
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(done_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.release_temporal_exact_time(beyond);
        self.release_temporal_exact_time(end);
        for local in date.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in end_duration.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in [beyond_sign_local, overflow_local, done_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn reserve_temporal_nudge_window(&mut self) -> TemporalNudgeWindow {
        TemporalNudgeWindow {
            r1: self.reserve_temp_local(),
            start_date: [
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
            ],
            end_date: [
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
            ],
            start: self.reserve_temporal_exact_time(),
            end: self.reserve_temporal_exact_time(),
        }
    }

    fn release_temporal_nudge_window(&mut self, window: TemporalNudgeWindow) {
        self.release_temporal_exact_time(window.end);
        self.release_temporal_exact_time(window.start);
        for local in window.end_date.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in window.start_date.into_iter().rev() {
            self.release_temp_local(local);
        }
        self.release_temp_local(window.r1);
    }

    /// `TemporalDurationFromInternal(internalDuration, hour)`: the time part
    /// balanced up to hours, the date part as is.
    pub(super) fn emit_temporal_duration_from_internal_hours(
        &mut self,
        duration: &TemporalInternalDuration,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = self.reserve_temporal_duration_field_locals();
        let largest_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::LocalSet(largest_local));
        self.emit_temporal_duration_balance(
            duration.time_seconds,
            duration.time_subsecond,
            largest_local,
            &fields,
            function,
        )?;
        for (index, unit) in CALENDAR_UNITS.into_iter().enumerate() {
            self.emit_temporal_duration_set_integer_field(
                &fields,
                unit,
                duration.date[index],
                function,
            );
        }
        self.emit_create_temporal_duration(&fields, function)?;
        self.release_temp_local(largest_local);
        self.release_temporal_duration_field_locals(fields);
        Ok(())
    }
}

/// The two ends of a `ComputeNudgeWindow` result.
struct TemporalNudgeWindow {
    r1: u32,
    start_date: [u32; 4],
    end_date: [u32; 4],
    start: TemporalExactTime,
    end: TemporalExactTime,
}
