use super::super::*;

impl<'a> FunctionBuilder<'a> {
    fn emit_settle_promise_record(
        &mut self,
        promise_record_local: u32,
        state: u64,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) {
        let state_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(PROMISE_STATE_PENDING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            state,
            function,
        );
        self.store_i64_local_at_offset(
            promise_record_local,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            promise_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(state_local);
    }

    pub(crate) fn emit_promise_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let executor_payload_local = self.reserve_temp_local();
        let executor_tag_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let resolve_function_local = self.reserve_temp_local();
        let reject_function_local = self.reserve_temp_local();
        let function_tag_local = self.reserve_temp_local();
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        let call_payload_local = self.reserve_temp_local();
        let call_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, executor_payload_local, executor_tag_local, function);
        self.emit_is_callable_i32(executor_tag_local, executor_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Promise executor is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_error_new_target_prototype_to_local(
            PROMISE_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(promise_payload_local));
        self.emit_heap_alloc_const(HEAP_PROMISE_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(promise_record_local));
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_STATE_OFFSET,
            PROMISE_STATE_PENDING,
            function,
        );
        self.store_i64_const_at_offset(
            promise_record_local,
            HEAP_PROMISE_RESULT_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        for offset in [
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            HEAP_PROMISE_FULFILL_REACTIONS_OFFSET,
            HEAP_PROMISE_REJECT_REACTIONS_OFFSET,
            HEAP_PROMISE_IS_HANDLED_OFFSET,
            HEAP_PROMISE_HOST_DATA_OFFSET,
        ] {
            self.store_i64_const_at_offset(promise_record_local, offset, 0, function);
        }
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            promise_record_local,
            HEAP_PROMISE_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            promise_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_PROMISE,
            function,
        );
        self.store_i64_const_at_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            promise_record_local,
            function,
        );

        for (builtin, resolving_function_local) in [
            (
                StandardBuiltinId::PromiseResolveFunction,
                resolve_function_local,
            ),
            (
                StandardBuiltinId::PromiseRejectFunction,
                reject_function_local,
            ),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(resolving_function_local));
            self.store_i64_local_at_offset(
                resolving_function_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                promise_record_local,
                function,
            );
        }

        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(function_tag_local));
        self.emit_pre_evaluated_arg_vector(
            &[
                (resolve_function_local, function_tag_local),
                (reject_function_local, function_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_handle_call_with_argv_without_throw_propagation(
            executor_payload_local,
            executor_tag_local,
            Some((undefined_payload_local, Some(undefined_tag_local))),
            argc_local,
            argv_local,
            call_payload_local,
            call_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            call_payload_local,
            call_tag_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(call_tag_local);
        self.release_temp_local(call_payload_local);
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(function_tag_local);
        self.release_temp_local(reject_function_local);
        self.release_temp_local(resolve_function_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(promise_payload_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(executor_tag_local);
        self.release_temp_local(executor_payload_local);
        Ok(())
    }

    pub(crate) fn emit_promise_resolving_function(&mut self, state: u64, function: &mut Function) {
        let promise_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(promise_record_local));
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.emit_settle_promise_record(
            promise_record_local,
            state,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(promise_record_local);
    }
}
