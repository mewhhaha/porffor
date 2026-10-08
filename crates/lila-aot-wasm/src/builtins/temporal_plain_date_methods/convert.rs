//! One complete PlainDate conversion, with the actual called FunctionContext.
use super::*;
use crate::runtime_helpers::{HelperParameters, RuntimeHelperOptionalValueParameter};

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_temporal_to_temporal_date(
        &mut self,
        argument: &ValueLocals,
        overflow_options: TemporalConversionOverflowOptions<'_>,
        fields: &[I64Local; 3],
        calendar_out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(function);
        let options = match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => Some(options),
            TemporalConversionOverflowOptions::Omit => None,
        };
        let context = self.current_function_context().ok_or_else(|| {
            EmitError::unsupported(
                "PlainDate conversion requires the actual called FunctionContext",
            )
        })?;
        schema
            .call_helper(
                crate::runtime_helpers::TemporalPlainDateConvertArguments::new(
                    argument,
                    options,
                    context,
                    self.current_environment(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(&pending, function);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<TemporalPlainDateObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_date_load_record(&record, fields, calendar_out, function);
        record.clear(function);
        function.instruction(&Instruction::Else);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        Ok(())
    }

    pub(crate) fn compile_temporal_plain_date_convert_helper(
        &mut self,
        real: bool,
    ) -> Result<Function, EmitError> {
        if !real {
            return Ok(
                self.temporal_calendar_helper_stub(RuntimeHelperId::TemporalPlainDateConvert)
            );
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalPlainDateConvert);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::TemporalPlainDateConvertParameters>(
                &mut function,
            );
        let schema = self.runtime_schema();
        let fields: [I64Local; 3] =
            std::array::from_fn(|_| schema.reserve_i64_local(&mut function));
        let calendar_value = schema.reserve_value_local(&mut function);
        self.emit_temporal_to_temporal_date_kernel(
            &parameters.input,
            &parameters.overflow_options,
            &fields,
            &calendar_value,
            &mut function,
        )?;
        // The original conversion has completed every observable read/check.
        // Transport its checked coordinates through the existing genuine GC record.
        let identifier = schema.reserve_gc_local(&mut function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, &mut function),
            &mut function,
        );
        let calendar =
            self.emit_temporal_calendar_slot_from_identifier(&identifier, &mut function)?;
        self.emit_alloc_temporal_plain_date(
            fields[0],
            fields[1],
            fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            &mut function,
        )?;
        calendar.release(self, &mut function);
        identifier.clear(&mut function);
        calendar_value.clear(&mut function);
        for field in fields {
            schema.release_i64_local(field, &mut function);
        }
        self.completion().emit(&mut function);
        self.clear_helper_function_context(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Internal-slot fast paths precede the property-bag path. All inputs and
    /// calendar outputs are whole values; the scalar fields are ISO coordinates.
    fn emit_temporal_to_temporal_date_kernel(
        &mut self,
        argument: &ValueLocals,
        overflow_options: &RuntimeHelperOptionalValueParameter,
        fields: &[I64Local; 3],
        calendar_out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let handled = schema.reserve_i64_local(function);
        let overflow = schema.reserve_i64_local(function);
        let present: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let month_code = schema.reserve_value_local(function);
        let encoded_code = schema.reserve_i64_local(function);
        let code_present = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        month_code.set_undefined(function);
        for local in fields.iter().chain(present.iter()).copied().chain([
            handled,
            encoded_code,
            code_present,
            any_present,
        ]) {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        let iso_calendar = self.emit_temporal_iso_calendar_slot(function)?;
        calendar_out.set_reference(iso_calendar.identifier(), schema, function);
        iso_calendar.release(self, function);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainDateObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_date_load_record(&record, fields, calendar_out, function);
        record.clear(function);
        overflow_options.emit_if_present(self, function, |options, builder, function| {
            builder.emit_temporal_plain_date_overflow_option(options, overflow, function)
        })?;
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainDateTimeObject>(schema, function),
            function,
        );
        let full = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_load_record(&record, &full, calendar_out, function);
        for (source, destination) in full.iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        self.release_temporal_plain_date_time_field_locals(full, function);
        record.clear(function);
        overflow_options.emit_if_present(self, function, |options, builder, function| {
            builder.emit_temporal_plain_date_overflow_option(options, overflow, function)
        })?;
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalZonedDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let branded = self.emit_temporal_branded_zoned_record_from_value(argument, function)?;
        let epoch = self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &epoch, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        for (source, destination) in iso.fields().iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        calendar_out.set_reference(calendar.identifier(), schema, function);
        calendar.release(self, function);
        iso.release(self, function);
        snapshot.release(self, function);
        zone.release(self, function);
        epoch.release(self, function);
        branded.release(function);
        overflow_options.emit_if_present(self, function, |options, builder, function| {
            builder.emit_temporal_plain_date_overflow_option(options, overflow, function)
        })?;
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let calendar_value = schema.reserve_value_local(function);
        self.emit_temporal_duration_option_get(argument, "calendar", &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        calendar_out.set_reference(calendar.identifier(), schema, function);
        calendar_value.clear(function);
        let era = self.emit_temporal_plain_date_read_fields(
            argument,
            &calendar,
            fields,
            &present,
            &month_code,
            code_present,
            any_present,
            function,
        )?;
        overflow_options.emit_if_present(self, function, |options, builder, function| {
            builder.emit_temporal_plain_date_overflow_option(options, overflow, function)
        })?;
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            present[0],
            function,
        )?;
        self.emit_temporal_plain_date_resolve_fields(
            resolved_year,
            fields[1],
            present[1],
            &month_code,
            encoded_code,
            code_present,
            fields[2],
            present[2],
            overflow,
            function,
        )?;
        calendar.release(self, function);
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_EXPECTS_A_STRING_A_PROPERTY_BAG_OR_A_TEMPORAL_PLAINDATE, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_parse_plain_date_string(
            &string,
            fields[0],
            fields[1],
            fields[2],
            calendar_out,
            function,
        )?;
        string.clear(function);
        overflow_options.emit_if_present(self, function, |options, builder, function| {
            builder.emit_temporal_plain_date_overflow_option(options, overflow, function)
        })?;
        self.emit_temporal_reject_iso_date(fields[0], fields[1], fields[2], function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        month_code.clear(function);
        for local in
            present
                .into_iter()
                .chain([any_present, code_present, encoded_code, overflow, handled])
        {
            schema.release_i64_local(local, function);
        }
        Ok(())
    }
}
