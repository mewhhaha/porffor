//! Complete mixed iterator phases retain the original IteratorRecord table row.

use super::generator_resource_scope::CompleteMixedResourceLifetime;
use super::*;
use lila_ir::{AsyncGeneratorForOfIr, AsyncGeneratorIteratorProtocolIr, ResumableRegionProtocolIr};

impl FunctionBuilder<'_> {
    pub(super) fn compile_async_generator_for_of(
        &mut self,
        plan: &AsyncGeneratorForOfIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_environment_owner;
        if plan.execution() == ResumableRegionProtocolIr::AsyncGenerator {
            self.checked_async_generator_environment_owner =
                Some(CheckedAsyncGeneratorEnvironmentOwner::for_for_of(plan));
        }
        let result = self.compile_complete_mixed_iterator(plan, labels, function);
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_complete_mixed_iterator(
        &mut self,
        plan: &AsyncGeneratorForOfIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected = match plan.execution() {
            ResumableRegionProtocolIr::Generator => FunctionExecutionKind::Generator,
            ResumableRegionProtocolIr::Async => FunctionExecutionKind::Async,
            ResumableRegionProtocolIr::AsyncGenerator => FunctionExecutionKind::AsyncGenerator,
        };
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == expected)
        {
            return Err(EmitError::unsupported(
                "compiler invariant: complete iterator requires its checked source execution owner",
            ));
        }
        let schema = self.runtime_schema();
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_checkpoint_generator_statement_list_value(function);
        self.push_scope();
        let frame = schema.reserve_gc_local(function).initialize(
            self.pending_completion_frame()?.load(schema, function),
            function,
        );
        let table = self.emit_for_await_state_table(&frame, plan.entry_state(), function)?;
        let index = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        index.store(function);
        let source = schema.reserve_value_local(function);
        let iterator_value = schema.reserve_value_local(function);
        let next_method = schema.reserve_value_local(function);
        let awaited = schema.reserve_value_local(function);
        let resumed = schema.reserve_value_local(function);
        let incoming = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let closed = schema.reserve_completion(function);
        let is_async = schema.reserve_i32_local(function);
        let rejected = schema.reserve_i32_local(function);
        let done = schema.reserve_i32_local(function);

        // Acquisition failures retire this row without closing an iterator.
        // The inner per-key close scope is installed only after acquisition.
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(cleanup);
        self.finally_stack.push(cleanup);
        let break_target = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_target);
        self.begin_async_generator_iterator_value(plan, break_target)?;
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        incoming.set_undefined(function);
        self.write_generator_statement_list_binding(plan.value_binding(), &incoming, function);
        self.emit_statement_result(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_resumable_state_in_range(
            plan.entry_state(),
            plan.head().region().end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        if let Some(environment) = plan
            .lexical_environment()
            .and_then(|e| e.tdz_environment.as_ref())
        {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.entry_state(),
                function,
            )?;
        }
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) = plan.lexical_environment() {
            for name in &environment.tdz_binding_names {
                let binding = self.lookup_current_scope_binding(name).unwrap_or_else(|| {
                    self.allocate_binding(
                        name.clone(),
                        plan.head_mode(),
                        ValueKind::Dynamic,
                        function,
                    )
                });
                self.initialize_binding_uninitialized(binding, function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.compile_resumable_operand_region(
            plan.head().region().block(),
            plan.entry_state(),
            function,
        )?;
        if plan
            .lexical_environment()
            .and_then(|e| e.tdz_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_save_resumable_environment(function)?;
        self.emit_set_resumable_resume_point(plan.acquisition_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_resumable_state_equals(plan.acquisition_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.read_generator_statement_list_binding(plan.head_binding(), &source, function);
        let acquired = match plan.protocol() {
            AsyncGeneratorIteratorProtocolIr::Sync => {
                let iterator =
                    self.emit_get_sync_iterator(&source, SyncIteratorConsumer::ForOf, function)?;
                let state = schema.reserve_gc_local(function).initialize(
                    schema.struct_type::<ForAwaitIteratorState>().construct(
                        (
                            GcOperand::reference(iterator.record(), schema),
                            GcOperand::boolean(false),
                        ),
                        function,
                    ),
                    function,
                );
                iterator.clear(function);
                state
            }
            AsyncGeneratorIteratorProtocolIr::Awaited { .. } => {
                self.emit_acquire_for_await_iterator_state(&source, function)?
            }
        };
        schema.array_type::<ForAwaitIteratorTable>().write(
            &table,
            index,
            GcOperand::nullable_reference(&acquired, schema),
            schema,
            function,
        );
        acquired.clear(function);
        source.set_undefined(function);
        self.write_generator_statement_list_binding(plan.head_binding(), &source, function);
        self.emit_set_resumable_resume_point(plan.advance_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let state = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ForAwaitIteratorTable>()
                .read(&table, index, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ForAwaitIteratorState>()
                .field(ForAwaitIteratorStateSchema::RECORD)
                .read(&state, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<ForAwaitIteratorState>()
            .field(ForAwaitIteratorStateSchema::ASYNC_ITERATOR)
            .read(&state, schema, function)
            .store(is_async, function);
        let stored_iterator = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IteratorRecord>()
                .field(IteratorRecordSchema::ITERATOR)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        let stored_next = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<IteratorRecord>()
                .field(IteratorRecordSchema::NEXT_METHOD)
                .read(&record, schema, function)
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
        stored_next.clear(function);
        stored_iterator.clear(function);

        if let AsyncGeneratorIteratorProtocolIr::Awaited {
            close_resume_state, ..
        } = plan.protocol()
        {
            self.emit_resumable_state_equals(close_resume_state, function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_load_async_continuation_resume(
                Self::complete_iterator_continuation_owner(plan)?,
                &resumed,
                rejected,
                function,
            )?;
            closed.set_normal(&resumed, function);
            rejected.load(function);
            self.open_frame(ControlFrameKind::If, function);
            closed.set_throw(&resumed, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.emit_restore_for_await_close(&closed, true, function)?;
            self.emit_complete_iterator_exit_route(break_target, cleanup, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        let iteration_loop = self.open_frame(ControlFrameKind::Loop, function);
        self.deactivate_generator_statement_list_value();
        self.emit_resumable_state_equals(plan.advance_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.read_generator_statement_list_binding(plan.value_binding(), &incoming, function);
        self.completion().set_normal(&incoming, function);
        match plan.protocol() {
            AsyncGeneratorIteratorProtocolIr::Sync => {
                let iterator = OwnedSyncIterator {
                    record: schema
                        .reserve_gc_local(function)
                        .initialize(record.load(schema, function), function),
                    consumer: SyncIteratorConsumer::ForOf,
                };
                self.emit_sync_iterator_step_value(&iterator, done, &incoming, function)?;
                iterator.clear(function);
                self.read_generator_statement_list_binding(plan.value_binding(), &source, function);
                self.completion().set_normal(&source, function);
                done.load(function);
                self.emit_branch_if_to_target(break_target, function);
                self.write_generator_statement_list_binding(
                    plan.incoming_binding(),
                    &incoming,
                    function,
                );
                self.emit_set_resumable_resume_point(
                    plan.initialization().entry_state(),
                    function,
                )?;
            }
            AsyncGeneratorIteratorProtocolIr::Awaited {
                next_resume_state, ..
            } => {
                self.emit_cached_for_await_next(
                    &record,
                    is_async,
                    &iterator_value,
                    &next_method,
                    &awaited,
                    function,
                )?;
                self.emit_set_resumable_resume_point(next_resume_state, function)?;
                self.emit_async_continuation_await(
                    Self::complete_iterator_continuation_owner(plan)?,
                    &awaited,
                    function,
                )?;
                self.emit_return_async_suspension(function)?;
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if let AsyncGeneratorIteratorProtocolIr::Awaited {
            next_resume_state, ..
        } = plan.protocol()
        {
            self.emit_resumable_state_equals(next_resume_state, function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_load_async_continuation_resume(
                Self::complete_iterator_continuation_owner(plan)?,
                &resumed,
                rejected,
                function,
            )?;
            rejected.load(function);
            self.open_frame(ControlFrameKind::If, function);
            self.completion().set_throw(&resumed, function);
            self.emit_propagate_current_throw(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.read_generator_statement_list_binding(plan.value_binding(), &incoming, function);
            self.completion().set_normal(&incoming, function);
            self.emit_for_await_result_value(&record, &resumed, done, &incoming, function)?;
            self.read_generator_statement_list_binding(plan.value_binding(), &source, function);
            self.completion().set_normal(&source, function);
            done.load(function);
            self.emit_branch_if_to_target(break_target, function);
            self.write_generator_statement_list_binding(
                plan.incoming_binding(),
                &incoming,
                function,
            );
            self.emit_set_resumable_resume_point(plan.initialization().entry_state(), function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        let resource_lifetime = CompleteMixedResourceLifetime::for_for_of(plan);
        match plan.resource() {
            Some(resource) => self.emit_resumable_state_in_range(
                plan.initialization().entry_state(),
                resource.exit_state(),
                false,
                function,
            )?,
            None => self.emit_resumable_state_in_range(
                plan.initialization().entry_state(),
                plan.body().end_state(),
                true,
                function,
            )?,
        }
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        if let Some(environment) = plan
            .lexical_environment()
            .and_then(|e| e.iteration_environment.as_ref())
        {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.initialization().entry_state(),
                function,
            )?;
        }
        let continue_target = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets {
            continue_frame: continue_target,
        });
        self.push_labels(labels, break_target, Some(continue_target));
        let close_scope = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(close_scope);
        if let Some(resource_lifetime) = resource_lifetime {
            // The original iteration record remains attached through disposal.
            // Initializer/GetMethod and body abrupts share this close scope.
            self.compile_complete_mixed_resource_lifetime_then(
                resource_lifetime,
                function,
                |builder, _, function| {
                    builder.compile_complete_mixed_iterator_initialization_and_body(plan, function)
                },
            )?;
        } else {
            self.compile_complete_mixed_iterator_initialization_and_body(plan, function)?;
        }
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.completion().kind().load(function);
        function.instruction(&Instruction::I32Const(
            CompletionKind::Continue.code() as i32
        ));
        function.instruction(&Instruction::I32Eq);
        self.completion().target().load(function);
        function.instruction(&Instruction::I32Const(continue_target.frame as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // A direct Continue can branch over the normal body tail. Capture only
        // after its actual target, as in the original synchronous loop owner.
        pending.copy_from(self.completion(), function);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.write_generator_statement_list_binding(
            plan.value_binding(),
            pending.value(),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if plan
            .lexical_environment()
            .and_then(|e| e.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_save_resumable_environment(function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        match plan.protocol() {
            AsyncGeneratorIteratorProtocolIr::Sync => {
                let iterator = OwnedSyncIterator {
                    record: schema
                        .reserve_gc_local(function)
                        .initialize(record.load(schema, function), function),
                    consumer: SyncIteratorConsumer::ForOf,
                };
                self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;
                iterator.clear(function);
                self.completion().copy_from(&closed, function);
                self.emit_complete_iterator_exit_route(break_target, cleanup, function);
            }
            AsyncGeneratorIteratorProtocolIr::Awaited {
                close_resume_state, ..
            } => {
                self.completion().copy_from(&pending, function);
                self.emit_push_async_pending_completion(function)?;
                self.emit_statement_result(function);
                schema
                    .struct_type::<IteratorRecord>()
                    .field(IteratorRecordSchema::DONE)
                    .write(&record, GcOperand::boolean(true), schema, function);
                self.emit_prepare_for_await_close(
                    &record,
                    is_async,
                    &iterator_value,
                    &awaited,
                    function,
                    |builder, outcome, function| {
                        builder.emit_restore_for_await_close(outcome, false, function)?;
                        builder.emit_complete_iterator_exit_route(break_target, cleanup, function);
                        Ok(())
                    },
                )?;
                self.emit_set_resumable_resume_point(close_resume_state, function)?;
                let await_failure = self.open_frame(ControlFrameKind::Block, function);
                self.throw_handler_stack.push(await_failure);
                self.emit_async_continuation_await(
                    Self::complete_iterator_continuation_owner(plan)?,
                    &awaited,
                    function,
                )?;
                self.emit_return_async_suspension(function)?;
                self.throw_handler_stack.pop();
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
                closed.copy_from(self.completion(), function);
                self.emit_restore_for_await_close(&closed, false, function)?;
                self.emit_complete_iterator_exit_route(break_target, cleanup, function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&pending, function);
        self.emit_set_resumable_resume_point(plan.advance_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_branch_to_target(iteration_loop, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.finally_stack.pop();
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        // No outward dispatch can retain a cache row or initializer result.
        // Suspension returns above retain both the original record and cells.
        schema.array_type::<ForAwaitIteratorTable>().write(
            &table,
            index,
            GcOperand::null(schema),
            schema,
            function,
        );
        incoming.set_undefined(function);
        for binding in [
            plan.head_binding(),
            plan.incoming_binding(),
            plan.value_binding(),
        ] {
            self.write_generator_statement_list_binding(binding, &incoming, function);
        }
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.emit_save_resumable_environment(function)?;
        self.end_generator_statement_list_value();
        self.emit_dispatch_current_completion(function)?;
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        record.clear(function);
        state.clear(function);
        table.clear(function);
        frame.clear(function);
        schema.release_i32_local(done, function);
        schema.release_i32_local(rejected, function);
        schema.release_i32_local(is_async, function);
        closed.clear(function);
        pending.clear(function);
        incoming.clear(function);
        resumed.clear(function);
        awaited.clear(function);
        next_method.clear(function);
        iterator_value.clear(function);
        source.clear(function);
        schema.release_i32_local(index, function);
        Ok(())
    }

    fn emit_complete_iterator_exit_route(
        &mut self,
        break_target: ControlTarget,
        cleanup: ControlTarget,
        function: &mut Function,
    ) {
        self.completion().kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Break.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.completion().target().load(function);
        function.instruction(&Instruction::I32Const(break_target.frame as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_branch_to_target(break_target, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_branch_to_target(cleanup, function);
    }

    fn complete_iterator_continuation_owner(
        plan: &AsyncGeneratorForOfIr,
    ) -> Result<AsyncContinuationOwner, EmitError> {
        match plan.execution() {
            ResumableRegionProtocolIr::Async => Ok(AsyncContinuationOwner::AsyncFunction),
            ResumableRegionProtocolIr::AsyncGenerator => Ok(AsyncContinuationOwner::AsyncGenerator),
            ResumableRegionProtocolIr::Generator => Err(EmitError::unsupported(
                "synchronous generator cannot own an implicit iterator Await",
            )),
        }
    }

    fn compile_complete_mixed_iterator_initialization_and_body(
        &mut self,
        plan: &AsyncGeneratorForOfIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_resumable_state_in_range(
            plan.initialization().entry_state(),
            plan.initialization().end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_resumable_operand_region_in_current_scope(
            plan.initialization().block(),
            plan.initialization().entry_state(),
            function,
            |_, _| Ok(()),
        )?;
        self.emit_set_resumable_resume_point(plan.body().entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.activate_generator_statement_list_value();
        self.emit_restore_generator_statement_list_value(function);
        self.compile_resumable_block_contents(
            plan.body().block(),
            plan.body().entry_state(),
            true,
            function,
        )?;
        self.emit_checkpoint_generator_statement_list_value(function);
        self.deactivate_generator_statement_list_value();
        Ok(())
    }
}
