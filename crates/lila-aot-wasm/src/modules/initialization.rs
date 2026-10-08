//! Reusable graph initialization captures only the selected Realm's global scope.

use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_module_initialize_in_realm(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema()
            .call_helper(
                ModuleInitializeArguments::new(realm),
                self.runtime_helper_base()?,
                function,
            )
            .store(result, function);
        Ok(())
    }

    pub(super) fn emit_module_initialization_runtime(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        result.initialize(function);
        let Some(plan) = self.functions.module_graph() else {
            return Ok(());
        };
        let meta = self
            .functions
            .get(plan.initializer())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("canonical module initializer must have a compiled entry")
            })?;
        let schema = self.runtime_schema();
        let records = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<RealmRecord>()
                .field(RealmRecordSchema::MODULES)
                .read(realm, schema, function)
                .reference(),
            function,
        );
        records.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::FunctionPrototype,
            &prototype,
            function,
        );
        // This producer supplies the selected global environment and a fresh
        // TemplateSource execution; it captures no caller lexical/private scope.
        let callable = schema.reserve_gc_local(function).initialize(
            self.emit_dynamic_source_function_record(&meta, realm, &prototype, function)?,
            function,
        );
        let callee = schema.reserve_value_local(function);
        callee.set_reference(&callable, schema, function);
        let this_value = schema.reserve_value_local(function);
        this_value.set_undefined(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_function_or_proxy_call_with_argv(
            &callee,
            &this_value,
            &arguments,
            result,
            function,
        )?;
        arguments.clear(function);
        this_value.clear(function);
        callable.clear(function);
        callee.clear(function);
        prototype.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        records.clear(function);
        Ok(())
    }
}
