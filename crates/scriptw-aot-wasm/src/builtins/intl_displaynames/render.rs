use super::*;
impl FunctionBuilder<'_> {
    pub(crate) fn emit_intl_display_names_of(&mut self, f: &mut Function) -> Result<(), EmitError> {
        // Intrinsic brand validation precedes every observation of the code operand.
        let record = self.emit_display_names_record_from_receiver(f)?;
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        let code = CompletedDisplayNameCodeLocals(self.emit_intl_number_to_string(&value, f)?);
        let response = self.emit_display_names_provider_call(
            DisplayNamesProviderRequest::Name {
                record: &record,
                code: &code,
            },
            f,
        )?;
        let reader = response.reader(schema, f);
        let present = schema.reserve_i64_local(f);
        let output = schema.reserve_value_local(f);
        output.set_undefined(f);
        reader.read_u64(present, schema, f);
        present.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        present.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        let name = reader.read_utf8(schema, f);
        output.set_reference(&name, schema, f);
        name.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        reader.finish(schema, f);
        self.completion().set_normal(&output, f);
        output.clear(f);
        schema.release_i64_local(present, f);
        response.clear(f);
        code.0.clear(f);
        value.clear(f);
        record.clear(f);
        Ok(())
    }
}
