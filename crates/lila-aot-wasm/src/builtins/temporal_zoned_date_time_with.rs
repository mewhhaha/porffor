//! Zoned date-time field replacement and fixed-zone epoch interpretation.

use super::super::*;
use super::temporal::{TemporalZonedDateTimeOptionsContext, SECONDS_PER_DAY};
use super::temporal_options::{OffsetOption, StringValuedOption};
use super::temporal_plain_date_time_methods::TemporalDateTimeFieldReadMode;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_temporal_zoned_date_time_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let offset_option_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let any_present_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let offset_nanoseconds_local = self.reserve_temp_local();
        let offset_present_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let present_locals = self.reserve_temporal_plain_date_time_field_locals();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
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
            [
                field_locals[0],
                field_locals[1],
                field_locals[2],
                field_locals[3],
                field_locals[4],
                field_locals[5],
                field_locals[6],
            ],
            function,
        )?;
        for local in field_locals.iter().take(7) {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::LocalGet(field_locals[1]));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(field_locals[1]));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(field_locals[7]));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(field_locals[8]));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(offset_nanoseconds_local));
        // The receiver supplies an offset even when the partial object omits it.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(offset_present_local));
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.prototype.with requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // `IsPartialTemporalObject` step 2 runs before the two `Get`s below.
        self.emit_temporal_reject_branded_partial_object(
            argument_payload_local,
            argument_tag_local,
            "Temporal.ZonedDateTime.prototype.with does not accept a Temporal object",
            function,
        )?;

        // `RejectTemporalLikeObject` reads both keys with `Get`, not with a
        // `HasProperty` probe, and Test262's `with/order-of-operations.js`
        // observes the two reads.
        for property in ["calendar", "timeZone"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                key_local,
                present_local,
                calendar_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(calendar_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Temporal.ZonedDateTime.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        for local in present_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        let era = self.emit_temporal_date_time_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            &field_locals,
            &present_locals,
            month_code_payload_local,
            month_code_present_local,
            any_present_local,
            TemporalDateTimeFieldReadMode::ZonedWith {
                offset_nanoseconds_local,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(any_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.prototype.with requires at least one date, time, or offset field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_zoned_date_time_options(
            TemporalZonedDateTimeOptionsContext::With,
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_local,
            function,
        )?;

        // Era resolution before the merge below: `{ era, eraYear }` excludes
        // the receiver's `year`, which is still sitting untouched in
        // `field_locals[0]` because `read_fields` only overwrites a slot the
        // bag actually supplied. `present_locals[0]` is therefore still 0 for
        // an era-only bag, so the era/year agreement check cannot fire against
        // a year the caller never wrote.
        let resolved_year = self.emit_temporal_resolve_era_to_iso_year(
            era,
            calendar_payload_local,
            field_locals[0],
            present_locals[0],
            function,
        )?;

        // `CalendarMergeFields` drops the receiver's `monthCode` as soon as the
        // argument supplies either `month` or `monthCode`, so a lone `month` is
        // never a conflict; every other absent key keeps the receiver's value,
        // which `emit_temporal_date_time_read_fields` already left in
        // place.
        function.instruction(&Instruction::LocalGet(present_locals[1]));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present_locals[1]));
        function.instruction(&Instruction::End);
        for index in [0_usize, 2] {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(present_locals[index]));
        }

        self.emit_temporal_plain_date_resolve_fields(
            &resolved_year,
            field_locals[1],
            present_locals[1],
            month_code_payload_local,
            month_code_present_local,
            field_locals[2],
            present_locals[2],
            overflow_local,
            function,
        )?;
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        self.emit_temporal_regulate_time(&time_locals, overflow_local, function)?;
        self.emit_temporal_fixed_zoned_date_time_epoch(
            &field_locals,
            time_zone_payload_local,
            offset_nanoseconds_local,
            offset_present_local,
            offset_option_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
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
        self.release_temporal_plain_date_time_field_locals(present_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            prototype_payload_local,
            local_time_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            offset_present_local,
            offset_nanoseconds_local,
            offset_seconds_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            epoch_tag_local,
            epoch_payload_local,
            record_local,
            any_present_local,
            month_code_present_local,
            month_code_payload_local,
            present_local,
            key_local,
            offset_option_local,
            overflow_local,
            calendar_tag_local,
            calendar_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_fixed_zoned_date_time_epoch(
        &mut self,
        field_locals: &[u32; 9],
        time_zone_payload_local: u32,
        offset_nanoseconds_local: u32,
        offset_present_local: u32,
        offset_option_local: u32,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [year_local, month_local, day_local, hour_local, minute_local, second_local, millisecond_local, microsecond_local, nanosecond_local] =
            *field_locals;
        let adjusted_year_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        self.emit_temporal_days_from_civil(
            year_local,
            month_local,
            day_local,
            adjusted_year_local,
            era_local,
            month_index_local,
            days_local,
            function,
        );
        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let selected_offset_subsecond_local = self.reserve_temp_local();
        let selected_offset_seconds_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            time_zone_offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::LocalGet(offset_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_temporal_zoned_date_time_offset_date_range(
            days_local,
            offset_option_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Const(OffsetOption::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(offset_nanoseconds_local));
        function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime offset does not match its fixed time zone",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Const(OffsetOption::Use.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(offset_nanoseconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::LocalGet(offset_nanoseconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(SECONDS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(second_local));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(selected_offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(millisecond_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(microsecond_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(nanosecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(selected_offset_subsecond_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_normalize_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        for local in [
            selected_offset_seconds_local,
            selected_offset_subsecond_local,
            time_zone_offset_seconds_local,
            subsecond_local,
            seconds_local,
            days_local,
            month_index_local,
            era_local,
            adjusted_year_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
