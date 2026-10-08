use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_for_await_result_value(
        &mut self,
        record: &GcLocal<IteratorRecord>,
        resumed: &ValueLocals,
        done: I32Local,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let method = schema.reserve_completion(function);
        self.emit_is_heap_object_like_tag_i32(resumed.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_control_flow_type_error(
            RuntimeErrorMessage::FOR_AWAIT_OF_ASYNC_ITERATOR_NEXT_RESULT_MUST_BE_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let done_key = self.emit_control_flow_string_key("done", function)?;
        self.emit_object_read(resumed, resumed, &done_key, &method, function)?;
        self.completion().copy_from(&method, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.compile_truthy_tagged_i32(method.value(), function)?;
        done.store(function);
        done_key.clear(function);
        schema
            .struct_type::<IteratorRecord>()
            .field(IteratorRecordSchema::DONE)
            .write(record, GcOperand::boolean_local(done), schema, function);
        done.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let value_key = self.emit_control_flow_string_key("value", function)?;
        self.emit_object_read(resumed, resumed, &value_key, &method, function)?;
        self.completion().copy_from(&method, function);
        self.emit_propagate_current_throw_if_needed(function);
        value.copy_from(method.value(), function);
        value_key.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        method.clear(function);
        Ok(())
    }

    // Immediate errors and an absent return consume the same pending whole
    // completion as the eventual close reaction. The caller owns its terminal
    // retirement target; this helper never transfers past that owner.
    pub(super) fn emit_prepare_for_await_close(
        &mut self,
        record: &GcLocal<IteratorRecord>,
        is_async: I32Local,
        iterator_value: &ValueLocals,
        awaited: &ValueLocals,
        function: &mut Function,
        mut finish: impl FnMut(&mut Self, &CompletionLocals, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let method = schema.reserve_completion(function);
        let called = schema.reserve_completion(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        is_async.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let return_key = self.emit_control_flow_string_key("return", function)?;
        self.emit_object_read(
            iterator_value,
            iterator_value,
            &return_key,
            &method,
            function,
        )?;
        return_key.clear(function);
        method.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        finish(self, &method, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_nullish_tagged_i32(method.value().tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        finish(self, &method, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_with_argv(
            method.value(),
            iterator_value,
            &arguments,
            &called,
            function,
        )?;
        called.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        finish(self, &called, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        awaited.copy_from(called.value(), function);
        function.instruction(&Instruction::Else);
        let realm = self.load_current_realm(function);
        let promise = self.emit_async_from_sync_iterator_method(
            record,
            AsyncFromSyncIteratorMethod::Return,
            None,
            &realm,
            function,
        )?;
        awaited.set_reference(&promise, schema, function);
        promise.clear(function);
        realm.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        arguments.clear(function);
        called.clear(function);
        method.clear(function);
        Ok(())
    }

    pub(super) fn emit_acquire_for_await_iterator_state(
        &mut self,
        source: &ValueLocals,
        function: &mut Function,
    ) -> Result<GcLocal<ForAwaitIteratorState>, EmitError> {
        let schema = self.runtime_schema();
        let iterator_value = schema.reserve_value_local(function);
        let next_method = schema.reserve_value_local(function);
        let method = schema.reserve_completion(function);
        let called = schema.reserve_completion(function);
        let is_async = schema.reserve_i32_local(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_for_await_well_known_symbol_read(
            ForAwaitIteratorSymbol::AsyncIterator,
            &source,
            &method,
            function,
        )?;
        self.completion().copy_from(&method, function);
        self.emit_propagate_current_throw_if_needed(function);
        function.instruction(&Instruction::I32Const(1));
        is_async.store(function);
        self.compile_nullish_tagged_i32(method.value().tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        is_async.store(function);
        self.emit_for_await_well_known_symbol_read(
            ForAwaitIteratorSymbol::Iterator,
            &source,
            &method,
            function,
        )?;
        self.completion().copy_from(&method, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_is_callable_i32(method.value(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_control_flow_type_error(
            RuntimeErrorMessage::FOR_AWAIT_OF_ITERATOR_METHOD_MUST_BE_CALLABLE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_with_argv(
            method.value(),
            &source,
            &arguments,
            &called,
            function,
        )?;
        self.completion().copy_from(&called, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.emit_is_heap_object_like_tag_i32(called.value().tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_control_flow_type_error(
            RuntimeErrorMessage::FOR_AWAIT_OF_ITERATOR_METHOD_MUST_RETURN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        iterator_value.copy_from(called.value(), function);
        let next_key = self.emit_control_flow_string_key("next", function)?;
        self.emit_object_read(
            &iterator_value,
            &iterator_value,
            &next_key,
            &method,
            function,
        )?;
        self.completion().copy_from(&method, function);
        self.emit_propagate_current_throw_if_needed(function);
        next_method.copy_from(method.value(), function);
        next_key.clear(function);
        // Acquisition observes next once. Its callability is checked by Call.
        let stored_iterator = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&iterator_value, function),
            function,
        );
        let stored_next = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&next_method, function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IteratorRecord>().construct(
                (
                    GcOperand::reference(&stored_iterator, schema),
                    GcOperand::reference(&stored_next, schema),
                    GcOperand::boolean(false),
                ),
                function,
            ),
            function,
        );
        let state = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ForAwaitIteratorState>().construct(
                (
                    GcOperand::reference(&record, schema),
                    GcOperand::boolean_local(is_async),
                ),
                function,
            ),
            function,
        );
        record.clear(function);
        stored_next.clear(function);
        stored_iterator.clear(function);
        arguments.clear(function);
        schema.release_i32_local(is_async, function);
        called.clear(function);
        method.clear(function);
        next_method.clear(function);
        iterator_value.clear(function);
        Ok(state)
    }

    pub(super) fn emit_cached_for_await_next(
        &mut self,
        record: &GcLocal<IteratorRecord>,
        is_async: I32Local,
        iterator_value: &ValueLocals,
        next_method: &ValueLocals,
        awaited: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let called = schema.reserve_completion(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        // The next method is cached; each call uses the retained original receiver.
        is_async.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_or_proxy_call_with_argv(
            &next_method,
            &iterator_value,
            &arguments,
            &called,
            function,
        )?;
        self.completion().copy_from(&called, function);
        self.emit_propagate_current_throw_if_needed(function);
        awaited.copy_from(called.value(), function);
        function.instruction(&Instruction::Else);
        let realm = self.load_current_realm(function);
        let promise = self.emit_async_from_sync_iterator_method(
            &record,
            AsyncFromSyncIteratorMethod::Next,
            None,
            &realm,
            function,
        )?;
        awaited.set_reference(&promise, schema, function);
        promise.clear(function);
        realm.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        arguments.clear(function);
        called.clear(function);
        Ok(())
    }
}
