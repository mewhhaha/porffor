use super::*;

enum EnumerableOwnProperties {
    Entries,
    Values,
}

impl<'a> FunctionBuilder<'a> {
    fn compile_object_enumerable_own_properties_builtin(
        &mut self,
        mode: EnumerableOwnProperties,
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
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let descriptor_field_key_local = self.reserve_temp_local();
        let descriptor_field_present_local = self.reserve_temp_local();
        let descriptor_field_payload_local = self.reserve_temp_local();
        let descriptor_field_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let entry_payload_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();
        let function_realm_local = self.reserve_temp_local();
        let array_prototype_local = self.reserve_temp_local();
        let nullish_message = match &mode {
            EnumerableOwnProperties::Entries => "Object.entries called on null or undefined",
            EnumerableOwnProperties::Values => "Object.values called on null or undefined",
        };

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
            nullish_message,
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
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            array_prototype_local,
            function,
        );

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
        self.emit_alloc_array_payload_with_length(
            own_keys_length_local,
            result_payload_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            array_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            result_payload_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Array.tag() as u64,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
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
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

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

        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
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

        self.emit_object_read_with_key_tag(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            own_key_payload_local,
            Some(own_key_tag_local),
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        match &mode {
            EnumerableOwnProperties::Entries => {
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::LocalSet(entry_index_local));
                self.emit_alloc_array_payload_with_length(
                    entry_index_local,
                    entry_payload_local,
                    function,
                )?;
                self.store_i64_local_at_offset(
                    entry_payload_local,
                    HEAP_PROTOTYPE_OFFSET,
                    array_prototype_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_payload_local,
                    HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Array.tag() as u64,
                    function,
                );
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(entry_index_local));
                self.emit_array_write(
                    entry_payload_local,
                    entry_index_local,
                    own_key_payload_local,
                    own_key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(entry_index_local));
                self.emit_array_write(
                    entry_payload_local,
                    entry_index_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(entry_tag_local));
                self.emit_array_write(
                    result_payload_local,
                    write_index_local,
                    entry_payload_local,
                    entry_tag_local,
                    function,
                )?;
            }
            EnumerableOwnProperties::Values => {
                self.emit_array_write(
                    result_payload_local,
                    write_index_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            }
        }
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_LEN_OFFSET,
            write_index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(array_prototype_local);
        self.release_temp_local(function_realm_local);
        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(entry_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_payload_local);
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
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_entries_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_enumerable_own_properties_builtin(
            EnumerableOwnProperties::Entries,
            function,
        )
    }

    pub(in crate::builtins) fn compile_object_values_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_enumerable_own_properties_builtin(
            EnumerableOwnProperties::Values,
            function,
        )
    }
}
