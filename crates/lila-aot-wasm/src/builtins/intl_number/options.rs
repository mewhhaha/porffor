use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_intl_number_get_option(
        &mut self,
        options: &ValueLocals,
        property: &str,
        out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let text = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(property, function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &text, function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        self.emit_object_read_with_throw_routing(
            options,
            options,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        out.copy_from(pending.value(), function);
        self.completion().copy_from(&pending, function);
        pending.clear(function);
        key.clear(function);
        text.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_number_to_string(
        &mut self,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        self.emit_value_to_string_payload(value, &pending, function)?;
        self.emit_intl_number_adopt_completion(&pending, function);
        let text = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        pending.clear(function);
        Ok(text)
    }
    pub(in crate::builtins) fn emit_intl_number_string_value(
        &mut self,
        options: &ValueLocals,
        property: &str,
        out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_number_get_option(options, property, out, function)?;
        emit_tag_is(out, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let text = self.emit_intl_number_to_string(out, function)?;
        out.set_reference(&text, self.runtime_schema(), function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    /// Selection can publish only a native closed-domain variant, and observes
    /// the source String once before testing the native spellings.
    pub(in crate::builtins) fn emit_intl_number_choice_option<V: GcI32Constant + Copy>(
        &mut self,
        options: &ValueLocals,
        property: IntlErrorOption,
        choices: impl IntoIterator<Item = (&'static str, V)>,
        default: V,
        out: &GcI32DomainLocal<V>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        self.emit_intl_number_string_value(options, property.property(), &value, function)?;
        out.set_constant(default, function);
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let text = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let recognized = schema.reserve_i32_local(function);
        set_i32(recognized, 0, function);
        for (spelling, variant) in choices {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(spelling, function)?,
                function,
            );
            self.emit_string_payload_equality_i32(&text, &expected, function);
            self.open_frame(ControlFrameKind::If, function);
            out.set_constant(variant, function);
            set_i32(recognized, 1, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            expected.clear(function);
        }
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(property.error_message(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(recognized, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_number_coerce_digit(
        &mut self,
        value: &ValueLocals,
        property: IntlErrorOption,
        minimum: u32,
        maximum: u32,
        fallback: I32Local,
        out: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        emit_tag_is(value, WasmRuntimeValueTag::Undefined, function);
        self.open_frame(ControlFrameKind::If, function);
        fallback.load(function);
        out.store(function);
        function.instruction(&Instruction::Else);
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        self.emit_value_to_number_payload(value, &pending, function)?;
        self.emit_intl_number_adopt_completion(&pending, function);
        for instruction in [Instruction::F64ReinterpretI64] {
            pending.value().scalar().load(function);
            function.instruction(&instruction);
        }
        pending.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        pending.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(minimum as f64)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Or);
        pending.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(maximum as f64)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(property.error_message(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I32TruncF64U);
        out.store(function);
        pending.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_number_number_option(
        &mut self,
        options: &ValueLocals,
        property: IntlErrorOption,
        minimum: u32,
        maximum: u32,
        fallback: I32Local,
        out: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = self.runtime_schema().reserve_value_local(function);
        self.emit_intl_number_get_option(options, property.property(), &value, function)?;
        self.emit_intl_number_coerce_digit(
            &value, property, minimum, maximum, fallback, out, function,
        )?;
        value.clear(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_number_options_object(
        &mut self,
        options: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        emit_tag_is(options, WasmRuntimeValueTag::Undefined, function);
        self.open_frame(ControlFrameKind::If, function);
        let schema = self.runtime_schema();
        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(None, function)?,
            function,
        );
        options.set_reference(&object, schema, function);
        object.clear(function);
        function.instruction(&Instruction::Else);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        self.emit_value_to_object_locals(options, &pending, function)?;
        self.emit_intl_number_adopt_completion(&pending, function);
        options.copy_from(pending.value(), function);
        pending.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
