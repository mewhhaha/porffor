use super::*;
use crate::prepared_script::GlobalPreparedScriptKind;

impl FunctionBuilder<'_> {
    pub(super) fn emit_eval_function_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        output.set_normal(&argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let source = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_prepared_script_dispatch(
            GlobalPreparedScriptKind::IndirectEval,
            &source,
            &output,
            function,
        )?;
        source.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        output.clear(function);
        argument.clear(function);
        Ok(())
    }
}
