use super::*;

impl FunctionBuilder<'_> {
    /// Runtime identity supplies the capability boundary when property lookup
    /// erased the call site's static realm-evaluation provenance.
    pub(crate) fn compile_host_realm_eval_script_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let source_string_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function);
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            source_payload_local,
            source_tag_local,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_string_local(
            primitive,
            source_string_local,
            function,
        )?;
        self.emit_prepared_script_dispatch(
            PreparedScriptKind::RealmScript,
            source_string_local,
            function,
        )?;
        self.emit_reject_dynamic_source(
            lila_ir::DynamicSourceRuntimeOperation::RealmEvalScript,
            function,
        );
        self.release_temp_local(source_string_local);
        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        Ok(())
    }
}
