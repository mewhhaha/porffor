use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_nf_grouping_option(
        &mut self,
        options: &ValueLocals,
        notation: &GcI32DomainLocal<NotationOption>,
        out: &GcI32DomainLocal<Grouping>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let default = GcI32DomainLocal::new(schema, Grouping::Auto, function);
        emit_domain_is(notation, NotationOption::Compact, function);
        self.open_frame(ControlFrameKind::If, function);
        default.set_constant(Grouping::MinTwo, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        out.copy_from(&default, function);
        let value = schema.reserve_value_local(function);
        self.emit_intl_number_get_option(options, "useGrouping", &value, function)?;
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_tagged_i32(&value, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        out.set_constant(Grouping::Never, function);
        function.instruction(&Instruction::Else);
        emit_tag_is(&value, WasmRuntimeValueTag::Boolean, function);
        self.open_frame(ControlFrameKind::If, function);
        out.set_constant(Grouping::Always, function);
        function.instruction(&Instruction::Else);
        let text = self.emit_intl_number_to_string(&value, function)?;
        let boolean_text = schema.reserve_i32_local(function);
        let recognized = schema.reserve_i32_local(function);
        set_i32(boolean_text, 0, function);
        set_i32(recognized, 0, function);
        for spelling in ["true", "false"] {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(spelling, function)?,
                function,
            );
            self.emit_string_payload_equality_i32(&text, &expected, function);
            boolean_text.load(function);
            function.instruction(&Instruction::I32Or);
            boolean_text.store(function);
            expected.clear(function);
        }
        boolean_text.load(function);
        self.open_frame(ControlFrameKind::If, function);
        out.copy_from(&default, function);
        function.instruction(&Instruction::Else);
        for (spelling, variant) in [
            ("auto", Grouping::Auto),
            ("always", Grouping::Always),
            ("min2", Grouping::MinTwo),
        ] {
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
        self.emit_intl_number_range_error(
            RuntimeErrorMessage::INVALID_USEGROUPING_OPTION,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(recognized, function);
        schema.release_i32_local(boolean_text, function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        default.clear(schema, function);
        Ok(())
    }
}
