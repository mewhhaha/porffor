use super::*;

#[must_use = "the observed then Reference must be consumed by its call"]
pub(super) struct ValidatedPromisePrototypeThenInvocationLocals {
    method: ValueLocals,
    receiver: ValueLocals,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_validate_promise_prototype_then_invocation(
        &mut self,
        method: &ValueLocals,
        receiver: &ValueLocals,
        function: &mut Function,
    ) -> Result<ValidatedPromisePrototypeThenInvocationLocals, EmitError> {
        let schema = self.runtime_schema();
        self.emit_is_callable_i32(method, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let error = schema.reserve_completion(function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::VALUE_IS_NOT_CALLABLE,
            &error,
            function,
        )?;
        self.completion().copy_from(&error, function);
        self.emit_propagate_current_throw(function);
        error.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let copied_method = schema.reserve_value_local(function);
        let copied_receiver = schema.reserve_value_local(function);
        copied_method.copy_from(method, function);
        copied_receiver.copy_from(receiver, function);
        Ok(ValidatedPromisePrototypeThenInvocationLocals {
            method: copied_method,
            receiver: copied_receiver,
        })
    }
    pub(super) fn emit_call_validated_promise_prototype_then_invocation(
        &mut self,
        invocation: ValidatedPromisePrototypeThenInvocationLocals,
        first: &ValueLocals,
        second: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arguments = self.emit_pre_evaluated_arg_vector(&[first, second], function);
        let emitted = self.emit_function_or_proxy_call_with_argv(
            &invocation.method,
            &invocation.receiver,
            &arguments,
            result,
            function,
        );
        arguments.clear(function);
        invocation.receiver.clear(function);
        invocation.method.clear(function);
        emitted
    }
}
