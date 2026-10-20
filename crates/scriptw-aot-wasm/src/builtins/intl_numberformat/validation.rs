use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_nf_currency_code(
        &mut self,
        input: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(input, schema, function)
                    .reference(),
                function,
            );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I32Const(3));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_CURRENCY_OPTION, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let output = StringConstruction::allocate(
            schema,
            schema.reserve_gc_local(function),
            length,
            function,
        );
        for position in 0..3 {
            set_i32(index, position, function);
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, index, schema, function)
                .store(unit, function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(0x20));
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I32Const(b'a' as i32));
            function.instruction(&Instruction::I32Sub);
            function.instruction(&Instruction::I32Const(25));
            function.instruction(&Instruction::I32GtU);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_intl_number_range_error(
                RuntimeErrorMessage::INVALID_CURRENCY_OPTION,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            unit.load(function);
            function.instruction(&Instruction::I32Const(!0x20));
            function.instruction(&Instruction::I32And);
            unit.store(function);
            output.write(index, unit, schema, function);
        }
        let text = schema
            .reserve_gc_local(function)
            .initialize(output.publish(schema, function), function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(length, function);
        units.clear(function);
        Ok(text)
    }
    fn emit_nf_single_unit_present(
        &mut self,
        text: &GcLocal<StringValue>,
        present: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        set_i32(present, 0, function);
        for unit in SingleUnit::ALL {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(unit.name(), function)?,
                function,
            );
            self.emit_string_payload_equality_i32(text, &expected, function);
            present.load(function);
            function.instruction(&Instruction::I32Or);
            present.store(function);
            expected.clear(function);
        }
        Ok(())
    }
    pub(super) fn emit_nf_unit_identifier(
        &mut self,
        text: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let recognized = schema.reserve_i32_local(function);
        self.emit_nf_single_unit_present(text, recognized, function)?;
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(text, schema, function)
                    .reference(),
                function,
            );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let position = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let equal = schema.reserve_i32_local(function);
        let split = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        set_i32(index, 0, function);
        set_i32(split, 0, function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        length.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        set_i32(equal, 1, function);
        for (offset, expected) in b"-per-".iter().enumerate() {
            index.load(function);
            function.instruction(&Instruction::I32Const(offset as i32));
            function.instruction(&Instruction::I32Add);
            position.store(function);
            schema
                .array_type::<CodeUnitArray>()
                .read(&units, position, schema, function)
                .store(unit, function);
            unit.load(function);
            function.instruction(&Instruction::I32Const(*expected as i32));
            function.instruction(&Instruction::I32Eq);
            equal.load(function);
            function.instruction(&Instruction::I32And);
            equal.store(function);
        }
        equal.load(function);
        self.open_frame(ControlFrameKind::If, function);
        index.load(function);
        split.store(function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        split.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let start = schema.reserve_i64_local(function);
        let end = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        start.store(function);
        split.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        end.store(function);
        let numerator = schema.reserve_gc_local(function).initialize(
            self.emit_gc_string_slice(text, start, end, function),
            function,
        );
        self.emit_nf_single_unit_present(&numerator, recognized, function)?;
        numerator.clear(function);
        recognized.load(function);
        self.open_frame(ControlFrameKind::If, function);
        split.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        start.store(function);
        length.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        end.store(function);
        let denominator = schema.reserve_gc_local(function).initialize(
            self.emit_gc_string_slice(text, start, end, function),
            function,
        );
        self.emit_nf_single_unit_present(&denominator, recognized, function)?;
        denominator.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(end, function);
        schema.release_i64_local(start, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [split, equal, unit, position, index, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(RuntimeErrorMessage::INVALID_UNIT_OPTION, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(recognized, function);
        Ok(())
    }
    pub(super) fn emit_nf_numbering_option(
        &mut self,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        let valid = schema.reserve_i32_local(function);
        let output = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        self.emit_intl_number_string_value(options, "numberingSystem", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_intl_is_unicode_type_i32(&text, valid, function);
        valid.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(
            RuntimeErrorMessage::INVALID_NUMBERINGSYSTEM_OPTION,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        output.replace(text.load(schema, function), function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(valid, function);
        value.clear(function);
        Ok(output)
    }
}
