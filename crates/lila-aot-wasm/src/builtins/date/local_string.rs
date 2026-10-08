use super::zone::DateLocalProjectionLocals;
use super::*;
use lila_intl::SystemTimeZoneKind;

enum DateTimeValueSource {
    ReceiverSlot,
    RealmHostClock,
}
enum DateLocalStringFormat {
    Date,
    Time,
    DateAndTime,
}

impl FunctionBuilder<'_> {
    fn emit_date_time_value_from_source(
        &mut self,
        source: DateTimeValueSource,
        destination: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match source {
            DateTimeValueSource::ReceiverSlot => {
                let value = self.runtime_schema().reserve_value_local(function);
                self.compile_this_to_locals(&value, function)?;
                self.emit_date_value_payload(&value, destination, function)?;
                value.clear(function);
                Ok(())
            }
            DateTimeValueSource::RealmHostClock => {
                self.emit_date_current_time_payload(destination, function)
            }
        }
    }

    pub(crate) fn emit_date_current_time_payload(
        &mut self,
        destination: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let import = self
            .functions
            .wall_clock_millis_import_function_index()
            .ok_or_else(|| {
                EmitError::unsupported("Date current time requires lila_host.wall_clock_millis")
            })?;
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::I64ReinterpretF64);
        destination.store(function);
        Ok(())
    }

    pub(crate) fn emit_date_function_call(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::RealmHostClock,
            DateLocalStringFormat::DateAndTime,
            function,
        )
    }

    fn emit_date_local_string(
        &mut self,
        source: DateTimeValueSource,
        format: DateLocalStringFormat,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (date, time) = match format {
            DateLocalStringFormat::Date => (true, false),
            DateLocalStringFormat::Time => (false, true),
            DateLocalStringFormat::DateAndTime => (true, true),
        };
        let schema = self.runtime_schema();
        let bits = schema.reserve_i64_local(function);
        let fields: [I64Local; 7] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let value = schema.reserve_value_local(function);
        let output = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        self.emit_date_time_value_from_source(source, bits, function)?;
        let prepared = self.reserve_date_clip_result(function);
        let clip = self.emit_date_time_clip_into(prepared, bits, function);
        clip.emit_valid_branch(
            self,
            function,
            |builder, function, finite| {
                let projection = builder.emit_date_local_projection(finite, function)?;
                builder.emit_date_components_from_time(
                    projection.local_payload(),
                    fields[0],
                    fields[1],
                    fields[2],
                    fields[3],
                    fields[4],
                    fields[5],
                    fields[6],
                    function,
                );
                if date {
                    builder.emit_date_append_calendar_date(
                        &output,
                        projection.local_payload(),
                        &fields,
                        false,
                        function,
                    )?;
                }
                if date && time {
                    builder.emit_date_append_literal(&output, " ", function)?;
                }
                if time {
                    builder.emit_date_append_clock_time(&output, &fields, function)?;
                    builder.emit_date_append_time_zone_string(&projection, &output, function)?;
                }
                projection.release(builder, function);
                Ok(())
            },
            |builder, function| {
                output.replace(
                    builder.emit_interned_string_reference("Invalid Date", function)?,
                    function,
                );
                Ok(())
            },
        )?;
        value.set_reference(&output, schema, function);
        self.completion().set_normal(&value, function);
        clip.release(self, function);
        output.clear(function);
        value.clear(function);
        for field in fields.into_iter().rev() {
            schema.release_i64_local(field, function);
        }
        schema.release_i64_local(bits, function);
        Ok(())
    }

    /// Root Date.parse accepts this complete grammar, including historical
    /// whole-second offsets; the required GMT component keeps its HHMM form.
    fn emit_date_append_time_zone_string(
        &mut self,
        projection: &DateLocalProjectionLocals,
        output: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let magnitude = schema.reserve_i64_local(function);
        let component = schema.reserve_i64_local(function);
        let sign = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("+", function)?,
            function,
        );
        projection.system_zone().kind_local().load(function);
        function.instruction(&Instruction::I64Const(SystemTimeZoneKind::Utc.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_date_append_literal(output, " GMT+0000 (Coordinated Universal Time)", function)?;
        function.instruction(&Instruction::Else);
        projection.offset_seconds().load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        sign.replace(
            self.emit_interned_string_reference("-", function)?,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        projection.offset_seconds().load(function);
        function.instruction(&Instruction::I64Sub);
        magnitude.store(function);
        function.instruction(&Instruction::Else);
        projection.offset_seconds().load(function);
        magnitude.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_date_append_literal(output, " GMT", function)?;
        output.replace(
            self.emit_concat_gc_strings(output, &sign, function),
            function,
        );
        for (divisor, modulus) in [(3600, 24), (60, 60)] {
            magnitude.load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Const(modulus));
            function.instruction(&Instruction::I64RemU);
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            component.store(function);
            self.emit_date_append_padded_decimal(output, component, 2, function)?;
        }
        self.emit_date_append_literal(output, " (", function)?;
        output.replace(
            self.emit_concat_gc_strings(output, projection.system_zone().identifier(), function),
            function,
        );
        self.emit_date_append_literal(output, ", ", function)?;
        output.replace(
            self.emit_concat_gc_strings(output, &sign, function),
            function,
        );
        for (divisor, modulus, separator) in [(3600, 24, ":"), (60, 60, ":"), (1, 60, ")")] {
            magnitude.load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Const(modulus));
            function.instruction(&Instruction::I64RemU);
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            component.store(function);
            self.emit_date_append_padded_decimal(output, component, 2, function)?;
            self.emit_date_append_literal(output, separator, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        sign.clear(function);
        schema.release_i64_local(component, function);
        schema.release_i64_local(magnitude, function);
        Ok(())
    }

    pub(crate) fn emit_date_to_date_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::ReceiverSlot,
            DateLocalStringFormat::Date,
            function,
        )
    }
    pub(crate) fn emit_date_to_time_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::ReceiverSlot,
            DateLocalStringFormat::Time,
            function,
        )
    }
    pub(crate) fn emit_date_to_string(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_date_local_string(
            DateTimeValueSource::ReceiverSlot,
            DateLocalStringFormat::DateAndTime,
            function,
        )
    }
}
