use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_with_environment_has_binding(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        name_local: u32,
        present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = RuntimeHelperId::WithEnvironmentHasBinding.index(
            self.heap_alloc_function_index
                .expect("With HasBinding requires the object runtime"),
        );
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalGet(name_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        self.emit_outlined_object_read_realm_argument(function);
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(payload_local, tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(present_local));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn compile_with_environment_has_binding_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::WithEnvironmentHasBinding);
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_with_environment_has_binding_inline(0, 1, 2, self.result_local, &mut function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_with_environment_has_binding_inline(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        name_local: u32,
        present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let exclusions_local = self.reserve_temp_local();
        let exclusions_tag_local = self.reserve_temp_local();
        let blocked_local = self.reserve_temp_local();
        let blocked_tag_local = self.reserve_temp_local();
        self.emit_object_has_property_i32(
            object_local,
            object_tag_local,
            name_local,
            present_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.unscopables"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            exclusions_local,
            exclusions_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(exclusions_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            exclusions_local,
            exclusions_tag_local,
            exclusions_local,
            exclusions_tag_local,
            name_local,
            blocked_local,
            blocked_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(blocked_tag_local, blocked_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(present_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(blocked_tag_local);
        self.release_temp_local(blocked_local);
        self.release_temp_local(exclusions_tag_local);
        self.release_temp_local(exclusions_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
