//! Intrinsic module Promise completion, async parents and deferred-import joins.

use super::*;
use crate::builtins::ModuleReactionContinuation;

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_cache_throw(
        &self,
        record: &GcLocal<ModuleRecord>,
        error: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(error, function),
            function,
        );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ERROR)
            .write(
                record,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::COMPLETION)
            .write(
                record,
                GcOperand::constant(ModuleEvaluationCompletion::Throw),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .write(
                record,
                GcOperand::constant(ModuleEvaluationState::Evaluated),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .write(record, GcOperand::i64(0), schema, function);
        stored.clear(function);
    }

    pub(super) fn emit_module_settle_evaluation_if_terminal(
        &mut self,
        module: &GcLocal<ModuleRecord>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let state = schema.reserve_i32_local(function);
        let promise = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::EVALUATION_PROMISE)
                .read(module, schema, function)
                .reference(),
            function,
        );
        promise.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_state_strict(module, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::Evaluated,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let selected = schema.reserve_gc_local(function).initialize(
            promise.load(schema, function).require_non_null(function),
            function,
        );
        let value = schema.reserve_value_local(function);
        self.emit_load_module_completion_strict(module, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::ERROR)
                .read(module, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &value, schema, function);
        self.emit_settle_promise_record(&selected, PromiseSettlement::Reject, &value, function)?;
        stored.clear(function);
        function.instruction(&Instruction::Else);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Normal,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.set_undefined(function);
        self.emit_settle_promise_record(&selected, PromiseSettlement::Fulfill, &value, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        selected.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        promise.clear(function);
        schema.release_i32_local(state, function);
        Ok(())
    }

    /// The job that resumes a module body. Only a program with module bodies
    /// can have queued one, so a program without them never installs the hook.
    pub(crate) fn emit_run_module_body_reaction(
        &mut self,
        module: &GcLocal<ModuleRecord>,
        rejected: I32Local,
        argument: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.emit_program_hook_dispatch(
            crate::runtime_helpers::ProgramHook::ModuleBodyReaction,
            |_, hook, function| {
                schema.call_hook(
                    hook,
                    ModuleBodyReactionArguments::new(module, rejected, argument),
                    function,
                );
                Ok(())
            },
            |_, function| {
                function.instruction(&Instruction::Unreachable);
                Ok(())
            },
            function,
        )?;
        self.completion().initialize(function);
        Ok(())
    }

    pub(crate) fn compile_module_body_reaction_hook(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::ModuleBodyReaction);
        let parameters = self.helper_parameters::<ModuleBodyReactionParameters>(&mut function);
        self.push_scope();
        self.emit_module_body_reaction_runtime(
            &parameters.module,
            parameters.rejected,
            &parameters.argument,
            &mut function,
        )?;
        self.pop_scope();
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_module_body_reaction_runtime(
        &mut self,
        module: &GcLocal<ModuleRecord>,
        rejected: I32Local,
        argument: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let state = schema.reserve_i32_local(function);
        let result = schema.reserve_completion(function);
        self.emit_load_module_body_state_strict(module, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleBodyState::Executing,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::BODY_STATE)
            .write(
                module,
                GcOperand::constant(ModuleBodyState::Completed),
                schema,
                function,
            );
        rejected.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(
                ModuleRejectedArguments::new(module, argument),
                base,
                function,
            )
            .store(&result, function);
        function.instruction(&Instruction::Else);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(
                ModuleFulfilledArguments::new(module, argument),
                base,
                function,
            )
            .store(&result, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.clear(function);
        schema.release_i32_local(state, function);
        self.completion().initialize(function);
        Ok(())
    }

    pub(super) fn emit_module_rejected_runtime(
        &mut self,
        initial: &GcLocal<ModuleRecord>,
        reason: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let graph = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::GRAPH)
                .read(initial, schema, function)
                .reference(),
            function,
        );
        let queue = self.emit_module_graph_list(&graph, function);
        let count = schema.reserve_i64_local(function);
        let index = schema.reserve_i64_local(function);
        let state = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I64Const(0));
        count.store(function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        self.emit_load_module_completion_strict(initial, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_state_strict(initial, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::EvaluatingAsync,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_module_cache_throw(initial, reason, function);
        self.emit_module_bounded_append(&queue, count, initial, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.open_frame(ControlFrameKind::Block, function);
        let again = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        let module = self.emit_module_list_entry(&queue, index, function);
        self.emit_module_settle_evaluation_if_terminal(&module, function)?;
        let occurrence = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::ASYNC_PARENTS_HEAD)
                .read(&module, schema, function)
                .reference(),
            function,
        );
        self.open_frame(ControlFrameKind::Block, function);
        let parents = self.open_frame(ControlFrameKind::Loop, function);
        occurrence.load(schema, function).is_null(function);
        function.instruction(&Instruction::BrIf(1));
        let selected = schema.reserve_gc_local(function).initialize(
            occurrence.load(schema, function).require_non_null(function),
            function,
        );
        let parent = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleParent>()
                .field(ModuleParentSchema::MODULE)
                .read(&selected, schema, function)
                .reference(),
            function,
        );
        self.emit_load_module_completion_strict(&parent, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_state_strict(&parent, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::EvaluatingAsync,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_module_cache_throw(&parent, reason, function);
        self.emit_module_bounded_append(&queue, count, &parent, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        occurrence.replace(
            schema
                .struct_type::<ModuleParent>()
                .field(ModuleParentSchema::NEXT)
                .read(&selected, schema, function)
                .reference(),
            function,
        );
        parent.clear(function);
        selected.clear(function);
        self.emit_branch_to_target(parents, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        occurrence.clear(function);
        module.clear(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        self.emit_branch_to_target(again, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        result.initialize(function);
        schema.release_i32_local(state, function);
        schema.release_i64_local(index, function);
        schema.release_i64_local(count, function);
        queue.clear(function);
        graph.clear(function);
        Ok(())
    }

    pub(crate) fn emit_run_module_join_reaction(
        &mut self,
        join: &GcLocal<ModuleJoin>,
        rejected: I32Local,
        argument: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let remaining = schema.reserve_i64_local(function);
        schema
            .struct_type::<ModuleJoin>()
            .field(ModuleJoinSchema::REMAINING)
            .read(join, schema, function)
            .store_i64(remaining, function);
        remaining.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        remaining.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        remaining.store(function);
        schema
            .struct_type::<ModuleJoin>()
            .field(ModuleJoinSchema::REMAINING)
            .write(join, GcOperand::i64_local(remaining), schema, function);
        let promise = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleJoin>()
                .field(ModuleJoinSchema::PROMISE)
                .read(join, schema, function)
                .reference(),
            function,
        );
        rejected.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_settle_promise_record(&promise, PromiseSettlement::Reject, argument, function)?;
        function.instruction(&Instruction::Else);
        remaining.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        self.emit_settle_promise_record(
            &promise,
            PromiseSettlement::Fulfill,
            &undefined,
            function,
        )?;
        undefined.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        promise.clear(function);
        schema.release_i64_local(remaining, function);
        self.completion().initialize(function);
        Ok(())
    }

    pub(super) fn emit_module_deferred_import_runtime(
        &mut self,
        module: &GcLocal<ModuleRecord>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let dependencies = self.emit_module_gather_call(module, function)?;
        let count = schema.reserve_i32_local(function);
        dependencies.count.load(function);
        function.instruction(&Instruction::I32WrapI64);
        count.store(function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        let empty = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&undefined, function),
            function,
        );
        // This is a compiler List of exact returned promises, not a JavaScript
        // Array and not a second module-record registry.
        let promises = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ValueArray>().filled(
                GcOperand::reference(&empty, schema),
                count,
                function,
            ),
            function,
        );
        let realm = self.emit_module_execution_realm_context(module, function);
        let promise = self.emit_alloc_promise_in_realm(realm.realm(), function)?;
        let join = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ModuleJoin>().construct(
                (
                    GcOperand::reference(&promise, schema),
                    GcOperand::i64_local(dependencies.count),
                    GcOperand::reference(realm.realm(), schema),
                ),
                function,
            ),
            function,
        );
        let index = schema.reserve_i64_local(function);
        let position = schema.reserve_i32_local(function);
        let evaluation = schema.reserve_completion(function);
        let failed = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        function.instruction(&Instruction::I32Const(0));
        failed.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        let evaluate = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        dependencies.count.load(function);
        function.instruction(&Instruction::I64GeU);
        failed.load(function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        let target = self.emit_module_list_entry(&dependencies.modules, index, function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(ModuleEvaluateArguments::new(&target), base, function)
            .store(&evaluation, function);
        evaluation.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        result.copy_from(&evaluation, function);
        function.instruction(&Instruction::I32Const(1));
        failed.store(function);
        function.instruction(&Instruction::Else);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(evaluation.value(), function),
            function,
        );
        index.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        schema.array_type::<ValueArray>().write(
            &promises,
            position,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        target.clear(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        self.emit_branch_to_target(evaluate, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // Complete every selected Evaluate before registering the first reaction.
        // Settled promises enqueue jobs; interleaving changes first-Await order.
        failed.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        let register = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        dependencies.count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        index.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ValueArray>()
                .read(&promises, position, schema, function)
                .reference(),
            function,
        );
        schema.struct_type::<StoredValue>().read_into(
            &stored,
            evaluation.value(),
            schema,
            function,
        );
        let selected = schema.reserve_gc_local(function).initialize(
            evaluation
                .value()
                .cast_reference::<PromiseObject>(schema, function),
            function,
        );
        self.emit_module_promise_reactions(
            &selected,
            &realm,
            ModuleReactionContinuation::Join(&join),
            function,
        )?;
        selected.clear(function);
        stored.clear(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        self.emit_branch_to_target(register, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        dependencies.count.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_settle_promise_record(
            &promise,
            PromiseSettlement::Fulfill,
            &undefined,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.initialize(function);
        result.value().set_reference(&promise, schema, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(failed, function);
        evaluation.clear(function);
        schema.release_i32_local(position, function);
        schema.release_i64_local(index, function);
        join.clear(function);
        promise.clear(function);
        self.release_async_execution_realm_context(realm, function);
        promises.clear(function);
        empty.clear(function);
        undefined.clear(function);
        schema.release_i32_local(count, function);
        dependencies.clear(schema, function);
        Ok(())
    }

    pub(super) fn emit_module_fulfilled_runtime(
        &mut self,
        initial: &GcLocal<ModuleRecord>,
        _value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let state = schema.reserve_i32_local(function);
        self.emit_load_module_completion_strict(initial, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_state_strict(initial, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::EvaluatingAsync,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .write(
                initial,
                GcOperand::constant(ModuleEvaluationState::Evaluated),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::COMPLETION)
            .write(
                initial,
                GcOperand::constant(ModuleEvaluationCompletion::Normal),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .write(initial, GcOperand::i64(0), schema, function);
        self.emit_module_settle_evaluation_if_terminal(initial, function)?;
        let graph = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::GRAPH)
                .read(initial, schema, function)
                .reference(),
            function,
        );
        let queue = self.emit_module_graph_list(&graph, function);
        let executions = self.emit_module_graph_list(&graph, function);
        let queue_count = schema.reserve_i64_local(function);
        let queue_index = schema.reserve_i64_local(function);
        let execution_count = schema.reserve_i64_local(function);
        let execution_index = schema.reserve_i64_local(function);
        let pending = schema.reserve_i64_local(function);
        let order = schema.reserve_i64_local(function);
        let smallest = schema.reserve_i64_local(function);
        let found = schema.reserve_i32_local(function);
        let kind = schema.reserve_i32_local(function);
        let selected_index = schema.reserve_i32_local(function);
        let position = schema.reserve_i32_local(function);
        let selected = schema
            .reserve_gc_local::<ModuleRecord, Nullable>(function)
            .initialize_null(schema, function);
        let execution = schema.reserve_completion(function);
        for local in [queue_count, queue_index, execution_count] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        self.emit_module_bounded_append(&queue, queue_count, initial, function);
        self.open_frame(ControlFrameKind::Block, function);
        let gather = self.open_frame(ControlFrameKind::Loop, function);
        queue_index.load(function);
        queue_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        let module = self.emit_module_list_entry(&queue, queue_index, function);
        let occurrence = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::ASYNC_PARENTS_HEAD)
                .read(&module, schema, function)
                .reference(),
            function,
        );
        self.open_frame(ControlFrameKind::Block, function);
        let parents = self.open_frame(ControlFrameKind::Loop, function);
        occurrence.load(schema, function).is_null(function);
        function.instruction(&Instruction::BrIf(1));
        let entry = schema.reserve_gc_local(function).initialize(
            occurrence.load(schema, function).require_non_null(function),
            function,
        );
        let parent = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleParent>()
                .field(ModuleParentSchema::MODULE)
                .read(&entry, schema, function)
                .reference(),
            function,
        );
        self.emit_module_list_contains(&executions, execution_count, &parent, found, function);
        self.emit_load_module_completion_strict(&parent, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        // A later sibling can fail before this parent's SCC closes. That cached
        // failure is final even while its CycleRoot is still absent.
        let cycle = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::CYCLE_ROOT)
                .read(&parent, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        self.emit_load_module_completion_strict(&cycle, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Ne);
        found.load(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_state_strict(&parent, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::EvaluatingAsync,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::PENDING_ASYNC_DEPENDENCIES)
            .read(&parent, schema, function)
            .store_i64(pending, function);
        pending.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        pending.store(function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::PENDING_ASYNC_DEPENDENCIES)
            .write(&parent, GcOperand::i64_local(pending), schema, function);
        pending.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_bounded_append(&executions, execution_count, &parent, function);
        self.emit_load_module_activation_kind_strict(&parent, kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleActivationKind::Synchronous,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_bounded_append(&queue, queue_count, &parent, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        cycle.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        occurrence.replace(
            schema
                .struct_type::<ModuleParent>()
                .field(ModuleParentSchema::NEXT)
                .read(&entry, schema, function)
                .reference(),
            function,
        );
        parent.clear(function);
        entry.clear(function);
        self.emit_branch_to_target(parents, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        occurrence.clear(function);
        module.clear(function);
        queue_index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        queue_index.store(function);
        self.emit_branch_to_target(gather, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // Gather all eligible synchronous ancestors before invoking any body,
        // then select their actual global async-evaluation order.
        self.open_frame(ControlFrameKind::Block, function);
        let execute = self.open_frame(ControlFrameKind::Loop, function);
        selected.set_null(schema, function);
        function.instruction(&Instruction::I64Const(0));
        execution_index.store(function);
        function.instruction(&Instruction::I64Const(-1));
        smallest.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        let choose = self.open_frame(ControlFrameKind::Loop, function);
        execution_index.load(function);
        execution_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        execution_index.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        let candidate = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ModuleRegistry>()
                .read(&executions, position, schema, function)
                .reference(),
            function,
        );
        candidate.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let parent = schema.reserve_gc_local(function).initialize(
            candidate.load(schema, function).require_non_null(function),
            function,
        );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .read(&parent, schema, function)
            .store_i64(order, function);
        order.load(function);
        smallest.load(function);
        function.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, function);
        selected.replace(parent.load(schema, function).nullable(), function);
        order.load(function);
        smallest.store(function);
        position.load(function);
        selected_index.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        parent.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        candidate.clear(function);
        execution_index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        execution_index.store(function);
        self.emit_branch_to_target(choose, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        selected.load(schema, function).is_null(function);
        function.instruction(&Instruction::BrIf(1));
        schema.array_type::<ModuleRegistry>().write(
            &executions,
            selected_index,
            GcOperand::null(schema),
            schema,
            function,
        );
        let parent = schema.reserve_gc_local(function).initialize(
            selected.load(schema, function).require_non_null(function),
            function,
        );
        self.emit_load_module_completion_strict(&parent, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_activation_kind_strict(&parent, kind, function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(ModuleExecuteArguments::new(&parent), base, function)
            .store(&execution, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleActivationKind::Synchronous,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        execution.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(
                ModuleRejectedArguments::new(&parent, execution.value()),
                base,
                function,
            )
            .store(&execution, function);
        function.instruction(&Instruction::Else);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .write(
                &parent,
                GcOperand::constant(ModuleEvaluationState::Evaluated),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::COMPLETION)
            .write(
                &parent,
                GcOperand::constant(ModuleEvaluationCompletion::Normal),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .write(&parent, GcOperand::i64(0), schema, function);
        self.emit_module_settle_evaluation_if_terminal(&parent, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        parent.clear(function);
        self.emit_branch_to_target(execute, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        execution.clear(function);
        selected.clear(function);
        schema.release_i32_local(position, function);
        schema.release_i32_local(selected_index, function);
        schema.release_i32_local(kind, function);
        schema.release_i32_local(found, function);
        for local in [
            smallest,
            order,
            pending,
            execution_index,
            execution_count,
            queue_index,
            queue_count,
        ] {
            schema.release_i64_local(local, function);
        }
        executions.clear(function);
        queue.clear(function);
        graph.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.initialize(function);
        schema.release_i32_local(state, function);
        Ok(())
    }
}
