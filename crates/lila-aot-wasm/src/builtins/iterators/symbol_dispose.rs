//! Iterator disposal retains the raw receiver, ordered GetV and whole abrupt completion.
use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_iterator_prototype_symbol_dispose(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let boxed = schema.reserve_value_local(function);
        let method = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.compile_this_to_locals(&receiver, function)?;
        self.emit_value_to_current_function_realm_object_locals(&receiver, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        boxed.copy_from(pending.value(), function);
        let key = self.emit_function_string_key("return", function)?;
        // GetV boxes only the lookup base and retains the raw receiver.
        self.emit_object_read_with_throw_routing(
            &boxed,
            &receiver,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        key.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        method.copy_from(pending.value(), function);
        self.compile_nullish_tagged_i32(method.tag(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_callable_i32(&method, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ITERATOR_PROTOTYPE_SYMBOL_DISPOSE_RETURN_METHOD_MUST_BE_CALLABLE,
            &pending,
            function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_function_or_proxy_call_with_argv(
            &method, &receiver, &arguments, &pending, function,
        )?;
        arguments.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().initialize(function);
        pending.clear(function);
        method.clear(function);
        boxed.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
