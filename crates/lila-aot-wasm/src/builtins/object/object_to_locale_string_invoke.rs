use super::*;
use crate::gc_types::{CompletionLocals, ValueLocals};

#[must_use]
struct ObjectToLocaleStringGetVLocals {
    original_receiver: ValueLocals,
    boxed_lookup: ValueLocals,
    method: ValueLocals,
}
#[must_use]
struct ValidatedObjectToLocaleStringInvocationLocals {
    method: ValueLocals,
    receiver: ValueLocals,
}

impl FunctionBuilder<'_> {
    fn emit_validate_object_to_locale_string_invocation(
        &mut self,
        get_v: ObjectToLocaleStringGetVLocals,
        pending: &CompletionLocals,
        function: &mut Function,
    ) -> Result<ValidatedObjectToLocaleStringInvocationLocals, EmitError> {
        self.emit_is_callable_i32(&get_v.method, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::OBJECT_PROTOTYPE_TOLOCALESTRING_TARGET_IS_NOT_CALLABLE,
            pending,
            function,
        )?;
        self.completion().copy_from(pending, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        get_v.boxed_lookup.clear(function);
        Ok(ValidatedObjectToLocaleStringInvocationLocals {
            method: get_v.method,
            receiver: get_v.original_receiver,
        })
    }

    fn emit_call_validated_object_to_locale_string_invocation(
        &mut self,
        invocation: ValidatedObjectToLocaleStringInvocationLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_function_or_proxy_call_with_argv(
            &invocation.method,
            &invocation.receiver,
            &arguments,
            result,
            function,
        )?;
        arguments.clear(function);
        invocation.method.clear(function);
        invocation.receiver.clear(function);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_prototype_to_locale_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let lookup = schema.reserve_value_local(function);
        let method = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.compile_this_to_locals(&receiver, function)?;
        self.emit_value_to_object_locals(&receiver, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        lookup.copy_from(pending.value(), function);
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("toString", function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &name, function);
        self.emit_object_read(&lookup, &receiver, &key, &pending, function)?;
        key.clear(function);
        name.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        method.copy_from(pending.value(), function);
        let invocation = self.emit_validate_object_to_locale_string_invocation(
            ObjectToLocaleStringGetVLocals {
                original_receiver: receiver,
                boxed_lookup: lookup,
                method,
            },
            &pending,
            function,
        )?;
        self.emit_call_validated_object_to_locale_string_invocation(
            invocation, &pending, function,
        )?;
        self.completion().copy_from(&pending, function);
        pending.clear(function);
        Ok(())
    }
}
