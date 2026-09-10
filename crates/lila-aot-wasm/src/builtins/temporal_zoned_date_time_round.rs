//! Zoned date-time rounding in the local clock of UTC and fixed-offset zones.

use super::super::*;
use super::temporal_options::{
    TemporalRoundingMode, TemporalUnit, TemporalUnitOptionProperty, TemporalUnitSlot,
};
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_temporal_zoned_date_time_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let within_day_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let components = std::array::from_fn::<_, 7, _>(|_| self.reserve_temp_local());

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.prototype.round requires a roundTo argument",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::LocalSet(mode_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_unit_from_payload(
            argument_payload_local,
            unit_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(
            argument_payload_local,
            argument_tag_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(
            argument_payload_local,
            argument_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            argument_payload_local,
            argument_tag_local,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            argument_payload_local,
            argument_tag_local,
            TemporalUnitOptionProperty::SmallestUnit,
            unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime.prototype.round requires smallestUnit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Day,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.ZonedDateTime unit option",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime rounding increment",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_time_validate_increment(unit_local, increment_local, function)?;
        function.instruction(&Instruction::End);

        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }

        // The one-nanosecond clone precedes local date conversion and its range
        // checks, including when the instant's offset moves it past an ISO limit.
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_zoned_date_time_local_components(
            record_local,
            epoch_payload_local,
            epoch_tag_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            offset_seconds_local,
            time_zone_payload_local,
            local_time_payload_local,
            components,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(components[3]));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        for (component, radix) in [
            (components[4], 60),
            (components[5], 60),
            (components[6], 1_000),
        ] {
            function.instruction(&Instruction::I64Const(radix));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(component));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(within_day_local));

        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_zoned_date_time_start_of_day_seconds(
            record_local,
            seconds_local,
            function,
        )?;
        self.emit_temporal_zoned_date_time_day_boundary(
            seconds_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        // GetStartOfDay validates both endpoints even when rounding selects
        // today's midnight. Fixed-offset day length is exactly 86,400 seconds.
        self.emit_temporal_zoned_date_time_day_boundary(
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::LocalSet(quantum_local));
        self.emit_temporal_plain_time_round_nanoseconds(
            within_day_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_time_rounding_quantum(
            unit_local,
            increment_local,
            quantum_local,
            function,
        );
        self.emit_temporal_round_time_nanoseconds(
            within_day_local,
            unit_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(local_time_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(86_400_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(86_400_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(within_day_local));

        // InterpretISODateTimeOffset with offset=prefer checks the rounded
        // original ISO day before applying the receiver's fixed offset.
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(-100_000_000));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(100_000_000));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime rounded ISO date is outside the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in components.into_iter().rev().chain([
            subsecond_local,
            seconds_local,
            within_day_local,
            day_local,
            local_time_payload_local,
            offset_seconds_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            prototype_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            quantum_local,
            mode_local,
            increment_local,
            unit_local,
            argument_tag_local,
            argument_payload_local,
            record_local,
        ]) {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
