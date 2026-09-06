use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn compile_object_get_own_property_descriptor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let proxy_key_payload_local = self.reserve_temp_local();
        let entry_buffer_local = self.reserve_temp_local();
        let entry_len_local = self.reserve_temp_local();
        let entry_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let writable_payload_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let configurable_payload_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let function_like_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_key_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let proxy_target_desc_found_local = self.reserve_temp_local();
        let proxy_result_configurable_present_local = self.reserve_temp_local();
        let proxy_result_configurable_payload_local = self.reserve_temp_local();
        let proxy_result_writable_present_local = self.reserve_temp_local();
        let proxy_result_writable_payload_local = self.reserve_temp_local();
        let proxy_result_field_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let typed_array_numeric_index_payload_local = self.reserve_temp_local();
        let typed_array_canonical_numeric_index_local = self.reserve_temp_local();
        let typed_array_valid_index_local = self.reserve_temp_local();
        let proxy_target_extensible_local = self.reserve_temp_local();
        let key_constant_local = self.reserve_temp_local();
        let proxy_target_descriptor_fact = self.reserve_own_descriptor_fact_locals();

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.getOwnPropertyDescriptor called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_property_key_payload_to_value_payload(key_string_local, function);
        function.instruction(&Instruction::LocalSet(proxy_key_payload_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_live_proxy_slots(
            target_payload_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(value_payload_local, value_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            ProxyRevocationRoute::CurrentFunctionRealm,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings.payload("getOwnPropertyDescriptor"),
        ));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_read_without_throw_propagation(
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            entry_key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(proxy_trap_tag_local, proxy_trap_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_key_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            proxy_handler_payload_local,
            proxy_handler_tag_local,
            &[
                (value_payload_local, value_tag_local),
                (proxy_key_payload_local, proxy_key_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result must be object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_own_descriptor_fact(
            value_payload_local,
            value_tag_local,
            key_string_local,
            key_tag_local,
            proxy_target_descriptor_fact,
            function,
        )?;
        proxy_target_descriptor_fact.emit_present_i32(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        proxy_target_descriptor_fact.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_object_is_extensible_i32(
            value_payload_local,
            value_tag_local,
            proxy_target_extensible_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_own_descriptor_fact(
            value_payload_local,
            value_tag_local,
            key_string_local,
            key_tag_local,
            proxy_target_descriptor_fact,
            function,
        )?;
        proxy_target_descriptor_fact.emit_present_i32(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(proxy_target_desc_found_local));
        proxy_target_descriptor_fact.emit_configurable_i32(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        proxy_target_descriptor_fact.emit_writable_i32(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_own_data_field_read(
            self.result_local,
            self.result_tag_local,
            entry_key_local,
            proxy_result_configurable_present_local,
            proxy_result_configurable_payload_local,
            proxy_result_field_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_own_data_field_read(
            self.result_local,
            self.result_tag_local,
            entry_key_local,
            proxy_result_writable_present_local,
            proxy_result_writable_payload_local,
            proxy_result_field_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(proxy_target_desc_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_is_extensible_i32(
            value_payload_local,
            value_tag_local,
            proxy_target_extensible_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_target_extensible_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result incompatible with non-extensible target",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proxy_target_desc_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_present_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_payload_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result cannot report configurable for non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_present_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(
            proxy_result_configurable_payload_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_desc_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(configurable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result cannot report non-configurable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_result_writable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(proxy_result_writable_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(writable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result cannot report non-writable target property",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_handled_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(
            typed_array_canonical_numeric_index_local,
        ));
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_canonical_numeric_index_string(
            key_string_local,
            typed_array_numeric_index_payload_local,
            typed_array_canonical_numeric_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(
            typed_array_canonical_numeric_index_local,
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_valid_integer_index_i32(
            target_payload_local,
            typed_array_numeric_index_payload_local,
            entry_index_local,
            typed_array_valid_index_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(typed_array_valid_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_typed_array_or_object_index_read_from_locals(
            target_payload_local,
            target_tag_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            true,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_like_local));
        self.emit_known_array_index_from_property_key(
            key_string_local,
            entry_index_local,
            function_like_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(function_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            entry_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entry_len_local));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_descriptor_kind_for_index(
            target_payload_local,
            entry_index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            target_payload_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let mapping =
            self.emit_arguments_index_mapping_from_descriptor_word(descriptor_kind_local, function);
        self.emit_arguments_parameter_map_read(
            target_payload_local,
            &mapping,
            value_payload_local,
            value_tag_local,
            function,
        );
        self.release_arguments_index_mapping(mapping);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_array_accessor_setter_for_index(
            target_payload_local,
            entry_index_local,
            setter_payload_local,
            setter_tag_local,
            function,
        );
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_length(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        self.emit_array_length_writable_i64(target_payload_local, writable_payload_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_descriptor_read(
            target_payload_local,
            key_string_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(entry_buffer_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(entry_len_local));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(entry_key_local));
        self.emit_object_read(
            entry_buffer_local,
            entry_len_local,
            entry_buffer_local,
            entry_len_local,
            entry_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            false,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_arguments_length(
            target_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        for (flag, flag_local) in [
            (DescriptorMask::WRITABLE, writable_payload_local),
            (DescriptorMask::ENUMERABLE, enumerable_payload_local),
            (DescriptorMask::CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
        }
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_LENGTH_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        for (flag, flag_local) in [
            (DescriptorMask::ENUMERABLE, enumerable_payload_local),
            (DescriptorMask::CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
        }
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (flag, flag_local) in [
            (DescriptorMask::ENUMERABLE, enumerable_payload_local),
            (DescriptorMask::CONFIGURABLE, configurable_payload_local),
        ] {
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(flag.as_i64()));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(flag_local));
        }
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_VALUE_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_ARGUMENTS_CALLEE_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_boxed_string_length_number_payload(
            value_payload_local,
            getter_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_string_local, entry_index_local, function);
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_index_read(
            value_payload_local,
            entry_index_local,
            getter_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(getter_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            true,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(5));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_boxed_string_length_number_payload(
            target_payload_local,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        self.emit_string_index_0_to_4_or_minus_one(key_string_local, entry_index_local, function);
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_index_read(
            target_payload_local,
            entry_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            true,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_like_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(function_like_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(function_like_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(function_like_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            true,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_constant_local));
        self.emit_property_key_payload_equality_i32(key_string_local, key_constant_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            false,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("ArrayBuffer")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::GlobalGet(DATA_VIEW_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(key_string_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("DataView")));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_alloc_data_descriptor_from_locals(
            value_payload_local,
            value_tag_local,
            false,
            false,
            true,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_PTR_OFFSET,
            entry_buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_LEN_OFFSET,
            entry_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::LocalGet(entry_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entry_buffer_local));
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_property_key_payload_equality_i32(entry_key_local, key_string_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(writable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_DESCRIPTOR_CONFIGURABLE as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(configurable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(enumerable_payload_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.emit_alloc_data_descriptor_from_locals_with_flag_locals(
            value_payload_local,
            value_tag_local,
            writable_payload_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_SETTER_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(getter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(setter_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        function.instruction(&Instruction::End);
        self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(
            getter_payload_local,
            getter_tag_local,
            setter_payload_local,
            setter_tag_local,
            enumerable_payload_local,
            configurable_payload_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(entry_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_own_descriptor_fact_locals(proxy_target_descriptor_fact);
        self.release_temp_local(key_constant_local);
        self.release_temp_local(proxy_target_extensible_local);
        self.release_temp_local(typed_array_valid_index_local);
        self.release_temp_local(typed_array_canonical_numeric_index_local);
        self.release_temp_local(typed_array_numeric_index_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(proxy_result_field_tag_local);
        self.release_temp_local(proxy_result_writable_payload_local);
        self.release_temp_local(proxy_result_writable_present_local);
        self.release_temp_local(proxy_result_configurable_payload_local);
        self.release_temp_local(proxy_result_configurable_present_local);
        self.release_temp_local(proxy_target_desc_found_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_key_tag_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(function_like_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(configurable_payload_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(writable_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(entry_index_local);
        self.release_temp_local(entry_len_local);
        self.release_temp_local(entry_buffer_local);
        self.release_temp_local(proxy_key_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }
}
