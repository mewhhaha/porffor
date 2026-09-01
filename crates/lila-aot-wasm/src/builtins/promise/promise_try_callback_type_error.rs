use super::*;

#[must_use = "Promise.try callback TypeError prototype must be consumed"]
pub(super) struct PromiseTryCallbackTypeErrorPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_load_promise_try_callback_type_error_prototype(
        &mut self,
        function: &mut Function,
    ) -> PromiseTryCallbackTypeErrorPrototypeLocal {
        let prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        PromiseTryCallbackTypeErrorPrototypeLocal(prototype_local)
    }

    pub(super) fn emit_throw_promise_try_non_callable_callback(
        &mut self,
        prototype: PromiseTryCallbackTypeErrorPrototypeLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result = self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "value is not callable",
            prototype.0,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(prototype.0);
        result
    }
}
