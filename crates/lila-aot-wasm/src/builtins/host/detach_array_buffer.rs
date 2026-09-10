use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_host_detach_array_buffer_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_payload_local = self.reserve_temp_local();
        let buffer_tag_local = self.reserve_temp_local();
        let detach_key_payload_local = self.reserve_temp_local();
        let detach_key_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, buffer_payload_local, buffer_tag_local, function);
        self.emit_builtin_arg_to_locals(
            1,
            detach_key_payload_local,
            detach_key_tag_local,
            function,
        );
        self.emit_detach_array_buffer(
            buffer_payload_local,
            buffer_tag_local,
            detach_key_payload_local,
            detach_key_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(detach_key_tag_local);
        self.release_temp_local(detach_key_payload_local);
        self.release_temp_local(buffer_tag_local);
        self.release_temp_local(buffer_payload_local);
        Ok(())
    }
}
