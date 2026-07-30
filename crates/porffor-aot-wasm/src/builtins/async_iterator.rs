use super::super::*;

const ASYNC_ITERATOR_DISPOSE_STATE_SIZE: u64 = 16;
const ASYNC_ITERATOR_DISPOSE_PROMISE_RECORD_OFFSET: u64 = 0;
const ASYNC_ITERATOR_DISPOSE_REALM_ENV_OFFSET: u64 = 8;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_async_iterator_prototype_async_dispose(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing AsyncIterator asyncDispose receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing AsyncIterator asyncDispose receiver tag",
            )
        })?;
        let promise_constructor_payload_local = self.reserve_temp_local();
        let promise_constructor_tag_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let receiver_object_payload_local = self.reserve_temp_local();
        let receiver_object_tag_local = self.reserve_temp_local();
        let return_key_local = self.reserve_temp_local();
        let return_payload_local = self.reserve_temp_local();
        let return_tag_local = self.reserve_temp_local();
        let return_result_payload_local = self.reserve_temp_local();
        let return_result_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();
        let throwaway_capability_local = self.reserve_temp_local();
        let throwaway_promise_payload_local = self.reserve_temp_local();
        let throwaway_promise_tag_local = self.reserve_temp_local();
        let fulfilled_payload_local = self.reserve_temp_local();
        let rejected_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(promise_constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(promise_constructor_tag_local));
        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            promise_record_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "AsyncIterator asyncDispose receiver is null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            receiver_object_payload_local,
            receiver_object_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("return")));
        function.instruction(&Instruction::LocalSet(return_key_local));
        self.emit_object_read_without_throw_propagation(
            receiver_object_payload_local,
            receiver_object_tag_local,
            receiver_object_payload_local,
            receiver_object_tag_local,
            return_key_local,
            return_payload_local,
            return_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_iterator_dispose_settle_undefined(
            promise_record_local,
            PROMISE_STATE_FULFILLED,
            function,
        )?;
        self.emit_async_iterator_dispose_return_promise(
            promise_payload_local,
            promise_tag_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.emit_is_callable_i32(return_tag_local, return_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "AsyncIterator asyncDispose return method is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            return_payload_local,
            return_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[(undefined_payload_local, undefined_tag_local)],
            return_result_payload_local,
            return_result_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;

        self.emit_heap_alloc_const(ASYNC_ITERATOR_DISPOSE_STATE_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(state_local));
        self.store_i64_local_at_offset(
            state_local,
            ASYNC_ITERATOR_DISPOSE_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            state_local,
            ASYNC_ITERATOR_DISPOSE_REALM_ENV_OFFSET,
            self.current_env_local,
            function,
        );

        for (builtin, callback_payload_local) in [
            (
                StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled,
                fulfilled_payload_local,
            ),
            (
                StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected,
                rejected_payload_local,
            ),
        ] {
            let callback_meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload(&callback_meta, function)?;
            function.instruction(&Instruction::LocalSet(callback_payload_local));
            self.store_i64_local_at_offset(
                callback_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                state_local,
                function,
            );
        }

        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            throwaway_capability_local,
            throwaway_promise_payload_local,
            throwaway_promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(callback_tag_local));
        self.emit_intrinsic_await_with_handlers(
            return_result_payload_local,
            return_result_tag_local,
            fulfilled_payload_local,
            callback_tag_local,
            rejected_payload_local,
            callback_tag_local,
            throwaway_capability_local,
            function,
        )?;
        self.emit_async_iterator_dispose_return_promise(
            promise_payload_local,
            promise_tag_local,
            function,
        );

        for local in [
            callback_tag_local,
            rejected_payload_local,
            fulfilled_payload_local,
            throwaway_promise_tag_local,
            throwaway_promise_payload_local,
            throwaway_capability_local,
            state_local,
            undefined_tag_local,
            undefined_payload_local,
            return_result_tag_local,
            return_result_payload_local,
            return_tag_local,
            return_payload_local,
            return_key_local,
            receiver_object_tag_local,
            receiver_object_payload_local,
            promise_record_local,
            promise_tag_local,
            promise_payload_local,
            capability_record_local,
            promise_constructor_tag_local,
            promise_constructor_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_async_iterator_prototype_async_dispose_fulfilled(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_async_iterator_dispose_restore_state(state_local, promise_record_local, function);
        self.emit_async_iterator_dispose_settle_undefined(
            promise_record_local,
            PROMISE_STATE_FULFILLED,
            function,
        )?;
        self.emit_async_iterator_dispose_return_undefined(function);

        self.release_temp_local(promise_record_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    pub(crate) fn emit_async_iterator_prototype_async_dispose_rejected(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let reason_payload_local = self.reserve_temp_local();
        let reason_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_async_iterator_dispose_restore_state(state_local, promise_record_local, function);
        self.emit_builtin_arg_to_locals(0, reason_payload_local, reason_tag_local, function);
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            reason_payload_local,
            reason_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_return_undefined(function);

        self.release_temp_local(reason_tag_local);
        self.release_temp_local(reason_payload_local);
        self.release_temp_local(promise_record_local);
        self.release_temp_local(state_local);
        Ok(())
    }

    fn emit_async_iterator_dispose_restore_state(
        &mut self,
        state_local: u32,
        promise_record_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            state_local,
            ASYNC_ITERATOR_DISPOSE_REALM_ENV_OFFSET,
            self.current_env_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            state_local,
            ASYNC_ITERATOR_DISPOSE_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
    }

    fn emit_async_iterator_dispose_settle_undefined(
        &mut self,
        promise_record_local: u32,
        promise_state: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_settle_promise_record(
            promise_record_local,
            promise_state,
            undefined_payload_local,
            undefined_tag_local,
            function,
        )?;

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        Ok(())
    }

    fn emit_async_iterator_dispose_reject_current_throw_and_return(
        &mut self,
        promise_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_settle_promise_record(
            promise_record_local,
            PROMISE_STATE_REJECTED,
            error_payload_local,
            error_tag_local,
            function,
        )?;
        self.emit_async_iterator_dispose_return_promise(
            promise_payload_local,
            promise_tag_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(error_tag_local);
        self.release_temp_local(error_payload_local);
        Ok(())
    }

    fn emit_async_iterator_dispose_return_promise(
        &mut self,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
    }

    fn emit_async_iterator_dispose_return_undefined(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
    }
}
