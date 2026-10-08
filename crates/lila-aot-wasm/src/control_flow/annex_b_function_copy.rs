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
        let schema = self.runtime_schema();
        let saved = self.save_statement_list_value(function);
        let value = schema.reserve_value_local(function);
        if let Some(admission) = admission {
            let storage = self
                .lookup_binding(&admission.name)
                .expect("prepared Annex B admission has an owned Boolean slot");
            self.read_binding_to_locals(storage, &value, function)?;
            self.compile_truthy_tagged_i32(&value, function)?;
            self.open_frame(ControlFrameKind::If, function);
        }
        let source = self.lookup_binding(block_storage_name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "Annex B declaration `{source_name}` is missing block binding `{block_storage_name}`"
            ))
        })?;
        self.read_binding_to_locals(source, &value, function)?;
        match target {
            AnnexBFunctionCopyTargetIr::OwnerBinding { storage_name } => {
                // B.3.2.1 writes the VariableEnvironment directly. A same-named
                // catch/lexical binding between block and owner is untouched.
                let target = self.lookup_owner_binding(storage_name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "Annex B declaration `{source_name}` is missing owner binding `{storage_name}`"
                    ))
                })?;
                self.write_binding_from_locals(target, &value, function);
            }
            AnnexBFunctionCopyTargetIr::DirectEvalVariable { name } => {
                let variable_environment = self
                    .body_entry_locals()
                    .and_then(|entry| entry.variable_environment())
                    .expect("Annex B eval copy belongs to a prepared source activation");
                let variable_environment = schema
                    .reserve_gc_local::<Environment, Nullable>(function)
                    .initialize(variable_environment.load(schema, function), function);
                self.emit_direct_eval_variable_binding_write(
                    name,
                    &variable_environment,
                    &value,
                    function,
                )?;
                variable_environment.clear(function);
            }
            AnnexBFunctionCopyTargetIr::ScriptGlobal { name } => {
                let target = self.lookup_owner_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "Annex B declaration `{source_name}` is missing script-global binding `{name}`"
                    ))
                })?;
                self.write_binding_from_locals(target, &value, function);
                self.mirror_binding_to_global_object(name, target, function)?;
            }
        }
        if admission.is_some() {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        value.clear(function);
        self.restore_statement_list_value(saved, function)?;
        Ok(())
    }
}
