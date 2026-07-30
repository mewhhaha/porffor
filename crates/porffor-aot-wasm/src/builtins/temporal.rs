use super::super::*;

const TEMPORAL_INSTANT_LIMIT_HIGH_LIMB: i64 = 468;
const TEMPORAL_INSTANT_LIMIT_LOW_LIMB: i64 = 6_923_773_503_929_843_712;
const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Copy)]
enum TemporalIsoParseGoal {
    Instant,
    TimeZoneIdentifier {
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
    },
    ZonedDateTimeSyntax {
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
    },
    ZonedDateTime {
        offset_option_local: u32,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
    },
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_temporal_instant_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let argument_brand_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            argument_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(argument_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            argument_brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(argument_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            argument_brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(argument_payload_local, argument_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(argument_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant.from requires a string or Temporal.Instant",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_parse_iso_string(
            argument_payload_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            TemporalIsoParseGoal::Instant,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(argument_brand_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let argument_brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let offset_option_local = self.reserve_temp_local();
        let overflow_option_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            argument_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(argument_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_zoned_date_time_options(
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            function,
        )?;
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
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_date_time_from_property_bag(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_zoned_date_time_from_property_bag(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.from requires a string or Temporal.ZonedDateTime",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_parse_iso_string(
            argument_payload_local,
            epoch_payload_local,
            epoch_tag_local,
            TemporalIsoParseGoal::ZonedDateTimeSyntax {
                time_zone_payload_local,
                time_zone_tag_local,
                calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
        self.emit_temporal_zoned_date_time_options(
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            function,
        )?;
        self.emit_temporal_parse_iso_string(
            argument_payload_local,
            epoch_payload_local,
            epoch_tag_local,
            TemporalIsoParseGoal::ZonedDateTime {
                offset_option_local,
                time_zone_payload_local,
                time_zone_tag_local,
                calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
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

        for local in [
            prototype_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            overflow_option_local,
            offset_option_local,
            options_tag_local,
            options_payload_local,
            record_local,
            argument_brand_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_zoned_date_time_from_property_bag(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        offset_option_local: u32,
        overflow_option_local: u32,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let microsecond_local = self.reserve_temp_local();
        let millisecond_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let nanosecond_local = self.reserve_temp_local();
        let offset_payload_local = self.reserve_temp_local();
        let offset_present_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("calendar")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_temporal_zoned_date_time_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;

        for (property, output_local, default, output_present_local) in [
            ("day", day_local, 0_i64, Some(day_present_local)),
            ("hour", hour_local, 0, None),
            ("microsecond", microsecond_local, 0, None),
            ("millisecond", millisecond_local, 0, None),
            ("minute", minute_local, 0, None),
            ("month", month_local, 0, Some(month_present_local)),
        ] {
            self.emit_temporal_property_bag_integer(
                argument_payload_local,
                argument_tag_local,
                property,
                property_key_local,
                value_payload_local,
                value_tag_local,
                present_local,
                output_local,
                default,
                function,
            )?;
            if let Some(output_present_local) = output_present_local {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::LocalSet(output_present_local));
            }
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("monthCode")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        self.emit_temporal_property_bag_string(
            value_payload_local,
            value_tag_local,
            "Temporal.ZonedDateTime monthCode must be a string",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(month_code_payload_local));

        for (property, output_local) in [("nanosecond", nanosecond_local)] {
            self.emit_temporal_property_bag_integer(
                argument_payload_local,
                argument_tag_local,
                property,
                property_key_local,
                value_payload_local,
                value_tag_local,
                present_local,
                output_local,
                0,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("offset")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(offset_present_local));
        self.emit_temporal_property_bag_string(
            value_payload_local,
            value_tag_local,
            "Temporal.ZonedDateTime offset must be a string",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(offset_payload_local));

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "second",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            second_local,
            0,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("timeZone")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "year",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            year_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires year",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires day",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_zoned_date_time_options(
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires timeZone",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        for month in 1_i64..=12 {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&format!("M{month:02}")),
            ));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_string_payload_equality_i32(
                month_code_payload_local,
                value_tag_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime month and monthCode must agree",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime month and day must be positive",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_regulate_property_bag_date_time(
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            millisecond_local,
            microsecond_local,
            nanosecond_local,
            overflow_option_local,
            function,
        )?;

        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let selected_offset_seconds_local = self.reserve_temp_local();
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
        self.emit_temporal_fixed_time_zone_offset_seconds(
            offset_payload_local,
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
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
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

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
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
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

        for local in [
            subsecond_local,
            seconds_local,
            days_local,
            month_index_local,
            era_local,
            adjusted_year_local,
            selected_offset_seconds_local,
            offset_seconds_local,
            time_zone_offset_seconds_local,
            year_local,
            second_local,
            offset_present_local,
            offset_payload_local,
            nanosecond_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            month_local,
            minute_local,
            millisecond_local,
            microsecond_local,
            hour_local,
            day_present_local,
            day_local,
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_regulate_property_bag_date_time(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        hour_local: u32,
        minute_local: u32,
        second_local: u32,
        millisecond_local: u32,
        microsecond_local: u32,
        nanosecond_local: u32,
        overflow_option_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(-271_821));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(275_760));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag year is outside the supported instant range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(overflow_option_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag month is out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        for month in [4_i64, 6, 9, 11] {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::LocalSet(maximum_day_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(28));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        for (local, minimum, maximum) in [
            (hour_local, 0_i64, 23_i64),
            (minute_local, 0, 59),
            (second_local, 0, 59),
            (millisecond_local, 0, 999),
            (microsecond_local, 0, 999),
            (nanosecond_local, 0, 999),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(invalid_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(overflow_option_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag date-time field is out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        for (local, minimum, maximum) in [
            (hour_local, 0_i64, 23_i64),
            (minute_local, 0, 59),
            (second_local, 0, 59),
            (millisecond_local, 0, 999),
            (microsecond_local, 0, 999),
            (nanosecond_local, 0, 999),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_property_bag_integer(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        property: &str,
        property_key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        present_local: u32,
        output_local: u32,
        default: i64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload(property)));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(present_local));
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(default));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag field must be finite",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_temporal_property_bag_string(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_options(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        offset_option_local: u32,
        overflow_option_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();
        let option_payload_local = self.reserve_temp_local();
        let option_tag_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(offset_option_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(overflow_option_local));
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.from options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (property, allowed_values) in [
            (
                "disambiguation",
                &[
                    ("compatible", None),
                    ("earlier", None),
                    ("later", None),
                    ("reject", None),
                ][..],
            ),
            (
                "offset",
                &[
                    ("reject", Some(0)),
                    ("use", Some(1)),
                    ("prefer", Some(2)),
                    ("ignore", Some(3)),
                ][..],
            ),
            (
                "overflow",
                &[("constrain", Some(0)), ("reject", Some(1))][..],
            ),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(property_key_local));
            self.emit_object_read(
                options_payload_local,
                options_tag_local,
                options_payload_local,
                options_tag_local,
                property_key_local,
                option_payload_local,
                option_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(option_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_value_to_string_payload(option_payload_local, option_tag_local, function)?;
            function.instruction(&Instruction::LocalSet(option_payload_local));
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(recognized_local));
            for (expected, offset_code) in allowed_values {
                function.instruction(&Instruction::I64Const(self.strings.payload(expected)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    option_payload_local,
                    expected_payload_local,
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(recognized_local));
                if let Some(offset_code) = offset_code {
                    function.instruction(&Instruction::I64Const(*offset_code));
                    function.instruction(&Instruction::LocalSet(if property == "overflow" {
                        overflow_option_local
                    } else {
                        offset_option_local
                    }));
                }
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::LocalGet(recognized_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                match property {
                    "disambiguation" => "Invalid Temporal.ZonedDateTime disambiguation option",
                    "offset" => "Invalid Temporal.ZonedDateTime offset option",
                    "overflow" => "Invalid Temporal.ZonedDateTime overflow option",
                    _ => unreachable!(),
                },
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(recognized_local);
        self.release_temp_local(expected_payload_local);
        self.release_temp_local(option_tag_local);
        self.release_temp_local(option_payload_local);
        self.release_temp_local(property_key_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let epoch_argument_payload_local = self.reserve_temp_local();
        let epoch_argument_tag_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(
            0,
            epoch_argument_payload_local,
            epoch_argument_tag_local,
            function,
        );
        self.emit_value_to_bigint_locals(
            epoch_argument_tag_local,
            epoch_argument_payload_local,
            false,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        self.emit_builtin_arg_to_locals(1, time_zone_payload_local, time_zone_tag_local, function);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(2, calendar_payload_local, calendar_tag_local, function);
        self.emit_temporal_zoned_date_time_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
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

        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(calendar_tag_local);
        self.release_temp_local(calendar_payload_local);
        self.release_temp_local(time_zone_tag_local);
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(epoch_argument_tag_local);
        self.release_temp_local(epoch_argument_payload_local);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_time_zone(
        &mut self,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let object_brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let first_byte_local = self.reserve_temp_local();
        let direct_identifier_local = self.reserve_temp_local();
        let unused_nanoseconds_payload_local = self.reserve_temp_local();
        let unused_nanoseconds_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            time_zone_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            object_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            time_zone_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
            time_zone_tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime time zone must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(direct_identifier_local));
        self.emit_unpack_string_payload(
            time_zone_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(
            string_offset_local,
            direct_identifier_local,
            first_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(direct_identifier_local));
        function.instruction(&Instruction::End);

        let expected_utc_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
        function.instruction(&Instruction::LocalSet(expected_utc_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(case_fold_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            time_zone_payload_local,
            expected_utc_payload_local,
            Some(case_fold_local),
            function,
        );
        function.instruction(&Instruction::LocalGet(direct_identifier_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            time_zone_offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_iso_string(
            time_zone_payload_local,
            unused_nanoseconds_payload_local,
            unused_nanoseconds_tag_local,
            TemporalIsoParseGoal::TimeZoneIdentifier {
                time_zone_payload_local,
                time_zone_tag_local,
            },
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_utc_payload_local);
        self.release_temp_local(unused_nanoseconds_tag_local);
        self.release_temp_local(unused_nanoseconds_payload_local);
        self.release_temp_local(direct_identifier_local);
        self.release_temp_local(first_byte_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        self.release_temp_local(record_local);
        self.release_temp_local(object_brand_local);
        self.release_temp_local(time_zone_offset_seconds_local);
        Ok(())
    }

    fn emit_temporal_fixed_time_zone_offset_seconds(
        &mut self,
        time_zone_payload_local: u32,
        time_zone_offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
        function.instruction(&Instruction::LocalSet(expected_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(case_fold_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            time_zone_payload_local,
            expected_payload_local,
            Some(case_fold_local),
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(time_zone_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(time_zone_offset_seconds_local));
        function.instruction(&Instruction::Else);
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let has_minute_local = self.reserve_temp_local();
        self.emit_unpack_string_payload(
            time_zone_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(minute_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            hour_local,
            2,
            function,
        );
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            minute_local,
            2,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime time zone",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(time_zone_offset_seconds_local));
        self.emit_temporal_format_fixed_time_zone_offset(
            time_zone_offset_seconds_local,
            time_zone_payload_local,
            function,
        )?;
        self.release_temp_local(has_minute_local);
        self.release_temp_local(minute_local);
        self.release_temp_local(hour_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(valid_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        function.instruction(&Instruction::End);
        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_calendar(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
        function.instruction(&Instruction::LocalSet(expected_payload_local));
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime calendar must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(case_fold_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            calendar_payload_local,
            expected_payload_local,
            Some(case_fold_local),
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime calendar",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_alloc_temporal_zoned_date_time(
        &mut self,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.store_i64_local_at_offset(record_local, offset, local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_epoch_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_epoch_milliseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            quotient_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(negative_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_offset(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_seconds_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_offset_seconds_from_receiver(
            offset_seconds_local,
            function,
        )?;
        self.emit_temporal_format_fixed_time_zone_offset(
            offset_seconds_local,
            output_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(output_payload_local);
        self.release_temp_local(offset_seconds_local);
        Ok(())
    }

    fn emit_temporal_format_fixed_time_zone_offset(
        &mut self,
        offset_seconds_local: u32,
        output_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let magnitude_seconds_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let separator_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(hour_payload_local));
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(minute_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            hour_payload_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload(":")));
        function.instruction(&Instruction::LocalSet(separator_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            separator_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            minute_payload_local,
            2,
            function,
        )?;

        self.release_temp_local(separator_payload_local);
        self.release_temp_local(minute_payload_local);
        self.release_temp_local(hour_payload_local);
        self.release_temp_local(magnitude_seconds_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_offset_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_seconds_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_offset_seconds_from_receiver(
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(offset_seconds_local);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_offset_seconds_from_receiver(
        &mut self,
        offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_payload_local,
            function,
        );
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            offset_seconds_local,
            function,
        )?;
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_time_zone_id(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_string_slot(
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
            function,
        )
    }

    pub(crate) fn emit_temporal_zoned_date_time_calendar_id(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_string_slot(
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
            function,
        )
    }

    pub(crate) fn emit_temporal_zoned_date_time_iso_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let day_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let millisecond_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(local_time_payload_local));
        self.emit_date_components_from_time(
            local_time_payload_local,
            year_payload_local,
            month_payload_local,
            day_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            millisecond_payload_local,
            function,
        );

        match builtin {
            StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter => {
                function.instruction(&Instruction::LocalGet(year_payload_local));
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter => {
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                function.instruction(&Instruction::F64Add);
                function.instruction(&Instruction::I64ReinterpretF64);
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter => {
                function.instruction(&Instruction::LocalGet(day_payload_local));
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter => {
                function.instruction(&Instruction::LocalGet(hour_payload_local));
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter => {
                function.instruction(&Instruction::LocalGet(minute_payload_local));
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter => {
                function.instruction(&Instruction::LocalGet(second_payload_local));
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter => {
                function.instruction(&Instruction::LocalGet(millisecond_payload_local));
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter => {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(1_000));
                function.instruction(&Instruction::I64DivU);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter => {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(1_000));
                function.instruction(&Instruction::I64RemU);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
            }
            StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter => {
                let month_number_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(month_number_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
                function.instruction(&Instruction::LocalSet(self.result_local));
                for month in 2..=12 {
                    function.instruction(&Instruction::LocalGet(month_number_local));
                    function.instruction(&Instruction::I64Const(month));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(&format!("M{month:02}")),
                    ));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(month_number_local);
            }
            _ => unreachable!(),
        }
        if builtin != StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter {
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
        }

        for local in [
            millisecond_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            day_payload_local,
            month_payload_local,
            year_payload_local,
            local_time_payload_local,
            time_zone_payload_local,
            offset_seconds_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_record_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let other_payload_local = self.reserve_temp_local();
        let other_tag_local = self.reserve_temp_local();
        let other_record_local = self.reserve_temp_local();
        let receiver_epoch_payload_local = self.reserve_temp_local();
        let receiver_epoch_tag_local = self.reserve_temp_local();
        let other_epoch_payload_local = self.reserve_temp_local();
        let other_epoch_tag_local = self.reserve_temp_local();
        let receiver_time_zone_local = self.reserve_temp_local();
        let other_time_zone_local = self.reserve_temp_local();
        let receiver_calendar_local = self.reserve_temp_local();
        let other_calendar_local = self.reserve_temp_local();
        let equal_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(receiver_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimeFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.from`",
                )
            })?;
        self.emit_direct_js_call(
            &from_meta,
            None,
            &[(argument_payload_local, argument_tag_local)],
            other_payload_local,
            other_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record_local,
            function,
        );
        for (record, offset, local) in [
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                receiver_epoch_payload_local,
            ),
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                receiver_epoch_tag_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                other_epoch_payload_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                other_epoch_tag_local,
            ),
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                receiver_time_zone_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                other_time_zone_local,
            ),
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                receiver_calendar_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                other_calendar_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record, offset, local, function);
        }
        function.instruction(&Instruction::LocalGet(receiver_epoch_tag_local));
        function.instruction(&Instruction::LocalGet(other_epoch_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_tagged_payload_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_tag_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_mixed_bigint_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(equal_local));
        self.emit_string_payload_equality_i32(
            receiver_time_zone_local,
            other_time_zone_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(equal_local));
        self.emit_string_payload_equality_i32(
            receiver_calendar_local,
            other_calendar_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            equal_local,
            other_calendar_local,
            receiver_calendar_local,
            other_time_zone_local,
            receiver_time_zone_local,
            other_epoch_tag_local,
            other_epoch_payload_local,
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_record_local,
            other_tag_local,
            other_payload_local,
            argument_tag_local,
            argument_payload_local,
            receiver_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_temporal_zoned_date_time_string_slot(
        &mut self,
        payload_offset: u64,
        tag_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            payload_offset,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            tag_offset,
            self.result_tag_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_to_instant(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            epoch_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            epoch_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            epoch_payload_local,
            epoch_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_with_time_zone(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, time_zone_payload_local, time_zone_tag_local, function);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
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

        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(calendar_tag_local);
        self.release_temp_local(calendar_payload_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(time_zone_tag_local);
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime receiver does not have [[InitializedTemporalZonedDateTime]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime receiver does not have [[InitializedTemporalZonedDateTime]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_value_to_bigint_locals(
            argument_tag_local,
            argument_payload_local,
            false,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_temporal_instant_validate_range(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    fn emit_temporal_parse_iso_string(
        &mut self,
        string_payload_local: u32,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        parse_goal: TemporalIsoParseGoal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let main_end_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let negative_year_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let fraction_local = self.reserve_temp_local();
        let fraction_digits_local = self.reserve_temp_local();
        let date_separated_local = self.reserve_temp_local();
        let time_separated_local = self.reserve_temp_local();
        let has_minute_local = self.reserve_temp_local();
        let has_second_local = self.reserve_temp_local();
        let has_time_local = self.reserve_temp_local();
        let offset_kind_local = self.reserve_temp_local();
        let offset_sign_local = self.reserve_temp_local();
        let offset_hour_local = self.reserve_temp_local();
        let offset_minute_local = self.reserve_temp_local();
        let offset_second_local = self.reserve_temp_local();
        let offset_has_second_local = self.reserve_temp_local();
        let offset_fraction_local = self.reserve_temp_local();
        let offset_fraction_digits_local = self.reserve_temp_local();
        let maximum_day_local = self.reserve_temp_local();
        let calendar_count_local = self.reserve_temp_local();
        let calendar_critical_local = self.reserve_temp_local();
        let timezone_count_local = self.reserve_temp_local();
        let annotation_start_local = self.reserve_temp_local();
        let annotation_equals_local = self.reserve_temp_local();
        let annotation_critical_local = self.reserve_temp_local();
        let annotation_key_uppercase_local = self.reserve_temp_local();
        let annotation_numeric_timezone_local = self.reserve_temp_local();
        let annotation_colon_count_local = self.reserve_temp_local();
        let time_zone_start_local = self.reserve_temp_local();
        let time_zone_end_local = self.reserve_temp_local();
        let calendar_start_local = self.reserve_temp_local();
        let calendar_end_local = self.reserve_temp_local();
        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let selected_offset_seconds_local = self.reserve_temp_local();
        let selected_offset_subsecond_local = self.reserve_temp_local();
        let offset_matches_time_zone_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let adjusted_year_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let parse_locals = [
            string_offset_local,
            string_len_local,
            main_end_local,
            cursor_local,
            byte_local,
            valid_local,
            negative_year_local,
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            fraction_local,
            fraction_digits_local,
            date_separated_local,
            time_separated_local,
            has_minute_local,
            has_second_local,
            has_time_local,
            offset_kind_local,
            offset_sign_local,
            offset_hour_local,
            offset_minute_local,
            offset_second_local,
            offset_has_second_local,
            offset_fraction_local,
            offset_fraction_digits_local,
            maximum_day_local,
            calendar_count_local,
            calendar_critical_local,
            timezone_count_local,
            annotation_start_local,
            annotation_equals_local,
            annotation_critical_local,
            annotation_key_uppercase_local,
            annotation_numeric_timezone_local,
            annotation_colon_count_local,
            time_zone_start_local,
            time_zone_end_local,
            calendar_start_local,
            calendar_end_local,
            time_zone_offset_seconds_local,
            selected_offset_seconds_local,
            selected_offset_subsecond_local,
            offset_matches_time_zone_local,
            days_local,
            era_local,
            adjusted_year_local,
            month_index_local,
            seconds_local,
            subsecond_local,
        ];

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        for (local, value) in [
            (cursor_local, 0),
            (valid_local, 1),
            (negative_year_local, 0),
            (date_separated_local, 0),
            (time_separated_local, 0),
            (has_minute_local, 0),
            (has_second_local, 0),
            (has_time_local, 0),
            (offset_kind_local, 0),
            (fraction_local, 0),
            (fraction_digits_local, 0),
            (offset_sign_local, 0),
            (offset_hour_local, 0),
            (offset_minute_local, 0),
            (offset_second_local, 0),
            (offset_has_second_local, 0),
            (offset_fraction_local, 0),
            (offset_fraction_digits_local, 0),
            (calendar_count_local, 0),
            (calendar_critical_local, 0),
            (timezone_count_local, 0),
            (time_zone_start_local, -1),
            (time_zone_end_local, -1),
            (calendar_start_local, -1),
            (calendar_end_local, -1),
            (time_zone_offset_seconds_local, 0),
            (selected_offset_seconds_local, 0),
            (selected_offset_subsecond_local, 0),
            (offset_matches_time_zone_local, 0),
        ] {
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::LocalSet(local));
        }

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(main_end_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(main_end_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            year_local,
            6,
            function,
        );
        function.instruction(&Instruction::LocalGet(negative_year_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            year_local,
            4,
            function,
        );
        function.instruction(&Instruction::End);

        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(date_separated_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            month_local,
            2,
            function,
        );
        function.instruction(&Instruction::LocalGet(date_separated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_temporal_expect_byte(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            b'-',
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            day_local,
            2,
            function,
        );

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_time_local));
        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            hour_local,
            2,
            function,
        );

        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(time_separated_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_minute_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(minute_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            minute_local,
            2,
            function,
        );
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(time_separated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_second_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(second_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            second_local,
            2,
            function,
        );
        self.emit_temporal_parse_optional_fraction(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            fraction_local,
            fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(offset_kind_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(offset_kind_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(offset_sign_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            offset_hour_local,
            2,
            function,
        );
        self.emit_temporal_parse_offset_tail(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            offset_minute_local,
            offset_second_local,
            offset_has_second_local,
            offset_fraction_local,
            offset_fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        if matches!(parse_goal, TemporalIsoParseGoal::Instant) {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        for local in [
            hour_local,
            minute_local,
            second_local,
            fraction_local,
            fraction_digits_local,
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        if matches!(parse_goal, TemporalIsoParseGoal::Instant) {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_validate_annotations(
            string_offset_local,
            main_end_local,
            string_len_local,
            cursor_local,
            byte_local,
            valid_local,
            calendar_count_local,
            calendar_critical_local,
            timezone_count_local,
            annotation_start_local,
            annotation_equals_local,
            annotation_critical_local,
            annotation_key_uppercase_local,
            annotation_numeric_timezone_local,
            annotation_colon_count_local,
            time_zone_start_local,
            time_zone_end_local,
            calendar_start_local,
            calendar_end_local,
            function,
        );

        self.emit_temporal_validate_date_time(
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            offset_hour_local,
            offset_minute_local,
            offset_second_local,
            maximum_day_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            match parse_goal {
                TemporalIsoParseGoal::Instant => "Invalid Temporal.Instant string",
                TemporalIsoParseGoal::TimeZoneIdentifier { .. } => {
                    "Invalid Temporal time zone identifier"
                }
                TemporalIsoParseGoal::ZonedDateTimeSyntax { .. }
                | TemporalIsoParseGoal::ZonedDateTime { .. } => {
                    "Invalid Temporal.ZonedDateTime string"
                }
            },
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if let TemporalIsoParseGoal::TimeZoneIdentifier {
            time_zone_payload_local,
            time_zone_tag_local,
        } = parse_goal
        {
            function.instruction(&Instruction::LocalGet(timezone_count_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal time zone string requires an offset or bracketed time zone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
            function.instruction(&Instruction::LocalSet(time_zone_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(offset_has_second_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(offset_fraction_digits_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal time zone offset must use minute precision",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(offset_sign_local));
            function.instruction(&Instruction::LocalGet(offset_hour_local));
            function.instruction(&Instruction::I64Const(3_600));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(offset_minute_local));
            function.instruction(&Instruction::I64Const(60));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(time_zone_offset_seconds_local));
            self.emit_temporal_format_fixed_time_zone_offset(
                time_zone_offset_seconds_local,
                time_zone_payload_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(time_zone_end_local));
            function.instruction(&Instruction::LocalGet(time_zone_start_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_slice_payload_from_locals(
                string_payload_local,
                time_zone_start_local,
                self.scratch_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(time_zone_payload_local));
            self.emit_temporal_fixed_time_zone_offset_seconds(
                time_zone_payload_local,
                time_zone_offset_seconds_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(time_zone_tag_local));

            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        if let TemporalIsoParseGoal::ZonedDateTimeSyntax {
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
        }
        | TemporalIsoParseGoal::ZonedDateTime {
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            ..
        } = parse_goal
        {
            function.instruction(&Instruction::LocalGet(timezone_count_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.ZonedDateTime string requires one bracketed time zone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(time_zone_end_local));
            function.instruction(&Instruction::LocalGet(time_zone_start_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_slice_payload_from_locals(
                string_payload_local,
                time_zone_start_local,
                self.scratch_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(time_zone_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(time_zone_tag_local));
            self.emit_temporal_fixed_time_zone_offset_seconds(
                time_zone_payload_local,
                time_zone_offset_seconds_local,
                function,
            )?;

            function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
            function.instruction(&Instruction::LocalSet(calendar_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(calendar_tag_local));
            function.instruction(&Instruction::LocalGet(calendar_count_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            let calendar_annotation_payload_local = self.reserve_temp_local();
            let expected_calendar_payload_local = self.reserve_temp_local();
            let case_fold_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(calendar_end_local));
            function.instruction(&Instruction::LocalGet(calendar_start_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_slice_payload_from_locals(
                string_payload_local,
                calendar_start_local,
                self.scratch_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(calendar_annotation_payload_local));
            function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
            function.instruction(&Instruction::LocalSet(expected_calendar_payload_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(case_fold_local));
            self.emit_string_payload_equality_i32_with_ascii_case_folding(
                calendar_annotation_payload_local,
                expected_calendar_payload_local,
                Some(case_fold_local),
                function,
            );
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.ZonedDateTime calendar annotation",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            self.release_temp_local(case_fold_local);
            self.release_temp_local(expected_calendar_payload_local);
            self.release_temp_local(calendar_annotation_payload_local);
            function.instruction(&Instruction::End);
        }

        if matches!(parse_goal, TemporalIsoParseGoal::ZonedDateTimeSyntax { .. }) {
            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        self.emit_temporal_scale_fraction_to_nanoseconds(
            fraction_local,
            fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::LocalSet(fraction_local));
        self.emit_temporal_scale_fraction_to_nanoseconds(
            offset_fraction_local,
            offset_fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::LocalSet(offset_fraction_local));
        function.instruction(&Instruction::LocalGet(offset_sign_local));
        function.instruction(&Instruction::LocalGet(offset_hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(offset_minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_second_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::LocalGet(offset_sign_local));
        function.instruction(&Instruction::LocalGet(offset_fraction_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));

        if let TemporalIsoParseGoal::ZonedDateTime {
            offset_option_local,
            ..
        } = parse_goal
        {
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
            function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(selected_offset_seconds_local));
            function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(selected_offset_subsecond_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(offset_matches_time_zone_local));

            function.instruction(&Instruction::LocalGet(offset_option_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(offset_matches_time_zone_local));
            function.instruction(&Instruction::I64Eqz);
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
            function.instruction(&Instruction::I64Const(3));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(offset_option_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(offset_matches_time_zone_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
            function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

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
        function.instruction(&Instruction::LocalGet(fraction_local));
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
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;

        for local in parse_locals.iter().rev() {
            self.release_temp_local(*local);
        }
        Ok(())
    }

    fn emit_temporal_advance_cursor(&mut self, cursor_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
    }

    fn emit_temporal_byte_is_digit(&mut self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    fn emit_temporal_peek_byte(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_peek_byte_if_available(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::End);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_expect_byte(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        expected: u8,
        function: &mut Function,
    ) {
        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_fixed_decimal(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        destination_local: u32,
        width: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(destination_local));
        for _ in 0..width {
            self.emit_temporal_peek_byte(
                string_offset_local,
                cursor_local,
                end_local,
                byte_local,
                valid_local,
                function,
            );
            self.emit_temporal_byte_is_digit(byte_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(destination_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'0' as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(destination_local));
            self.emit_temporal_advance_cursor(cursor_local, function);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_optional_fraction(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        fraction_local: u32,
        fraction_digits_local: u32,
        function: &mut Function,
    ) {
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_digits_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_offset_tail(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        minute_local: u32,
        second_local: u32,
        has_second_local: u32,
        fraction_local: u32,
        fraction_digits_local: u32,
        function: &mut Function,
    ) {
        let separated_local = self.reserve_temp_local();
        let has_minute_local = self.reserve_temp_local();
        for local in [separated_local, has_minute_local, has_second_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(separated_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_minute_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            minute_local,
            2,
            function,
        );
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(separated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_second_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            second_local,
            2,
            function,
        );
        self.emit_temporal_parse_optional_fraction(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            fraction_local,
            fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(has_minute_local);
        self.release_temp_local(separated_local);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_validate_annotations(
        &mut self,
        string_offset_local: u32,
        main_end_local: u32,
        string_len_local: u32,
        cursor_local: u32,
        byte_local: u32,
        valid_local: u32,
        calendar_count_local: u32,
        calendar_critical_local: u32,
        timezone_count_local: u32,
        annotation_start_local: u32,
        annotation_equals_local: u32,
        annotation_critical_local: u32,
        annotation_key_uppercase_local: u32,
        annotation_numeric_timezone_local: u32,
        annotation_colon_count_local: u32,
        time_zone_start_local: u32,
        time_zone_end_local: u32,
        calendar_start_local: u32,
        calendar_end_local: u32,
        function: &mut Function,
    ) {
        let annotation_end_local = self.reserve_temp_local();
        let key_is_calendar_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_expect_byte(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            b'[',
            function,
        );
        for (local, value) in [
            (annotation_critical_local, 0),
            (annotation_key_uppercase_local, 0),
            (annotation_numeric_timezone_local, 0),
            (annotation_colon_count_local, 0),
            (annotation_equals_local, -1),
        ] {
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'!' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(annotation_critical_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_start_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_end_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_end_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'=' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_equals_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(annotation_key_uppercase_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_colon_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(annotation_colon_count_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_key_uppercase_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_is_calendar_local));
        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (offset, expected) in [(0, b'u'), (1, b'-'), (2, b'c'), (3, b'a')] {
            function.instruction(&Instruction::LocalGet(annotation_start_local));
            function.instruction(&Instruction::I64Const(offset));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_load_string_byte(
                string_offset_local,
                self.scratch_local,
                byte_local,
                function,
            );
            // emit_load_string_byte consumes a local index, so materialize it.
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Eq);
            if offset == 0 {
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(key_is_calendar_local));
            } else {
                function.instruction(&Instruction::LocalGet(key_is_calendar_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(key_is_calendar_local));
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_is_calendar_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(calendar_start_local));
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalSet(calendar_end_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(calendar_count_local));
        function.instruction(&Instruction::LocalGet(calendar_critical_local));
        function.instruction(&Instruction::LocalGet(annotation_critical_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(calendar_critical_local));
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(calendar_critical_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(annotation_critical_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(timezone_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::LocalSet(time_zone_start_local));
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalSet(time_zone_end_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(timezone_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(timezone_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(
            string_offset_local,
            annotation_start_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(key_is_calendar_local);
        self.release_temp_local(annotation_end_local);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_validate_date_time(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        hour_local: u32,
        minute_local: u32,
        second_local: u32,
        offset_hour_local: u32,
        offset_minute_local: u32,
        offset_second_local: u32,
        maximum_day_local: u32,
        valid_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        for month in [4_i64, 6, 9, 11] {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::LocalSet(maximum_day_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(28));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        function.instruction(&Instruction::End);

        for (local, minimum, maximum) in [
            (month_local, 1_i64, 12_i64),
            (day_local, 1, 31),
            (hour_local, 0, 23),
            (minute_local, 0, 59),
            (second_local, 0, 60),
            (offset_hour_local, 0, 23),
            (offset_minute_local, 0, 59),
            (offset_second_local, 0, 59),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_scale_fraction_to_nanoseconds(
        &mut self,
        fraction_local: u32,
        digit_count_local: u32,
        function: &mut Function,
    ) {
        let counter_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalSet(counter_local));
        for _ in 0..9 {
            function.instruction(&Instruction::LocalGet(counter_local));
            function.instruction(&Instruction::I64Const(9));
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(fraction_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(fraction_local));
            function.instruction(&Instruction::LocalGet(counter_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(counter_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(fraction_local));
        self.release_temp_local(counter_local);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_days_from_civil(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        adjusted_year_local: u32,
        era_local: u32,
        month_index_local: u32,
        days_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(adjusted_year_local));
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::I64Const(399));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(era_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-9));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(month_index_local));
        function.instruction(&Instruction::LocalGet(era_local));
        function.instruction(&Instruction::I64Const(146_097));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::LocalGet(era_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(days_local));
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(month_index_local));
        function.instruction(&Instruction::I64Const(153));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(719_468));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
    }

    fn emit_temporal_normalize_seconds_and_subseconds(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_epoch_nanoseconds_bigint(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let negative_local = self.reserve_temp_local();
        let magnitude_seconds_local = self.reserve_temp_local();
        let magnitude_subsecond_local = self.reserve_temp_local();
        let low_word_local = self.reserve_temp_local();
        let low_product_local = self.reserve_temp_local();
        let high_product_local = self.reserve_temp_local();
        let low_limb_local = self.reserve_temp_local();
        let high_limb_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::LocalSet(magnitude_subsecond_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(magnitude_subsecond_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_subsecond_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(low_word_local));
        function.instruction(&Instruction::LocalGet(low_word_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(low_product_local));
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(high_product_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_limb_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_limb_local));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(magnitude_subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_limb_local));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(magnitude_subsecond_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_limb_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(nanoseconds_tag_local));
        function.instruction(&Instruction::Else);
        let record_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.emit_heap_alloc_const(16, function)?;
        function.instruction(&Instruction::LocalSet(limbs_local));
        self.store_i64_local_at_offset(limbs_local, 0, low_limb_local, function);
        self.store_i64_local_at_offset(limbs_local, 8, high_limb_local, function);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(limb_count_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(low_word_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_SIGN_OFFSET,
            low_word_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_CAP_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::LocalSet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(nanoseconds_tag_local));
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(record_local);
        function.instruction(&Instruction::End);

        self.release_temp_local(high_limb_local);
        self.release_temp_local(low_limb_local);
        self.release_temp_local(high_product_local);
        self.release_temp_local(low_product_local);
        self.release_temp_local(low_word_local);
        self.release_temp_local(magnitude_subsecond_local);
        self.release_temp_local(magnitude_seconds_local);
        self.release_temp_local(negative_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_temporal_instant(
        &mut self,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let instant_payload_local = self.reserve_temp_local();
        let instant_record_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(instant_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_INSTANT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(instant_record_local));
        self.store_i64_local_at_offset(
            instant_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            instant_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            instant_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT,
            function,
        );
        self.store_i64_local_at_offset(
            instant_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            instant_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(instant_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(instant_record_local);
        self.release_temp_local(instant_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_epoch_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_epoch_milliseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();

        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            quotient_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(negative_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_record_local = self.reserve_temp_local();
        let receiver_epoch_payload_local = self.reserve_temp_local();
        let receiver_epoch_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let other_instant_payload_local = self.reserve_temp_local();
        let other_instant_tag_local = self.reserve_temp_local();
        let other_record_local = self.reserve_temp_local();
        let other_epoch_payload_local = self.reserve_temp_local();
        let other_epoch_tag_local = self.reserve_temp_local();

        self.emit_temporal_instant_record_from_receiver(receiver_record_local, function)?;
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            receiver_epoch_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            receiver_epoch_tag_local,
            function,
        );

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        let instant_from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.Instant.from`",
                )
            })?;
        self.emit_direct_js_call(
            &instant_from_meta,
            None,
            &[(argument_payload_local, argument_tag_local)],
            other_instant_payload_local,
            other_instant_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_instant_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            other_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            other_epoch_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            other_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            other_epoch_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(receiver_epoch_tag_local));
        function.instruction(&Instruction::LocalGet(other_epoch_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_tagged_payload_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_tag_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_mixed_bigint_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            other_epoch_tag_local,
            other_epoch_payload_local,
            other_record_local,
            other_instant_tag_local,
            other_instant_payload_local,
            argument_tag_local,
            argument_payload_local,
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            receiver_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_to_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let millisecond_payload_local = self.reserve_temp_local();
        let fraction_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let absolute_year_payload_local = self.reserve_temp_local();

        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(time_payload_local));
        self.emit_date_components_from_time(
            time_payload_local,
            year_payload_local,
            month_payload_local,
            date_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            millisecond_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(millisecond_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(9_999.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(month_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(month_payload_local));
        for (component_payload_local, minimum_width, separator) in [
            (month_payload_local, 2, "-"),
            (date_payload_local, 2, "T"),
            (hour_payload_local, 2, ":"),
            (minute_payload_local, 2, ":"),
        ] {
            self.emit_date_append_padded_decimal(
                output_payload_local,
                component_payload_local,
                minimum_width,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }
        self.emit_date_append_padded_decimal(
            output_payload_local,
            second_payload_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            piece_payload_local,
            3,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            piece_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            piece_payload_local,
            9,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("Z")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            absolute_year_payload_local,
            piece_payload_local,
            output_payload_local,
            fraction_local,
            millisecond_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            date_payload_local,
            month_payload_local,
            year_payload_local,
            time_payload_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_temporal_instant_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant receiver does not have [[InitializedTemporalInstant]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant receiver does not have [[InitializedTemporalInstant]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_temporal_instant_validate_range(
        &mut self,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        let high_limb_local = self.reserve_temp_local();
        let low_limb_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            nanoseconds_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            nanoseconds_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            nanoseconds_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_instant_range_error(function)?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(limbs_local, 0, low_limb_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(high_limb_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(limbs_local, 8, high_limb_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_INSTANT_LIMIT_HIGH_LIMB));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_INSTANT_LIMIT_HIGH_LIMB));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_INSTANT_LIMIT_LOW_LIMB));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_instant_range_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(low_limb_local);
        self.release_temp_local(high_limb_local);
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    fn emit_temporal_instant_range_error(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Temporal.Instant epoch nanoseconds are outside the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_temporal_heap_bigint_millisecond_quotient(
        &mut self,
        bigint_payload_local: u32,
        quotient_local: u32,
        remainder_local: u32,
        negative_local: u32,
        function: &mut Function,
    ) {
        let sign_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let word_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(word_local));
        function.instruction(&Instruction::LocalGet(word_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::LocalGet(word_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(word_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(word_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(word_local);
        self.release_temp_local(limb_local);
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(sign_local);
    }
}
