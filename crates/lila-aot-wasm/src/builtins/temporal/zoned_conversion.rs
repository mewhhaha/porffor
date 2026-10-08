//! The complete ZonedDateTime conversion has one emitted body. The caller's
//! real FunctionContext owns intrinsic allocation and errors across every hook.
use super::*;
use crate::runtime_helpers::{
    HelperParameters, TemporalZonedDateTimeConvertArguments, TemporalZonedDateTimeConvertParameters,
};

impl FunctionBuilder<'_> {
    pub(super) fn emit_temporal_zoned_date_time_from_input(
        &mut self,
        input: &ValueLocals,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context = self.current_function_context().ok_or_else(|| {
            EmitError::unsupported("ZonedDateTime conversion requires an actual callable context")
        })?;
        self.runtime_schema()
            .call_helper(
                TemporalZonedDateTimeConvertArguments::new(
                    input,
                    options,
                    context,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    pub(crate) fn compile_temporal_zoned_date_time_convert_helper(
        &mut self,
        real: bool,
    ) -> Result<Function, EmitError> {
        if !real {
            return Ok(
                self.temporal_calendar_helper_stub(RuntimeHelperId::TemporalZonedDateTimeConvert)
            );
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalZonedDateTimeConvert);
        let parameters =
            self.helper_parameters::<TemporalZonedDateTimeConvertParameters>(&mut function);
        self.emit_temporal_zoned_date_time_from_input_kernel(
            &parameters.input,
            &parameters.options,
            &mut function,
        )?;
        self.completion().emit(&mut function);
        // On an abrupt return the whole Wasm frame releases these roots. The
        // normal epilogue keeps them alive through final completion publication.
        self.clear_helper_function_context(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_temporal_zoned_date_time_from_input_kernel(
        &mut self,
        input: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let handled = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        handled.store(f);
        self.emit_temporal_zoned_value_branch(input, f, |builder, f, record| {
            let policies = builder.emit_temporal_zoned_date_time_options(
                TemporalZonedDateTimeOptionsContext::From,
                options,
                f,
            )?;
            let instant = builder.emit_temporal_normalized_instant_from_zoned_record(record, f)?;
            let zone = builder.emit_temporal_zone_from_zoned_record(record, f)?;
            let calendar = builder.emit_temporal_calendar_from_zoned_record(record, f)?;
            builder.emit_alloc_temporal_zoned_date_time(
                TemporalZonedAllocationInput::new(&instant, &zone, &calendar),
                TemporalPrototypeSource::Intrinsic,
                f,
            )?;
            calendar.release(builder, f);
            zone.release(builder, f);
            instant.release(builder, f);
            policies.release(builder, f);
            f.instruction(&Instruction::I32Const(1));
            handled.store(f);
            Ok(())
        })?;
        handled.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_is_heap_object_like_tag_i32(input.tag(), f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_zoned_date_time_from_property_bag(input, options, f)?;
        f.instruction(&Instruction::Else);
        input.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_FROM_REQUIRES_A_STRING_OR_TEMPORAL_ZONEDDATETIME,f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let string = schema
            .reserve_gc_local(f)
            .initialize(input.cast_reference::<StringValue>(schema, f), f);
        let zone_value = schema.reserve_value_local(f);
        let calendar_value = schema.reserve_value_local(f);
        self.emit_temporal_parse_iso_string(
            &string,
            TemporalIsoParseGoal::ZonedDateTimeSyntax {
                time_zone: &zone_value,
                calendar: &calendar_value,
            },
            f,
        )?;
        let identifier = schema
            .reserve_gc_local(f)
            .initialize(zone_value.cast_reference::<StringValue>(schema, f), f);
        let zone = self.emit_temporal_resolve_zone_identifier(&identifier, f)?;
        let calendar = self.emit_temporal_constructor_calendar_slot(&calendar_value, f)?;
        let policies = self.emit_temporal_zoned_date_time_options(
            TemporalZonedDateTimeOptionsContext::From,
            options,
            f,
        )?;
        let output = self.reserve_temporal_instant_result(f);
        let instant = self
            .emit_temporal_parse_iso_string(
                &string,
                TemporalIsoParseGoal::ZonedDateTime {
                    policies: &policies,
                    zone: &zone,
                    output,
                },
                f,
            )?
            .ok_or_else(|| {
                EmitError::unsupported("Zoned parse goal did not publish its completed epoch")
            })?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&instant, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            f,
        )?;
        instant.release(self, f);
        policies.release(self, f);
        calendar.release(self, f);
        zone.release(self, f);
        identifier.clear(f);
        calendar_value.clear(f);
        zone_value.clear(f);
        string.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(handled, f);
        Ok(())
    }
    fn emit_temporal_zoned_date_time_from_property_bag(
        &mut self,
        input: &ValueLocals,
        options: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let fields = self.reserve_temporal_plain_date_time_field_locals(f);
        let present: [I64Local; 9] = std::array::from_fn(|_| schema.reserve_i64_local(f));
        let value = schema.reserve_value_local(f);
        let code = schema.reserve_value_local(f);
        let zone_value = schema.reserve_value_local(f);
        let code_present = schema.reserve_i64_local(f);
        let encoded = schema.reserve_i64_local(f);
        let offset_present = schema.reserve_i64_local(f);
        let offset = schema.reserve_i64_local(f);
        encoded.set_constant(0, f);
        offset.set_constant(0, f);
        code.set_undefined(f);
        self.emit_temporal_duration_option_get(input, "calendar", &value, f)?;
        let calendar = self.emit_temporal_calendar_slot_from_value(&value, f)?;
        let error = RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROPERTY_BAG_FIELD_MUST_BE_FINITE;
        self.emit_temporal_property_bag_integer(input, "day", present[2], fields[2], 0, error, f)?;
        let era_slots = self.reserve_temporal_era_slots(f);
        let era =
            self.emit_temporal_read_era_fields(era_slots, input, calendar.calendar_id(), f)?;
        for (name, index) in [
            ("hour", 3),
            ("microsecond", 7),
            ("millisecond", 6),
            ("minute", 4),
            ("month", 1),
        ] {
            self.emit_temporal_property_bag_integer(
                input,
                name,
                present[index],
                fields[index],
                0,
                error,
                f,
            )?;
        }
        self.emit_temporal_duration_option_get(input, "monthCode", &code, f)?;
        code.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I64ExtendI32U);
        code_present.store(f);
        self.emit_temporal_month_code_string(
            &code,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_MONTHCODE_MUST_BE_A_STRING,
            RuntimeErrorMessage::INVALID_TEMPORAL_ZONEDDATETIME_MONTHCODE,
            f,
        )?;
        self.emit_temporal_property_bag_integer(
            input,
            "nanosecond",
            present[8],
            fields[8],
            0,
            error,
            f,
        )?;
        self.emit_temporal_duration_option_get(input, "offset", &value, f)?;
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I64ExtendI32U);
        offset_present.store(f);
        self.emit_temporal_offset_string(
            &value,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_OFFSET_MUST_BE_A_STRING,
            f,
        )?;
        offset_present.load(f);
        f.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, f);
        let offset_text = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<StringValue>(schema, f), f);
        self.emit_temporal_utc_offset_nanoseconds(&offset_text, offset, f)?;
        offset_text.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_property_bag_integer(
            input, "second", present[5], fields[5], 0, error, f,
        )?;
        self.emit_temporal_duration_option_get(input, "timeZone", &zone_value, f)?;
        zone_value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROPERTY_BAG_REQUIRES_TIMEZONE,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let zone = self.emit_temporal_zoned_date_time_time_zone(
            &zone_value,
            TemporalTimeZoneStringGoal::Object,
            f,
        )?;
        self.emit_temporal_property_bag_integer(input, "year", present[0], fields[0], 0, error, f)?;
        let policies = self.emit_temporal_zoned_date_time_options(
            TemporalZonedDateTimeOptionsContext::From,
            options,
            f,
        )?;
        let year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            present[0],
            f,
        )?;
        for (missing, message) in [
            (
                year.year_present_local(),
                RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROPERTY_BAG_REQUIRES_YEAR,
            ),
            (
                present[2],
                RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROPERTY_BAG_REQUIRES_DAY,
            ),
        ] {
            missing.load(f);
            f.instruction(&Instruction::I64Eqz);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError, message, f)?;
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        present[1].load(f);
        f.instruction(&Instruction::I64Eqz);
        code_present.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_PROPERTY_BAG_REQUIRES_MONTH_OR_MONTHCODE,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let month = self.emit_temporal_resolve_calendar_month(
            year,
            fields[1],
            present[1],
            &code,
            encoded,
            code_present,
            TemporalMonthFieldContext::ZonedDateTime,
            f,
        )?;
        fields[2].load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64LtS);
        fields[1].load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64LtS);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_MONTH_AND_DAY_MUST_BE_POSITIVE,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_calendar_date_to_iso(month, fields[2], policies.overflow_local(), f)?;
        self.emit_temporal_regulate_property_bag_date_time(
            fields[0],
            fields[1],
            fields[2],
            fields[3],
            fields[4],
            fields[5],
            fields[6],
            fields[7],
            fields[8],
            policies.overflow_local(),
            f,
        )?;
        self.emit_temporal_reject_iso_date(fields[0], fields[1], fields[2], f)?;
        let iso = self.emit_temporal_regulated_iso_record(&fields, f)?;
        let supplied = self.emit_temporal_validated_offset_nanoseconds(offset, f)?;
        let output = self.reserve_temporal_instant_result(f);
        let instant = self.emit_temporal_parsed_zoned_epoch_into(
            output,
            &iso,
            &zone,
            &supplied,
            &policies,
            TemporalZonedParseFlags::Bag { offset_present },
            f,
        )?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&instant, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            f,
        )?;
        instant.release(self, f);
        supplied.release(self, f);
        iso.release(self, f);
        policies.release(self, f);
        zone.release(self, f);
        calendar.release(self, f);
        for local in [offset, offset_present, encoded, code_present] {
            schema.release_i64_local(local, f);
        }
        zone_value.clear(f);
        code.clear(f);
        value.clear(f);
        for local in present.into_iter().rev() {
            schema.release_i64_local(local, f);
        }
        self.release_temporal_plain_date_time_field_locals(fields, f);
        Ok(())
    }
}
