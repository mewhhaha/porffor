use super::*;

mod plan;
pub(crate) use plan::ResumableSyncForOfPlan;
use plan::{ResumableSyncForOfOwner, ResumableSyncForOfValueStorage};

impl ResumableSyncForOfOwner {
    fn execution_kind(self) -> FunctionExecutionKind {
        match self {
            Self::Async => FunctionExecutionKind::Async,
            Self::Generator => FunctionExecutionKind::Generator,
        }
    }
    fn emit_dispatch_completion(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self {
            Self::Async => builder.emit_dispatch_async_completion(function),
            Self::Generator => builder.emit_dispatch_current_completion(function),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_resumable_sync_for_of_iterator(
        &mut self,
        iterable: &TypedExpr,
        plan: ResumableSyncForOfPlan<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let owner = plan.owner();
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == owner.execution_kind())
        {
            return Err(EmitError::unsupported(
                "resumable synchronous for-of requires its checked plan's activation kind",
            ));
        }
        let schema = self.runtime_schema();
        let point = self.emit_resumable_resume_point(function)?;
        point.load(function);
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        function.instruction(&Instruction::I32GeU);
        point.load(function);
        function.instruction(&Instruction::I32Const(plan.body_exit_state() as i32));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        let entry_local = match plan.value_storage() {
            ResumableSyncForOfValueStorage::Activation(binding) => {
                let storage = if binding.mode == BindingMode::Var {
                    self.lookup_binding(&binding.name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unbound resumable for-of var `{}`",
                            binding.name
                        ))
                    })?
                } else {
                    self.allocate_binding(
                        binding.name.clone(),
                        binding.mode,
                        ValueKind::Dynamic,
                        function,
                    )
                };
                if !matches!(storage, BindingStorage::EnvSlot { .. }) {
                    return Err(EmitError::unsupported(
                        "resumable for-of source binding is not activation-owned",
                    ));
                }
                if binding.mode == BindingMode::Var {
                    self.binding_scopes
                        .last_mut()
                        .expect("binding scope exists")
                        .insert(binding.name.clone(), storage);
                }
                None
            }
            ResumableSyncForOfValueStorage::IterationEnvironment(binding) => {
                if !iteration_environment_owns_binding(plan.head_environment(), &binding.name) {
                    return Err(EmitError::unsupported(
                        "resumable for-of iteration environment does not own its source binding",
                    ));
                }
                None
            }
            ResumableSyncForOfValueStorage::EntryLocal(name) => {
                // This incoming prefix sink has no source BindingMode or TDZ.
                let undefined = schema.reserve_value_local(function);
                undefined.set_undefined(function);
                let storage = self.allocate_compiler_temporary_binding(&undefined, function);
                undefined.clear(function);
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope exists")
                    .insert(name.to_string(), storage);
                Some(storage)
            }
        };
        let iterator_storage = self.allocate_binding(
            plan.record().iterator().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Object,
            function,
        );
        let next_storage = self.allocate_binding(
            plan.record().next_method().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Dynamic,
            function,
        );
        let done_storage = self.allocate_binding(
            plan.record().done().as_str().to_string(),
            BindingMode::Let,
            ValueKind::Boolean,
            function,
        );
        for storage in [iterator_storage, next_storage, done_storage] {
            if !matches!(storage, BindingStorage::EnvSlot { .. }) {
                return Err(EmitError::unsupported(
                    "resumable for-of Iterator Record must retain all three activation-owned cells",
                ));
            }
        }
        let record = schema
            .reserve_gc_local::<IteratorRecord, Nullable>(function)
            .initialize_null(schema, function);
        let source = schema.reserve_value_local(function);
        let iterator_value = schema.reserve_value_local(function);
        let next_method = schema.reserve_value_local(function);
        let done_value = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let done = schema.reserve_i32_local(function);
        let pending = schema.reserve_completion(function);
        let closed = schema.reserve_completion(function);
        point.load(function);
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        if let Some((mode, environment)) = plan.head_binding_environment() {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_value(iterable, &source, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        if let Some(environment) = plan.head_environment() {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }
        let acquired =
            self.emit_get_sync_iterator(&source, SyncIteratorConsumer::ForOf, function)?;
        let stored_iterator = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IteratorRecord>()
                .field(IteratorRecordSchema::ITERATOR)
                .read(acquired.record(), schema, function)
                .reference(),
            function,
        );
        let stored_next = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IteratorRecord>()
                .field(IteratorRecordSchema::NEXT_METHOD)
                .read(acquired.record(), schema, function)
                .reference(),
            function,
        );
        schema.struct_type::<StoredValue>().read_into(
            &stored_iterator,
            &iterator_value,
            schema,
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored_next, &next_method, schema, function);
        self.write_binding_from_locals(iterator_storage, &iterator_value, function);
        self.write_binding_from_locals(next_storage, &next_method, function);
        done_value.set_scalar(ScalarValue::Boolean(false), function);
        self.write_binding_from_locals(done_storage, &done_value, function);
        record.replace(
            acquired.record().load(schema, function).nullable(),
            function,
        );
        stored_next.clear(function);
        stored_iterator.clear(function);
        acquired.clear(function);
        function.instruction(&Instruction::Else);
        self.read_binding_to_locals(iterator_storage, &iterator_value, function)?;
        self.read_binding_to_locals(next_storage, &next_method, function)?;
        self.read_binding_to_locals(done_storage, &done_value, function)?;
        self.compile_truthy_tagged_i32(&done_value, function)?;
        done.store(function);
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
        let restored = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IteratorRecord>().construct(
                (
                    GcOperand::reference(&stored_iterator, schema),
                    GcOperand::reference(&stored_next, schema),
                    GcOperand::boolean_local(done),
                ),
                function,
            ),
            function,
        );
        record.replace(restored.load(schema, function).nullable(), function);
        restored.clear(function);
        stored_next.clear(function);
        stored_iterator.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let iterator = OwnedSyncIterator {
            record: schema.reserve_gc_local(function).initialize(
                record.load(schema, function).require_non_null(function),
                function,
            ),
            consumer: SyncIteratorConsumer::ForOf,
        };
        record.clear(function);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        point.load(function);
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_sync_iterator_step_value(&iterator, done, &value, function)?;
        done_value.set_boolean(done, function);
        self.write_binding_from_locals(done_storage, &done_value, function);
        done.load(function);
        self.emit_branch_if_to_target(break_frame, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let fresh_iteration = match plan.iteration_environment() {
            ResumableLoopIterationEnvironmentIr::StorageOnly => false,
            ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment) => {
                self.push_scope();
                self.emit_enter_resumable_lexical_environment(
                    environment,
                    plan.entry_state(),
                    function,
                )?;
                true
            }
        };
        point.load(function);
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        if fresh_iteration {
            self.emit_save_resumable_environment(function)?;
        }
        self.initialize_direct_lexical_bindings(plan.body_statements(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let (value_storage, temporary) = match plan.value_storage() {
            ResumableSyncForOfValueStorage::Activation(binding) => (
                self.lookup_binding(&binding.name).ok_or_else(|| {
                    EmitError::unsupported("resumable for-of activation value storage is missing")
                })?,
                false,
            ),
            ResumableSyncForOfValueStorage::IterationEnvironment(binding) => (
                self.lookup_current_scope_binding(&binding.name)
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "resumable for-of iteration value storage is missing",
                        )
                    })?,
                false,
            ),
            ResumableSyncForOfValueStorage::EntryLocal(_) => {
                (entry_local.expect("prefix sink is allocated"), true)
            }
        };
        self.breakable_stack.push(break_frame);
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        let close_frame = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(close_frame);
        point.load(function);
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        if plan
            .binding_mode()
            .is_some_and(|mode| mode != BindingMode::Var)
        {
            self.initialize_binding_uninitialized(value_storage, function);
        }
        self.write_binding_from_locals(value_storage, &value, function);
        if !temporary {
            self.mirror_binding_to_global_object(plan.value_name(), value_storage, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_resumable_statement_sequence(
            plan.body_statements(),
            plan.entry_state(),
            function,
        )?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().kind().load(function);
        function.instruction(&Instruction::I32Const(
            CompletionKind::Continue.code() as i32
        ));
        function.instruction(&Instruction::I32Eq);
        self.completion().target().load(function);
        function.instruction(&Instruction::I32Const(continue_frame.frame as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.copy_from(self.completion(), function);
        if fresh_iteration {
            self.emit_leave_lexical_environment(function);
            self.pop_scope();
            self.emit_save_resumable_environment(function)?;
        }
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;
        self.completion().copy_from(&closed, function);
        owner.emit_dispatch_completion(self, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_set_resumable_resume_point(plan.entry_state(), function)?;
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        point.store(function);
        self.emit_branch_to_target(loop_frame, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.emit_statement_result(function);
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        closed.clear(function);
        pending.clear(function);
        schema.release_i32_local(done, function);
        value.clear(function);
        done_value.clear(function);
        next_method.clear(function);
        iterator_value.clear(function);
        source.clear(function);
        iterator.clear(function);
        if let Some(BindingStorage::Local(id)) = entry_local {
            self.release_local_binding(id, function);
        }
        schema.release_i32_local(point, function);
        Ok(())
    }
}
