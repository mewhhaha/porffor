use super::super::*;

enum GlobalNumericBuiltin {
    IsFinite,
    IsNaN,
}

impl FunctionBuilder<'_> {
    fn emit_global_numeric_builtin(
        &mut self,
        builtin: GlobalNumericBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let truth = schema.reserve_i32_local(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_value_to_number_payload(&argument, &pending, function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let bits = pending.value().scalar();
        bits.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        bits.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        match builtin {
            GlobalNumericBuiltin::IsNaN => {}
            GlobalNumericBuiltin::IsFinite => {
                function.instruction(&Instruction::I32Eqz);
                for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
                    bits.load(function);
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
                    function.instruction(&Instruction::F64Ne);
                    function.instruction(&Instruction::I32And);
                }
            }
        }
        truth.store(function);
        argument.set_boolean(truth, function);
        pending.set_normal(&argument, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&pending, function);
        schema.release_i32_local(truth, function);
        pending.clear(function);
        argument.clear(function);
        Ok(())
    }

    pub(super) fn emit_global_is_finite_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_global_numeric_builtin(GlobalNumericBuiltin::IsFinite, function)
    }

    pub(super) fn emit_global_is_nan_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_global_numeric_builtin(GlobalNumericBuiltin::IsNaN, function)
    }
}
