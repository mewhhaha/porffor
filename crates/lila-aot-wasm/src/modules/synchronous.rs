//! Private module records reuse canonical generator activations and environment cells.

use super::*;
use crate::objects::TaggedLocals;
use lila_ir::{
    DeferredModuleEvaluationIr, ModuleCellIr, ModuleImportBindingIr, ModuleNamespaceModeIr,
    SynchronousModuleEvaluationIr, SynchronousModuleGraphIr,
};

#[derive(Clone, Copy)]
enum ModuleState {
    Linked,
    Evaluating,
    Evaluated,
    Errored,
}
impl ModuleState {
    const fn word(self) -> u64 {
        match self {
            Self::Linked => 0,
            Self::Evaluating => 1,
            Self::Evaluated => 2,
            Self::Errored => 3,
        }
    }
}

pub(crate) fn synchronous_module_record_count(script: &ScriptIr) -> u32 {
    script
        .executable_script_bodies()
        .flat_map(|body| &body.statements)
        .filter_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::SynchronousModuleGraph(graph),
                ..
            }) => Some(graph.record_count),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

impl FunctionBuilder<'_> {
    fn module_record_local(&mut self, module: u32, function: &mut Function) -> u32 {
        let record = self.reserve_temp_local();
        let index = GLOBAL_INDEX_REGISTRY.len() as u32
            + self.strings.template_objects.len() as u32
            + self.functions.module_unit_guard_count()
            + module;
        function.instruction(&Instruction::GlobalGet(index));
        function.instruction(&Instruction::LocalSet(record));
        record
    }

    fn module_namespace_cell_offset(mode: ModuleNamespaceModeIr) -> u64 {
        match mode {
            ModuleNamespaceModeIr::Eager => MODULE_NAMESPACE_CELL_OFFSET,
            ModuleNamespaceModeIr::Deferred => MODULE_DEFERRED_NAMESPACE_CELL_OFFSET,
        }
    }

    fn module_cell_address(&mut self, target: &ModuleCellIr, function: &mut Function) -> u32 {
        let (module, offset) = match target {
            ModuleCellIr::Binding { module, slot } => {
                (*module, ENV_SLOT_BASE_OFFSET + *slot as u64 * ENV_SLOT_SIZE)
            }
            ModuleCellIr::Namespace { module, mode } => {
                (*module, Self::module_namespace_cell_offset(*mode))
            }
        };
        let cell = self.module_record_local(module, function);
        if matches!(target, ModuleCellIr::Binding { .. }) {
            self.load_i64_to_local_from_offset(cell, MODULE_ACTIVATION_OFFSET, cell, function);
            self.load_i64_to_local_from_offset(cell, HEAP_GENERATOR_ENV_OFFSET, cell, function);
        }
        function.instruction(&Instruction::LocalGet(cell));
        function.instruction(&Instruction::I64Const(offset as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cell));
        cell
    }

    pub(crate) fn emit_module_import_binding(
        &mut self,
        import: &ModuleImportBindingIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let slot = self
            .owned_env_slot(&import.name)
            .ok_or_else(|| EmitError::unsupported("module import must own an environment slot"))?;
        let cell = self.module_cell_address(&import.target, function);
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
        let cell = self.module_cell_address(target, function);
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
        let record = self.module_record_local(module, function);
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

    fn emit_private_module_resume(
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

    fn emit_call_module_evaluator(
        &mut self,
        module: u32,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.module_record_local(module, function);
        let callee = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.load_i64_to_local_from_offset(
            record,
            MODULE_EVALUATOR_OFFSET,
            callee.payload,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(callee.tag));
        self.emit_function_handle_call_without_throw_propagation(
            callee.payload,
            callee.tag,
            None,
            &[],
            result.payload,
            result.tag,
            function,
        )?;
        self.release_temp_local(callee.tag);
        self.release_temp_local(callee.payload);
        self.release_temp_local(record);
        Ok(())
    }

    pub(crate) fn emit_synchronous_module_graph(
        &mut self,
        graph: &SynchronousModuleGraphIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let callee = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let result = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        for module in 0..graph.record_count {
            self.emit_heap_alloc_const(MODULE_RECORD_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(record));
            let index = GLOBAL_INDEX_REGISTRY.len() as u32
                + self.strings.template_objects.len() as u32
                + self.functions.module_unit_guard_count()
                + module;
            function.instruction(&Instruction::LocalGet(record));
            function.instruction(&Instruction::GlobalSet(index));
            for offset in (0..MODULE_RECORD_SIZE).step_by(8) {
                self.store_i64_const_at_offset(record, offset, 0, function);
            }
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
        // The generator INITIALIZING ABI allocates its environment before returning.
        for activation in &graph.activations {
            let target = self.module_record_local(activation.module, function);
            self.compile_expr_to_locals(
                &TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Function),
                    ExprIr::FunctionValue(activation.function.clone()),
                ),
                callee.payload,
                callee.tag,
                function,
            )?;
            self.emit_function_handle_call_without_throw_propagation(
                callee.payload,
                callee.tag,
                None,
                &[],
                result.payload,
                result.tag,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(result.payload, result.tag, function)?;
            self.store_i64_local_at_offset(
                target,
                MODULE_ACTIVATION_OFFSET,
                result.payload,
                function,
            );
            self.compile_expr_to_locals(
                &TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Function),
                    ExprIr::FunctionValue(activation.evaluator.clone()),
                ),
                callee.payload,
                callee.tag,
                function,
            )?;
            self.store_i64_local_at_offset(
                target,
                MODULE_EVALUATOR_OFFSET,
                callee.payload,
                function,
            );
            self.release_temp_local(target);
        }
        // Every target environment exists before any import cell points into it.
        for activation in &graph.activations {
            let target = self.module_record_local(activation.module, function);
            self.emit_private_module_resume(target, result, function)?;
            self.emit_propagate_throw_from_locals_if_needed(result.payload, result.tag, function)?;
            self.release_temp_local(target);
        }
        self.release_temp_local(result.tag);
        self.release_temp_local(result.payload);
        self.release_temp_local(callee.tag);
        self.release_temp_local(callee.payload);
        self.release_temp_local(record);
        function.instruction(&Instruction::I64Const(0));
        Ok(())
    }

    pub(crate) fn emit_synchronous_module_evaluate(
        &mut self,
        plan: &SynchronousModuleEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.module_record_local(plan.module(), function);
        let state = self.reserve_temp_local();
        let component_owner = self.reserve_temp_local();
        let result = TaggedLocals::new(payload, tag);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.load_i64_to_local_from_offset(record, MODULE_STATE_OFFSET, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(ModuleState::Errored.word() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(record, MODULE_ERROR_PAYLOAD_OFFSET, payload, function);
        self.load_i64_to_local_from_offset(record, MODULE_ERROR_TAG_OFFSET, tag, function);
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(ModuleState::Linked.word() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));
        // With no asynchronous members, the first active frame in this static
        // SCC remains on the call stack until the component completes.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(component_owner));
        for member in plan.component_members() {
            let member_record = self.module_record_local(*member, function);
            self.load_i64_from_offset(member_record, MODULE_STATE_OFFSET, function);
            function.instruction(&Instruction::I64Const(ModuleState::Evaluating.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(component_owner));
            function.instruction(&Instruction::End);
            self.release_temp_local(member_record);
        }
        self.store_i64_const_at_offset(
            record,
            MODULE_STATE_OFFSET,
            ModuleState::Evaluating.word(),
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        for dependency in plan.dependencies() {
            self.emit_call_module_evaluator(*dependency, result, function)?;
            self.emit_break_current_completion_if_throw(1, function);
        }
        self.emit_private_module_resume(record, result, function)?;
        function.instruction(&Instruction::End);
        for member in plan.component_members() {
            function.instruction(&Instruction::LocalGet(component_owner));
            if *member == plan.module() {
                // A failed non-root frame caches its own error while unwinding;
                // the component owner also finalizes members whose bodies returned.
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::I64Or);
            }
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            let member_record = self.module_record_local(*member, function);
            self.load_i64_from_offset(member_record, MODULE_STATE_OFFSET, function);
            function.instruction(&Instruction::I64Const(ModuleState::Evaluating.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            // Only entered members change state: an earlier failure can leave
            // other members Linked, with dependencies they must still visit.
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_local_at_offset(
                member_record,
                MODULE_ERROR_PAYLOAD_OFFSET,
                payload,
                function,
            );
            self.store_i64_local_at_offset(member_record, MODULE_ERROR_TAG_OFFSET, tag, function);
            self.store_i64_const_at_offset(
                member_record,
                MODULE_STATE_OFFSET,
                ModuleState::Errored.word(),
                function,
            );
            function.instruction(&Instruction::Else);
            self.store_i64_const_at_offset(
                member_record,
                MODULE_STATE_OFFSET,
                ModuleState::Evaluated.word(),
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.release_temp_local(member_record);
        }
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(component_owner);
        self.release_temp_local(state);
        self.release_temp_local(record);
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)
    }

    pub(crate) fn emit_deferred_module_evaluate(
        &mut self,
        plan: &DeferredModuleEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // A pure fixed point implements the spec's seen set. Evaluated (including
        // errored) records stop traversal; no observable call occurs during it.
        let reached = plan
            .readiness
            .iter()
            .map(|node| node.module)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|module| (module, self.reserve_temp_local()))
            .collect::<BTreeMap<_, _>>();
        let changed = self.reserve_temp_local();
        for (module, local) in &reached {
            function.instruction(&Instruction::I64Const(i64::from(*module == plan.module)));
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(changed));
        for node in &plan.readiness {
            function.instruction(&Instruction::LocalGet(reached[&node.module]));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            let record = self.module_record_local(node.module, function);
            self.load_i64_from_offset(record, MODULE_STATE_OFFSET, function);
            function.instruction(&Instruction::I64Const(ModuleState::Evaluating.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                "TypeError",
                "module graph is already evaluating",
                payload,
                tag,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)?;
            function.instruction(&Instruction::End);
            self.load_i64_from_offset(record, MODULE_STATE_OFFSET, function);
            function.instruction(&Instruction::I64Const(ModuleState::Linked.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            for dependency in &node.dependencies {
                let local = reached[dependency];
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(changed));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
            self.release_temp_local(record);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(changed));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::End);
        self.release_temp_local(changed);
        for local in reached.values().rev() {
            self.release_temp_local(*local);
        }
        self.emit_call_module_evaluator(plan.module, TaggedLocals::new(payload, tag), function)?;
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)
    }
}
