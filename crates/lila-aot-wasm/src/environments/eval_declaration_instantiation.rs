use super::named_environment::NamedEnvironmentKind;
use super::*;
use crate::gc_types::{FunctionObject, NamedBinding, NamedBindingSchema, StringValue};
use lila_ir::PreparedScriptUnit;

#[must_use]
struct ValidatedEvalDeclarationInstantiation<'a> {
    unit: &'a PreparedScriptUnit,
    variable_environment: GcLocal<Environment, Nullable>,
}

impl FunctionBuilder<'_> {
    /// Strict eval owns its variable cells. Initialize every var declaration
    /// before any body statement, including loop heads that never iterate.
    /// Lexical declarations keep their TDZ, and closures retain these same
    /// planned cells when they escape the eval activation.
    pub(crate) fn emit_initialize_strict_eval_variable_cells(
        &mut self,
        unit: &PreparedScriptUnit,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        value.set_undefined(function);
        for name in &unit.declarations.var_names {
            let slot = unit
                .owned_env_bindings
                .iter()
                .find(|binding| &binding.name == name)
                .map(|binding| binding.slot)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "strict eval var declaration `{name}` lacks its planned owned cell",
                    ))
                })?;
            self.write_env_slot_from_locals(slot, 0, &value, function);
        }
        value.clear(function);
        Ok(())
    }

    pub(crate) fn emit_direct_eval_variable_binding_write(
        &mut self,
        name: &str,
        variable_environment: &GcLocal<Environment, Nullable>,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let saved = schema.reserve_value_local(function);
        saved.copy_from(value, function);
        let parent = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<Environment>()
                    .field(EnvironmentSchema::PARENT)
                    .read(variable_environment, schema, function)
                    .reference(),
                function,
            );
        parent.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_global_property_write(name, &saved, function)?;
        function.instruction(&Instruction::Else);
        let key = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(
                self.emit_interned_string_reference(name, function)?,
                function,
            );
        self.emit_set_named_environment_binding(
            variable_environment,
            &key,
            Strictness::Sloppy,
            &saved,
            function,
        )?;
        key.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        parent.clear(function);
        saved.clear(function);
        Ok(())
    }

    pub(crate) fn emit_eval_variable_environment_to_local(
        &mut self,
        function: &mut Function,
    ) -> GcLocal<Environment, Nullable> {
        let schema = self.runtime_schema();
        let environment = self.resolve_env_handle_local(0, function);
        let parent = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize_null(schema, function);
        let kind = schema.reserve_i32_local(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        parent.replace(
            schema
                .struct_type::<Environment>()
                .field(EnvironmentSchema::PARENT)
                .read(&environment, schema, function)
                .reference(),
            function,
        );
        parent.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::BrIf(1));
        schema
            .struct_type::<Environment>()
            .field(EnvironmentSchema::KIND)
            .read(&environment, schema, function)
            .store(kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(
            NamedEnvironmentKind::Variable.code(),
        ));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::BrIf(1));
        environment.replace(parent.load(schema, function), function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i32_local(kind, function);
        parent.clear(function);
        environment
    }

    pub(crate) fn emit_instantiate_direct_eval_declarations(
        &mut self,
        unit: &PreparedScriptUnit,
        variable_environment: &GcLocal<Environment, Nullable>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let validated =
            self.emit_validate_direct_eval_declarations(unit, variable_environment, function)?;
        self.emit_install_direct_eval_declarations(validated, function)
    }

    fn emit_validate_direct_eval_declarations<'a>(
        &mut self,
        unit: &'a PreparedScriptUnit,
        variable_environment: &GcLocal<Environment, Nullable>,
        function: &mut Function,
    ) -> Result<ValidatedEvalDeclarationInstantiation<'a>, EmitError> {
        let schema = self.runtime_schema();
        let accepted = schema.reserve_i32_local(function);
        let value = schema.reserve_value_local(function);
        for name in unit.declarations.var_names.iter().chain(
            unit.declarations
                .functions_in_reverse_order
                .iter()
                .map(|declaration| &declaration.name),
        ) {
            let key = schema
                .reserve_gc_local::<StringValue, NonNullable>(function)
                .initialize(
                    self.emit_interned_string_reference(name, function)?,
                    function,
                );
            self.emit_eval_variable_name_admission(variable_environment, &key, accepted, function);
            key.clear(function);
            accepted.load(function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_environment_native_error(
                NativeErrorKind::SyntaxError,
                RuntimeErrorMessage::EVAL_DECLARATION_CONFLICTS_WITH_LEXICAL_BINDING,
                &value,
                function,
            )?;
            self.emit_return_current_completion(function);
            self.pop_control(ControlFrameKind::If);
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
            let key = schema
                .reserve_gc_local::<StringValue, NonNullable>(function)
                .initialize(
                    self.emit_interned_string_reference(&candidate.name, function)?,
                    function,
                );
            self.emit_eval_variable_name_admission(variable_environment, &key, accepted, function);
            key.clear(function);
            value.set_boolean(accepted, function);
            self.write_env_slot_from_locals(candidate.admission.slot, 0, &value, function);
        }
        value.clear(function);
        schema.release_i32_local(accepted, function);
        let environment = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(variable_environment.load(schema, function), function);
        Ok(ValidatedEvalDeclarationInstantiation {
            unit,
            variable_environment: environment,
        })
    }

    fn emit_eval_variable_name_admission(
        &mut self,
        variable_environment: &GcLocal<Environment, Nullable>,
        key: &GcLocal<StringValue>,
        accepted: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let environment = self.resolve_env_handle_local(0, function);
        let parent = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize_null(schema, function);
        let kind = schema.reserve_i32_local(function);
        let entry = schema
            .reserve_gc_local::<NamedBinding, Nullable>(function)
            .initialize_null(schema, function);
        function.instruction(&Instruction::I32Const(1));
        accepted.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        parent.replace(
            schema
                .struct_type::<Environment>()
                .field(EnvironmentSchema::PARENT)
                .read(&environment, schema, function)
                .reference(),
            function,
        );
        parent.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        // The captured VariableEnvironment must be on this lexical chain.
        environment.load(schema, function);
        variable_environment.load(schema, function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_global_lexical_entry_to_local(key, &entry, function);
        entry.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        accepted.store(function);
        function.instruction(&Instruction::Br(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<Environment>()
            .field(EnvironmentSchema::KIND)
            .read(&environment, schema, function)
            .store(kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(
            NamedEnvironmentKind::WithObject.code(),
        ));
        function.instruction(&Instruction::I32Ne);
        kind.load(function);
        function.instruction(&Instruction::I32Const(
            NamedEnvironmentKind::SimpleCatch.code(),
        ));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_find_own_named_binding(&environment, key, &entry, function);
        entry.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<NamedBinding>()
            .field(NamedBindingSchema::LEXICAL_CONFLICT)
            .read(&entry, schema, function)
            .store(accepted, function);
        environment.load(schema, function);
        variable_environment.load(schema, function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::I32Eqz);
        accepted.load(function);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        accepted.store(function);
        function.instruction(&Instruction::Br(4));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Const(1));
        accepted.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        environment.load(schema, function);
        variable_environment.load(schema, function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::BrIf(1));
        environment.replace(parent.load(schema, function), function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        entry.clear(function);
        schema.release_i32_local(kind, function);
        parent.clear(function);
        environment.clear(function);
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
        let schema = self.runtime_schema();
        let parent = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<Environment>()
                    .field(EnvironmentSchema::PARENT)
                    .read(&variable_environment, schema, function)
                    .reference(),
                function,
            );
        let value = schema.reserve_value_local(function);
        parent.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_instantiate_global_declaration_plan(unit, function)?;
        function.instruction(&Instruction::Else);
        for candidate in &unit.declarations.annex_b_candidates {
            let initialized =
                self.read_env_slot_to_locals(candidate.admission.slot, 0, &value, function);
            schema.release_i32_local(initialized, function);
            self.compile_truthy_tagged_i32(&value, function)?;
            self.open_frame(ControlFrameKind::If, function);
            let key = schema
                .reserve_gc_local::<StringValue, NonNullable>(function)
                .initialize(
                    self.emit_interned_string_reference(&candidate.name, function)?,
                    function,
                );
            value.set_undefined(function);
            self.emit_create_eval_variable_binding(&variable_environment, &key, &value, function)?;
            key.clear(function);
            self.pop_control(ControlFrameKind::If);
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
            let key = schema
                .reserve_gc_local::<StringValue, NonNullable>(function)
                .initialize(
                    self.emit_interned_string_reference(&declaration.name, function)?,
                    function,
                );
            let callable = schema
                .reserve_gc_local::<FunctionObject, NonNullable>(function)
                .initialize(self.emit_function_value_payload(&meta, function)?, function);
            value.set_reference(&callable, schema, function);
            callable.clear(function);
            self.emit_create_eval_variable_binding(&variable_environment, &key, &value, function)?;
            self.emit_set_named_environment_binding(
                &variable_environment,
                &key,
                Strictness::Sloppy,
                &value,
                function,
            )?;
            key.clear(function);
        }
        for name in &unit.declarations.var_names {
            let key = schema
                .reserve_gc_local::<StringValue, NonNullable>(function)
                .initialize(
                    self.emit_interned_string_reference(name, function)?,
                    function,
                );
            value.set_undefined(function);
            self.emit_create_eval_variable_binding(&variable_environment, &key, &value, function)?;
            key.clear(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.clear(function);
        parent.clear(function);
        variable_environment.clear(function);
        Ok(())
    }
}
