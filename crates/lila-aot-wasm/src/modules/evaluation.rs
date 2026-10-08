//! Phase-aware iterative DFS retains strong canonical records and full failures.

use super::runtime::GatheredModules;
use super::*;
use crate::builtins::ModuleReactionContinuation;

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_execute_runtime(
        &mut self,
        record: &GcLocal<ModuleRecord>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let state = schema.reserve_i32_local(function);
        let kind = schema.reserve_i32_local(function);
        self.emit_load_module_body_state_strict(record, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleBodyState::NotStarted,
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
                record,
                GcOperand::constant(ModuleBodyState::Executing),
                schema,
                function,
            );
        self.emit_load_module_activation_kind_strict(record, kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleActivationKind::Synchronous,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_private_module_resume(record, result, function)?;
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::BODY_STATE)
            .write(
                record,
                GcOperand::constant(ModuleBodyState::Completed),
                schema,
                function,
            );
        // A private sync activation yields only its instantiation pause. Source
        // completion exposes no public iterator result or Return value.
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        result.initialize(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        let activation = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::ASYNC_ACTIVATION)
                .read(record, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        schema
            .struct_type::<AsyncActivation>()
            .field(AsyncActivationSchema::MODULE_ENTRY_MODE)
            .read(&activation, schema, function)
            .store(state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            AsyncModuleEntryMode::Execute,
        )));
        function.instruction(&Instruction::I32Ne);
        schema
            .struct_type::<AsyncActivation>()
            .field(AsyncActivationSchema::RESUME_POINT)
            .read(&activation, schema, function)
            .store(state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let promise = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<AsyncActivation>()
                .field(AsyncActivationSchema::PROMISE)
                .read(&activation, schema, function)
                .reference(),
            function,
        );
        let realm = self.emit_module_execution_realm_context(record, function);
        self.emit_module_promise_reactions(
            &promise,
            &realm,
            ModuleReactionContinuation::Body(record),
            function,
        )?;
        let body = schema.reserve_completion(function);
        self.emit_saved_async_body_call(&activation, &body, function);
        self.emit_complete_async_entry_invocation(&activation, &body, function)?;
        result.initialize(function);
        body.clear(function);
        self.release_async_execution_realm_context(realm, function);
        promise.clear(function);
        activation.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(kind, function);
        schema.release_i32_local(state, function);
        Ok(())
    }

    fn emit_module_effective_dependencies(
        &mut self,
        record: &GcLocal<ModuleRecord>,
        function: &mut Function,
    ) -> Result<GatheredModules, EmitError> {
        let schema = self.runtime_schema();
        let graph = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::GRAPH)
                .read(record, schema, function)
                .reference(),
            function,
        );
        let modules = self.emit_module_graph_list(&graph, function);
        let count = schema.reserve_i64_local(function);
        let index = schema.reserve_i32_local(function);
        let found = schema.reserve_i32_local(function);
        let phase = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I64Const(0));
        count.store(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let requests = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::REQUESTS)
                .read(record, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let end = self.open_frame(ControlFrameKind::Block, function);
        let again = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        schema
            .array_type::<ModuleRequestTable>()
            .length(&requests, schema, function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        let request = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ModuleRequestTable>()
                .read(&requests, index, schema, function)
                .reference(),
            function,
        );
        self.emit_load_module_request_phase_strict(&request, phase, function);
        let target = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRequest>()
                .field(ModuleRequestSchema::TARGET)
                .read(&request, schema, function)
                .reference(),
            function,
        );
        phase.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleRequestPhase::Evaluation,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_list_contains(&modules, count, &target, found, function);
        found.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_bounded_append(&modules, count, &target, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        let gathered = self.emit_module_gather_call(&target, function)?;
        let next = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        next.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        let gather_again = self.open_frame(ControlFrameKind::Loop, function);
        next.load(function);
        gathered.count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        let selected = self.emit_module_list_entry(&gathered.modules, next, function);
        self.emit_module_list_contains(&modules, count, &selected, found, function);
        found.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_bounded_append(&modules, count, &selected, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        selected.clear(function);
        next.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        next.store(function);
        self.emit_branch_to_target(gather_again, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i64_local(next, function);
        gathered.clear(schema, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        target.clear(function);
        request.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(again, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        let _ = end;
        requests.clear(function);
        schema.release_i32_local(phase, function);
        schema.release_i32_local(found, function);
        schema.release_i32_local(index, function);
        graph.clear(function);
        Ok(GatheredModules { modules, count })
    }

    fn emit_module_enter_evaluation(
        &mut self,
        record: &GcLocal<ModuleRecord>,
        frames: &GcLocal<ModuleDfsStack>,
        depth: I64Local,
        active: &GcLocal<ModuleRegistry>,
        active_count: I64Local,
        dfs_index: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let position = schema.reserve_i32_local(function);
        depth.load(function);
        schema
            .array_type::<ModuleDfsStack>()
            .length(frames, schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .write(
                record,
                GcOperand::constant(ModuleEvaluationState::Evaluating),
                schema,
                function,
            );
        for field in [
            ModuleRecordSchema::DFS_INDEX,
            ModuleRecordSchema::DFS_ANCESTOR,
        ] {
            schema.struct_type::<ModuleRecord>().field(field).write(
                record,
                GcOperand::i64_local(dfs_index),
                schema,
                function,
            );
        }
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::PENDING_ASYNC_DEPENDENCIES)
            .write(record, GcOperand::i64(0), schema, function);
        dfs_index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        dfs_index.store(function);
        let dependencies = self.emit_module_effective_dependencies(record, function)?;
        self.emit_module_bounded_append(active, active_count, record, function);
        let frame = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ModuleDfsFrame>().construct(
                (
                    GcOperand::reference(record, schema),
                    GcOperand::nullable_reference(&dependencies.modules, schema),
                    GcOperand::i64_local(dependencies.count),
                    GcOperand::i64(0),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        depth.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        schema.array_type::<ModuleDfsStack>().write(
            frames,
            position,
            GcOperand::nullable_reference(&frame, schema),
            schema,
            function,
        );
        depth.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        depth.store(function);
        frame.clear(function);
        dependencies.clear(schema, function);
        schema.release_i32_local(position, function);
        Ok(())
    }

    fn emit_module_register_parent(
        &self,
        child: &GcLocal<ModuleRecord>,
        parent: &GcLocal<ModuleRecord>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let graph = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::GRAPH)
                .read(child, schema, function)
                .reference(),
            function,
        );
        let count = schema.reserve_i64_local(function);
        let limit = schema.reserve_i64_local(function);
        schema
            .struct_type::<ModuleGraph>()
            .field(ModuleGraphSchema::PARENT_COUNT)
            .read(&graph, schema, function)
            .store_i64(count, function);
        schema
            .struct_type::<ModuleGraph>()
            .field(ModuleGraphSchema::PARENT_BUDGET)
            .read(&graph, schema, function)
            .store_i64(limit, function);
        count.load(function);
        limit.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        count.store(function);
        schema
            .struct_type::<ModuleGraph>()
            .field(ModuleGraphSchema::PARENT_COUNT)
            .write(&graph, GcOperand::i64_local(count), schema, function);
        let occurrence = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ModuleParent>().construct(
                (
                    GcOperand::reference(parent, schema),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        let tail = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::ASYNC_PARENTS_TAIL)
                .read(child, schema, function)
                .reference(),
            function,
        );
        tail.load(schema, function).is_null(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_PARENTS_HEAD)
            .write(
                child,
                GcOperand::nullable_reference(&occurrence, schema),
                schema,
                function,
            );
        function.instruction(&Instruction::Else);
        let selected = schema.reserve_gc_local(function).initialize(
            tail.load(schema, function).require_non_null(function),
            function,
        );
        schema
            .struct_type::<ModuleParent>()
            .field(ModuleParentSchema::NEXT)
            .write(
                &selected,
                GcOperand::nullable_reference(&occurrence, schema),
                schema,
                function,
            );
        selected.clear(function);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_PARENTS_TAIL)
            .write(
                child,
                GcOperand::nullable_reference(&occurrence, schema),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::PENDING_ASYNC_DEPENDENCIES)
            .read(parent, schema, function)
            .store_i64(count, function);
        count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        count.store(function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::PENDING_ASYNC_DEPENDENCIES)
            .write(parent, GcOperand::i64_local(count), schema, function);
        tail.clear(function);
        occurrence.clear(function);
        schema.release_i64_local(limit, function);
        schema.release_i64_local(count, function);
        graph.clear(function);
    }

    fn emit_module_read_cached_error(
        &self,
        record: &GcLocal<ModuleRecord>,
        output: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::ERROR)
                .read(record, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, output, schema, function);
        stored.clear(function);
    }

    fn emit_module_reconcile_returned_child(
        &mut self,
        record: &GcLocal<ModuleRecord>,
        child: &GcLocal<ModuleRecord>,
        error: &ValueLocals,
        failed: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let state = schema.reserve_i32_local(function);
        let ancestor = schema.reserve_i64_local(function);
        let child_ancestor = schema.reserve_i64_local(function);
        let order = schema.reserve_i64_local(function);
        self.emit_load_module_completion_strict(child, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_read_cached_error(child, error, function);
        function.instruction(&Instruction::I32Const(1));
        failed.store(function);
        function.instruction(&Instruction::Else);
        self.emit_load_module_state_strict(child, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::Evaluating,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::DFS_ANCESTOR)
            .read(record, schema, function)
            .store_i64(ancestor, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::DFS_ANCESTOR)
            .read(child, schema, function)
            .store_i64(child_ancestor, function);
        ancestor.load(function);
        child_ancestor.load(function);
        ancestor.load(function);
        child_ancestor.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::Select);
        ancestor.store(function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::DFS_ANCESTOR)
            .write(record, GcOperand::i64_local(ancestor), schema, function);
        // In-cycle ancestors retain their own async order. Finished children
        // instead expose the canonical SCC root before dependency registration.
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .read(child, schema, function)
            .store_i64(order, function);
        order.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_register_parent(child, record, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        let cycle = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::CYCLE_ROOT)
                .read(child, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        self.emit_load_module_completion_strict(&cycle, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationCompletion::Throw,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_read_cached_error(&cycle, error, function);
        function.instruction(&Instruction::I32Const(1));
        failed.store(function);
        function.instruction(&Instruction::Else);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .read(&cycle, schema, function)
            .store_i64(order, function);
        order.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_register_parent(&cycle, record, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        cycle.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(order, function);
        schema.release_i64_local(child_ancestor, function);
        schema.release_i64_local(ancestor, function);
        schema.release_i32_local(state, function);
        Ok(())
    }

    pub(super) fn emit_module_evaluation_runtime(
        &mut self,
        input: &GcLocal<ModuleRecord>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let root = schema
            .reserve_gc_local(function)
            .initialize(input.load(schema, function), function);
        let state = schema.reserve_i32_local(function);
        self.emit_load_module_state_strict(&root, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::EvaluatingAsync,
        )));
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        let cycle = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::CYCLE_ROOT)
                .read(&root, schema, function)
                .reference(),
            function,
        );
        cycle.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        root.replace(
            cycle.load(schema, function).require_non_null(function),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        cycle.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let promise = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::EVALUATION_PROMISE)
                .read(&root, schema, function)
                .reference(),
            function,
        );
        promise.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_state_strict(&root, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::Evaluating,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        // A private reentry may reuse a capability, never restart an active body.
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let realm = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::REALM)
                .read(&root, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let allocated = self.emit_alloc_promise_in_realm(&realm, function)?;
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::EVALUATION_PROMISE)
            .write(
                &root,
                GcOperand::nullable_reference(&allocated, schema),
                schema,
                function,
            );
        promise.replace(allocated.load(schema, function).nullable(), function);
        allocated.clear(function);
        realm.clear(function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::Linked,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let graph = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleRecord>()
                .field(ModuleRecordSchema::GRAPH)
                .read(&root, schema, function)
                .reference(),
            function,
        );
        let graph_modules = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleGraph>()
                .field(ModuleGraphSchema::MODULES)
                .read(&graph, schema, function)
                .reference(),
            function,
        );
        let capacity = schema.reserve_i32_local(function);
        schema
            .array_type::<ModuleRegistry>()
            .length(&graph_modules, schema, function);
        capacity.store(function);
        let frames = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ModuleDfsStack>().filled(
                GcOperand::null(schema),
                capacity,
                function,
            ),
            function,
        );
        let active = self.emit_module_graph_list(&graph, function);
        let depth = schema.reserve_i64_local(function);
        let active_count = schema.reserve_i64_local(function);
        let dfs_index = schema.reserve_i64_local(function);
        let index = schema.reserve_i64_local(function);
        let count = schema.reserve_i64_local(function);
        let next = schema.reserve_i64_local(function);
        let ancestor = schema.reserve_i64_local(function);
        let order = schema.reserve_i64_local(function);
        let pending = schema.reserve_i64_local(function);
        let position = schema.reserve_i32_local(function);
        let kind = schema.reserve_i32_local(function);
        let failed = schema.reserve_i32_local(function);
        let advanced = schema.reserve_i32_local(function);
        let error = schema.reserve_value_local(function);
        error.set_undefined(function);
        let execution = schema.reserve_completion(function);
        for local in [depth, active_count, dfs_index] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        function.instruction(&Instruction::I32Const(0));
        failed.store(function);
        self.emit_module_enter_evaluation(
            &root,
            &frames,
            depth,
            &active,
            active_count,
            dfs_index,
            function,
        )?;
        self.open_frame(ControlFrameKind::Block, function);
        let again = self.open_frame(ControlFrameKind::Loop, function);
        depth.load(function);
        function.instruction(&Instruction::I64Eqz);
        failed.load(function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        depth.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        let frame = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ModuleDfsStack>()
                .read(&frames, position, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleDfsFrame>()
                .field(ModuleDfsFrameSchema::MODULE)
                .read(&frame, schema, function)
                .reference(),
            function,
        );
        let returned = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleDfsFrame>()
                .field(ModuleDfsFrameSchema::RETURNED_CHILD)
                .read(&frame, schema, function)
                .reference(),
            function,
        );
        returned.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let child = schema.reserve_gc_local(function).initialize(
            returned.load(schema, function).require_non_null(function),
            function,
        );
        schema
            .struct_type::<ModuleDfsFrame>()
            .field(ModuleDfsFrameSchema::RETURNED_CHILD)
            .write(&frame, GcOperand::null(schema), schema, function);
        self.emit_module_reconcile_returned_child(&record, &child, &error, failed, function)?;
        child.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        returned.clear(function);
        function.instruction(&Instruction::I32Const(0));
        advanced.store(function);
        failed.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<ModuleDfsFrame>()
            .field(ModuleDfsFrameSchema::NEXT_DEPENDENCY)
            .read(&frame, schema, function)
            .store_i64(next, function);
        schema
            .struct_type::<ModuleDfsFrame>()
            .field(ModuleDfsFrameSchema::DEPENDENCY_COUNT)
            .read(&frame, schema, function)
            .store_i64(count, function);
        next.load(function);
        count.load(function);
        function.instruction(&Instruction::I64LtU);
        self.open_frame(ControlFrameKind::If, function);
        let dependencies = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ModuleDfsFrame>()
                .field(ModuleDfsFrameSchema::DEPENDENCIES)
                .read(&frame, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let child = self.emit_module_list_entry(&dependencies, next, function);
        next.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        next.store(function);
        schema
            .struct_type::<ModuleDfsFrame>()
            .field(ModuleDfsFrameSchema::NEXT_DEPENDENCY)
            .write(&frame, GcOperand::i64_local(next), schema, function);
        schema
            .struct_type::<ModuleDfsFrame>()
            .field(ModuleDfsFrameSchema::RETURNED_CHILD)
            .write(
                &frame,
                GcOperand::nullable_reference(&child, schema),
                schema,
                function,
            );
        self.emit_load_module_state_strict(&child, state, function);
        state.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleEvaluationState::Linked,
        )));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_module_enter_evaluation(
            &child,
            &frames,
            depth,
            &active,
            active_count,
            dfs_index,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        child.clear(function);
        dependencies.clear(function);
        function.instruction(&Instruction::I32Const(1));
        advanced.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        failed.load(function);
        advanced.load(function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_module_activation_kind_strict(&record, kind, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::PENDING_ASYNC_DEPENDENCIES)
            .read(&record, schema, function)
            .store_i64(pending, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            ModuleActivationKind::Async,
        )));
        function.instruction(&Instruction::I32Eq);
        pending.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<ModuleGraph>()
            .field(ModuleGraphSchema::NEXT_ASYNC_ORDER)
            .read(&graph, schema, function)
            .store_i64(order, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .write(&record, GcOperand::i64_local(order), schema, function);
        order.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        order.store(function);
        schema
            .struct_type::<ModuleGraph>()
            .field(ModuleGraphSchema::NEXT_ASYNC_ORDER)
            .write(&graph, GcOperand::i64_local(order), schema, function);
        pending.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(ModuleExecuteArguments::new(&record), base, function)
            .store(&execution, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(ModuleExecuteArguments::new(&record), base, function)
            .store(&execution, function);
        execution.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        error.copy_from(execution.value(), function);
        function.instruction(&Instruction::I32Const(1));
        failed.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        failed.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::DFS_INDEX)
            .read(&record, schema, function)
            .store_i64(index, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::DFS_ANCESTOR)
            .read(&record, schema, function)
            .store_i64(ancestor, function);
        index.load(function);
        ancestor.load(function);
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let scc = self.open_frame(ControlFrameKind::Loop, function);
        active_count.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        active_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        active_count.store(function);
        let child = self.emit_module_list_entry(&active, active_count, function);
        active_count.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        schema.array_type::<ModuleRegistry>().write(
            &active,
            position,
            GcOperand::null(schema),
            schema,
            function,
        );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::CYCLE_ROOT)
            .write(
                &child,
                GcOperand::nullable_reference(&record, schema),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ASYNC_ORDER)
            .read(&child, schema, function)
            .store_i64(order, function);
        order.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .write(
                &child,
                GcOperand::constant(ModuleEvaluationState::Evaluated),
                schema,
                function,
            );
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::COMPLETION)
            .write(
                &child,
                GcOperand::constant(ModuleEvaluationCompletion::Normal),
                schema,
                function,
            );
        function.instruction(&Instruction::Else);
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .write(
                &child,
                GcOperand::constant(ModuleEvaluationState::EvaluatingAsync),
                schema,
                function,
            );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        child.load(schema, function);
        record.load(schema, function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::I32Eqz);
        advanced.store(function);
        child.clear(function);
        advanced.load(function);
        self.emit_branch_if_to_target(scc, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        depth.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        depth.store(function);
        depth.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        schema.array_type::<ModuleDfsStack>().write(
            &frames,
            position,
            GcOperand::null(schema),
            schema,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        record.clear(function);
        frame.clear(function);
        self.emit_branch_to_target(again, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        failed.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.open_frame(ControlFrameKind::Block, function);
        let unwind = self.open_frame(ControlFrameKind::Loop, function);
        active_count.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        active_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        active_count.store(function);
        let record = self.emit_module_list_entry(&active, active_count, function);
        self.emit_module_cache_throw(&record, &error, function);
        self.emit_module_settle_evaluation_if_terminal(&record, function)?;
        record.clear(function);
        self.emit_branch_to_target(unwind, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        execution.clear(function);
        error.clear(function);
        schema.release_i32_local(advanced, function);
        schema.release_i32_local(failed, function);
        schema.release_i32_local(kind, function);
        schema.release_i32_local(position, function);
        for local in [
            pending,
            order,
            ancestor,
            next,
            count,
            index,
            dfs_index,
            active_count,
            depth,
        ] {
            schema.release_i64_local(local, function);
        }
        active.clear(function);
        frames.clear(function);
        schema.release_i32_local(capacity, function);
        graph_modules.clear(function);
        graph.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_module_settle_evaluation_if_terminal(&root, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let selected = schema.reserve_gc_local(function).initialize(
            promise.load(schema, function).require_non_null(function),
            function,
        );
        result.initialize(function);
        result.value().set_reference(&selected, schema, function);
        selected.clear(function);
        promise.clear(function);
        schema.release_i32_local(state, function);
        root.clear(function);
        Ok(())
    }
}
