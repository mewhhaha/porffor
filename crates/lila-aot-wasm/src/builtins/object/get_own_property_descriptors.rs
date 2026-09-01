use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn compile_object_get_own_property_descriptors_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let own_key_internal_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let function_realm_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();

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
        self.compile_nullish_tagged_i32(arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.getOwnPropertyDescriptors called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            arg_payload_local,
            arg_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(object_payload_local, object_tag_local)],
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
        function.instruction(&Instruction::LocalSet(function_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            function_realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_global(
            function_realm_local,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            object_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(result_payload_local));

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
                (object_payload_local, object_tag_local),
                (own_key_payload_local, own_key_tag_local),
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
        self.store_i64_local_at_offset(
            descriptor_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            object_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            descriptor_payload_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        // Keys arrive as JS values from `Reflect.ownKeys`; storing them needs
        // the internal property-key encoding back.
        self.emit_property_key_payload_from_value_local(
            own_key_payload_local,
            own_key_tag_local,
            own_key_internal_local,
            function,
        );
        self.emit_object_define_enumerable_data(
            result_payload_local,
            own_key_internal_local,
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(result_payload_local);
        self.release_temp_local(object_prototype_local);
        self.release_temp_local(function_realm_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_internal_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }
}
