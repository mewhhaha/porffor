//! Host adoption retains the exact intrinsic Promise in a dedicated GC root.

use super::*;
use lila_ir::{ModuleEntryEvaluationIr, ModuleEntryEvaluationKindIr};

impl FunctionBuilder<'_> {
    fn emit_store_module_entry_status(
        &self,
        status: WasmModuleEvaluationStatus,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(status.word()));
        function.instruction(&Instruction::GlobalSet(
            MODULE_EVALUATION_STATUS_GLOBAL_INDEX,
        ));
    }

    pub(crate) fn initialize_module_entry_completion(&self, function: &mut Function) {
        self.runtime_schema()
            .clear_module_evaluation_promise(function);
        self.emit_store_module_entry_status(WasmModuleEvaluationStatus::NotStarted, function);
    }

    pub(crate) fn emit_module_entry_evaluation(
        &mut self,
        entry: &ModuleEntryEvaluationIr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        assert!(
            self.is_main(),
            "only the artifact main owns entry evaluation"
        );
        self.compile_expr_to_value(entry.evaluation(), output, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        let schema = self.runtime_schema();
        match entry.kind() {
            ModuleEntryEvaluationKindIr::Promise => {
                let promise = schema.reserve_gc_local(function).initialize(
                    output.cast_reference::<PromiseObject>(schema, function),
                    function,
                );
                schema.replace_module_evaluation_promise(&promise, function);
                // Adoption reads neither .then nor constructor/species and
                // allocates no second Promise. The rejection FIFO is separate.
                schema
                    .struct_type::<PromiseObject>()
                    .field(PromiseObjectSchema::HANDLED)
                    .write(&promise, GcOperand::boolean(true), schema, function);
                self.emit_store_module_entry_status(WasmModuleEvaluationStatus::Pending, function);
                promise.clear(function);
            }
        }
        output.set_undefined(function);
        self.completion().initialize(function);
        Ok(())
    }

    pub(crate) fn emit_module_entry_checkpoint(
        &mut self,
        kind: ModuleEntryEvaluationKindIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        self.completion().kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        // Prelude/instantiation Throw wins even before Promise adoption.
        self.emit_store_module_entry_status(WasmModuleEvaluationStatus::Settled, function);
        function.instruction(&Instruction::Else);
        match kind {
            ModuleEntryEvaluationKindIr::Promise => {
                let adopted = schema.load_module_evaluation_promise(function);
                let promise = schema.reserve_gc_local(function).initialize(
                    adopted.load(schema, function).require_non_null(function),
                    function,
                );
                let state = schema.reserve_i32_local(function);
                self.emit_load_promise_state_strict(&promise, state, function);
                let mut arms = 0;
                for selected in PromiseState::ALL {
                    state.load(function);
                    function.instruction(&Instruction::I32Const(GcI32Constant::encode(selected)));
                    function.instruction(&Instruction::I32Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    match selected {
                        PromiseState::Pending => {
                            self.completion().initialize(function);
                            self.emit_store_module_entry_status(
                                WasmModuleEvaluationStatus::Pending,
                                function,
                            );
                        }
                        PromiseState::Fulfilled => {
                            self.completion().initialize(function);
                            self.emit_store_module_entry_status(
                                WasmModuleEvaluationStatus::Settled,
                                function,
                            );
                        }
                        PromiseState::Rejected => {
                            let stored = schema.reserve_gc_local(function).initialize(
                                schema
                                    .struct_type::<PromiseObject>()
                                    .field(PromiseObjectSchema::RESULT)
                                    .read(&promise, schema, function)
                                    .reference(),
                                function,
                            );
                            let reason = schema.reserve_value_local(function);
                            schema
                                .struct_type::<StoredValue>()
                                .read_into(&stored, &reason, schema, function);
                            self.completion().set_throw(&reason, function);
                            self.emit_store_module_entry_status(
                                WasmModuleEvaluationStatus::Settled,
                                function,
                            );
                            self.emit_capture_throw_error_name(&reason, function)?;
                            reason.clear(function);
                            stored.clear(function);
                        }
                    }
                    function.instruction(&Instruction::Else);
                    arms += 1;
                }
                function.instruction(&Instruction::Unreachable);
                for _ in 0..arms {
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
                schema.release_i32_local(state, function);
                promise.clear(function);
                adopted.clear(function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
