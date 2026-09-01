use super::*;

#[derive(Clone, Copy)]
enum PromisePrototypeReceiverError {
    ThenIncompatible,
    FinallyNonObject,
}

impl PromisePrototypeReceiverError {
    const fn message(self) -> &'static str {
        match self {
            Self::ThenIncompatible => "Promise.prototype.then called on incompatible receiver",
            Self::FinallyNonObject => "Promise.prototype.finally called on non-object receiver",
        }
    }
}

#[must_use = "Promise prototype receiver TypeError prototype must be consumed"]
pub(super) struct PromisePrototypeReceiverTypeErrorPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_load_promise_prototype_receiver_type_error_prototype(
        &mut self,
        function: &mut Function,
    ) -> PromisePrototypeReceiverTypeErrorPrototypeLocal {
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

        PromisePrototypeReceiverTypeErrorPrototypeLocal(prototype_local)
    }

    fn emit_throw_promise_prototype_receiver_error(
        &mut self,
        prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal,
        error: PromisePrototypeReceiverError,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result = self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            error.message(),
            prototype.0,
            payload_local,
            tag_local,
            function,
        );
        self.release_temp_local(prototype.0);
        result
    }

    pub(super) fn emit_throw_promise_then_incompatible_receiver_error(
        &mut self,
        prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_promise_prototype_receiver_error(
            prototype,
            PromisePrototypeReceiverError::ThenIncompatible,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn emit_throw_promise_finally_non_object_receiver_error(
        &mut self,
        prototype: PromisePrototypeReceiverTypeErrorPrototypeLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_promise_prototype_receiver_error(
            prototype,
            PromisePrototypeReceiverError::FinallyNonObject,
            payload_local,
            tag_local,
            function,
        )
    }
}
