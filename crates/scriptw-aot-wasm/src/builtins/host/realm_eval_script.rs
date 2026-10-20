use super::*;
use crate::gc_types::StringValue;
use crate::prepared_script::GlobalPreparedScriptKind;

impl FunctionBuilder<'_> {
    /// Source conversion completes in the captured callable's Realm before
    /// entering a compiled Script. Arbitrary conversion Throws remain whole.
    pub(crate) fn compile_host_realm_eval_script_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        output.initialize(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        self.emit_value_to_string_payload(&argument, &pending, function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(&pending, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let source = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_prepared_script_dispatch(
            GlobalPreparedScriptKind::RealmScript,
            &source,
            &output,
            function,
        )?;
        source.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        output.clear(function);
        pending.clear(function);
        argument.clear(function);
        Ok(())
    }
}
