//! Zoned date-time rounding with actual day spans and retained inverse policies.

use super::super::*;
use super::temporal_options::{
    Disambiguation, OffsetOption, TemporalRoundingMode, TemporalUnit, TemporalUnitOptionProperty,
    TemporalUnitSlot,
};
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::temporal_zone_provider::{
    RegulatedTemporalIsoRecordLocals, TemporalOffsetBehavior, TemporalOffsetMatchBehavior,
    TemporalRoundingModeLocals, TemporalZonedAllocationInput,
};
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

impl FunctionBuilder<'_> {
    pub(super) fn emit_temporal_zoned_date_time_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let unit_local = schema.reserve_i64_local(function);
        let increment_local = schema.reserve_i64_local(function);
        let mode_local = schema.reserve_i64_local(function);
        let offset_nanoseconds = schema.reserve_i64_local(function);
        let branded = self.emit_temporal_branded_zoned_receiver(function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROTOTYPE_ROUND_REQUIRES_A_ROUNDTO_ARGUMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        increment_local.store(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        mode_local.store(function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_plain_time_unit_from_string(&string, unit_local, function)?;
        string.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(&argument, function)?;
        self.emit_temporal_duration_rounding_increment_option(
            &argument,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            &argument,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            &argument,
            TemporalUnitOptionProperty::SmallestUnit,
            unit_local,
            function,
        )?;
        unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROTOTYPE_ROUND_REQUIRES_SMALLESTUNIT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Day,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_UNIT_OPTION,
            function,
        )?;
        unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        increment_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_ROUNDING_INCREMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_time_validate_increment(unit_local, increment_local, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let mode = self.emit_temporal_validated_rounding_mode(mode_local, function)?;
        let instant =
            self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        // Preserve the nanosecond clone before any local-date projection.
        unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Eq);
        increment_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&instant, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &instant, function)?;
        unit_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let boundaries = self.emit_temporal_zone_day_boundaries(&snapshot, function)?;
        let replacement = self.emit_temporal_round_instant_to_day(&boundaries, &mode, function)?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&replacement, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        replacement.release(self, function);
        boundaries.release(self, function);
        function.instruction(&Instruction::Else);
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        let rounded = self.emit_temporal_zoned_round_iso_date_time(
            &iso,
            unit_local,
            increment_local,
            &mode,
            function,
        )?;
        snapshot.offset_seconds().load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        offset_nanoseconds.store(function);
        let original_offset =
            self.emit_temporal_validated_offset_nanoseconds(offset_nanoseconds, function)?;
        let compatible =
            self.emit_temporal_constant_disambiguation(Disambiguation::Compatible, function);
        let prefer = self.emit_temporal_constant_offset_option(OffsetOption::Prefer, function);
        let replacement = self.emit_temporal_interpret_iso_date_time_offset(
            &rounded,
            &zone,
            TemporalOffsetBehavior::Option,
            &original_offset,
            &compatible,
            &prefer,
            TemporalOffsetMatchBehavior::MatchExactly,
            function,
        )?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&replacement, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        replacement.release(self, function);
        prefer.release(self, function);
        compatible.release(self, function);
        original_offset.release(self, function);
        rounded.release(self, function);
        iso.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        snapshot.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        calendar.release(self, function);
        zone.release(self, function);
        instant.release(self, function);
        mode.release(self, function);
        branded.release(function);
        for local in [offset_nanoseconds, mode_local, increment_local, unit_local] {
            schema.release_i64_local(local, function);
        }
        argument.clear(function);
        Ok(())
    }

    fn emit_temporal_zoned_round_iso_date_time(
        &mut self,
        iso: &RegulatedTemporalIsoRecordLocals,
        unit: I64Local,
        increment: I64Local,
        mode: &TemporalRoundingModeLocals,
        function: &mut Function,
    ) -> Result<RegulatedTemporalIsoRecordLocals, EmitError> {
        let destination = self.reserve_temporal_iso_record_result(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        let within_day = self.runtime_schema().reserve_i64_local(function);
        let quantum = self.runtime_schema().reserve_i64_local(function);
        let days = self.runtime_schema().reserve_i64_local(function);
        let adjusted_year = self.runtime_schema().reserve_i64_local(function);
        let era = self.runtime_schema().reserve_i64_local(function);
        let month_index = self.runtime_schema().reserve_i64_local(function);
        for (source, target) in iso.fields().iter().zip(fields) {
            (*source).load(function);
            (target).store(function);
        }
        let time = Self::temporal_plain_date_time_time_locals(&fields);
        self.emit_temporal_plain_time_total_nanoseconds(&time, within_day, function);
        self.emit_temporal_plain_time_rounding_quantum(unit, increment, quantum, function);
        self.emit_temporal_round_time_nanoseconds(
            within_day,
            unit,
            quantum,
            mode.local(),
            function,
        );
        self.emit_temporal_days_from_civil(
            fields[0],
            fields[1],
            fields[2],
            adjusted_year,
            era,
            month_index,
            days,
            function,
        );
        (days).load(function);
        (within_day).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (days).store(function);
        (within_day).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64RemS);
        (within_day).store(function);
        self.emit_temporal_civil_from_days(days, fields[0], fields[1], fields[2], function);
        self.emit_temporal_plain_time_from_nanoseconds(within_day, &time, function);
        let result =
            self.emit_temporal_regulated_iso_record_into(destination, &fields, function)?;
        for local in [month_index, era, adjusted_year, days, quantum, within_day] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.release_temporal_plain_date_time_field_locals(fields, function);
        Ok(result)
    }
}
