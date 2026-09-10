//! Zoned date-time day boundaries and transition queries for UTC and fixed offsets.

use super::super::*;
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;

const MILLISECONDS_PER_DAY: i64 = NANOSECONDS_PER_TEMPORAL_DAY / 1_000_000;
const SECONDS_PER_DAY: i64 = NANOSECONDS_PER_TEMPORAL_DAY / 1_000_000_000;

#[derive(Clone, Copy)]
enum TimeZoneTransitionDirection {
    Next,
    Previous,
}

impl TimeZoneTransitionDirection {
    const ALL: &'static [Self] = &[Self::Next, Self::Previous];

    const fn name(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Previous => "previous",
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_temporal_zoned_date_time_get_time_zone_transition(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let direction_payload_local = self.reserve_temp_local();
        let direction_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();
        let time_zone_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // The specification's null-prototype shorthand object is unobservable.
        function.instruction(&Instruction::LocalGet(options_payload_local));
        function.instruction(&Instruction::LocalSet(direction_payload_local));
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::LocalSet(direction_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime transition direction must be a string or object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("direction")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            key_local,
            direction_payload_local,
            direction_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        // GetDirectionOption is required even when this zone has no transition.
        self.emit_value_to_string_payload(direction_payload_local, direction_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(direction_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(recognized_local));
        for direction in TimeZoneTransitionDirection::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(direction.name()),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_string_payload_equality_i32(direction_payload_local, key_local, function);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalGet(recognized_local));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(recognized_local));
        }
        function.instruction(&Instruction::LocalGet(recognized_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime transition direction",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_local,
            function,
        );
        // The existing zone boundary accepts only UTC and numeric offsets.
        // Both have no transitions; a future named-zone implementation must
        // supply the corresponding transition operations at this boundary.
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_local,
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            offset_seconds_local,
            time_zone_local,
            recognized_local,
            key_local,
            direction_tag_local,
            direction_payload_local,
            options_tag_local,
            options_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_start_of_day_seconds(
        &mut self,
        record_local: u32,
        seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let milliseconds_local = self.reserve_temp_local();
        let time_zone_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();

        self.emit_temporal_epoch_nanoseconds_record_to_milliseconds(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            milliseconds_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_local,
            function,
        );
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_local,
            offset_seconds_local,
            function,
        )?;
        // The record conversion floors negative submilliseconds and produces
        // an exact Number below 2^53 throughout the Temporal instant range.
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(MILLISECONDS_PER_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(MILLISECONDS_PER_DAY));
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
        function.instruction(&Instruction::I64Const(SECONDS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));

        for local in [
            day_local,
            offset_seconds_local,
            time_zone_local,
            milliseconds_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_day_boundary(
        &mut self,
        seconds_local: u32,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let subsecond_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        self.release_temp_local(subsecond_local);
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_hours_in_day(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let today_seconds_local = self.reserve_temp_local();
        let tomorrow_seconds_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_temporal_zoned_date_time_start_of_day_seconds(
            record_local,
            today_seconds_local,
            function,
        )?;
        self.emit_temporal_zoned_date_time_day_boundary(
            today_seconds_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        // UTC and fixed-offset zones have successive midnights 86,400 seconds
        // apart. Both endpoints still must be representable instants.
        function.instruction(&Instruction::LocalGet(today_seconds_local));
        function.instruction(&Instruction::I64Const(SECONDS_PER_DAY));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(tomorrow_seconds_local));
        self.emit_temporal_zoned_date_time_day_boundary(
            tomorrow_seconds_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(tomorrow_seconds_local));
        function.instruction(&Instruction::LocalGet(today_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Const(Ieee64::from(3_600.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            epoch_tag_local,
            epoch_payload_local,
            tomorrow_seconds_local,
            today_seconds_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_zoned_date_time_start_of_day(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
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
        for (offset, local) in [
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
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_local,
            function,
        )?;

        for local in [
            prototype_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            seconds_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
