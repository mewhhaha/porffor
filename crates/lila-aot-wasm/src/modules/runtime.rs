//! Shared private operations selected only by a validated compiled graph.

use super::*;
use crate::runtime_helpers::RuntimeHelperId;

#[derive(Clone, Copy)]
pub(crate) enum ModuleRuntimeOperation {
    Evaluate,
    Ready,
    Gather,
    Execute,
    Fulfilled,
    Rejected,
    DeferredImport,
}
impl ModuleRuntimeOperation {
    pub(crate) const ALL: [Self; 7] = [
        Self::Evaluate,
        Self::Ready,
        Self::Gather,
        Self::Execute,
        Self::Fulfilled,
        Self::Rejected,
        Self::DeferredImport,
    ];
    pub(crate) const fn helper(self) -> RuntimeHelperId {
        match self {
            Self::Evaluate => RuntimeHelperId::ModuleEvaluate,
            Self::Ready => RuntimeHelperId::ModuleReady,
            Self::Gather => RuntimeHelperId::ModuleGather,
            Self::Execute => RuntimeHelperId::ModuleExecute,
            Self::Fulfilled => RuntimeHelperId::ModuleFulfilled,
            Self::Rejected => RuntimeHelperId::ModuleRejected,
            Self::DeferredImport => RuntimeHelperId::ModuleDeferredImport,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_module_runtime_operation(
        &mut self,
        operation: ModuleRuntimeOperation,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(operation.helper());
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        match operation {
            ModuleRuntimeOperation::Evaluate => {
                self.emit_module_evaluation_runtime(&mut function)?
            }
            ModuleRuntimeOperation::Ready => self.emit_module_readiness_runtime(&mut function)?,
            ModuleRuntimeOperation::Gather => self.emit_module_gather_runtime(&mut function)?,
            ModuleRuntimeOperation::Execute => self.emit_module_execute_runtime(&mut function)?,
            ModuleRuntimeOperation::Fulfilled => {
                self.emit_module_fulfilled_runtime(&mut function)?
            }
            ModuleRuntimeOperation::Rejected => self.emit_module_rejected_runtime(&mut function)?,
            ModuleRuntimeOperation::DeferredImport => {
                self.emit_module_deferred_import_runtime(&mut function)?
            }
        }
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(super) fn emit_module_runtime_call(
        &mut self,
        operation: ModuleRuntimeOperation,
        arguments: &[u32],
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.functions.module_execution_record_count() == 0 {
            return Err(EmitError::unsupported(
                "private module operation requires its validated execution graph",
            ));
        }
        assert!(arguments.len() <= 7);
        let index = self
            .heap_alloc_function_index
            .map(|base| operation.helper().index(base))
            .ok_or_else(|| EmitError::unsupported("module execution requires heap runtime"))?;
        for &argument in arguments {
            function.instruction(&Instruction::LocalGet(argument));
        }
        for _ in arguments.len()..7 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(index));
        self.store_call_results(payload, tag, function);
        Ok(())
    }

    pub(super) fn emit_module_array_address(
        &self,
        base: u32,
        index: u32,
        stride: u64,
        address: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(base));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(stride as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address));
    }

    pub(super) fn emit_module_list_contains(
        &mut self,
        base: u32,
        count: u32,
        sought: u32,
        found: u32,
        function: &mut Function,
    ) {
        let index = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        for local in [index, found] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_module_array_address(base, index, 8, address, function);
        self.load_i64_to_local_from_offset(address, 0, address, function);
        function.instruction(&Instruction::LocalGet(address));
        function.instruction(&Instruction::LocalGet(sought));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(address);
        self.release_temp_local(index);
    }

    pub(super) fn emit_module_increment(&self, local: u32, amount: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(amount));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(local));
    }

    pub(super) fn emit_module_bounded_append(
        &mut self,
        base: u32,
        count: u32,
        limit: u32,
        value: u32,
        function: &mut Function,
    ) {
        let address = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::LocalGet(limit));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_module_array_address(base, count, 8, address, function);
        self.store_i64_local_at_offset(address, 0, value, function);
        self.emit_module_increment(count, 1, function);
        self.release_temp_local(address);
    }

    pub(super) fn emit_module_allocate_words(
        &mut self,
        count: u32,
        stride: u64,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let size = self.reserve_temp_local();
        // Count originated from the validated graph's u32 cardinality. The
        // bound also rejects corrupted private records before multiplication.
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Const((u32::MAX as u64 / stride) as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Const(stride as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(size));
        self.emit_heap_alloc_from_local(size, function)?;
        function.instruction(&Instruction::LocalSet(destination));
        self.release_temp_local(size);
        Ok(())
    }

    pub(super) fn emit_module_read_settled_evaluation(
        &mut self,
        promise: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let state = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            promise,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        self.store_i64_const_at_offset(record, HEAP_PROMISE_IS_HANDLED_OFFSET, 1, function);
        self.emit_load_promise_state_strict(record, state, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(PromiseState::Pending.word() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            record,
            HEAP_PROMISE_RESULT_PAYLOAD_OFFSET,
            promise,
            function,
        );
        self.load_i64_to_local_from_offset(record, HEAP_PROMISE_RESULT_TAG_OFFSET, tag, function);
        function.instruction(&Instruction::LocalGet(state));
        function.instruction(&Instruction::I64Const(PromiseState::Rejected.word() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Throw, function);
        function.instruction(&Instruction::Else);
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(state);
        self.release_temp_local(record);
        Ok(())
    }
}
