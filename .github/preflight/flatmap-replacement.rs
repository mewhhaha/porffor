    pub(crate) fn compile_array_prototype_flat_map_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported("missing Array.prototype.flatMap receiver")
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing Array.prototype.flatMap receiver tag")
        })?;
        let mapper_payload_local = self.reserve_temp_local();
        let mapper_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let source_index_local = self.reserve_temp_local();
        let target_index_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let mapped_payload_local = self.reserve_temp_local();
        let mapped_tag_local = self.reserve_temp_local();
        let mapped_len_local = self.reserve_temp_local();
        let mapped_index_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let number_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        // ToObject and LengthOfArrayLike precede IsCallable and ArraySpeciesCreate.
        // In particular, a TypedArray's observable length property is not its
        // private element count. Get/HasProperty below own live buffer witnesses.
        self.emit_array_iteration_length_before_callback_validation(
            this_payload_local,
            this_tag_local,
            key_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(length_payload_local));
        function.instruction(&Instruction::LocalSet(source_len_local));
        self.emit_builtin_arg_to_locals(0, mapper_payload_local, mapper_tag_local, function);
        self.emit_is_callable_i32(mapper_tag_local, mapper_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap mapper is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_array_species_create(
            this_payload_local,
            this_tag_local,
            zero_local,
            target_payload_local,
            target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(number_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(target_index_local));

        // FlattenIntoArray with a mapper and depth one: only present source
        // properties invoke the callback; only actual Arrays flatten its result.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            source_index_local, index_payload_local, key_local, function,
        )?;
        self.emit_object_has_property_i32(
            this_payload_local, this_tag_local, key_local, present_local, function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            this_payload_local, this_tag_local, this_payload_local, this_tag_local,
            key_local, element_payload_local, element_tag_local, function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            element_payload_local, element_tag_local, function,
        )?;
        function.instruction(&Instruction::LocalGet(source_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (element_payload_local, element_tag_local),
                (index_payload_local, number_tag_local),
                (this_payload_local, this_tag_local),
            ],
            argc_local, argv_local, function,
        )?;
        self.emit_function_handle_call_with_argv(
            mapper_payload_local, mapper_tag_local,
            Some((this_arg_payload_local, Some(this_arg_tag_local))),
            argc_local, argv_local, mapped_payload_local, mapped_tag_local, function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            mapped_payload_local, mapped_tag_local, function,
        )?;
        self.emit_is_array_i64(mapped_payload_local, mapped_tag_local, is_array_local, function)?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_flat_map_append(
            target_payload_local, target_tag_local, target_index_local,
            mapped_payload_local, mapped_tag_local, key_local, index_payload_local, function,
        )?;
        function.instruction(&Instruction::Else);
        // Keep the original Proxy as the Get/HasProperty receiver: IsArray may
        // inspect its target, but must not bypass traps or revocation afterward.
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            mapped_payload_local, mapped_tag_local, mapped_payload_local, mapped_tag_local,
            key_local, length_payload_local, length_tag_local, function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            length_payload_local, length_tag_local, function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            length_tag_local, length_payload_local, mapped_len_local, function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::LocalGet(mapped_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_index_to_flat_map_key_local(
            mapped_index_local, index_payload_local, key_local, function,
        )?;
        self.emit_object_has_property_i32(
            mapped_payload_local, mapped_tag_local, key_local, present_local, function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            mapped_payload_local, mapped_tag_local, mapped_payload_local, mapped_tag_local,
            key_local, child_payload_local, child_tag_local, function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            child_payload_local, child_tag_local, function,
        )?;
        self.emit_flat_map_append(
            target_payload_local, target_tag_local, target_index_local,
            child_payload_local, child_tag_local, key_local, index_payload_local, function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(mapped_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mapped_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
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
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(number_tag_local);
        self.release_temp_local(index_payload_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(present_local);
        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(mapped_index_local);
        self.release_temp_local(mapped_len_local);
        self.release_temp_local(mapped_tag_local);
        self.release_temp_local(mapped_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(zero_local);
        self.release_temp_local(target_index_local);
        self.release_temp_local(source_index_local);
        self.release_temp_local(source_len_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(mapper_tag_local);
        self.release_temp_local(mapper_payload_local);
        Ok(())
    }

    fn emit_flat_map_append(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        target_index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        key_local: u32,
        index_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(MAX_SAFE_INTEGER as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Array.prototype.flatMap result exceeds the maximum safe length",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_index_to_flat_map_key_local(
            target_index_local, index_payload_local, key_local, function,
        )?;
        self.emit_array_target_create_data_property_or_throw(
            target_payload_local, target_tag_local, key_local,
            value_payload_local, value_tag_local, function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(target_index_local));
        Ok(())
    }
