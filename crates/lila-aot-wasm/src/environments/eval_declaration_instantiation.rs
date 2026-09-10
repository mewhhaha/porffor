use super::named_environment::{NamedEnvironmentKind, NAMED_BINDING_LEXICAL_CONFLICT_OFFSET};
use super::*;
use lila_ir::PreparedScriptUnit;

#[must_use]
struct ValidatedEvalDeclarationInstantiation<'a> {
    unit: &'a PreparedScriptUnit,
    variable_environment: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_direct_eval_variable_binding_write(
        &mut self,
        name: &str,
        variable_environment: u32,
        value_payload: u32,
        value_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let parent = self.reserve_temp_local();
        let key = self.reserve_temp_local();
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(value_payload));
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::LocalGet(value_tag));
        function.instruction(&Instruction::LocalSet(tag));
        self.load_i64_to_local_from_offset(
            variable_environment,
            ENV_PARENT_OFFSET,
            parent,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_global_property_write(name, payload, tag, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key));
        self.emit_set_named_environment_binding(
            variable_environment,
            key,
            Strictness::Sloppy,
            payload,
            tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(tag);
        self.release_temp_local(payload);
        self.release_temp_local(key);
        self.release_temp_local(parent);
        Ok(())
    }

    pub(crate) fn emit_eval_variable_environment_to_local(
        &mut self,
        environment_local: u32,
        function: &mut Function,
    ) {
        let parent_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_PARENT_OFFSET,
            parent_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_RECORD_KIND_OFFSET,
            kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            NamedEnvironmentKind::Variable.code() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(kind_local);
        self.release_temp_local(parent_local);
    }

    pub(crate) fn emit_instantiate_direct_eval_declarations(
        &mut self,
        unit: &PreparedScriptUnit,
        variable_environment: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let validated =
            self.emit_validate_direct_eval_declarations(unit, variable_environment, function)?;
        self.emit_install_direct_eval_declarations(validated, function)
    }

    fn emit_validate_direct_eval_declarations<'a>(
        &mut self,
        unit: &'a PreparedScriptUnit,
        variable_environment: u32,
        function: &mut Function,
    ) -> Result<ValidatedEvalDeclarationInstantiation<'a>, EmitError> {
        let key_local = self.reserve_temp_local();
        let accepted_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        for name in unit.declarations.var_names.iter().chain(
            unit.declarations
                .functions_in_reverse_order
                .iter()
                .map(|declaration| &declaration.name),
        ) {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_eval_variable_name_admission(
                variable_environment,
                key_local,
                accepted_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(accepted_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                "SyntaxError",
                "eval declaration conflicts with lexical binding",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        for candidate in &unit.declarations.annex_b_candidates {
            self.binding_scopes
                .last_mut()
                .expect("eval initial binding scope")
                .insert(
                    candidate.admission.name.clone(),
                    BindingStorage::EnvSlot {
                        slot: candidate.admission.slot,
                        hops: 0,
                    },
                );
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&candidate.name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_eval_variable_name_admission(
                variable_environment,
                key_local,
                accepted_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.write_env_slot_from_locals(
                candidate.admission.slot,
                0,
                accepted_local,
                tag_local,
                function,
            );
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(accepted_local);
        self.release_temp_local(key_local);
        Ok(ValidatedEvalDeclarationInstantiation {
            unit,
            variable_environment,
        })
    }

    fn emit_eval_variable_name_admission(
        &mut self,
        variable_environment: u32,
        key_local: u32,
        accepted_local: u32,
        function: &mut Function,
    ) {
        let environment_local = self.reserve_temp_local();
        let parent_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_PARENT_OFFSET,
            parent_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // The captured VariableEnvironment must be on this lexical chain.
        function.instruction(&Instruction::LocalGet(environment_local));
        function.instruction(&Instruction::LocalGet(variable_environment));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_global_lexical_entry_to_local(key_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_RECORD_KIND_OFFSET,
            kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            NamedEnvironmentKind::WithObject.code() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            NamedEnvironmentKind::SimpleCatch.code() as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_find_own_named_binding(environment_local, key_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            NAMED_BINDING_LEXICAL_CONFLICT_OFFSET,
            accepted_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(environment_local));
        function.instruction(&Instruction::LocalGet(variable_environment));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(accepted_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(environment_local));
        function.instruction(&Instruction::LocalGet(variable_environment));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(entry_local);
        self.release_temp_local(kind_local);
        self.release_temp_local(parent_local);
        self.release_temp_local(environment_local);
    }

    fn emit_install_direct_eval_declarations(
        &mut self,
        validated: ValidatedEvalDeclarationInstantiation<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ValidatedEvalDeclarationInstantiation {
            unit,
            variable_environment,
        } = validated;
        let parent_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            variable_environment,
            ENV_PARENT_OFFSET,
            parent_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_instantiate_global_declaration_plan(unit, function)?;
        function.instruction(&Instruction::Else);
        for candidate in &unit.declarations.annex_b_candidates {
            self.read_env_slot_to_locals(
                candidate.admission.slot,
                0,
                payload_local,
                tag_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&candidate.name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_create_eval_variable_binding(
                variable_environment,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        for declaration in unit.declarations.functions_in_reverse_order.iter().rev() {
            let meta = self
                .functions
                .get(&declaration.function_id)
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "eval declaration `{}` lacks function `{}`",
                        declaration.name, declaration.function_id
                    ))
                })?;
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&declaration.name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_create_eval_variable_binding(
                variable_environment,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_set_named_environment_binding(
                variable_environment,
                key_local,
                Strictness::Sloppy,
                payload_local,
                tag_local,
                function,
            )?;
        }
        for name in &unit.declarations.var_names {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_create_eval_variable_binding(
                variable_environment,
                key_local,
                payload_local,
                tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(parent_local);
        Ok(())
    }
}
