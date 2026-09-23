use super::*;

#[derive(Clone, Copy)]
pub(crate) enum FunctionNamePrefix {
    None,
    Getter,
    Setter,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_set_function_name(
        &mut self,
        callable_local: u32,
        property_key_local: u32,
        prefix: FunctionNamePrefix,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();
        let fragment_local = self.reserve_temp_local();
        let symbol_local = self.reserve_temp_local();
        self.emit_property_key_payload_is_symbol_i32(property_key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_property_key_value_payload_to_local(property_key_local, symbol_local, function);
        self.emit_symbol_description_to_locals(symbol_local, name_local, name_tag_local, function);
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(name_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("[")));
        function.instruction(&Instruction::LocalSet(fragment_local));
        self.emit_concat_string_payloads_local(fragment_local, name_local, function)?;
        function.instruction(&Instruction::LocalSet(name_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(fragment_local));
        self.emit_concat_string_payloads_local(name_local, fragment_local, function)?;
        function.instruction(&Instruction::LocalSet(name_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(property_key_local));
        function.instruction(&Instruction::LocalSet(name_local));
        function.instruction(&Instruction::End);
        let prefix = match prefix {
            FunctionNamePrefix::None => None,
            FunctionNamePrefix::Getter => Some("get "),
            FunctionNamePrefix::Setter => Some("set "),
        };
        if let Some(prefix) = prefix {
            function.instruction(&Instruction::I64Const(self.strings.payload(prefix)));
            function.instruction(&Instruction::LocalSet(fragment_local));
            self.emit_concat_string_payloads_local(fragment_local, name_local, function)?;
            function.instruction(&Instruction::LocalSet(name_local));
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(fragment_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(name_tag_local));
        self.emit_object_define_data_with_configurable(
            callable_local,
            fragment_local,
            name_local,
            name_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(symbol_local);
        self.release_temp_local(fragment_local);
        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_local);
        Ok(())
    }
}
