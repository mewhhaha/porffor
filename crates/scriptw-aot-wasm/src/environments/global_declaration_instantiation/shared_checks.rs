//! One R body per repeated global declaration operation; caller-owned targets
//! and Realm remain explicit until the full declaration plan is installed.
use super::*;
use crate::runtime_helpers::HelperParameters;

impl FunctionBuilder<'_> {
    pub(super) fn emit_reject_existing_global_lexical(
        &mut self,
        key: &GcLocal<StringValue>,
        environment: &GcLocal<Environment>,
        realm: &GcLocal<RealmRecord>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                crate::runtime_helpers::GlobalLexicalDeclarationCheckArguments::new(
                    key,
                    environment,
                    realm,
                    self.current_environment(),
                    self.current_program(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(self.completion(), function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    pub(super) fn emit_can_declare_global(
        &mut self,
        admission: GlobalDeclarationAdmission,
        object: &ValueLocals,
        key: &PropertyKeyLocals,
        admitted: I32Local,
        realm: &GcLocal<RealmRecord>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match admission {
            GlobalDeclarationAdmission::Function => {
                self.emit_can_declare_global_inline(admission, object, key, admitted, function)
            }
            GlobalDeclarationAdmission::Var => {
                self.runtime_schema()
                    .call_helper(
                        crate::runtime_helpers::GlobalVarDeclarationCheckArguments::new(
                            object,
                            key,
                            realm,
                            self.current_environment(),
                            self.current_program(),
                        ),
                        self.runtime_helper_base()?,
                        function,
                    )
                    .store(self.completion(), function);
                self.emit_propagate_current_throw_if_needed(function);
                self.completion().value().scalar().load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                admitted.store(function);
                Ok(())
            }
        }
    }

    pub(super) fn emit_create_global_var_binding(
        &mut self,
        object: &ValueLocals,
        key: &PropertyKeyLocals,
        deletable: bool,
        realm: &GcLocal<RealmRecord>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let configurable = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(i32::from(deletable)));
        configurable.store(function);
        schema
            .call_helper(
                crate::runtime_helpers::GlobalVarDeclarationInstallArguments::new(
                    object,
                    key,
                    configurable,
                    realm,
                    self.current_environment(),
                    self.current_program(),
                ),
                self.runtime_helper_base()?,
                function,
            )
            .store(self.completion(), function);
        schema.release_i32_local(configurable, function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    pub(crate) fn compile_global_lexical_declaration_check_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::GlobalLexicalDeclarationCheck);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::GlobalLexicalDeclarationCheckParameters>(
                &mut function,
            );
        let schema = self.runtime_schema();
        let entry = schema
            .reserve_gc_local::<NamedBinding, Nullable>(&mut function)
            .initialize_null(schema, &mut function);
        // RuntimeOperation is not source code: consume the caller's typed root
        // directly, without the source-frame lookup's early-return condition.
        self.emit_find_own_named_binding(
            &parameters.global_environment,
            &parameters.key,
            &entry,
            &mut function,
        );
        entry.load(schema, &mut function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, &mut function);
        self.emit_global_declaration_failure(
            GlobalDeclarationFailure::ExistingLexical,
            &mut function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion()
            .set_kind(CompletionKind::Normal, &mut function);
        self.completion().value().set_undefined(&mut function);
        self.completion().emit(&mut function);
        entry.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn compile_global_var_declaration_check_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::GlobalVarDeclarationCheck);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::GlobalVarDeclarationCheckParameters>(
                &mut function,
            );
        let schema = self.runtime_schema();
        let admitted = schema.reserve_i32_local(&mut function);
        self.emit_can_declare_global_inline(
            GlobalDeclarationAdmission::Var,
            &parameters.object,
            &parameters.key,
            admitted,
            &mut function,
        )?;
        self.completion()
            .set_kind(CompletionKind::Normal, &mut function);
        self.completion()
            .value()
            .set_boolean(admitted, &mut function);
        self.completion().emit(&mut function);
        schema.release_i32_local(admitted, &mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn compile_global_var_declaration_install_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::GlobalVarDeclarationInstall);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::GlobalVarDeclarationInstallParameters>(
                &mut function,
            );
        self.completion()
            .set_kind(CompletionKind::Normal, &mut function);
        self.completion().value().set_undefined(&mut function);
        self.emit_create_global_var_binding_inline(
            &parameters.object,
            &parameters.key,
            parameters.deletable,
            &mut function,
        )?;
        self.completion().emit(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
