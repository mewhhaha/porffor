use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsyncGeneratorDelegationKind {
    YieldStar,
    ForAwaitYield,
}

const ASYNC_GENERATOR_DELEGATE_PENDING_CLOSE_THROW: u64 = 5;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_async_generator_delegation(
        &mut self,
        value: &TypedExpr,
        suspend_state: u32,
        resume_state: u32,
        resume_mode: &GeneratorResumeModeIr,
        delegation_kind: AsyncGeneratorDelegationKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(resume_mode, GeneratorResumeModeIr::AssignProperty(_)) {
            return Err(EmitError::unsupported(
                "async-generator body dispatcher does not yet support property-assignment yield resumption",
            ));
        }

        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("async-generator delegation requires the function call ABI")
        })?;
        let state_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let resume_kind_local = self.reserve_temp_local();
        let pending_kind_local = self.reserve_temp_local();
        let async_iterator_local = self.reserve_temp_local();
        let awaiting_sync_value_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(suspend_state)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(resume_state)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(suspend_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.compile_expr_to_locals(value, value_payload_local, value_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(value_tag_local, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "yield* target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_generator_delegate_property_read(
            value_payload_local,
            value_tag_local,
            "Symbol.asyncIterator",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(async_iterator_local));
        self.emit_generator_delegate_method_is_missing_i32(method_tag_local, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(async_iterator_local));
        self.emit_generator_delegate_property_read(
            value_payload_local,
            value_tag_local,
            "Symbol.iterator",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            value_payload_local,
            value_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            "yield* iterator method must be callable",
            function,
        )?;
        self.emit_require_generator_delegate_object(
            iterator_tag_local,
            "yield* iterator method must return object",
            function,
        )?;
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "next",
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_heap_alloc_const(HEAP_GENERATOR_DELEGATE_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, source_local) in [
            (
                HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET,
                iterator_payload_local,
            ),
            (
                HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET,
                iterator_tag_local,
            ),
            (
                HEAP_GENERATOR_DELEGATE_NEXT_PAYLOAD_OFFSET,
                next_payload_local,
            ),
            (HEAP_GENERATOR_DELEGATE_NEXT_TAG_OFFSET, next_tag_local),
        ] {
            self.store_i64_local_at_offset(record_local, offset, source_local, function);
        }
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_ASYNC_ITERATOR_OFFSET,
            async_iterator_local,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
            0,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
            record_local,
            function,
        );
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(argument_tag_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_NORMAL as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::Else);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
            record_local,
            function,
        );
        for (offset, destination_local) in [
            (
                HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET,
                iterator_payload_local,
            ),
            (
                HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET,
                iterator_tag_local,
            ),
            (
                HEAP_GENERATOR_DELEGATE_NEXT_PAYLOAD_OFFSET,
                next_payload_local,
            ),
            (HEAP_GENERATOR_DELEGATE_NEXT_TAG_OFFSET, next_tag_local),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, destination_local, function);
        }
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_ASYNC_ITERATOR_OFFSET,
            async_iterator_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
            awaiting_sync_value_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_FULFILL as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,
            pending_kind_local,
            function,
        );
        if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            function.instruction(&Instruction::LocalGet(pending_kind_local));
            function.instruction(&Instruction::I64Const(
                ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
                self.result_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
                self.result_tag_local,
                function,
            );
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
                0,
                function,
            );
            self.set_completion_kind(CompletionKind::Throw, function);
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(awaiting_sync_value_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_RESULT_DONE_PAYLOAD_OFFSET,
            done_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_RESULT_DONE_TAG_OFFSET,
            done_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_require_generator_delegate_object(
            argument_tag_local,
            "yield* iterator result must be object",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(pending_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_DELEGATE_PENDING_CLOSE_THROW as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
            0,
            function,
        );
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "yield* iterator has no throw method",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            function.instruction(&Instruction::LocalGet(pending_kind_local));
            function.instruction(&Instruction::I64Const(
                ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
                self.result_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
                self.result_tag_local,
                function,
            );
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
                0,
                function,
            );
            self.set_completion_kind(CompletionKind::Return, function);
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.emit_generator_delegate_property_read(
            argument_payload_local,
            argument_tag_local,
            "done",
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_generator_delegate_property_read(
            argument_payload_local,
            argument_tag_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(async_iterator_local));
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_AWAITING_SYNC_VALUE_OFFSET,
            1,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_RESULT_DONE_PAYLOAD_OFFSET,
            done_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_RESULT_DONE_TAG_OFFSET,
            done_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(resume_state),
            function,
        );
        self.emit_async_generator_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_AWAIT,
            function,
        );
        self.emit_store_async_generator_execution_state(
            activation_local,
            AsyncGeneratorExecutionState::Executing,
            function,
        );
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::LocalGet(pending_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        match resume_mode {
            GeneratorResumeModeIr::Ignore => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            GeneratorResumeModeIr::Return => {
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.set_completion_kind(CompletionKind::Return, function);
                self.emit_return_current_completion(function);
            }
            GeneratorResumeModeIr::AssignIdentifier(name) => {
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.emit_global_property_write(
                        name,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                } else {
                    let storage = self.lookup_binding(name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                        ))
                    })?;
                    self.write_binding_from_locals(
                        storage,
                        value_payload_local,
                        value_tag_local,
                        function,
                    );
                    self.mirror_binding_to_global_object(name, storage, function)?;
                }
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            GeneratorResumeModeIr::AssignProperty(_) => unreachable!(),
        }
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(resume_state),
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_RESULT_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_YIELD,
            function,
        );
        self.emit_store_async_generator_execution_state(
            activation_local,
            AsyncGeneratorExecutionState::SuspendedYield,
            function,
        );
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_REJECT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,
                pending_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(pending_kind_local));
            function.instruction(&Instruction::I64Const(
                ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
                self.result_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                record_local,
                HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
                self.result_tag_local,
                function,
            );
            self.set_completion_kind(CompletionKind::Throw, function);
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Throw, function);
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        if delegation_kind == AsyncGeneratorDelegationKind::YieldStar {
            function.instruction(&Instruction::LocalGet(resume_kind_local));
            function.instruction(&Instruction::I64Const(
                ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
            ));
            function.instruction(&Instruction::I64Eq);
        } else {
            function.instruction(&Instruction::I32Const(0));
        }
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "throw",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_generator_delegate_method_is_missing_i32(method_tag_local, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "return",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_generator_delegate_method_is_missing_i32(method_tag_local, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "yield* iterator has no throw method",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            result_payload_local,
            result_tag_local,
            "yield* return method must be callable",
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_DELEGATE_PENDING_CLOSE_THROW as i64,
        ));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[(argument_payload_local, argument_tag_local)],
            result_payload_local,
            result_tag_local,
            "yield* throw method must be callable",
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(
            ASYNC_GENERATOR_RESUME_KIND_RETURN as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            function.instruction(&Instruction::LocalGet(resume_kind_local));
            function.instruction(&Instruction::I64Const(
                ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "return",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_generator_delegate_method_is_missing_i32(method_tag_local, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            function.instruction(&Instruction::LocalGet(resume_kind_local));
            function.instruction(&Instruction::I64Const(
                ASYNC_GENERATOR_RESUME_KIND_THROW as i64,
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.set_completion_kind(CompletionKind::Throw, function);
            function.instruction(&Instruction::Else);
            self.set_completion_kind(CompletionKind::Return, function);
            function.instruction(&Instruction::End);
        } else {
            self.set_completion_kind(CompletionKind::Return, function);
        }
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        let close_arguments = if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            &[][..]
        } else {
            &[(argument_payload_local, argument_tag_local)][..]
        };
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            close_arguments,
            result_payload_local,
            result_tag_local,
            "yield* return method must be callable",
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        if delegation_kind == AsyncGeneratorDelegationKind::ForAwaitYield {
            self.emit_undefined_payload(function);
            function.instruction(&Instruction::LocalSet(argument_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(argument_tag_local));
        }
        self.emit_generator_delegate_call(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[(argument_payload_local, argument_tag_local)],
            result_payload_local,
            result_tag_local,
            "yield* next method must be callable",
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(resume_state),
            function,
        );
        self.emit_async_generator_await_reactions(
            activation_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET,
            ASYNC_GENERATOR_BODY_STATUS_AWAIT,
            function,
        );
        self.emit_store_async_generator_execution_state(
            activation_local,
            AsyncGeneratorExecutionState::Executing,
            function,
        );
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            i64::from(resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.release_temp_local(awaiting_sync_value_local);
        self.release_temp_local(async_iterator_local);
        self.release_temp_local(pending_kind_local);
        self.release_temp_local(resume_kind_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(record_local);
        self.release_temp_local(state_local);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_generator_delegation(
        &mut self,
        value: &TypedExpr,
        suspend_state: u32,
        resume_state: u32,
        resume_mode: &GeneratorResumeModeIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported("generator delegation requires the function call ABI")
        })?;
        let state_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let resume_kind_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(suspend_state)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(resume_state)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(suspend_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
            self.prepare_suspended_property_reference(reference, activation_local, function)?;
        }
        self.compile_expr_to_locals(value, value_payload_local, value_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.compile_nullish_tagged_i32(value_tag_local, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "yield* target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_generator_delegate_property_read(
            value_payload_local,
            value_tag_local,
            "Symbol.iterator",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            value_payload_local,
            value_tag_local,
            &[],
            iterator_payload_local,
            iterator_tag_local,
            "yield* iterator method must be callable",
            function,
        )?;
        self.emit_require_generator_delegate_object(
            iterator_tag_local,
            "yield* iterator method must return object",
            function,
        )?;
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "next",
            next_payload_local,
            next_tag_local,
            function,
        )?;

        self.emit_heap_alloc_const(HEAP_GENERATOR_DELEGATE_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET,
            iterator_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET,
            iterator_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_NEXT_PAYLOAD_OFFSET,
            next_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_NEXT_TAG_OFFSET,
            next_tag_local,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            activation_local,
            HEAP_GENERATOR_DELEGATE_RECORD_OFFSET,
            record_local,
            function,
        );
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(argument_tag_local));
        function.instruction(&Instruction::I64Const(GENERATOR_RESUME_KIND_NORMAL as i64));
        function.instruction(&Instruction::LocalSet(resume_kind_local));
        function.instruction(&Instruction::Else);

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_DELEGATE_RECORD_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_ITERATOR_PAYLOAD_OFFSET,
            iterator_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_ITERATOR_TAG_OFFSET,
            iterator_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_NEXT_PAYLOAD_OFFSET,
            next_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_GENERATOR_DELEGATE_NEXT_TAG_OFFSET,
            next_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET,
            argument_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_TAG_OFFSET,
            argument_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_KIND_OFFSET,
            resume_kind_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(GENERATOR_RESUME_KIND_THROW as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "throw",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_generator_delegate_method_is_missing_i32(method_tag_local, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_iterator_close(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "yield* iterator has no throw method",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[(argument_payload_local, argument_tag_local)],
            result_payload_local,
            result_tag_local,
            "yield* throw method must be callable",
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(GENERATOR_RESUME_KIND_RETURN as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            iterator_payload_local,
            iterator_tag_local,
            "return",
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_generator_delegate_method_is_missing_i32(method_tag_local, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(argument_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            method_payload_local,
            method_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[(argument_payload_local, argument_tag_local)],
            result_payload_local,
            result_tag_local,
            "yield* return method must be callable",
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::Else);
        self.emit_generator_delegate_call(
            next_payload_local,
            next_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[(argument_payload_local, argument_tag_local)],
            result_payload_local,
            result_tag_local,
            "yield* next method must be callable",
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_require_generator_delegate_object(
            result_tag_local,
            "yield* iterator result must be object",
            function,
        )?;
        self.emit_generator_delegate_property_read(
            result_payload_local,
            result_tag_local,
            "done",
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_generator_delegate_property_read(
            result_payload_local,
            result_tag_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(resume_kind_local));
        function.instruction(&Instruction::I64Const(GENERATOR_RESUME_KIND_RETURN as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_dispatch_current_completion(function)?;
        function.instruction(&Instruction::Else);
        match resume_mode {
            GeneratorResumeModeIr::Ignore => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            GeneratorResumeModeIr::Return => {
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.set_completion_kind(CompletionKind::Return, function);
                self.emit_dispatch_current_completion(function)?;
            }
            GeneratorResumeModeIr::AssignIdentifier(name) => {
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.emit_global_property_write(
                        name,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                } else {
                    let storage = self.lookup_binding(name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                        ))
                    })?;
                    self.write_binding_from_locals(
                        storage,
                        value_payload_local,
                        value_tag_local,
                        function,
                    );
                    self.mirror_binding_to_global_object(name, storage, function)?;
                }
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            GeneratorResumeModeIr::AssignProperty(reference) => {
                self.write_suspended_property_reference(
                    reference,
                    activation_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            activation_local,
            HEAP_GENERATOR_RESUME_STATE_OFFSET,
            u64::from(resume_state),
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Normal,
            GENERATOR_DELEGATED_RESULT_AUX_FLAG | i64::from(resume_state),
            function,
        );
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.release_temp_local(resume_kind_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(record_local);
        self.release_temp_local(state_local);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_generator_delegate_call(
        &mut self,
        callee_payload_local: u32,
        callee_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        args: &[(u32, u32)],
        result_payload_local: u32,
        result_tag_local: u32,
        not_callable_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_callable_i32(callee_tag_local, callee_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            not_callable_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(callee_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_function_handle_call(
            callee_payload_local,
            callee_tag_local,
            Some((this_payload_local, Some(this_tag_local))),
            args,
            result_payload_local,
            result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            args,
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            result_payload_local,
            result_tag_local,
            function,
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_generator_delegate_property_read(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        key: &str,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if key.starts_with("Symbol.") {
            let target = TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags()
                        .without(ValueKind::Undefined)
                        .without(ValueKind::Null),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::Undefined,
            );
            let symbol_key = TypedExpr::from_info(
                ValueInfo::new(ValueKind::Symbol),
                ExprIr::String(key.to_string()),
            );
            self.compile_property_read_from_locals(
                &target,
                &PropertyKeyIr::StringExpr(Box::new(symbol_key)),
                target_payload_local,
                target_tag_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            return self.emit_propagate_throw_from_locals_if_needed(
                value_payload_local,
                value_tag_local,
                function,
            );
        }

        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read_without_throw_propagation(
            target_payload_local,
            target_tag_local,
            target_payload_local,
            target_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.release_temp_local(key_local);
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )
    }

    fn emit_generator_delegate_method_is_missing_i32(
        &self,
        method_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_require_generator_delegate_object(
        &mut self,
        value_tag_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
