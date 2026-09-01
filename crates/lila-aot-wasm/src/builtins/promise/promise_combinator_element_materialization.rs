use super::*;

#[must_use = "Promise combinator element function context must be explicitly released"]
pub(super) struct PromiseCombinatorElementFunctionMaterializationContext {
    internal: PromiseInternalFunctionMaterializationContext,
    aggregate_error_prototype_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_current_function_promise_combinator_element_materialization_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseCombinatorElementFunctionMaterializationContext {
        let aggregate_error_prototype_local = self.reserve_temp_local();
        let internal =
            self.emit_current_function_promise_internal_function_materialization_context(function);
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
            aggregate_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(aggregate_error_prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.release_temp_local(active_function_local);

        PromiseCombinatorElementFunctionMaterializationContext {
            internal,
            aggregate_error_prototype_local,
        }
    }

    pub(super) fn emit_promise_combinator_element_function_value(
        &mut self,
        meta: &WasmFunctionMeta,
        context: &PromiseCombinatorElementFunctionMaterializationContext,
        closure_context_local: u32,
        function_object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_internal_function_value(
            meta,
            &context.internal,
            closure_context_local,
            function_object_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            context.aggregate_error_prototype_local,
            function,
        );
        Ok(())
    }

    pub(super) fn release_promise_combinator_element_function_materialization_context(
        &mut self,
        context: PromiseCombinatorElementFunctionMaterializationContext,
    ) {
        self.release_promise_internal_function_materialization_context(context.internal);
        self.release_temp_local(context.aggregate_error_prototype_local);
    }
}
