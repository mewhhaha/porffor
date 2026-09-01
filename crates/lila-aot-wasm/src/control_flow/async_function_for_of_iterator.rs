use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_async_function_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &AsyncFunctionForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol.execution_kind() == FunctionExecutionKind::Async)
        {
            return Err(EmitError::unsupported(
                "resumable synchronous for-of requires a plain async function activation",
            ));
        }
        let activation_local = self.new_target_payload_local().ok_or_else(|| {
            EmitError::unsupported(
                "resumable synchronous for-of requires the async function call ABI",
            )
        })?;
        let state_local = self.reserve_temp_local();
        let iterator_locals = self.reserve_sync_iterator_locals();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();
        let consumer = SyncIteratorConsumer::ForOf;
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            activation_local,
            HEAP_ASYNC_RESUME_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(plan.resume_state())));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);

        self.push_scope();
        let entry_local_storage = match plan.value_storage() {
            AsyncFunctionForOfIteratorValueStorageIr::Activation(binding) => {
                let storage = if binding.mode == BindingMode::Var {
                    self.lookup_binding(&binding.name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unbound resumable for-of var `{}`",
                            binding.name
                        ))
                    })?
                } else {
                    self.allocate_binding(binding.name.clone(), binding.mode, ValueKind::Dynamic)
                };
                if !matches!(storage, BindingStorage::EnvSlot { .. }) {
                    return Err(EmitError::unsupported(format!(
                        "resumable for-of binding `{}` is not activation-owned",
                        binding.name
                    )));
                }
                if binding.mode == BindingMode::Var {
                    self.binding_scopes
                        .last_mut()
                        .expect("binding scope stack must exist")
                        .insert(binding.name.clone(), storage);
                }
                None
            }
            AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding) => {
                if !iteration_environment_owns_binding(plan.head_environment(), &binding.name) {
                    return Err(EmitError::unsupported(format!(
                        "resumable for-of iteration environment does not own binding `{}`",
                        binding.name
                    )));
                }
                None
            }
            AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name } => {
                let storage =
                    self.allocate_binding(name.clone(), BindingMode::Let, ValueKind::Dynamic);
                if !matches!(storage, BindingStorage::Dynamic { .. }) {
                    return Err(EmitError::unsupported(format!(
                        "resumable for-of entry-local value `{name}` did not allocate local storage"
                    )));
                }
                Some(storage)
            }
        };

        let iterator_storage = self.allocate_binding(
            plan.record().iterator().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Object,
        );
        let next_storage = self.allocate_binding(
            plan.record().next_method().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Dynamic,
        );
        let done_storage = self.allocate_binding(
            plan.record().done().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Boolean,
        );
        for (slot_name, storage) in [
            (plan.record().iterator().as_str(), iterator_storage),
            (plan.record().next_method().as_str(), next_storage),
            (plan.record().done().as_str(), done_storage),
        ] {
            if !matches!(storage, BindingStorage::EnvSlot { .. }) {
                return Err(EmitError::unsupported(format!(
                    "resumable for-of Iterator Record slot `{slot_name}` is not activation-owned"
                )));
            }
        }

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) = plan.head_environment() {
            self.emit_enter_for_in_of_tdz_scope(plan.value_mode(), environment, function)?;
        }
        self.compile_expr_to_locals(
            iterable,
            iterator_locals.value_payload,
            iterator_locals.value_tag,
            function,
        )?;
        if let Some(environment) = plan.head_environment() {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }
        self.emit_get_iterator_from_value_locals(
            iterable.value_info(),
            iterator_locals.value_payload,
            iterator_locals.value_tag,
            method_payload_local,
            method_tag_local,
            &iterator_locals,
            &consumer,
            function,
        )?;
        self.write_binding_from_locals(
            iterator_storage,
            iterator_locals.iterator_payload,
            iterator_locals.iterator_tag,
            function,
        );
        self.write_binding_from_locals(
            next_storage,
            iterator_locals.next_payload,
            iterator_locals.next_tag,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::LocalSet(iterator_locals.done_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_locals.done_tag));
        self.write_binding_from_locals(
            done_storage,
            iterator_locals.done_payload,
            iterator_locals.done_tag,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_sync_iterator_step_value(&iterator_locals, done_local, &consumer, function)?;
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::LocalSet(iterator_locals.done_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(iterator_locals.done_tag));
        self.write_binding_from_locals(
            done_storage,
            iterator_locals.done_payload,
            iterator_locals.done_tag,
            function,
        );
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I32WrapI64);
        function.branch_if_to_label(break_frame.label);

        let has_iteration_environment = match plan.iteration_environment() {
            ResumableLoopIterationEnvironmentIr::StorageOnly => false,
            ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment) => {
                self.push_scope();
                self.emit_enter_lexical_environment(environment, function)?;
                self.store_i64_local_at_offset(
                    activation_local,
                    HEAP_ASYNC_ENV_OFFSET,
                    self.current_env_local,
                    function,
                );
                true
            }
        };
        self.initialize_direct_lexical_bindings(plan.before_await(), function);
        self.initialize_direct_lexical_bindings(plan.after_await(), function);
        function.instruction(&Instruction::Else);
        let resumed_iterator_storage = self
            .lookup_binding(plan.record().iterator().as_str())
            .expect("resumable for-of iterator slot must remain in scope");
        let resumed_next_storage = self
            .lookup_binding(plan.record().next_method().as_str())
            .expect("resumable for-of next-method slot must remain in scope");
        let resumed_done_storage = self
            .lookup_binding(plan.record().done().as_str())
            .expect("resumable for-of done slot must remain in scope");
        self.read_binding_to_locals(
            resumed_iterator_storage,
            iterator_locals.iterator_payload,
            iterator_locals.iterator_tag,
            function,
        )?;
        self.read_binding_to_locals(
            resumed_next_storage,
            iterator_locals.next_payload,
            iterator_locals.next_tag,
            function,
        )?;
        self.read_binding_to_locals(
            resumed_done_storage,
            iterator_locals.done_payload,
            iterator_locals.done_tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(iterator_locals.done_payload));
        function.instruction(&Instruction::LocalSet(done_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let (value_storage, value_is_entry_local) = match plan.value_storage() {
            AsyncFunctionForOfIteratorValueStorageIr::Activation(binding) => (
                self.lookup_binding(&binding.name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "resumable for-of activation value `{}` has no storage after iteration entry",
                        binding.name
                    ))
                })?,
                false,
            ),
            AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding) => (
                self.lookup_current_scope_binding(&binding.name)
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "resumable for-of iteration value `{}` is absent from the current environment",
                            binding.name
                        ))
                    })?,
                false,
            ),
            AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { .. } => (
                entry_local_storage.expect(
                    "resumable for-of entry-local storage must be allocated before iteration",
                ),
                true,
            ),
        };
        let close_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(close_frame);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if !value_is_entry_local && plan.value_mode() != BindingMode::Var {
            self.initialize_binding_uninitialized(value_storage, function);
        }
        self.write_binding_from_locals(
            value_storage,
            iterator_locals.value_payload,
            iterator_locals.value_tag,
            function,
        );
        if !value_is_entry_local {
            self.mirror_binding_to_global_object(plan.value_name(), value_storage, function)?;
        }
        for statement in plan.before_await() {
            self.compile_statement(statement, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_statement(plan.await_statement(), function)?;
        for statement in plan.after_await() {
            self.compile_statement(statement, function)?;
        }
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        if has_iteration_environment {
            self.emit_leave_lexical_environment(function);
            self.pop_scope();
            self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_ENV_OFFSET,
                self.current_env_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_set_async_resume_state(activation_local, plan.exit_state(), function);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local: iterator_locals.iterator_payload,
                iterator_tag_local: iterator_locals.iterator_tag,
                key_local: iterator_locals.key,
                return_payload_local: method_payload_local,
                return_tag_local: method_tag_local,
                result_payload_local: iterator_locals.result_payload,
                result_tag_local: iterator_locals.result_tag,
                saved_payload_local: close_saved_payload_local,
                saved_tag_local: close_saved_tag_local,
                saved_completion_local: close_saved_completion_local,
                saved_aux_local: close_saved_aux_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_iterator_close(
            iterator_locals.iterator_payload,
            iterator_locals.iterator_tag,
            iterator_locals.key,
            method_payload_local,
            method_tag_local,
            iterator_locals.result_payload,
            iterator_locals.result_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_dispatch_async_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_set_async_resume_state(activation_local, plan.entry_state(), function);
        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));
        function.instruction(&Instruction::LocalSet(state_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.emit_set_async_resume_state(activation_local, plan.exit_state(), function);
        self.emit_statement_result(function, ValueKind::Undefined);
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        self.release_temp_local(done_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_sync_iterator_locals(iterator_locals);
        self.release_temp_local(state_local);
        Ok(())
    }
}
