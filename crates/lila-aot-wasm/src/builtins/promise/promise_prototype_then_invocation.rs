use super::*;

#[must_use = "a validated Promise prototype then invocation must be called"]
pub(super) struct ValidatedPromisePrototypeThenInvocationLocals {
    method: TaggedLocals,
    receiver: TaggedLocals,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_validate_promise_prototype_then_invocation(
        &mut self,
        method: TaggedLocals,
        receiver: TaggedLocals,
        function: &mut Function,
    ) -> Result<ValidatedPromisePrototypeThenInvocationLocals, EmitError> {
        self.emit_is_callable_i32(method.tag, method.payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "value is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        Ok(ValidatedPromisePrototypeThenInvocationLocals { method, receiver })
    }

    pub(super) fn emit_call_validated_promise_prototype_then_invocation(
        &mut self,
        invocation: ValidatedPromisePrototypeThenInvocationLocals,
        first_argument: TaggedLocals,
        second_argument: TaggedLocals,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ValidatedPromisePrototypeThenInvocationLocals { method, receiver } = invocation;

        self.emit_function_or_proxy_call_leave_throw_completion(
            method.payload,
            method.tag,
            receiver.payload,
            receiver.tag,
            &[
                (first_argument.payload, first_argument.tag),
                (second_argument.payload, second_argument.tag),
            ],
            result.payload,
            result.tag,
            function,
        )
    }
}
