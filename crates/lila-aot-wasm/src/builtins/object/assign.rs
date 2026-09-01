use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn compile_object_assign_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_arg_payload_local = self.reserve_temp_local();
        let target_arg_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let source_arg_payload_local = self.reserve_temp_local();
        let source_arg_tag_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let own_key_internal_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let set_result_payload_local = self.reserve_temp_local();
        let set_result_tag_local = self.reserve_temp_local();

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
        let set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.set builtin"))?;

        self.emit_builtin_arg_to_locals(
            0,
            target_arg_payload_local,
            target_arg_tag_local,
            function,
        );
        self.compile_nullish_tagged_i32(target_arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.assign called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            target_arg_payload_local,
            target_arg_tag_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            self.argv_param_local(),
            source_index_local,
            source_arg_payload_local,
            source_arg_tag_local,
            function,
        );
        self.compile_nullish_tagged_i32(source_arg_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_current_function_realm_object_locals(
            source_arg_payload_local,
            source_arg_tag_local,
            source_payload_local,
            source_tag_local,
            function,
        )?;
        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(source_payload_local, source_tag_local)],
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
                (source_payload_local, source_tag_local),
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
        self.emit_object_own_data_field_read(
            descriptor_payload_local,
            descriptor_tag_local,
            enumerable_key_local,
            enumerable_present_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `own_key_payload_local` holds the key as a JS *value*; the internal
        // [[Get]] path keys on the marked property-key payload.
        self.emit_property_key_payload_from_value_local(
            own_key_payload_local,
            own_key_tag_local,
            own_key_internal_local,
            function,
        );
        self.emit_object_read_with_key_tag(
            source_payload_local,
            source_tag_local,
            source_payload_local,
            source_tag_local,
            own_key_internal_local,
            Some(own_key_tag_local),
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_direct_js_call(
            &set_meta,
            None,
            &[
                (target_payload_local, target_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (value_payload_local, value_tag_local),
            ],
            set_result_payload_local,
            set_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(set_result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot assign to read only property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
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
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(set_result_tag_local);
        self.release_temp_local(set_result_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(enumerable_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_internal_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(source_arg_tag_local);
        self.release_temp_local(source_arg_payload_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(target_arg_tag_local);
        self.release_temp_local(target_arg_payload_local);
        Ok(())
    }
}
