//! Zoned date-time day boundaries and transition queries, through the
//! time-zone kernel for offset and named zones alike.

use super::super::*;
use super::temporal_time_zone::TemporalTransitionSearch;

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

    const fn search(self) -> TemporalTransitionSearch {
        match self {
            Self::Next => TemporalTransitionSearch::Next,
            Self::Previous => TemporalTransitionSearch::Previous,
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
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        let transition_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

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

        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_local,
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
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_temporal_epoch_value_seconds(
            epoch_payload_local,
            epoch_tag_local,
            seconds_local,
            subsecond_local,
            function,
        );
        // Offset zones have no transitions; the kernel answers that too.
        let next = TimeZoneTransitionDirection::Next;
        function.instruction(&Instruction::I64Const(self.strings.payload(next.name())));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_string_payload_equality_i32(direction_payload_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_time_zone_transition(
            time_zone_local,
            seconds_local,
            subsecond_local,
            next.search(),
            found_local,
            transition_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_time_zone_transition(
            time_zone_local,
            seconds_local,
            subsecond_local,
            TimeZoneTransitionDirection::Previous.search(),
            found_local,
            transition_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_zoned_date_time_day_boundary(
            transition_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        for local in [
            prototype_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            epoch_tag_local,
            epoch_payload_local,
            transition_local,
            found_local,
            subsecond_local,
            seconds_local,
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

    /// The epoch day of `GetISODateTimeFor(timeZone, epochNs)` for the
    /// receiver's record.
    pub(super) fn emit_temporal_zoned_date_time_local_epoch_day(
        &mut self,
        record_local: u32,
        day_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let offset_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_local,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_pair(
            record_local,
            super::temporal::TemporalEpochNanosecondsRecord::ZonedDateTime,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_normalize_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_time_zone_offset_seconds(
            time_zone_local,
            seconds_local,
            offset_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        for local in [offset_local, subsecond_local, seconds_local, time_zone_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `GetStartOfDay(timeZone, date)` for the receiver's local date shifted by
    /// `day_shift` days, in whole seconds.
    pub(super) fn emit_temporal_zoned_date_time_start_of_day_seconds(
        &mut self,
        record_local: u32,
        day_shift: i64,
        seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_local_epoch_day(record_local, day_local, function)?;
        if day_shift != 0 {
            function.instruction(&Instruction::LocalGet(day_local));
            function.instruction(&Instruction::I64Const(day_shift));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(day_local));
        }
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_local,
            function,
        );
        self.emit_temporal_start_of_epoch_day(time_zone_local, day_local, seconds_local, function)?;
        self.release_temp_local(day_local);
        self.release_temp_local(time_zone_local);
        Ok(())
    }

    /// Whole epoch `seconds_local` as an epoch-nanoseconds value, which must be
    /// a representable instant.
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

    /// `Temporal.ZonedDateTime.prototype.hoursInDay`: the exact length of the
    /// receiver's local day, from `GetStartOfDay` of the day and of the next.
    pub(super) fn emit_temporal_zoned_date_time_hours_in_day(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let today_seconds_local = self.reserve_temp_local();
        let tomorrow_seconds_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_temporal_zoned_date_time_start_of_day_seconds(
            record_local,
            0,
            today_seconds_local,
            function,
        )?;
        self.emit_temporal_zoned_date_time_start_of_day_seconds(
            record_local,
            1,
            tomorrow_seconds_local,
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

        for local in [tomorrow_seconds_local, today_seconds_local, record_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Temporal.ZonedDateTime.prototype.startOfDay`.
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
            0,
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
