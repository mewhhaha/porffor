//! Private module records own canonical resumable activations and environment cells.

use super::*;
use crate::objects::TaggedLocals;
use lila_ir::{
    DeferredModuleEvaluationIr, ModuleActivationKindIr, ModuleCellIr, ModuleEvaluationIr,
    ModuleExecutionGraphIr, ModuleImportBindingIr, ModuleNamespaceModeIr, ModuleRequestPhaseIr,
};

pub(crate) fn module_execution_record_count(script: &ScriptIr) -> u32 {
    script
        .executable_script_bodies()
        .flat_map(|body| &body.statements)
        .filter_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ModuleExecutionGraph(graph),
                ..
            }) => Some(graph.record_count()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

impl FunctionBuilder<'_> {
    pub(super) fn module_record_local(
        &mut self,
        module: u32,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        if module >= self.functions.module_execution_record_count() {
            return Err(EmitError::unsupported(
                "private module operation requires its validated execution graph",
            ));
        }
        let record = self.reserve_temp_local();
        let index = GLOBAL_INDEX_REGISTRY.len() as u32
            + self.strings.template_objects.len() as u32
            + self.functions.module_unit_guard_count()
            + module;
        function.instruction(&Instruction::GlobalGet(index));
        function.instruction(&Instruction::LocalSet(record));
        Ok(record)
    }

    fn module_namespace_cell_offset(mode: ModuleNamespaceModeIr) -> u64 {
        match mode {
            ModuleNamespaceModeIr::Eager => MODULE_NAMESPACE_CELL_OFFSET,
            ModuleNamespaceModeIr::Deferred => MODULE_DEFERRED_NAMESPACE_CELL_OFFSET,
        }
    }

    fn module_cell_address(
        &mut self,
        target: &ModuleCellIr,
        function: &mut Function,
    ) -> Result<u32, EmitError> {
        let (module, offset) = match target {
            ModuleCellIr::Binding { module, slot } => {
                (*module, ENV_SLOT_BASE_OFFSET + *slot as u64 * ENV_SLOT_SIZE)
            }
            ModuleCellIr::Namespace { module, mode } => {
                (*module, Self::module_namespace_cell_offset(*mode))
            }
        };
        let cell = self.module_record_local(module, function)?;
        if matches!(target, ModuleCellIr::Binding { .. }) {
            self.load_i64_to_local_from_offset(cell, MODULE_ENVIRONMENT_OFFSET, cell, function);
        }
        function.instruction(&Instruction::LocalGet(cell));
        function.instruction(&Instruction::I64Const(offset as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cell));
        Ok(cell)
    }

    pub(crate) fn emit_module_import_binding(
        &mut self,
        import: &ModuleImportBindingIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let slot = self
            .owned_env_slot(&import.name)
            .ok_or_else(|| EmitError::unsupported("module import must own an environment slot"))?;
        let cell = self.module_cell_address(&import.target, function)?;
        self.emit_initialize_indirect_binding(slot, cell, function);
        self.release_temp_local(cell);
        Ok(())
    }

    pub(crate) fn emit_module_binding_read(
        &mut self,
        target: &ModuleCellIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cell = self.module_cell_address(target, function)?;
        self.emit_read_environment_cell(cell, payload, tag, function);
        self.release_temp_local(cell);
        function.instruction(&Instruction::LocalGet(tag));
        function.instruction(&Instruction::I64Const(ENV_SLOT_UNINITIALIZED_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "ReferenceError",
            "module binding accessed before initialization",
            payload,
            tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)
    }

    pub(crate) fn emit_module_namespace_publish(
        &mut self,
        module: u32,
        mode: ModuleNamespaceModeIr,
        namespace: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.compile_expr_to_locals(namespace, value.payload, value.tag, function)?;
        let record = self.module_record_local(module, function)?;
        let offset = Self::module_namespace_cell_offset(mode);
        self.store_i64_local_at_offset(record, offset + ENV_SLOT_TAG_OFFSET, value.tag, function);
        self.store_i64_local_at_offset(
            record,
            offset + ENV_SLOT_PAYLOAD_OFFSET,
            value.payload,
            function,
        );
        self.release_temp_local(record);
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        function.instruction(&Instruction::I64Const(0));
        Ok(())
    }

    pub(super) fn emit_private_module_resume(
        &mut self,
        record: u32,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let generator = self.reserve_temp_local();
        let generator_tag = self.reserve_temp_local();
        let callee = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(record, MODULE_ACTIVATION_OFFSET, generator, function);
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(generator_tag));
        self.compile_expr_to_locals(
            &TypedExpr::from_info(
                ValueInfo::new(ValueKind::Function),
                ExprIr::FunctionValue(StandardBuiltinId::GeneratorPrototypeNext.function_id()),
            ),
            callee.payload,
            callee.tag,
            function,
        )?;
        self.emit_function_handle_call_without_throw_propagation(
            callee.payload,
            callee.tag,
            Some((generator, Some(generator_tag))),
            &[],
            result.payload,
            result.tag,
            function,
        )?;
        self.release_temp_local(callee.tag);
        self.release_temp_local(callee.payload);
        self.release_temp_local(generator_tag);
        self.release_temp_local(generator);
        Ok(())
    }

    pub(crate) fn emit_module_execution_graph(
        &mut self,
        graph: &ModuleExecutionGraphIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for activation in graph.activations() {
            let expected = match activation.kind() {
                ModuleActivationKindIr::Synchronous => FunctionProtocolIr::ModuleActivation,
                ModuleActivationKindIr::Async => FunctionProtocolIr::AsyncModuleActivation,
            };
            if !self
                .functions
                .get(activation.function())
                .is_some_and(|owner| owner.protocol == expected)
            {
                return Err(EmitError::unsupported(
                    "module activation requires its matching private function protocol",
                ));
            }
        }
        let graph_record = self.reserve_temp_local();
        let records = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let requests = self.reserve_temp_local();
        let env = self.reserve_temp_local();
        let table = self.reserve_temp_local();
        let zero = self.reserve_temp_local();
        let undefined = self.reserve_temp_local();
        let promise = self.reserve_temp_local();
        let promise_record = self.reserve_temp_local();
        let callee = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let result = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined));
        self.emit_heap_alloc_const(MODULE_GRAPH_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(graph_record));
        self.emit_heap_alloc_const(u64::from(graph.record_count()) * 8, function)?;
        function.instruction(&Instruction::LocalSet(records));
        self.store_i64_const_at_offset(
            graph_record,
            MODULE_GRAPH_COUNT_OFFSET,
            u64::from(graph.record_count()),
            function,
        );
        self.store_i64_local_at_offset(
            graph_record,
            MODULE_GRAPH_RECORDS_OFFSET,
            records,
            function,
        );
        self.store_i64_const_at_offset(
            graph_record,
            MODULE_GRAPH_NEXT_ASYNC_ORDER_OFFSET,
            1,
            function,
        );
        self.store_i64_const_at_offset(
            graph_record,
            MODULE_GRAPH_PARENT_BUDGET_OFFSET,
            u64::from(graph.record_count()).pow(2),
            function,
        );
        self.store_i64_const_at_offset(graph_record, MODULE_GRAPH_PARENT_COUNT_OFFSET, 0, function);
        for module in 0..graph.record_count() {
            self.emit_heap_alloc_const(MODULE_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(record));
            let index = GLOBAL_INDEX_REGISTRY.len() as u32
                + self.strings.template_objects.len() as u32
                + self.functions.module_unit_guard_count()
                + module;
            function.instruction(&Instruction::LocalGet(record));
            function.instruction(&Instruction::GlobalSet(index));
            self.store_i64_local_at_offset(records, u64::from(module) * 8, record, function);
            for offset in (0..MODULE_RECORD_SIZE).step_by(8) {
                self.store_i64_const_at_offset(record, offset, 0, function);
            }
            self.store_i64_local_at_offset(record, MODULE_GRAPH_OFFSET, graph_record, function);
            for offset in [
                MODULE_NAMESPACE_CELL_OFFSET,
                MODULE_DEFERRED_NAMESPACE_CELL_OFFSET,
            ] {
                self.store_i64_const_at_offset(
                    record,
                    offset + ENV_SLOT_TAG_OFFSET,
                    ENV_SLOT_UNINITIALIZED_TAG as u64,
                    function,
                );
            }
        }
        for activation in graph.activations() {
            let target = self.module_record_local(activation.module(), function)?;
            self.compile_expr_to_locals(
                &TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Function),
                    ExprIr::FunctionValue(activation.function().clone()),
                ),
                callee.payload,
                callee.tag,
                function,
            )?;
            self.store_i64_local_at_offset(
                target,
                MODULE_FUNCTION_OFFSET,
                callee.payload,
                function,
            );
            self.load_i64_to_local_from_offset(
                callee.payload,
                HEAP_FUNCTION_DEFINING_REALM_OFFSET,
                env,
                function,
            );
            self.store_i64_local_at_offset(target, MODULE_REALM_OFFSET, env, function);
            match activation.kind() {
                ModuleActivationKindIr::Synchronous => {
                    self.store_i64_const_at_offset(
                        target,
                        MODULE_ACTIVATION_KIND_OFFSET,
                        ModuleActivationKind::Synchronous.word(),
                        function,
                    );
                    self.emit_function_handle_call_without_throw_propagation(
                        callee.payload,
                        callee.tag,
                        None,
                        &[],
                        result.payload,
                        result.tag,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        result.payload,
                        result.tag,
                        function,
                    )?;
                    self.store_i64_local_at_offset(
                        target,
                        MODULE_ACTIVATION_OFFSET,
                        result.payload,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        result.payload,
                        HEAP_GENERATOR_ENV_OFFSET,
                        env,
                        function,
                    );
                }
                ModuleActivationKindIr::Async => {
                    self.store_i64_const_at_offset(
                        target,
                        MODULE_ACTIVATION_KIND_OFFSET,
                        ModuleActivationKind::Async.word(),
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        callee.payload,
                        HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                        env,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        callee.payload,
                        HEAP_FUNCTION_TABLE_INDEX_OFFSET,
                        table,
                        function,
                    );
                    self.emit_allocate_async_activation(
                        callee.payload,
                        env,
                        table,
                        zero,
                        undefined,
                        zero,
                        zero,
                        AsyncModuleEntryMode::Allocate,
                        record,
                        promise,
                        promise_record,
                        function,
                    )?;
                    self.store_i64_local_at_offset(
                        target,
                        MODULE_ACTIVATION_OFFSET,
                        record,
                        function,
                    );
                    self.emit_invoke_async_activation(record, result.payload, result.tag, function);
                    self.emit_propagate_throw_from_locals_if_needed(
                        result.payload,
                        result.tag,
                        function,
                    )?;
                    self.load_i64_to_local_from_offset(
                        record,
                        HEAP_ASYNC_INVOCATION_ENV_OFFSET,
                        env,
                        function,
                    );
                }
            }
            self.store_i64_local_at_offset(target, MODULE_ENVIRONMENT_OFFSET, env, function);
            if !activation.requests().is_empty() {
                self.emit_heap_alloc_const(
                    activation.requests().len() as u64 * MODULE_REQUEST_SIZE,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(requests));
                self.store_i64_local_at_offset(target, MODULE_REQUESTS_OFFSET, requests, function);
                self.store_i64_const_at_offset(
                    target,
                    MODULE_REQUEST_COUNT_OFFSET,
                    activation.requests().len() as u64,
                    function,
                );
                for (index, request) in activation.requests().iter().enumerate() {
                    let dependency = self.module_record_local(request.target(), function)?;
                    let phase = match request.phase() {
                        ModuleRequestPhaseIr::Evaluation => ModuleRequestPhase::Evaluation,
                        ModuleRequestPhaseIr::Defer => ModuleRequestPhase::Defer,
                    };
                    self.store_i64_const_at_offset(
                        requests,
                        index as u64 * MODULE_REQUEST_SIZE + MODULE_REQUEST_PHASE_OFFSET,
                        phase.word(),
                        function,
                    );
                    self.store_i64_local_at_offset(
                        requests,
                        index as u64 * MODULE_REQUEST_SIZE + MODULE_REQUEST_TARGET_OFFSET,
                        dependency,
                        function,
                    );
                    self.release_temp_local(dependency);
                }
            }
            self.release_temp_local(target);
        }
        // Canonical environments and request vectors exist before any import aliases them.
        for activation in graph.activations() {
            let target = self.module_record_local(activation.module(), function)?;
            match activation.kind() {
                ModuleActivationKindIr::Synchronous => {
                    self.emit_private_module_resume(target, result, function)?
                }
                ModuleActivationKindIr::Async => {
                    self.load_i64_to_local_from_offset(
                        target,
                        MODULE_ACTIVATION_OFFSET,
                        record,
                        function,
                    );
                    self.emit_store_async_module_entry_mode(
                        record,
                        AsyncModuleEntryMode::Instantiate,
                        function,
                    );
                    self.emit_invoke_async_activation(record, result.payload, result.tag, function);
                }
            }
            self.emit_propagate_throw_from_locals_if_needed(result.payload, result.tag, function)?;
            self.release_temp_local(target);
        }
        for local in [
            result.tag,
            result.payload,
            callee.tag,
            callee.payload,
            promise_record,
            promise,
            undefined,
            zero,
            table,
            env,
            requests,
            record,
            records,
            graph_record,
        ] {
            self.release_temp_local(local);
        }
        function.instruction(&Instruction::I64Const(0));
        Ok(())
    }

    pub(crate) fn emit_module_evaluate(
        &mut self,
        evaluation: &ModuleEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.module_record_local(evaluation.module(), function)?;
        self.emit_module_runtime_call(
            super::runtime::ModuleRuntimeOperation::Evaluate,
            &[record],
            payload,
            tag,
            function,
        )?;
        self.release_temp_local(record);
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)
    }

    pub(crate) fn emit_module_has_async_dependencies(
        &mut self,
        evaluation: &ModuleEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.module_record_local(evaluation.module(), function)?;
        self.emit_module_runtime_call(
            super::runtime::ModuleRuntimeOperation::Gather,
            &[record],
            payload,
            tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(tag));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        self.release_temp_local(record);
        Ok(())
    }

    pub(crate) fn emit_module_deferred_import_evaluate(
        &mut self,
        evaluation: &ModuleEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.module_record_local(evaluation.module(), function)?;
        self.emit_module_runtime_call(
            super::runtime::ModuleRuntimeOperation::DeferredImport,
            &[record],
            payload,
            tag,
            function,
        )?;
        self.release_temp_local(record);
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)
    }

    pub(crate) fn emit_deferred_module_evaluate(
        &mut self,
        evaluation: &DeferredModuleEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.module_record_local(evaluation.module(), function)?;
        self.emit_module_runtime_call(
            super::runtime::ModuleRuntimeOperation::Ready,
            &[record],
            payload,
            tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(payload));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "deferred module is not ready for synchronous evaluation",
            payload,
            tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)?;
        self.emit_module_runtime_call(
            super::runtime::ModuleRuntimeOperation::Evaluate,
            &[record],
            payload,
            tag,
            function,
        )?;
        self.emit_module_read_settled_evaluation(payload, tag, function)?;
        self.release_temp_local(record);
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)
    }
}
