use super::*;

enum IntegrityTest {
    Sealed,
    Frozen,
}

impl<'a> FunctionBuilder<'a> {
    fn compile_object_integrity_test_builtin(
        &mut self,
        mode: IntegrityTest,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let extensible_result_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let descriptor_field_key_local = self.reserve_temp_local();
        let descriptor_field_present_local = self.reserve_temp_local();
        let descriptor_field_payload_local = self.reserve_temp_local();
        let descriptor_field_tag_local = self.reserve_temp_local();
        let reject_descriptor_local = self.reserve_temp_local();
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_is_extensible_i32(
            arg_payload_local,
            arg_tag_local,
            extensible_result_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(extensible_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(arg_payload_local, arg_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(reject_descriptor_local));
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_field_present_local,
            descriptor_field_payload_local,
            descriptor_field_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_field_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(descriptor_field_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(reject_descriptor_local));
        function.instruction(&Instruction::End);

        match &mode {
            IntegrityTest::Sealed => {}
            IntegrityTest::Frozen => {
                function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
                function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
                self.emit_object_own_data_field_read(
                    descriptor_payload_local,
                    descriptor_tag_local,
                    descriptor_field_key_local,
                    descriptor_field_present_local,
                    descriptor_field_payload_local,
                    descriptor_field_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(descriptor_field_present_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(descriptor_field_payload_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(reject_descriptor_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(reject_descriptor_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(reject_descriptor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(reject_descriptor_local);
        self.release_temp_local(descriptor_field_tag_local);
        self.release_temp_local(descriptor_field_payload_local);
        self.release_temp_local(descriptor_field_present_local);
        self.release_temp_local(descriptor_field_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(extensible_result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_is_sealed_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_integrity_test_builtin(IntegrityTest::Sealed, function)
    }

    pub(in crate::builtins) fn compile_object_is_frozen_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_integrity_test_builtin(IntegrityTest::Frozen, function)
    }
}
