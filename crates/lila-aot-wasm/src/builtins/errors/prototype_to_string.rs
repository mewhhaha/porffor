use super::*;

#[must_use = "the prepared Error name must be consumed before reading message"]
struct PreparedErrorNameLocal(u32);

impl PreparedErrorNameLocal {
    fn into_local(self) -> u32 {
        self.0
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_error_prototype_to_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Error.prototype.toString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Error.prototype.toString receiver",
            )
        })?;

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        let prepared_name = self.emit_error_to_string_prepare_name(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_error_to_string_message_and_result(
            receiver_payload_local,
            receiver_tag_local,
            prepared_name,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Error.prototype.toString receiver is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_error_to_string_prepare_name(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<PreparedErrorNameLocal, EmitError> {
        // Reserve the local that crosses the phase boundary before all of this
        // phase's transient locals so the latter can be released in LIFO order.
        let name_string_local = self.reserve_temp_local();
        let name_key_local = self.reserve_temp_local();
        let name_payload_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(name_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            name_key_local,
            name_payload_local,
            name_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(ERROR_NAME)));
        function.instruction(&Instruction::LocalSet(name_string_local));
        function.instruction(&Instruction::Else);
        self.emit_error_to_string_value_to_string_local(
            name_payload_local,
            name_tag_local,
            name_string_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_payload_local);
        self.release_temp_local(name_key_local);
        Ok(PreparedErrorNameLocal(name_string_local))
    }

    fn emit_error_to_string_message_and_result(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        prepared_name: PreparedErrorNameLocal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name_string_local = prepared_name.into_local();
        let message_string_local = self.reserve_temp_local();
        let message_key_local = self.reserve_temp_local();
        let message_payload_local = self.reserve_temp_local();
        let message_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(message_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            message_key_local,
            message_payload_local,
            message_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(message_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(message_string_local));
        function.instruction(&Instruction::Else);
        self.emit_error_to_string_value_to_string_local(
            message_payload_local,
            message_tag_local,
            message_string_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(message_tag_local);
        self.release_temp_local(message_payload_local);
        self.release_temp_local(message_key_local);

        let separator_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(separator_local));
        self.emit_string_payload_equality_i32(name_string_local, separator_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(message_string_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_payload_equality_i32(message_string_local, separator_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_string_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(": ")));
        function.instruction(&Instruction::LocalSet(separator_local));
        self.emit_concat_string_payloads_local(name_string_local, separator_local, function)?;
        function.instruction(&Instruction::LocalSet(name_string_local));
        self.emit_concat_string_payloads_local(name_string_local, message_string_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(separator_local);
        self.release_temp_local(message_string_local);
        self.release_temp_local(name_string_local);
        Ok(())
    }

    fn emit_error_to_string_value_to_string_local(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_string_local(
            primitive,
            string_payload_local,
            function,
        )
    }
}
