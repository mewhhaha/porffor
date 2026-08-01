use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_proxy_define_property_trap_invariants(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        value_present_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        writable_present_local: u32,
        writable_payload_local: u32,
        enumerable_present_local: u32,
        enumerable_payload_local: u32,
        configurable_present_local: u32,
        configurable_payload_local: u32,
        getter_present_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        setter_present_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                )
            })?;
        let get_own_property_descriptor_payload_local = self.reserve_temp_local();
        let get_own_property_descriptor_tag_local = self.reserve_temp_local();
        let target_descriptor_payload_local = self.reserve_temp_local();
        let target_descriptor_tag_local = self.reserve_temp_local();
        let target_descriptor_found_local = self.reserve_temp_local();
        let target_extensible_local = self.reserve_temp_local();
        let target_field_key_local = self.reserve_temp_local();
        let target_value_present_local = self.reserve_temp_local();
        let target_value_payload_local = self.reserve_temp_local();
        let target_value_tag_local = self.reserve_temp_local();
        let target_writable_present_local = self.reserve_temp_local();
        let target_writable_payload_local = self.reserve_temp_local();
        let target_enumerable_present_local = self.reserve_temp_local();
        let target_enumerable_payload_local = self.reserve_temp_local();
        let target_configurable_present_local = self.reserve_temp_local();
        let target_configurable_payload_local = self.reserve_temp_local();
        let target_getter_present_local = self.reserve_temp_local();
        let target_getter_payload_local = self.reserve_temp_local();
        let target_getter_tag_local = self.reserve_temp_local();
        let target_setter_present_local = self.reserve_temp_local();
        let target_setter_payload_local = self.reserve_temp_local();
        let target_setter_tag_local = self.reserve_temp_local();

        self.emit_function_value_payload(&get_own_property_descriptor_meta, function)?;
        function.instruction(&Instruction::LocalSet(
            get_own_property_descriptor_payload_local,
        ));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(
            get_own_property_descriptor_tag_local,
        ));
        self.emit_function_handle_call(
            get_own_property_descriptor_payload_local,
            get_own_property_descriptor_tag_local,
            None,
            &[
                (target_payload_local, target_tag_local),
                (key_payload_local, key_tag_local),
            ],
            target_descriptor_payload_local,
            target_descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(target_descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(target_descriptor_found_local));

        function.instruction(&Instruction::LocalGet(target_descriptor_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        for (field, present_local, payload_local, tag_local) in [
            (
                "value",
                target_value_present_local,
                target_value_payload_local,
                target_value_tag_local,
            ),
            (
                "writable",
                target_writable_present_local,
                target_writable_payload_local,
                get_own_property_descriptor_tag_local,
            ),
            (
                "enumerable",
                target_enumerable_present_local,
                target_enumerable_payload_local,
                get_own_property_descriptor_tag_local,
            ),
            (
                "configurable",
                target_configurable_present_local,
                target_configurable_payload_local,
                get_own_property_descriptor_tag_local,
            ),
            (
                "get",
                target_getter_present_local,
                target_getter_payload_local,
                target_getter_tag_local,
            ),
            (
                "set",
                target_setter_present_local,
                target_setter_payload_local,
                target_setter_tag_local,
            ),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(field)));
            function.instruction(&Instruction::LocalSet(target_field_key_local));
            self.emit_object_own_data_field_read(
                target_descriptor_payload_local,
                target_descriptor_tag_local,
                target_field_key_local,
                present_local,
                payload_local,
                tag_local,
                function,
            );
        }
        function.instruction(&Instruction::End);

        self.emit_object_is_extensible_i32(
            target_payload_local,
            target_tag_local,
            target_extensible_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(target_descriptor_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap cannot add property to non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap cannot define non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(target_configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap cannot define non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::LocalGet(target_enumerable_payload_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(target_getter_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(target_getter_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_getter_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap cannot report a writable target property as non-writable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_tagged_payload_same_value_i32(
            value_tag_local,
            value_payload_local,
            target_value_tag_local,
            target_value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap result is incompatible with target descriptor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        for (
            request_present_local,
            request_payload_local,
            request_tag_local,
            target_payload_local,
            target_tag_local,
        ) in [
            (
                getter_present_local,
                getter_payload_local,
                getter_tag_local,
                target_getter_payload_local,
                target_getter_tag_local,
            ),
            (
                setter_present_local,
                setter_payload_local,
                setter_tag_local,
                target_setter_payload_local,
                target_setter_tag_local,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(request_present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_tagged_payload_same_value_i32(
                request_tag_local,
                request_payload_local,
                target_tag_local,
                target_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Proxy defineProperty trap result is incompatible with target descriptor",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(target_setter_tag_local);
        self.release_temp_local(target_setter_payload_local);
        self.release_temp_local(target_setter_present_local);
        self.release_temp_local(target_getter_tag_local);
        self.release_temp_local(target_getter_payload_local);
        self.release_temp_local(target_getter_present_local);
        self.release_temp_local(target_configurable_payload_local);
        self.release_temp_local(target_configurable_present_local);
        self.release_temp_local(target_enumerable_payload_local);
        self.release_temp_local(target_enumerable_present_local);
        self.release_temp_local(target_writable_payload_local);
        self.release_temp_local(target_writable_present_local);
        self.release_temp_local(target_value_tag_local);
        self.release_temp_local(target_value_payload_local);
        self.release_temp_local(target_value_present_local);
        self.release_temp_local(target_field_key_local);
        self.release_temp_local(target_extensible_local);
        self.release_temp_local(target_descriptor_found_local);
        self.release_temp_local(target_descriptor_tag_local);
        self.release_temp_local(target_descriptor_payload_local);
        self.release_temp_local(get_own_property_descriptor_tag_local);
        self.release_temp_local(get_own_property_descriptor_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_construct_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let args_payload_local = self.reserve_temp_local();
        let args_tag_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let target_constructable_local = self.reserve_temp_local();
        let new_target_constructable_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, args_payload_local, args_tag_local, function);
        self.emit_builtin_arg_to_locals(
            2,
            new_target_payload_local,
            new_target_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(new_target_tag_local));
        function.instruction(&Instruction::End);

        self.emit_is_constructor_i32(target_tag_local, target_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(target_constructable_local));
        function.instruction(&Instruction::LocalGet(target_constructable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Reflect.construct target is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_is_constructor_i32(new_target_tag_local, new_target_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(new_target_constructable_local));
        function.instruction(&Instruction::LocalGet(new_target_constructable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.construct newTarget is not a constructor",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(args_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.construct argumentsList must be array-like",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_array_like_snapshot_payload(
            args_payload_local,
            args_tag_local,
            argv_local,
            "Reflect.construct argumentsList must be array-like",
            function,
        )?;
        self.load_i64_to_local_from_offset(argv_local, HEAP_LEN_OFFSET, argc_local, function);
        self.emit_function_or_proxy_construct_with_argv(
            target_payload_local,
            target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            argc_local,
            argv_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(new_target_constructable_local);
        self.release_temp_local(target_constructable_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(args_tag_local);
        self.release_temp_local(args_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_apply_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let this_arg_payload_local = self.reserve_temp_local();
        let this_arg_tag_local = self.reserve_temp_local();
        let args_payload_local = self.reserve_temp_local();
        let args_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
        self.emit_builtin_arg_to_locals(2, args_payload_local, args_tag_local, function);

        self.emit_is_callable_i32(target_tag_local, target_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.apply target must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(args_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.apply argumentsList must be array-like",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_array_like_snapshot_payload(
            args_payload_local,
            args_tag_local,
            argv_local,
            "Reflect.apply argumentsList must be an array",
            function,
        )?;
        self.load_i64_to_local_from_offset(argv_local, HEAP_LEN_OFFSET, argc_local, function);
        self.emit_function_or_proxy_call_with_argv_without_throw_propagation(
            target_payload_local,
            target_tag_local,
            this_arg_payload_local,
            this_arg_tag_local,
            argc_local,
            argv_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(args_tag_local);
        self.release_temp_local(args_payload_local);
        self.release_temp_local(this_arg_tag_local);
        self.release_temp_local(this_arg_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_get_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);
        self.emit_builtin_arg_to_locals(2, receiver_payload_local, receiver_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.get target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::End);
        self.emit_value_to_property_key_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_local));

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(key_string_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(
            target_payload_local,
            self.result_local,
            self.result_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_string_local, index_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get_with_prototype(
            target_payload_local,
            index_local,
            receiver_payload_local,
            receiver_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_string_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_string_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_get_own_property_descriptor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let object_get_payload_local = self.reserve_temp_local();
        let object_get_tag_local = self.reserve_temp_local();

        let object_get_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                )
            })?;

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.getOwnPropertyDescriptor target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_function_value_payload(&object_get_meta, function)?;
        function.instruction(&Instruction::LocalSet(object_get_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_get_tag_local));
        self.emit_function_handle_call(
            object_get_payload_local,
            object_get_tag_local,
            None,
            &[
                (target_payload_local, target_tag_local),
                (key_payload_local, key_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.release_temp_local(object_get_tag_local);
        self.release_temp_local(object_get_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_get_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.getPrototypeOf target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                target_payload_local,
                target_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_object_get_prototype_of_without_proxy(
                target_payload_local,
                target_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_set_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let key_value_payload_local = self.reserve_temp_local();
        let key_property_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let trap_key_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();
        let nested_kind_local = self.reserve_temp_local();
        let reflect_set_payload_local = self.reserve_temp_local();
        let reflect_set_tag_local = self.reserve_temp_local();

        let reflect_set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);
        self.emit_builtin_arg_to_locals(2, value_payload_local, value_tag_local, function);
        self.emit_builtin_arg_to_locals(3, receiver_payload_local, receiver_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.set target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(receiver_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(receiver_tag_local));
        function.instruction(&Instruction::End);

        self.emit_value_to_property_key_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_property_key_tag_from_source_tag(key_tag_local, key_property_tag_local, function);
        // `key_string_local` is the internal property-key payload; anything
        // handed back to JS (the `set` trap, or a nested `Reflect.set` call)
        // must see the unmarked symbol value instead.
        self.emit_property_key_value_payload_to_local(
            key_string_local,
            key_value_payload_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("set")));
        function.instruction(&Instruction::LocalSet(trap_key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            trap_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;

        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            trap_payload_local,
            trap_tag_local,
            Some((handler_payload_local, Some(handler_tag_local))),
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (key_value_payload_local, key_property_tag_local),
                (value_payload_local, value_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_set_invariant_check(
            proxy_target_payload_local,
            proxy_target_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy set trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            nested_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nested_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_value_payload(&reflect_set_meta, function)?;
        function.instruction(&Instruction::LocalSet(reflect_set_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(reflect_set_tag_local));
        self.emit_function_handle_call(
            reflect_set_payload_local,
            reflect_set_tag_local,
            None,
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (key_value_payload_local, key_property_tag_local),
                (value_payload_local, value_tag_local),
                (receiver_payload_local, receiver_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(trap_result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(trap_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_result_via_helper(
            proxy_target_payload_local,
            proxy_target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_string_local,
            key_property_tag_local,
            value_payload_local,
            value_tag_local,
            self.result_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_set_result_via_helper(
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_string_local,
            key_property_tag_local,
            value_payload_local,
            value_tag_local,
            self.result_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(reflect_set_tag_local);
        self.release_temp_local(reflect_set_payload_local);
        self.release_temp_local(nested_kind_local);
        self.release_temp_local(handled_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(trap_key_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_property_tag_local);
        self.release_temp_local(key_value_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_has_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.has target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_property_key_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_property_key_tag_from_source_tag(key_tag_local, key_tag_local, function);
        self.emit_object_has_property_with_key_tag_i32(
            target_payload_local,
            target_tag_local,
            key_string_local,
            key_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_define_property_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let key_value_payload_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let value_key_local = self.reserve_temp_local();
        let writable_key_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let configurable_key_local = self.reserve_temp_local();
        let get_key_local = self.reserve_temp_local();
        let set_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        // Keep the tags for ToBoolean.  Descriptor fields are tagged values;
        // their payload alone is not a JavaScript truth value (notably -0 and
        // NaN, but also empty strings and objects).
        let writable_tag_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let configurable_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let value_present_local = self.reserve_temp_local();
        let writable_present_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let configurable_present_local = self.reserve_temp_local();
        let getter_present_local = self.reserve_temp_local();
        let setter_present_local = self.reserve_temp_local();
        let descriptor_field_tag_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let proxy_key_tag_local = self.reserve_temp_local();
        let object_define_payload_local = self.reserve_temp_local();
        let object_define_tag_local = self.reserve_temp_local();
        let reflect_define_payload_local = self.reserve_temp_local();
        let reflect_define_tag_local = self.reserve_temp_local();
        let cap_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let scratch_payload_local = self.reserve_temp_local();
        let scratch_tag_local = self.reserve_temp_local();
        let target_entry_buffer_local = self.reserve_temp_local();
        let target_entry_len_local = self.reserve_temp_local();
        let target_entry_index_local = self.reserve_temp_local();
        let target_entry_local = self.reserve_temp_local();
        let target_desc_configurable_local = self.reserve_temp_local();
        let target_desc_writable_local = self.reserve_temp_local();
        let target_desc_accessor_local = self.reserve_temp_local();
        let target_value_payload_local = self.reserve_temp_local();
        let target_value_tag_local = self.reserve_temp_local();
        let array_length_success_local = self.reserve_temp_local();
        let array_named_success_local = self.reserve_temp_local();

        let object_define_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                )
            })?;
        let reflect_define_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                )
            })?;

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);
        self.emit_builtin_arg_to_locals(
            2,
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.defineProperty target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_property_key_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_property_key_tag_from_payload(key_string_local, proxy_key_tag_local, function);
        // The `defineProperty` trap (and the nested Reflect/Object
        // re-dispatches below) observe the key, so they need the unmarked
        // symbol value rather than the internal property-key payload.
        self.emit_property_key_value_payload_to_local(
            key_string_local,
            key_value_payload_local,
            function,
        );

        self.emit_is_heap_object_like_tag_i32(descriptor_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.defineProperty attributes must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // ToPropertyDescriptor observes these fields in specification order.
        // Each present field is read exactly once after its HasProperty check.
        for (key, key_local, present_local, payload_local, tag_local) in [
            (
                "enumerable",
                enumerable_key_local,
                enumerable_present_local,
                enumerable_payload_local,
                enumerable_tag_local,
            ),
            (
                "configurable",
                configurable_key_local,
                configurable_present_local,
                configurable_payload_local,
                configurable_tag_local,
            ),
            (
                "value",
                value_key_local,
                value_present_local,
                value_payload_local,
                value_tag_local,
            ),
            (
                "writable",
                writable_key_local,
                writable_present_local,
                writable_payload_local,
                writable_tag_local,
            ),
            (
                "get",
                get_key_local,
                getter_present_local,
                getter_payload_local,
                getter_tag_local,
            ),
            (
                "set",
                set_key_local,
                setter_present_local,
                setter_payload_local,
                setter_tag_local,
            ),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_property_key_tag_from_payload(
                key_local,
                descriptor_field_tag_local,
                function,
            );
            self.emit_object_has_property_with_key_tag_i32(
                descriptor_payload_local,
                descriptor_tag_local,
                key_local,
                descriptor_field_tag_local,
                present_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_object_read_without_throw_propagation(
                descriptor_payload_local,
                descriptor_tag_local,
                descriptor_payload_local,
                descriptor_tag_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);

            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;

            if matches!(key, "enumerable" | "configurable" | "writable") {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_to_boolean_payload_from_tagged_locals(
                    tag_local,
                    payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::End);
            }

            if matches!(key, "get" | "set") {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                self.emit_is_callable_i32(tag_local, payload_local, function)?;
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Property descriptor getter/setter must be callable or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
        }

        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::LocalGet(writable_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Property descriptor cannot be both accessor and data",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // The Proxy trap and the generic Object.defineProperty fallback must
        // receive the completed descriptor, not the observable attributes
        // object. This also guarantees descriptor getters run exactly once.
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));
        for (key_local, present_local, payload_local, tag_local, boolean_value) in [
            (
                value_key_local,
                value_present_local,
                value_payload_local,
                value_tag_local,
                false,
            ),
            (
                writable_key_local,
                writable_present_local,
                writable_payload_local,
                writable_tag_local,
                true,
            ),
            (
                get_key_local,
                getter_present_local,
                getter_payload_local,
                getter_tag_local,
                false,
            ),
            (
                set_key_local,
                setter_present_local,
                setter_payload_local,
                setter_tag_local,
                false,
            ),
            (
                enumerable_key_local,
                enumerable_present_local,
                enumerable_payload_local,
                enumerable_tag_local,
                true,
            ),
            (
                configurable_key_local,
                configurable_present_local,
                configurable_payload_local,
                configurable_tag_local,
                true,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            if boolean_value {
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            self.emit_object_define_enumerable_data(
                descriptor_payload_local,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        }

        self.emit_function_value_payload(&object_define_meta, function)?;
        function.instruction(&Instruction::LocalSet(object_define_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_define_tag_local));
        self.emit_function_value_payload(&reflect_define_meta, function)?;
        function.instruction(&Instruction::LocalSet(reflect_define_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(reflect_define_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("defineProperty"),
        ));
        function.instruction(&Instruction::LocalSet(get_key_local));
        self.emit_object_read_ordinary(
            handler_payload_local,
            handler_tag_local,
            handler_payload_local,
            handler_tag_local,
            get_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;

        self.emit_is_callable_i32(trap_tag_local, trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            trap_payload_local,
            trap_tag_local,
            handler_payload_local,
            handler_tag_local,
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (key_value_payload_local, proxy_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(trap_result_tag_local, trap_result_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_define_property_trap_invariants(
            proxy_target_payload_local,
            proxy_target_tag_local,
            key_string_local,
            proxy_key_tag_local,
            value_present_local,
            value_payload_local,
            value_tag_local,
            writable_present_local,
            writable_payload_local,
            enumerable_present_local,
            enumerable_payload_local,
            configurable_present_local,
            configurable_payload_local,
            getter_present_local,
            getter_payload_local,
            getter_tag_local,
            setter_present_local,
            setter_payload_local,
            setter_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            reflect_define_payload_local,
            reflect_define_tag_local,
            None,
            &[
                (proxy_target_payload_local, proxy_target_tag_local),
                (key_value_payload_local, proxy_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            scratch_payload_local,
            scratch_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(scratch_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(scratch_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Proxy defineProperty trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_ordinary_is_extensible_i32(
            target_payload_local,
            target_tag_local,
            cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_own_data_field_read(
            target_payload_local,
            target_tag_local,
            key_string_local,
            present_local,
            scratch_payload_local,
            scratch_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Do not delegate Array length to Object.defineProperty: Reflect must
        // report the ordinary DefineProperty failure as false, not a throw.
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(get_key_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_string_payload_equality_i32(key_string_local, get_key_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(configurable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cap_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_set_length_from_value(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            writable_present_local,
            cap_local,
            array_length_success_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(cap_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_length_success_local));
        function.instruction(&Instruction::Else);
        self.emit_array_set_length_without_value(
            target_payload_local,
            writable_payload_local,
            writable_present_local,
            array_length_success_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(array_length_success_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Array named properties use the array-specific descriptor storage;
        // leave indices, length, constructor, and symbols to the fallback.
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_string_local,
            cap_local,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(get_key_local));
        self.emit_string_payload_equality_i32(key_string_local, get_key_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(array_named_success_local));
        function.instruction(&Instruction::LocalGet(getter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(setter_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_define_named_accessor_descriptor(
            target_payload_local,
            key_string_local,
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(getter_present_local),
            Some(setter_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_success_local),
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_array_define_named_data_descriptor(
            target_payload_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            Some(value_present_local),
            Some(writable_present_local),
            Some(enumerable_present_local),
            Some(configurable_present_local),
            Some(array_named_success_local),
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(array_named_success_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_without_throw_propagation(
            object_define_payload_local,
            object_define_tag_local,
            None,
            &[
                (target_payload_local, target_tag_local),
                (key_value_payload_local, proxy_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            scratch_payload_local,
            scratch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(array_named_success_local);
        self.release_temp_local(array_length_success_local);
        self.release_temp_local(target_value_tag_local);
        self.release_temp_local(target_value_payload_local);
        self.release_temp_local(target_desc_accessor_local);
        self.release_temp_local(target_desc_writable_local);
        self.release_temp_local(target_desc_configurable_local);
        self.release_temp_local(target_entry_local);
        self.release_temp_local(target_entry_index_local);
        self.release_temp_local(target_entry_len_local);
        self.release_temp_local(target_entry_buffer_local);
        self.release_temp_local(scratch_tag_local);
        self.release_temp_local(scratch_payload_local);
        self.release_temp_local(present_local);
        self.release_temp_local(cap_local);
        self.release_temp_local(reflect_define_tag_local);
        self.release_temp_local(reflect_define_payload_local);
        self.release_temp_local(object_define_tag_local);
        self.release_temp_local(object_define_payload_local);
        self.release_temp_local(proxy_key_tag_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(handled_local);
        self.release_temp_local(descriptor_field_tag_local);
        self.release_temp_local(setter_present_local);
        self.release_temp_local(getter_present_local);
        self.release_temp_local(configurable_present_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(writable_present_local);
        self.release_temp_local(value_present_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(configurable_tag_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(writable_tag_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(set_key_local);
        self.release_temp_local(get_key_local);
        self.release_temp_local(configurable_key_local);
        self.release_temp_local(enumerable_key_local);
        self.release_temp_local(writable_key_local);
        self.release_temp_local(value_key_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_value_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_delete_property_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.deleteProperty target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_property_key_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_object_delete(
            target_payload_local,
            target_tag_local,
            key_string_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_prevent_extensions_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.preventExtensions target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_object_prevent_extensions_i32(
            target_payload_local,
            target_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_is_extensible_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.isExtensible target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_object_is_extensible_i32(
            target_payload_local,
            target_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_set_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let set_result_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, proto_payload_local, proto_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf prototype must be object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_object_set_prototype_of_i32(
            target_payload_local,
            target_tag_local,
            proto_payload_local,
            proto_tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(set_result_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn compile_reflect_own_keys_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let trap_result_payload_local = self.reserve_temp_local();
        let trap_result_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let names_function_payload_local = self.reserve_temp_local();
        let names_function_tag_local = self.reserve_temp_local();
        let symbols_function_payload_local = self.reserve_temp_local();
        let symbols_function_tag_local = self.reserve_temp_local();
        let names_payload_local = self.reserve_temp_local();
        let names_tag_local = self.reserve_temp_local();
        let symbols_payload_local = self.reserve_temp_local();
        let symbols_tag_local = self.reserve_temp_local();
        let names_len_local = self.reserve_temp_local();
        let symbols_len_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Reflect.ownKeys target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_proxy_own_keys_trap_result(
            target_payload_local,
            target_tag_local,
            proxy_handled_local,
            proxy_target_payload_local,
            proxy_target_tag_local,
            handler_payload_local,
            handler_tag_local,
            trap_payload_local,
            trap_tag_local,
            trap_result_payload_local,
            trap_result_tag_local,
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_own_keys_array_result(
            proxy_target_payload_local,
            proxy_target_tag_local,
            trap_result_payload_local,
            trap_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        let names_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyNames.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyNames`",
                )
            })?;
        self.emit_function_value_payload(&names_meta, function)?;
        function.instruction(&Instruction::LocalSet(names_function_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(names_function_tag_local));
        self.emit_function_handle_call(
            names_function_payload_local,
            names_function_tag_local,
            None,
            &[(target_payload_local, target_tag_local)],
            names_payload_local,
            names_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        let symbols_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertySymbols.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertySymbols`",
                )
            })?;
        self.emit_function_value_payload(&symbols_meta, function)?;
        function.instruction(&Instruction::LocalSet(symbols_function_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(symbols_function_tag_local));
        self.emit_function_handle_call(
            symbols_function_payload_local,
            symbols_function_tag_local,
            None,
            &[(target_payload_local, target_tag_local)],
            symbols_payload_local,
            symbols_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.load_i64_to_local_from_offset(
            names_payload_local,
            HEAP_LEN_OFFSET,
            names_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            symbols_payload_local,
            HEAP_LEN_OFFSET,
            symbols_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(names_len_local));
        function.instruction(&Instruction::LocalGet(symbols_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));
        self.emit_alloc_array_payload_with_length(total_len_local, result_payload_local, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(names_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            names_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(symbols_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            symbols_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(symbols_len_local);
        self.release_temp_local(names_len_local);
        self.release_temp_local(symbols_tag_local);
        self.release_temp_local(symbols_payload_local);
        self.release_temp_local(names_tag_local);
        self.release_temp_local(names_payload_local);
        self.release_temp_local(symbols_function_tag_local);
        self.release_temp_local(symbols_function_payload_local);
        self.release_temp_local(names_function_tag_local);
        self.release_temp_local(names_function_payload_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(trap_result_tag_local);
        self.release_temp_local(trap_result_payload_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }
}
