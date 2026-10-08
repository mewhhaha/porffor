use super::*;
use crate::gc_types::ValueLocals;

impl FunctionBuilder<'_> {
    /// The caller has extracted thisNumberValue/thisBigIntValue. Retain that
    /// whole value through locales/options hooks and invoke the recorded Intl
    /// constructor/getter, independent of mutable public Intl properties.
    pub(crate) fn emit_intrinsic_number_locale_format(
        &mut self,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor = self
            .functions
            .get(&StandardBuiltinId::IntlNumberFormatConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing planned Intl.NumberFormat constructor")
            })?;
        let getter = self
            .functions
            .get(&StandardBuiltinId::IntlNumberFormatPrototypeFormatGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing planned Intl.NumberFormat format getter")
            })?;
        let schema = self.runtime_schema();
        let locales = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let formatter = schema.reserve_completion(function);
        let format = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        let receiver = schema.reserve_value_local(function);
        receiver.set_undefined(function);
        self.emit_builtin_arg_to_value(0, &locales, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.emit_direct_js_call(
            &constructor,
            None,
            &[&locales, &options],
            &formatter,
            function,
        )?;
        output.copy_from(&formatter, function);
        formatter.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(cleanup, function);
        self.emit_direct_js_call(&getter, Some(formatter.value()), &[], &format, function)?;
        output.copy_from(&format, function);
        format.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(cleanup, function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[value], function);
        self.emit_function_or_proxy_call_with_argv(
            format.value(),
            &receiver,
            &arguments,
            &output,
            function,
        )?;
        arguments.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        receiver.clear(function);
        output.clear(function);
        format.clear(function);
        formatter.clear(function);
        options.clear(function);
        locales.clear(function);
        Ok(())
    }
}
