use super::*;

#[derive(Clone, Copy)]
enum PromiseKeyedCombinatorMode {
    Values,
    SettledRecords,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_promise_all_keyed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_keyed(PromiseKeyedCombinatorMode::Values, function)
    }

    pub(crate) fn emit_promise_all_settled_keyed(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_keyed(PromiseKeyedCombinatorMode::SettledRecords, function)
    }

    fn emit_promise_keyed(
        &mut self,
        mode: PromiseKeyedCombinatorMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let builtin_name = match mode {
            PromiseKeyedCombinatorMode::Values => "Promise.allKeyed",
            PromiseKeyedCombinatorMode::SettledRecords => "Promise.allSettledKeyed",
        };
        let constructor_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {builtin_name} receiver"
            ))
        })?;
        let constructor_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing {builtin_name} receiver tag"
            ))
        })?;
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let resolve_payload_local = self.reserve_temp_local();
        let resolve_tag_local = self.reserve_temp_local();
        let reject_payload_local = self.reserve_temp_local();
        let reject_tag_local = self.reserve_temp_local();
        let promise_resolve_payload_local = self.reserve_temp_local();
        let promise_resolve_tag_local = self.reserve_temp_local();
        let promises_payload_local = self.reserve_temp_local();
        let promises_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let keys_length_local = self.reserve_temp_local();
        let key_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_property_payload_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let property_value_payload_local = self.reserve_temp_local();
        let property_value_tag_local = self.reserve_temp_local();
        let next_promise_payload_local = self.reserve_temp_local();
        let next_promise_tag_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let shared_context_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let element_context_local = self.reserve_temp_local();
        let resolve_element_payload_local = self.reserve_temp_local();
        let resolve_element_tag_local = self.reserve_temp_local();
        let reject_element_payload_local = self.reserve_temp_local();
        let reject_element_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();

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
        let resolve_element_builtin = match mode {
            PromiseKeyedCombinatorMode::Values => StandardBuiltinId::PromiseAllKeyedResolveElement,
            PromiseKeyedCombinatorMode::SettledRecords => {
                StandardBuiltinId::PromiseAllSettledKeyedResolveElement
            }
        };
        let resolve_element_meta = self
            .functions
            .get(&resolve_element_builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing keyed Promise resolve element builtin")
            })?;
        let reject_element_meta = match mode {
            PromiseKeyedCombinatorMode::Values => None,
            PromiseKeyedCombinatorMode::SettledRecords => Some(
                self.functions
                    .get(&StandardBuiltinId::PromiseAllSettledKeyedRejectElement.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "missing Promise.allSettledKeyed reject element builtin",
                        )
                    })?,
            ),
        };

        self.emit_new_promise_capability(
            constructor_payload_local,
            constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            capability_record_local,
            HEAP_PROMISE_CAPABILITY_REJECT_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("resolve")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        self.emit_object_read_without_throw_propagation(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            enumerable_key_local,
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        let algorithm_error_realm =
            self.emit_promise_combinator_algorithm_error_realm_context(function);
        self.emit_is_callable_i32(
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_promise_combinator_type_error(
            &algorithm_error_realm,
            "Promise keyed constructor resolve property is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, promises_payload_local, promises_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(promises_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_promise_combinator_type_error(
            &algorithm_error_realm,
            "Promise keyed input must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(promises_payload_local, promises_tag_local)],
            keys_payload_local,
            keys_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            keys_payload_local,
            HEAP_LEN_OFFSET,
            keys_length_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.emit_heap_alloc_const(HEAP_PROMISE_ALL_SHARED_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(shared_context_local));
        self.store_i64_const_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            1,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_VALUES_OFFSET,
            result_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            resolve_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            resolve_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::LocalGet(keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_payload_local,
            key_index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        // `Reflect.ownKeys` yields String/Symbol *values*; every internal
        // property-key consumer below (the `[[Get]]` on the source object, the
        // `[[DefineOwnProperty]]` on the result, and the key handed to the
        // resolve-element closure) needs the internal encoding, which re-applies
        // `PROPERTY_KEY_SYMBOL_MARKER` for symbols. Without it a symbol key is
        // stored as a bogus string key: `Object.keys` then reports it and reads
        // a garbage payload. The value form stays for
        // `Reflect.getOwnPropertyDescriptor`, which applies ToPropertyKey itself.
        self.emit_property_key_payload_from_value_local(
            key_payload_local,
            key_tag_local,
            key_property_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (promises_payload_local, promises_tag_local),
                (key_payload_local, key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        self.emit_object_read_without_throw_propagation(
            descriptor_payload_local,
            descriptor_tag_local,
            descriptor_payload_local,
            descriptor_tag_local,
            enumerable_key_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(enumerable_tag_local, enumerable_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        self.emit_object_read_without_throw_propagation(
            promises_payload_local,
            promises_tag_local,
            promises_payload_local,
            promises_tag_local,
            key_property_payload_local,
            property_value_payload_local,
            property_value_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_object_define_enumerable_data(
            result_payload_local,
            key_property_payload_local,
            undefined_payload_local,
            undefined_tag_local,
            function,
        )?;
        self.emit_function_or_proxy_call_leave_throw_completion(
            promise_resolve_payload_local,
            promise_resolve_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            &[(property_value_payload_local, property_value_tag_local)],
            next_promise_payload_local,
            next_promise_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;

        self.emit_heap_alloc_const(HEAP_PROMISE_KEYED_ELEMENT_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(element_context_local));
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_PAYLOAD_OFFSET,
            key_property_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_KEY_TAG_OFFSET,
            key_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_SHARED_OFFSET,
            shared_context_local,
            function,
        );
        self.store_i64_const_at_offset(
            element_context_local,
            HEAP_PROMISE_KEYED_ELEMENT_ALREADY_CALLED_OFFSET,
            0,
            function,
        );
        let materialization_context =
            self.emit_current_function_promise_internal_function_materialization_context(function);
        self.emit_promise_internal_function_value(
            &resolve_element_meta,
            &materialization_context,
            element_context_local,
            resolve_element_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(resolve_element_tag_local));
        if let Some(reject_element_meta) = &reject_element_meta {
            self.emit_promise_internal_function_value(
                reject_element_meta,
                &materialization_context,
                element_context_local,
                reject_element_payload_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(reject_element_tag_local));
        } else {
            function.instruction(&Instruction::LocalGet(reject_payload_local));
            function.instruction(&Instruction::LocalSet(reject_element_payload_local));
            function.instruction(&Instruction::LocalGet(reject_tag_local));
            function.instruction(&Instruction::LocalSet(reject_element_tag_local));
        }
        self.release_promise_internal_function_materialization_context(materialization_context);
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        self.emit_object_read_without_throw_propagation(
            next_promise_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            enumerable_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            &[
                (resolve_element_payload_local, resolve_element_tag_local),
                (reject_element_payload_local, reject_element_tag_local),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(property_value_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            resolve_payload_local,
            resolve_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[(result_payload_local, property_value_tag_local)],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        self.emit_promise_keyed_reject_current_throw(
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            error_payload_local,
            error_tag_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_promise_combinator_algorithm_error_realm_context(algorithm_error_realm);
        for local in [
            error_tag_local,
            error_payload_local,
            call_tag_local,
            call_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            reject_element_tag_local,
            reject_element_payload_local,
            resolve_element_tag_local,
            resolve_element_payload_local,
            element_context_local,
            remaining_local,
            shared_context_local,
            result_payload_local,
            then_tag_local,
            then_payload_local,
            next_promise_tag_local,
            next_promise_payload_local,
            property_value_tag_local,
            property_value_payload_local,
            enumerable_tag_local,
            enumerable_payload_local,
            enumerable_key_local,
            descriptor_tag_local,
            descriptor_payload_local,
            key_property_payload_local,
            key_tag_local,
            key_payload_local,
            key_index_local,
            keys_length_local,
            keys_tag_local,
            keys_payload_local,
            promises_tag_local,
            promises_payload_local,
            promise_resolve_tag_local,
            promise_resolve_payload_local,
            reject_tag_local,
            reject_payload_local,
            resolve_tag_local,
            resolve_payload_local,
            promise_tag_local,
            promise_payload_local,
            capability_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
