use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn compile_annex_b_function_copy(
        &mut self,
        source_name: &str,
        block_storage_name: &str,
        target: &AnnexBFunctionCopyTargetIr,
        admission: &Option<OwnedEnvBindingIr>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved = self.save_statement_list_value(function);
        if let Some(admission) = admission {
            let storage = self
                .lookup_binding(&admission.name)
                .expect("prepared Annex B admission has an owned Boolean slot");
            self.read_binding_to_locals(
                storage,
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::If(BlockType::Empty));
        }
        let source = self.lookup_binding(block_storage_name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "Annex B declaration `{source_name}` is missing block binding `{block_storage_name}`"
            ))
        })?;
        self.read_binding_to_locals(source, self.scratch_local, self.result_tag_local, function)?;
        match target {
            AnnexBFunctionCopyTargetIr::OwnerBinding { storage_name } => {
                // B.3.2.1, evaluating f: `fenv.SetMutableBinding(F, fobj,
                // false)` writes the function's VariableEnvironment directly.
                // It does not resolve F through the running LexicalEnvironment,
                // so a same-named binding between the block and the function
                // body — the catch parameter that B.3.4 lets coexist with the
                // var-scoped name, which this builder also registers under its
                // source name — must keep its value.
                let target = self.lookup_owner_binding(storage_name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "Annex B declaration `{source_name}` is missing owner binding `{storage_name}`"
                    ))
                })?;
                self.write_binding_from_locals(
                    target,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
            AnnexBFunctionCopyTargetIr::DirectEvalVariable { name } => {
                self.emit_direct_eval_variable_binding_write(
                    name,
                    7,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
            }
            AnnexBFunctionCopyTargetIr::ScriptGlobal { name } => {
                let target = self.lookup_owner_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "Annex B declaration `{source_name}` is missing script-global binding `{name}`"
                    ))
                })?;
                self.write_binding_from_locals(
                    target,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
                self.mirror_binding_to_global_object(name, target, function)?;
            }
        }
        if admission.is_some() {
            function.instruction(&Instruction::End);
        }
        self.restore_statement_list_value(saved, function)?;
        Ok(())
    }
}
