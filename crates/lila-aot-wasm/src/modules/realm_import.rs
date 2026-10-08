//! ShadowRealm import jobs use the canonical per-Realm module cache.

use super::*;

impl FunctionBuilder<'_> {
    /// The native importValue owner already converted the specifier and
    /// validated the export name without coercing it.
    /// The returned inner promise performs the load/evaluation jobs; the
    /// namespace stays separate so export `then` is never assimilated.
    pub(crate) fn emit_realm_module_import(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        specifier: &GcLocal<StringValue>,
        namespace: &ValueLocals,
        evaluation: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        namespace.set_undefined(function);
        self.emit_program_hook_dispatch(
            ProgramHook::RealmModuleImport,
            |builder, hook, function| {
                schema
                    .call_hook(
                        hook,
                        RealmModuleImportArguments::new(
                            realm,
                            specifier,
                            builder.current_environment(),
                        ),
                        function,
                    )
                    .store(evaluation, namespace, function);
                Ok(())
            },
            |builder, function| {
                builder.emit_throw_runtime_error(
                    NativeErrorKind::TypeError,
                    RuntimeErrorMessage::MODULE_REQUEST_NOT_IN_COMPILED_CATALOG,
                    evaluation,
                    function,
                )
            },
            function,
        )
    }

    /// The program half: initialize the Realm's module graph, then resolve the
    /// request against the catalogued Realm requests and call the dispatcher.
    pub(crate) fn compile_realm_module_import_hook(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::RealmModuleImport);
        let parameters = self.helper_parameters::<RealmModuleImportParameters>(&mut function);
        self.push_scope();
        let schema = self.runtime_schema();
        let namespace = schema.reserve_value_local(&mut function);
        let evaluation = schema.reserve_completion(&mut function);
        evaluation.initialize(&mut function);
        self.emit_realm_module_import_runtime(
            &parameters.realm,
            &parameters.specifier,
            &namespace,
            &evaluation,
            &mut function,
        )?;
        evaluation.emit(&mut function);
        namespace.emit(&mut function);
        namespace.clear(&mut function);
        evaluation.clear(&mut function);
        self.pop_scope();
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_realm_module_import_runtime(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        specifier: &GcLocal<StringValue>,
        namespace: &ValueLocals,
        evaluation: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        namespace.set_undefined(function);
        let plan =
            self.functions.module_graph().cloned().ok_or_else(|| {
                EmitError::unsupported("Realm import hook requires a module graph")
            })?;
        let schema = self.runtime_schema();
        let done = self.open_frame(ControlFrameKind::Block, function);
        self.emit_module_initialize_in_realm(realm, evaluation, function)?;
        evaluation.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let folding = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        folding.store(function);
        let equal = schema.reserve_i32_local(function);
        for (request, resolution) in plan.realm_requests() {
            let lila_ir::RealmModuleResolutionIr::Loaded(module) = resolution else {
                continue;
            };
            if !request.attributes().is_empty() {
                continue;
            }
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(request.specifier(), function)?,
                function,
            );
            self.emit_gc_string_equality(specifier, &expected, folding, equal, function);
            expected.clear(function);
            equal.load(function);
            self.open_frame(ControlFrameKind::If, function);
            let record = self.module_record_in_realm(*module, realm, function)?;
            let cell = self.module_namespace_cell(
                &record,
                lila_ir::ModuleNamespaceModeIr::Eager,
                function,
            );
            let initialized = self.emit_read_environment_cell(&cell, namespace, function);
            // The same successful initializer published every namespace cell.
            initialized.load(function);
            function.instruction(&Instruction::I32Eqz);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::Unreachable);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            schema.release_i32_local(initialized, function);
            cell.clear(function);
            record.clear(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        schema.release_i32_local(equal, function);
        schema.release_i32_local(folding, function);
        let meta = self
            .functions
            .get(plan.realm_import_dispatcher())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("Realm import dispatcher must have a compiled entry")
            })?;
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::AsyncFunctionPrototype,
            &prototype,
            function,
        );
        let callable = schema.reserve_gc_local(function).initialize(
            self.emit_dynamic_source_function_record(&meta, realm, &prototype, function)?,
            function,
        );
        let callee = schema.reserve_value_local(function);
        callee.set_reference(&callable, schema, function);
        let argument = schema.reserve_value_local(function);
        argument.set_reference(specifier, schema, function);
        let receiver = schema.reserve_value_local(function);
        receiver.set_undefined(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[&argument], function);
        self.emit_function_or_proxy_call_with_argv(
            &callee, &receiver, &arguments, evaluation, function,
        )?;
        arguments.clear(function);
        receiver.clear(function);
        argument.clear(function);
        callee.clear(function);
        callable.clear(function);
        prototype.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
