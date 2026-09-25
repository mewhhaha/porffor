use super::*;

#[derive(Clone, Copy)]
enum ResumableSyncForOfOwner {
    AsyncFunction,
    Generator,
}

impl ResumableSyncForOfOwner {
    fn execution_kind(self) -> FunctionExecutionKind {
        match self {
            Self::AsyncFunction => FunctionExecutionKind::Async,
            Self::Generator => FunctionExecutionKind::Generator,
        }
    }

    fn resume_state_offset(self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_RESUME_STATE_OFFSET,
            Self::Generator => HEAP_GENERATOR_RESUME_STATE_OFFSET,
        }
    }

    fn environment_offset(self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_ENV_OFFSET,
            Self::Generator => HEAP_GENERATOR_LEXICAL_ENV_OFFSET,
        }
    }
}

struct ResumableSyncForOfView<'a> {
    value_storage: AsyncFunctionForOfIteratorValueStorageIr,
    value_mode: BindingMode,
    record: &'a IteratorRecordIr,
    head_environment: Option<&'a ForInOfEnvironmentIr>,
    iteration_environment: &'a ResumableLoopIterationEnvironmentIr,
    body: &'a [StatementIr],
    entry_state: u32,
    body_exit_state: u32,
    exit_state: u32,
}

impl ResumableSyncForOfView<'_> {
    fn body_entry_state(&self) -> u32 {
        self.entry_state + 1
    }

    fn value_name(&self) -> &str {
        match &self.value_storage {
            AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)
            | AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding) => {
                &binding.name
            }
            AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name } => name,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_async_function_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &AsyncFunctionForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_sync_for_of_iterator(
            iterable,
            ResumableSyncForOfView {
                value_storage: plan.value_storage().clone(),
                value_mode: plan.value_mode(),
                record: plan.record(),
                head_environment: plan.head_environment(),
                iteration_environment: plan.iteration_environment(),
                body: plan.body().statements(),
                entry_state: plan.entry_state(),
                body_exit_state: plan.body().exit_state(),
                exit_state: plan.exit_state(),
            },
            ResumableSyncForOfOwner::AsyncFunction,
            function,
        )
    }

    pub(crate) fn compile_generator_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: &GeneratorForOfIteratorPlanIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_storage = match plan.iteration_environment() {
            ResumableLoopIterationEnvironmentIr::StorageOnly => {
                AsyncFunctionForOfIteratorValueStorageIr::Activation(plan.head().clone())
            }
            ResumableLoopIterationEnvironmentIr::FreshPerIteration(_) => {
                AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(plan.head().clone())
            }
        };
        self.compile_resumable_sync_for_of_iterator(
            iterable,
            ResumableSyncForOfView {
                value_storage,
                value_mode: plan.head().mode,
                record: plan.record(),
                head_environment: plan.head_environment(),
                iteration_environment: plan.iteration_environment(),
                body: plan.body(),
                entry_state: plan.entry_state(),
                body_exit_state: plan.body_exit_state(),
                exit_state: plan.exit_state(),
            },
            ResumableSyncForOfOwner::Generator,
            function,
        )
    }

    fn compile_resumable_sync_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        view: ResumableSyncForOfView<'_>,
        owner: ResumableSyncForOfOwner,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol.execution_kind() == owner.execution_kind())
        {
            return Err(EmitError::unsupported(
                "resumable synchronous for-of requires its matching activation",
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
            owner.resume_state_offset(),
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(view.body_exit_state)));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);

        self.push_scope();
        let entry_local_storage = match &view.value_storage {
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
                if !iteration_environment_owns_binding(view.head_environment, &binding.name) {
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
            view.record.iterator().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Object,
        );
        let next_storage = self.allocate_binding(
            view.record.next_method().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Dynamic,
        );
        let done_storage = self.allocate_binding(
            view.record.done().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Boolean,
        );
        for (slot_name, storage) in [
            (view.record.iterator().as_str(), iterator_storage),
            (view.record.next_method().as_str(), next_storage),
            (view.record.done().as_str(), done_storage),
        ] {
            if !matches!(storage, BindingStorage::EnvSlot { .. }) {
                return Err(EmitError::unsupported(format!(
                    "resumable for-of Iterator Record slot `{slot_name}` is not activation-owned"
                )));
            }
        }

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) = view.head_environment {
            self.emit_enter_for_in_of_tdz_scope(view.value_mode, environment, function)?;
        }
        self.compile_expr_to_locals(
            iterable,
            iterator_locals.value_payload,
            iterator_locals.value_tag,
            function,
        )?;
        if let Some(environment) = view.head_environment {
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
        function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
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

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let has_iteration_environment = match view.iteration_environment {
            ResumableLoopIterationEnvironmentIr::StorageOnly => false,
            ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment) => {
                self.push_scope();
                // The saved chain may end inside a body or catch block. Reattach
                // this iteration's child before those owners restore theirs.
                self.emit_enter_resumable_lexical_environment(
                    environment,
                    view.entry_state,
                    function,
                )?;
                true
            }
        };
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if has_iteration_environment {
            self.store_i64_local_at_offset(
                activation_local,
                owner.environment_offset(),
                self.current_env_local,
                function,
            );
        }
        self.initialize_direct_lexical_bindings(view.body, function);
        function.instruction(&Instruction::Else);
        let resumed_iterator_storage = self
            .lookup_binding(view.record.iterator().as_str())
            .expect("resumable for-of iterator slot must remain in scope");
        let resumed_next_storage = self
            .lookup_binding(view.record.next_method().as_str())
            .expect("resumable for-of next-method slot must remain in scope");
        let resumed_done_storage = self
            .lookup_binding(view.record.done().as_str())
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

        let (value_storage, value_is_entry_local) = match &view.value_storage {
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
        function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        if !value_is_entry_local && view.value_mode != BindingMode::Var {
            self.initialize_binding_uninitialized(value_storage, function);
        }
        self.write_binding_from_locals(
            value_storage,
            iterator_locals.value_payload,
            iterator_locals.value_tag,
            function,
        );
        if !value_is_entry_local {
            self.mirror_binding_to_global_object(view.value_name(), value_storage, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if matches!(owner, ResumableSyncForOfOwner::Generator) {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.store_i64_const_at_offset(
                activation_local,
                owner.resume_state_offset(),
                u64::from(view.body_entry_state()),
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        match owner {
            ResumableSyncForOfOwner::AsyncFunction => self.compile_async_statement_sequence(
                view.body,
                view.entry_state,
                owner.resume_state_offset(),
                function,
            )?,
            ResumableSyncForOfOwner::Generator => self.compile_generator_statement_sequence(
                view.body,
                view.body_entry_state(),
                function,
            )?,
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
                owner.environment_offset(),
                self.current_env_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.store_i64_const_at_offset(
            activation_local,
            owner.resume_state_offset(),
            u64::from(view.exit_state),
            function,
        );
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
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.store_i64_const_at_offset(
            activation_local,
            owner.resume_state_offset(),
            u64::from(view.entry_state),
            function,
        );
        function.instruction(&Instruction::I64Const(i64::from(view.entry_state)));
        function.instruction(&Instruction::LocalSet(state_local));
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.store_i64_const_at_offset(
            activation_local,
            owner.resume_state_offset(),
            u64::from(view.exit_state),
            function,
        );
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
