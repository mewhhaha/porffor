use super::*;

#[must_use = "a saved StatementList value must be restored after normal evaluation"]
pub(crate) struct SavedStatementListValue {
    payload: u32,
    tag: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_statement_result(&self, function: &mut Function, kind: ValueKind) {
        self.emit_undefined_payload(function);
        self.finish_statement_payload(function, kind);
    }

    pub(crate) fn finish_statement_payload(&self, function: &mut Function, kind: ValueKind) {
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
    }

    pub(crate) fn emit_undefined_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
    }

    pub(crate) fn save_current_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(aux_local));
    }

    pub(crate) fn restore_saved_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    // StatementList folds empty completions into its preceding value. Keep that
    // accumulator away from expression temporaries while evaluating statements
    // whose normal completion is empty (ECMA-262 14.2.2).
    pub(crate) fn save_statement_list_value(
        &mut self,
        function: &mut Function,
    ) -> SavedStatementListValue {
        let saved = SavedStatementListValue {
            payload: self.reserve_temp_local(),
            tag: self.reserve_temp_local(),
        };
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(saved.payload));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(saved.tag));
        saved
    }

    pub(crate) fn restore_statement_list_value(
        &mut self,
        saved: SavedStatementListValue,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(saved.payload));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(saved.tag));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(saved.tag);
        self.release_temp_local(saved.payload);
        Ok(())
    }

    pub(super) fn compile_iteration_condition(
        &mut self,
        condition: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved = self.save_statement_list_value(function);
        self.compile_truthy_i32(condition, function)?;
        self.restore_statement_list_value(saved, function)
    }
}
