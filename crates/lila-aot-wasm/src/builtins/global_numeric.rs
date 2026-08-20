use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GlobalNumericBuiltin {
    IsFinite,
    IsNaN,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_global_numeric_builtin(
        &mut self,
        builtin: GlobalNumericBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            GlobalNumericBuiltin::IsFinite | GlobalNumericBuiltin::IsNaN => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(arg_payload_local));
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Ne);
                match builtin {
                    GlobalNumericBuiltin::IsFinite => {
                        function.instruction(&Instruction::I32Eqz);
                        for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
                            function.instruction(&Instruction::LocalGet(arg_payload_local));
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
                            function.instruction(&Instruction::F64Ne);
                            function.instruction(&Instruction::I32And);
                        }
                    }
                    GlobalNumericBuiltin::IsNaN => {}
                }
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
        }
        Ok(())
    }
}
