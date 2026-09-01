use super::*;

enum PrototypeLookup {
    Getter,
    Setter,
}

impl<'a> FunctionBuilder<'a> {
    fn compile_object_prototype_lookup_builtin(
        &mut self,
        mode: PrototypeLookup,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype accessor lookup receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype accessor lookup receiver",
            )
        })?;
        let object_get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                )
            })?;
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let accessor_key_local = self.reserve_temp_local();
        let accessor_present_local = self.reserve_temp_local();
        let accessor_payload_local = self.reserve_temp_local();
        let accessor_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
        self.emit_value_to_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_direct_js_call(
            &object_get_own_property_descriptor_meta,
            None,
            &[
                (object_payload_local, object_tag_local),
                (key_payload_local, key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        let accessor_name = match &mode {
            PrototypeLookup::Getter => "get",
            PrototypeLookup::Setter => "set",
        };
        function.instruction(&Instruction::I64Const(self.strings.payload(accessor_name)));
        function.instruction(&Instruction::LocalSet(accessor_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            accessor_key_local,
            accessor_present_local,
            accessor_payload_local,
            accessor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(accessor_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(accessor_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.emit_object_get_prototype_of(
            object_payload_local,
            object_tag_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::LocalSet(object_payload_local));
        function.instruction(&Instruction::LocalGet(prototype_tag_local));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(accessor_tag_local);
        self.release_temp_local(accessor_payload_local);
        self.release_temp_local(accessor_present_local);
        self.release_temp_local(accessor_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_prototype_lookup_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_prototype_lookup_builtin(PrototypeLookup::Getter, function)
    }

    pub(in crate::builtins) fn compile_object_prototype_lookup_setter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_prototype_lookup_builtin(PrototypeLookup::Setter, function)
    }
}
