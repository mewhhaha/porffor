use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn emit_self_backed_promise_any_aggregate_error_allocation_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseAnyAggregateErrorAllocationContext {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        PromiseAnyAggregateErrorAllocationContext { prototype_local }
    }

    pub(in crate::builtins) fn emit_promise_combinator_aggregate_error_allocation_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseAnyAggregateErrorAllocationContext {
        let prototype_local = self.reserve_temp_local();
        let active_function_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(active_function_local));
        self.load_i64_to_local_from_offset(
            active_function_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.release_temp_local(active_function_local);
        PromiseAnyAggregateErrorAllocationContext { prototype_local }
    }

    pub(in crate::builtins) fn emit_promise_any_aggregate_error_from_context(
        &mut self,
        errors_payload_local: u32,
        context: PromiseAnyAggregateErrorAllocationContext,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prepared = self
            .emit_prepare_promise_any_aggregate_error_instance(context.prototype_local, function)?;
        self.emit_finish_aggregate_error_instance(
            prepared,
            errors_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(context.prototype_local);
        Ok(())
    }
}
