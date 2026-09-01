use super::*;

/// The original completion restored after a `finally` cleanup resolves.
///
/// This is deliberately distinct from Promise record settlement and reaction
/// type. Named wrappers below select the variant, so the builtin dispatcher
/// cannot invert `ThenFinally`/`CatchFinally` with an unlabelled boolean.
enum PromiseFinallyCompletion {
    Fulfill,
    Reject,
}

impl PromiseFinallyCompletion {
    const fn continuation_builtin(self) -> StandardBuiltinId {
        match self {
            Self::Fulfill => StandardBuiltinId::PromiseValueThunk,
            Self::Reject => StandardBuiltinId::PromiseThrower,
        }
    }

    const fn completion_kind(self) -> CompletionKind {
        match self {
            Self::Fulfill => CompletionKind::Normal,
            Self::Reject => CompletionKind::Throw,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_promise_then_finally(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_continuation(PromiseFinallyCompletion::Fulfill, function)
    }

    pub(crate) fn emit_promise_catch_finally(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_continuation(PromiseFinallyCompletion::Reject, function)
    }

    fn emit_promise_finally_continuation(
        &mut self,
        completion: PromiseFinallyCompletion,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context_local = self.reserve_temp_local();
        let original_payload_local = self.reserve_temp_local();
        let original_tag_local = self.reserve_temp_local();
        let on_finally_payload_local = self.reserve_temp_local();
        let on_finally_tag_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let cleanup_payload_local = self.reserve_temp_local();
        let cleanup_tag_local = self.reserve_temp_local();
        let cleanup_promise_payload_local = self.reserve_temp_local();
        let cleanup_promise_tag_local = self.reserve_temp_local();
        let value_context_local = self.reserve_temp_local();
        let continuation_payload_local = self.reserve_temp_local();
        let continuation_tag_local = self.reserve_temp_local();
        let then_key_local = self.reserve_temp_local();
        let then_payload_local = self.reserve_temp_local();
        let then_tag_local = self.reserve_temp_local();

        let continuation_builtin = completion.continuation_builtin();
        let continuation_meta = self
            .functions
            .get(&continuation_builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Promise finally continuation builtin")
            })?;

        self.emit_load_promise_internal_function_context(context_local, function);
        self.emit_builtin_arg_to_locals(0, original_payload_local, original_tag_local, function);
        for (offset, value_local) in [
            (
                HEAP_PROMISE_FINALLY_ON_FINALLY_PAYLOAD_OFFSET,
                on_finally_payload_local,
            ),
            (
                HEAP_PROMISE_FINALLY_ON_FINALLY_TAG_OFFSET,
                on_finally_tag_local,
            ),
            (
                HEAP_PROMISE_FINALLY_CONSTRUCTOR_PAYLOAD_OFFSET,
                constructor_payload_local,
            ),
            (
                HEAP_PROMISE_FINALLY_CONSTRUCTOR_TAG_OFFSET,
                constructor_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(context_local, offset, value_local, function);
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_function_or_proxy_call_leave_throw_completion(
            on_finally_payload_local,
            on_finally_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &[],
            cleanup_payload_local,
            cleanup_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        let resolve_context = self.emit_promise_resolve_operation_realm_context(
            PromiseResolveRealmAuthority::CurrentFunction,
            function,
        )?;
        let resolve_result = self.emit_call_promise_resolve_operation(
            &resolve_context,
            constructor_payload_local,
            constructor_tag_local,
            cleanup_payload_local,
            cleanup_tag_local,
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            function,
        );
        self.release_promise_resolve_operation_realm_context(resolve_context);
        resolve_result?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_heap_alloc_const(HEAP_PROMISE_FINALLY_VALUE_CONTEXT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(value_context_local));
        self.store_i64_local_at_offset(
            value_context_local,
            HEAP_PROMISE_FINALLY_VALUE_PAYLOAD_OFFSET,
            original_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            value_context_local,
            HEAP_PROMISE_FINALLY_VALUE_TAG_OFFSET,
            original_tag_local,
            function,
        );
        let materialization_context =
            self.emit_current_function_promise_internal_function_materialization_context(function);
        self.emit_promise_internal_function_value(
            &continuation_meta,
            &materialization_context,
            value_context_local,
            continuation_payload_local,
            function,
        )?;
        self.release_promise_internal_function_materialization_context(materialization_context);
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(continuation_tag_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("then")));
        function.instruction(&Instruction::LocalSet(then_key_local));
        self.emit_object_read(
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            then_key_local,
            then_payload_local,
            then_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            cleanup_promise_payload_local,
            cleanup_promise_tag_local,
            &[(continuation_payload_local, continuation_tag_local)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.set_completion_kind(CompletionKind::Normal, function);

        self.release_temp_local(then_tag_local);
        self.release_temp_local(then_payload_local);
        self.release_temp_local(then_key_local);
        self.release_temp_local(continuation_tag_local);
        self.release_temp_local(continuation_payload_local);
        self.release_temp_local(value_context_local);
        self.release_temp_local(cleanup_promise_tag_local);
        self.release_temp_local(cleanup_promise_payload_local);
        self.release_temp_local(cleanup_tag_local);
        self.release_temp_local(cleanup_payload_local);
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(on_finally_tag_local);
        self.release_temp_local(on_finally_payload_local);
        self.release_temp_local(original_tag_local);
        self.release_temp_local(original_payload_local);
        self.release_temp_local(context_local);
        Ok(())
    }

    pub(crate) fn emit_promise_value_thunk(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_value_thunk(PromiseFinallyCompletion::Fulfill, function)
    }

    pub(crate) fn emit_promise_thrower(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_value_thunk(PromiseFinallyCompletion::Reject, function)
    }

    fn emit_promise_finally_value_thunk(
        &mut self,
        completion: PromiseFinallyCompletion,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context_local = self.reserve_temp_local();

        self.emit_load_promise_internal_function_context(context_local, function);
        self.load_i64_to_local_from_offset(
            context_local,
            HEAP_PROMISE_FINALLY_VALUE_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            context_local,
            HEAP_PROMISE_FINALLY_VALUE_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(completion.completion_kind(), function);

        self.release_temp_local(context_local);
        Ok(())
    }
}
