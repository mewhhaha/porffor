//! Adoption and host projection of the exact compiler-owned entry evaluation.

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
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(
            MODULE_EVALUATION_PROMISE_GLOBAL_INDEX,
        ));
        self.emit_store_module_entry_status(WasmModuleEvaluationStatus::NotStarted, function);
    }

    pub(crate) fn emit_module_entry_evaluation(
        &mut self,
        entry: &ModuleEntryEvaluationIr,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        assert!(
            self.is_main(),
            "only the artifact main owns entry evaluation"
        );
        self.compile_expr_to_locals(entry.evaluation(), payload, tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(payload, tag, function)?;
        match entry.kind() {
            ModuleEntryEvaluationKindIr::Synchronous => {
                self.emit_store_module_entry_status(WasmModuleEvaluationStatus::Settled, function);
            }
            ModuleEntryEvaluationKindIr::Promise => {
                let record = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(tag));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Unreachable);
                function.instruction(&Instruction::End);
                self.load_i64_to_local_from_offset(
                    payload,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    record,
                    function,
                );
                function.instruction(&Instruction::LocalGet(record));
                function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_PROMISE as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Unreachable);
                function.instruction(&Instruction::End);
                // Retain the Promise object, including its intrinsic record and
                // exact settled value, independently of the rejection FIFO.
                function.instruction(&Instruction::LocalGet(payload));
                function.instruction(&Instruction::GlobalSet(
                    MODULE_EVALUATION_PROMISE_GLOBAL_INDEX,
                ));
                self.load_i64_to_local_from_offset(
                    payload,
                    HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                    record,
                    function,
                );
                // Intrinsic host adoption must not read a mutable .then or
                // constructor/species, nor manufacture a second Promise.
                self.store_i64_const_at_offset(record, HEAP_PROMISE_IS_HANDLED_OFFSET, 1, function);
                self.emit_store_module_entry_status(WasmModuleEvaluationStatus::Pending, function);
                self.release_temp_local(record);
            }
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        self.set_completion_kind(CompletionKind::Normal, function);
        Ok(())
    }

    pub(crate) fn emit_module_entry_checkpoint(
        &mut self,
        kind: ModuleEntryEvaluationKindIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(CompletionKind::Throw.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A prelude/instantiation throw remains primary even if entry adoption
        // never happened. Do not inspect an absent Promise root on this path.
        self.emit_store_module_entry_status(WasmModuleEvaluationStatus::Settled, function);
        function.instruction(&Instruction::Else);
        match kind {
            ModuleEntryEvaluationKindIr::Synchronous => {
                self.emit_statement_result(function, ValueKind::Undefined);
                self.set_completion_kind(CompletionKind::Normal, function);
                self.emit_store_module_entry_status(WasmModuleEvaluationStatus::Settled, function);
            }
            ModuleEntryEvaluationKindIr::Promise => {
                let promise = self.reserve_temp_local();
                let record = self.reserve_temp_local();
                let state = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(
                    MODULE_EVALUATION_PROMISE_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalTee(promise));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Unreachable);
                function.instruction(&Instruction::End);
                self.load_i64_to_local_from_offset(
                    promise,
                    HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                    record,
                    function,
                );
                self.emit_load_promise_state_strict(record, state, function);
                for value in PromiseState::ALL {
                    function.instruction(&Instruction::LocalGet(state));
                    function.instruction(&Instruction::I64Const(value.word() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    match value {
                        PromiseState::Pending => {
                            self.emit_statement_result(function, ValueKind::Undefined);
                            self.set_completion_kind(CompletionKind::Normal, function);
                            self.emit_store_module_entry_status(
                                WasmModuleEvaluationStatus::Pending,
                                function,
                            );
                        }
                        PromiseState::Fulfilled => {
                            self.emit_statement_result(function, ValueKind::Undefined);
                            self.set_completion_kind(CompletionKind::Normal, function);
                            self.emit_store_module_entry_status(
                                WasmModuleEvaluationStatus::Settled,
                                function,
                            );
                        }
                        PromiseState::Rejected => {
                            self.load_i64_to_local_from_offset(
                                record,
                                HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
                                self.result_local,
                                function,
                            );
                            self.load_i64_to_local_from_offset(
                                record,
                                HEAP_PROMISE_RESULT_TAG_OFFSET,
                                self.result_tag_local,
                                function,
                            );
                            self.set_completion_kind(CompletionKind::Throw, function);
                            self.emit_store_module_entry_status(
                                WasmModuleEvaluationStatus::Settled,
                                function,
                            );
                            // Capture only data properties for the legacy error
                            // display. No conversion decides the completion.
                            self.emit_capture_throw_error_name(
                                self.result_local,
                                self.result_tag_local,
                                function,
                            )?;
                        }
                    }
                    function.instruction(&Instruction::End);
                }
                self.release_temp_local(state);
                self.release_temp_local(record);
                self.release_temp_local(promise);
            }
        }
        function.instruction(&Instruction::End);
        Ok(())
    }
}
