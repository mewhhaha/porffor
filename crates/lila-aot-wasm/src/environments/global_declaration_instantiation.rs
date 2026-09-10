use super::global_environment::{
    GLOBAL_ENV_LEXICAL_COUNT_OFFSET, GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET, GLOBAL_LEXICAL_CELL_OFFSET,
    GLOBAL_LEXICAL_ENTRY_SIZE, GLOBAL_LEXICAL_KEY_OFFSET, GLOBAL_LEXICAL_MUTABLE_OFFSET,
};
use super::*;
use lila_ir::{GlobalLexicalBindingModeIr, PreparedScriptKind, PreparedScriptUnit};

#[derive(Clone, Copy)]
enum GlobalDeclarationAdmission {
    Function,
    Var,
}

#[derive(Clone, Copy)]
enum GlobalDeclarationFailure {
    ExistingLexical,
    RestrictedProperty,
    FunctionNotDefinable,
    VarNotDefinable,
}

#[must_use]
struct ValidatedGlobalDeclarationInstantiation<'a> {
    unit: &'a PreparedScriptUnit,
    environment_local: u32,
    object_local: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_instantiate_prepared_script_declarations(
        &mut self,
        unit: &PreparedScriptUnit,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(&unit.kind, PreparedScriptKind::DirectEval(_)) {
            return if unit.strict {
                Ok(())
            } else {
                self.emit_instantiate_direct_eval_declarations(unit, 7, function)
            };
        }
        if !unit.has_global_variable_environment() {
            return Ok(());
        }
        self.emit_instantiate_global_declaration_plan(unit, function)
    }

    pub(crate) fn emit_instantiate_global_declaration_plan(
        &mut self,
        unit: &PreparedScriptUnit,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let environment_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        self.emit_source_global_environment_to_local(environment_local, function);
        self.emit_execution_global_object_payload(function);
        function.instruction(&Instruction::LocalSet(object_local));
        let validated = self.emit_validate_global_declarations(
            unit,
            environment_local,
            object_local,
            function,
        )?;
        self.emit_install_global_declarations(validated, function)?;
        self.release_temp_local(object_local);
        self.release_temp_local(environment_local);
        Ok(())
    }

    fn emit_validate_global_declarations<'a>(
        &mut self,
        unit: &'a PreparedScriptUnit,
        environment_local: u32,
        object_local: u32,
        function: &mut Function,
    ) -> Result<ValidatedGlobalDeclarationInstantiation<'a>, EmitError> {
        let plan = &unit.declarations;
        let kind = &unit.kind;
        for candidate in &plan.annex_b_candidates {
            self.binding_scopes
                .last_mut()
                .expect("prepared Script binding scope")
                .insert(
                    candidate.admission.name.clone(),
                    BindingStorage::EnvSlot {
                        slot: candidate.admission.slot,
                        hops: 0,
                    },
                );
        }
        let object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let admitted_local = self.reserve_temp_local();
        let fact = self.reserve_own_descriptor_fact_locals();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        if kind == &PreparedScriptKind::RealmScript {
            for name in &plan.lexical_names_in_source_order {
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_global_lexical_entry_to_local(key_local, entry_local, function);
                function.instruction(&Instruction::LocalGet(entry_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_global_declaration_failure(
                    GlobalDeclarationFailure::ExistingLexical,
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.emit_direct_own_descriptor_fact(
                    object_local,
                    object_tag_local,
                    key_local,
                    key_tag_local,
                    fact,
                    function,
                )?;
                fact.emit_present_i32(function);
                fact.emit_configurable_i32(function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_global_declaration_failure(
                    GlobalDeclarationFailure::RestrictedProperty,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
        }
        for name in plan
            .functions_in_reverse_order
            .iter()
            .map(|declaration| &declaration.name)
            .chain(plan.var_names.iter())
        {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_global_lexical_entry_to_local(key_local, entry_local, function);
            function.instruction(&Instruction::LocalGet(entry_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_global_declaration_failure(
                GlobalDeclarationFailure::ExistingLexical,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        for declaration in &plan.functions_in_reverse_order {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&declaration.name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_can_declare_global(
                GlobalDeclarationAdmission::Function,
                object_local,
                key_local,
                admitted_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(admitted_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_global_declaration_failure(
                GlobalDeclarationFailure::FunctionNotDefinable,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        for name in &plan.var_names {
            if plan
                .functions_in_reverse_order
                .iter()
                .any(|declaration| &declaration.name == name)
            {
                continue;
            }
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_can_declare_global(
                GlobalDeclarationAdmission::Var,
                object_local,
                key_local,
                admitted_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(admitted_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_global_declaration_failure(
                GlobalDeclarationFailure::VarNotDefinable,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        for candidate in &plan.annex_b_candidates {
            if matches!(kind, PreparedScriptKind::DirectEval(_)) {
                self.read_env_slot_to_locals(
                    candidate.admission.slot,
                    0,
                    admitted_local,
                    entry_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(admitted_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&candidate.name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(admitted_local));
            self.emit_global_lexical_entry_to_local(key_local, entry_local, function);
            function.instruction(&Instruction::LocalGet(entry_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_can_declare_global(
                GlobalDeclarationAdmission::Var,
                object_local,
                key_local,
                admitted_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
            function.instruction(&Instruction::LocalSet(entry_local));
            self.write_env_slot_from_locals(
                candidate.admission.slot,
                0,
                admitted_local,
                entry_local,
                function,
            );
            if matches!(kind, PreparedScriptKind::DirectEval(_)) {
                function.instruction(&Instruction::End);
            }
        }
        self.release_own_descriptor_fact_locals(fact);
        self.release_temp_local(admitted_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_tag_local);
        Ok(ValidatedGlobalDeclarationInstantiation {
            unit,
            environment_local,
            object_local,
        })
    }

    fn emit_can_declare_global(
        &mut self,
        admission: GlobalDeclarationAdmission,
        object_local: u32,
        key_local: u32,
        admitted_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_tag_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let fact = self.reserve_own_descriptor_fact_locals();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_direct_own_descriptor_fact(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            fact,
            function,
        )?;
        fact.emit_present_i32(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        match admission {
            GlobalDeclarationAdmission::Var => {
                function.instruction(&Instruction::I64Const(1));
            }
            GlobalDeclarationAdmission::Function => {
                fact.emit_configurable_i32(function);
                fact.emit_accessor_i32(function);
                function.instruction(&Instruction::I32Eqz);
                fact.emit_writable_i32(function);
                function.instruction(&Instruction::I32And);
                fact.emit_enumerable_i32(function);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I64ExtendI32U);
            }
        }
        function.instruction(&Instruction::LocalSet(admitted_local));
        function.instruction(&Instruction::Else);
        self.emit_object_is_extensible_i32(
            object_local,
            object_tag_local,
            admitted_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_own_descriptor_fact_locals(fact);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(object_tag_local);
        Ok(())
    }

    fn emit_global_declaration_failure(
        &mut self,
        failure: GlobalDeclarationFailure,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (name, message) = match failure {
            GlobalDeclarationFailure::ExistingLexical => (
                SYNTAX_ERROR_NAME,
                "global declaration conflicts with existing lexical binding",
            ),
            GlobalDeclarationFailure::RestrictedProperty => (
                SYNTAX_ERROR_NAME,
                "global lexical declaration conflicts with non-configurable property",
            ),
            GlobalDeclarationFailure::FunctionNotDefinable => (
                TYPE_ERROR_NAME,
                "global function declaration is not permitted",
            ),
            GlobalDeclarationFailure::VarNotDefinable => (
                TYPE_ERROR_NAME,
                "global variable declaration is not permitted",
            ),
        };
        self.emit_throw_runtime_error(
            name,
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}

impl FunctionBuilder<'_> {
    fn emit_install_global_declarations(
        &mut self,
        validated: ValidatedGlobalDeclarationInstantiation<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ValidatedGlobalDeclarationInstantiation {
            unit,
            environment_local,
            object_local,
        } = validated;
        let kind = &unit.kind;
        let plan = &unit.declarations;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let deletable = match kind {
            PreparedScriptKind::RealmScript => false,
            PreparedScriptKind::IndirectEval | PreparedScriptKind::DirectEval(_) => true,
        };
        for candidate in &plan.annex_b_candidates {
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
            if !plan
                .functions_in_reverse_order
                .iter()
                .any(|declaration| declaration.name == candidate.name)
                && !plan.var_names.contains(&candidate.name)
            {
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(&candidate.name),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_create_global_var_binding(object_local, key_local, deletable, function)?;
            }
            function.instruction(&Instruction::End);
        }
        if kind == &PreparedScriptKind::RealmScript {
            self.emit_append_global_lexical_bindings(unit, environment_local, function)?;
        }
        for declaration in plan.functions_in_reverse_order.iter().rev() {
            let meta = self
                .functions
                .get(&declaration.function_id)
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "prepared global declaration `{}` lacks function `{}`",
                        declaration.name, declaration.function_id
                    ))
                })?;
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&declaration.name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_create_global_function_binding(
                object_local,
                key_local,
                payload_local,
                tag_local,
                deletable,
                function,
            )?;
        }
        for name in &plan.var_names {
            if plan
                .functions_in_reverse_order
                .iter()
                .any(|declaration| &declaration.name == name)
            {
                continue;
            }
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_create_global_var_binding(object_local, key_local, deletable, function)?;
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    fn emit_append_global_lexical_bindings(
        &mut self,
        unit: &PreparedScriptUnit,
        environment_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let bindings = unit.global_bindings.lexical_bindings();
        if bindings.is_empty() {
            return Ok(());
        }
        let old_entries_local = self.reserve_temp_local();
        let old_count_local = self.reserve_temp_local();
        let new_entries_local = self.reserve_temp_local();
        let new_count_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let word_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET,
            old_entries_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_COUNT_OFFSET,
            old_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(old_count_local));
        function.instruction(&Instruction::I64Const(bindings.len() as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_count_local));
        function.instruction(&Instruction::LocalGet(new_count_local));
        function.instruction(&Instruction::I64Const(GLOBAL_LEXICAL_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(word_local));
        self.emit_heap_alloc_from_local(word_local, function)?;
        function.instruction(&Instruction::LocalSet(new_entries_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(old_count_local));
        function.instruction(&Instruction::I64Const(GLOBAL_LEXICAL_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(old_entries_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(entry_local, 0, word_local, function);
        function.instruction(&Instruction::LocalGet(new_entries_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, 0, word_local, function);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_entries_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        for (index, (name, mode)) in bindings.iter().enumerate() {
            let slot = unit
                .owned_env_bindings
                .iter()
                .find(|binding| &binding.name == name)
                .unwrap_or_else(|| {
                    panic!("prepared global lexical `{name}` must own its analyzed cell")
                })
                .slot;
            let offset = index as u64 * GLOBAL_LEXICAL_ENTRY_SIZE;
            self.store_i64_const_at_offset(
                entry_local,
                offset + GLOBAL_LEXICAL_KEY_OFFSET,
                self.strings.payload(name) as u64,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            function.instruction(&Instruction::I64Const(Self::env_slot_offset(slot, 0) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(word_local));
            self.store_i64_local_at_offset(
                entry_local,
                offset + GLOBAL_LEXICAL_CELL_OFFSET,
                word_local,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                offset + GLOBAL_LEXICAL_MUTABLE_OFFSET,
                match mode {
                    GlobalLexicalBindingModeIr::Mutable => 1,
                    GlobalLexicalBindingModeIr::Immutable => 0,
                },
                function,
            );
        }
        self.store_i64_local_at_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET,
            new_entries_local,
            function,
        );
        self.store_i64_local_at_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_COUNT_OFFSET,
            new_count_local,
            function,
        );
        self.release_temp_local(word_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(new_count_local);
        self.release_temp_local(new_entries_local);
        self.release_temp_local(old_count_local);
        self.release_temp_local(old_entries_local);
        Ok(())
    }

    fn emit_create_global_var_binding(
        &mut self,
        object_local: u32,
        key_local: u32,
        deletable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_tag_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let fact = self.reserve_own_descriptor_fact_locals();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_direct_own_descriptor_fact(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            fact,
            function,
        )?;
        fact.emit_present_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            true,
            deletable,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_own_descriptor_fact_locals(fact);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(object_tag_local);
        Ok(())
    }

    fn emit_create_global_function_binding(
        &mut self,
        object_local: u32,
        key_local: u32,
        payload_local: u32,
        tag_local: u32,
        deletable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_tag_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let writable_local = self.reserve_temp_local();
        let configurable_local = self.reserve_temp_local();
        let fact = self.reserve_own_descriptor_fact_locals();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(writable_local));
        function.instruction(&Instruction::I64Const(i64::from(deletable)));
        function.instruction(&Instruction::LocalSet(configurable_local));
        self.emit_direct_own_descriptor_fact(
            object_local,
            object_tag_local,
            key_local,
            key_tag_local,
            fact,
            function,
        )?;
        fact.emit_present_i32(function);
        fact.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(configurable_local));
        function.instruction(&Instruction::End);
        self.emit_object_define_data_with_flag_locals(
            object_local,
            key_local,
            payload_local,
            tag_local,
            writable_local,
            writable_local,
            configurable_local,
            function,
        )?;
        self.release_own_descriptor_fact_locals(fact);
        self.release_temp_local(configurable_local);
        self.release_temp_local(writable_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(object_tag_local);
        Ok(())
    }
}
