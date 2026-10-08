//! DifferenceZonedDateTime uses elapsed time between compatible local probes.

use super::*;
use crate::builtins::temporal_options::TemporalUnit;
use crate::builtins::temporal_plain_date_time_methods::TemporalDateTimeDifferenceSettingsPlan;

impl<'a> FunctionBuilder<'a> {
    /// The two Instant pairs are range-proved and floor-normalized. Their
    /// signed difference fits the Duration seconds domain without scaling a
    /// whole epoch into nanoseconds.
    pub(super) fn emit_temporal_zoned_elapsed_difference(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        dest: &NormalizedTemporalInstantLocals,
        seconds: I64Local,
        subsecond: I64Local,
        function: &mut Function,
    ) {
        (dest.floor_seconds()).load(function);
        (origin.floor_seconds()).load(function);
        function.instruction(&Instruction::I64Sub);
        (seconds).store(function);
        (dest.nanosecond()).load(function);
        (origin.nanosecond()).load(function);
        function.instruction(&Instruction::I64Sub);
        (subsecond).store(function);
        self.emit_temporal_duration_renormalize(seconds, subsecond, function);
    }

    fn emit_temporal_zoned_epoch_equality_i32(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        dest: &NormalizedTemporalInstantLocals,
        function: &mut Function,
    ) {
        (origin.floor_seconds()).load(function);
        (dest.floor_seconds()).load(function);
        function.instruction(&Instruction::I64Eq);
        (origin.nanosecond()).load(function);
        (dest.nanosecond()).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
    }

    /// Combine endDate + correction with the origin time. AddDaysToISODate is
    /// balanced ISO arithmetic here; it does not add a PlainDate allocation
    /// or an extra ISODateTimeWithinLimits check before the prescribed inverse.
    fn emit_temporal_zoned_difference_probe_fields(
        &mut self,
        origin: &RegulatedTemporalIsoRecordLocals,
        end_date: [I64Local; 3],
        signed_correction: I64Local,
        function: &mut Function,
    ) -> Result<RegulatedTemporalIsoRecordLocals, EmitError> {
        let destination = self.reserve_temporal_iso_record_result(function);
        let fields: [I64Local; 9] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let epoch_days = self.runtime_schema().reserve_i64_local(function);
        for (source, destination) in origin.fields().iter().zip(fields) {
            (*source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_plain_date_epoch_days(
            end_date[0],
            end_date[1],
            end_date[2],
            epoch_days,
            function,
        );
        (epoch_days).load(function);
        (signed_correction).load(function);
        function.instruction(&Instruction::I64Add);
        (epoch_days).store(function);
        self.emit_temporal_civil_from_days(epoch_days, fields[0], fields[1], fields[2], function);
        self.runtime_schema()
            .release_i64_local(epoch_days, function);
        let completed = CompletedTemporalIsoArithmeticLocals { fields };
        let result =
            self.emit_temporal_iso_record_from_arithmetic_into(destination, &completed, function)?;
        completed.release(self, function);
        Ok(result)
    }

    pub(in crate::builtins) fn emit_temporal_zoned_date_difference(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        dest: &NormalizedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        largest: TemporalZonedRelativeLargestUnit<'_>,
        function: &mut Function,
    ) -> Result<TemporalZonedInternalDurationLocals, EmitError> {
        let date: [I64Local; 4] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        for local in date.into_iter().chain([seconds, subsecond]) {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.emit_temporal_zoned_epoch_equality_i32(origin, dest, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let origin_snapshot = self.emit_temporal_zone_snapshot(zone, origin, function)?;
        let dest_snapshot = self.emit_temporal_zone_snapshot(zone, dest, function)?;
        let origin_iso = self.emit_temporal_zone_snapshot_iso_record(&origin_snapshot, function)?;
        let dest_iso = self.emit_temporal_zone_snapshot_iso_record(&dest_snapshot, function)?;
        let date_comparison = self.runtime_schema().reserve_i64_local(function);
        let origin_time = self.runtime_schema().reserve_i64_local(function);
        let dest_time = self.runtime_schema().reserve_i64_local(function);
        let time_difference = self.runtime_schema().reserve_i64_local(function);
        let sign = self.runtime_schema().reserve_i64_local(function);
        let max_correction = self.runtime_schema().reserve_i64_local(function);
        let correction = self.runtime_schema().reserve_i64_local(function);
        let success = self.runtime_schema().reserve_i64_local(function);
        let signed_correction = self.runtime_schema().reserve_i64_local(function);
        let time_sign = self.runtime_schema().reserve_i64_local(function);
        let date_largest = self.runtime_schema().reserve_i64_local(function);
        let intermediate_date: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let first = origin_iso.fields();
        let second = dest_iso.fields();
        self.emit_temporal_compare_iso_date(
            [first[0], first[1], first[2]],
            [second[0], second[1], second[2]],
            date_comparison,
            function,
        );
        (date_comparison).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_zoned_elapsed_difference(origin, dest, seconds, subsecond, function);
        function.instruction(&Instruction::Else);
        // Sign is opposite the epoch difference. The initial correction
        // follows the sign of DifferenceTime, as the spec requires.
        self.emit_temporal_zoned_elapsed_difference(origin, dest, seconds, subsecond, function);
        self.emit_temporal_relative_time_sign(seconds, subsecond, time_sign, function);
        (time_sign).load(function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Mul);
        (sign).store(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        (max_correction).store(function);
        let first_time = Self::temporal_plain_date_time_time_locals(first);
        let second_time = Self::temporal_plain_date_time_time_locals(second);
        self.emit_temporal_plain_time_total_nanoseconds(&first_time, origin_time, function);
        self.emit_temporal_plain_time_total_nanoseconds(&second_time, dest_time, function);
        (dest_time).load(function);
        (origin_time).load(function);
        function.instruction(&Instruction::I64Sub);
        (time_difference).store(function);
        (time_difference).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        (time_difference).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        (time_sign).store(function);
        (time_sign).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        (correction).store(function);
        function.instruction(&Instruction::I64Const(0));
        (success).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (correction).load(function);
        (max_correction).load(function);
        function.instruction(&Instruction::I64GtS);
        (success).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        (correction).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Mul);
        (signed_correction).store(function);
        let probe = self.emit_temporal_zoned_difference_probe_fields(
            &origin_iso,
            [second[0], second[1], second[2]],
            signed_correction,
            function,
        )?;
        for (source, destination) in probe.fields()[..3].iter().zip(intermediate_date) {
            (*source).load(function);
            (destination).store(function);
        }
        let local = self.emit_temporal_local_coordinate_from_iso_record(&probe, function)?;
        let disambiguation = self.emit_temporal_constant_disambiguation(
            crate::builtins::temporal_options::Disambiguation::Compatible,
            function,
        );
        let intermediate =
            self.emit_temporal_get_epoch_nanoseconds_for(zone, &local, &disambiguation, function)?;
        self.emit_temporal_zoned_elapsed_difference(
            &intermediate,
            dest,
            seconds,
            subsecond,
            function,
        );
        self.emit_temporal_relative_time_sign(seconds, subsecond, time_sign, function);
        (sign).load(function);
        (time_sign).load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        (success).store(function);
        intermediate.release(self, function);
        disambiguation.release(self, function);
        local.release(self, function);
        probe.release(self, function);
        (correction).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (correction).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // The spec asserts that the bounded correction succeeds. A broken
        // provider/compiler invariant traps instead of fabricating a result.
        (success).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (largest.local()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (largest.local()).load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::End);
        (date_largest).store(function);
        self.emit_temporal_difference_calendar_date(
            calendar,
            [first[0], first[1], first[2]],
            intermediate_date,
            date_largest,
            date[0],
            date[1],
            date[2],
            date[3],
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in intermediate_date.into_iter().rev().chain([
            date_largest,
            time_sign,
            signed_correction,
            success,
            correction,
            max_correction,
            sign,
            time_difference,
            dest_time,
            origin_time,
            date_comparison,
        ]) {
            self.runtime_schema().release_i64_local(local, function);
        }
        dest_iso.release(self, function);
        origin_iso.release(self, function);
        dest_snapshot.release(self, function);
        origin_snapshot.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(TemporalZonedInternalDurationLocals {
            date,
            seconds,
            subsecond,
        })
    }

    pub(in crate::builtins) fn emit_temporal_difference_zoned_date_time(
        &mut self,
        receiver: &BrandedTemporalZonedRecordLocals,
        operation: TemporalZonedDifferenceOperation,
        other_input: &ValueLocals,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let other = self.emit_temporal_to_zoned_record(other_input, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(receiver, function)?;
        let other_calendar = self.emit_temporal_calendar_from_zoned_record(&other, function)?;
        self.emit_temporal_require_same_calendar(
            calendar.calendar_id(),
            other_calendar.calendar_id(),
            TemporalDifferenceGuard::ZonedDateTimeSameCalendar,
            function,
        )?;
        let plan = match operation {
            TemporalZonedDifferenceOperation::Until => {
                TemporalDateTimeDifferenceSettingsPlan::ZonedUntil
            }
            TemporalZonedDifferenceOperation::Since => {
                TemporalDateTimeDifferenceSettingsPlan::ZonedSince
            }
        };
        let settings = self.emit_temporal_date_time_difference_settings(options, plan, function)?;
        let origin = self.emit_temporal_normalized_instant_from_zoned_record(receiver, function)?;
        let dest = self.emit_temporal_normalized_instant_from_zoned_record(&other, function)?;
        (settings.largest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_zoned_elapsed_difference(&origin, &dest, seconds, subsecond, function);
        self.emit_temporal_round_difference_time(
            seconds,
            subsecond,
            settings.smallest_unit(),
            settings.rounding_increment(),
            settings.rounding_mode(),
            function,
        );
        match operation {
            TemporalZonedDifferenceOperation::Until => {}
            TemporalZonedDifferenceOperation::Since => {
                for local in [seconds, subsecond] {
                    function.instruction(&Instruction::I64Const(0));
                    (local).load(function);
                    function.instruction(&Instruction::I64Sub);
                    (local).store(function);
                }
            }
        }
        self.emit_temporal_duration_balance(
            seconds,
            subsecond,
            settings.largest_unit(),
            &fields,
            function,
        )?;
        self.emit_create_temporal_duration(&fields, function)?;
        self.release_temporal_duration_field_locals(fields, function);
        self.runtime_schema().release_i64_local(subsecond, function);
        self.runtime_schema().release_i64_local(seconds, function);
        function.instruction(&Instruction::Else);
        let zone = self.emit_temporal_zone_from_zoned_record(receiver, function)?;
        let other_zone = self.emit_temporal_zone_from_zoned_record(&other, function)?;
        self.emit_string_payload_equality_i32(
            zone.primary_identifier(),
            other_zone.primary_identifier(),
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            TemporalDifferenceGuard::ZonedDateTimeSameTimeZone.message(),
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_epoch_equality_i32(&origin, &dest, function);
        self.open_frame(ControlFrameKind::If, function);
        let empty = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_duration_zero_fields(&empty, function);
        self.emit_create_temporal_duration(&empty, function)?;
        self.emit_return_current_completion(function);
        self.release_temporal_duration_field_locals(empty, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let mut duration = self.emit_temporal_zoned_date_difference(
            &origin,
            &dest,
            &zone,
            &calendar,
            TemporalZonedRelativeLargestUnit::Rounding(
                TemporalZonedRelativeRoundingSettings::Difference(&settings),
            ),
            function,
        )?;
        // Nanosecond/1 preserves the exact unrounded internal difference.
        (settings.smallest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Ne);
        (settings.rounding_increment()).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_round_zoned_relative_duration(
            &origin,
            &dest,
            &zone,
            &calendar,
            &mut duration,
            TemporalZonedRelativeRoundingSettings::Difference(&settings),
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_zoned_balance_internal(
            &duration,
            TemporalZonedRelativeLargestUnit::Rounding(
                TemporalZonedRelativeRoundingSettings::Difference(&settings),
            ),
            &fields,
            function,
        )?;
        match operation {
            TemporalZonedDifferenceOperation::Until => {}
            TemporalZonedDifferenceOperation::Since => {
                self.emit_temporal_duration_negate_fields(&fields, function);
            }
        }
        self.emit_create_temporal_duration(&fields, function)?;
        self.release_temporal_duration_field_locals(fields, function);
        duration.release(self, function);
        other_zone.release(self, function);
        zone.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        dest.release(self, function);
        origin.release(self, function);
        settings.release(self, function);
        other_calendar.release(self, function);
        calendar.release(self, function);
        other.release(function);
        Ok(())
    }
}
