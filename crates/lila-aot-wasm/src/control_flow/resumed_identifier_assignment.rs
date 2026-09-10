use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_resumed_binding_assignment(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
            return self.emit_global_property_write(name, payload_local, tag_local, function);
        }
        let storage = self.lookup_binding(name).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
            ))
        })?;
        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)
    }

    pub(crate) fn emit_resumed_global_assignment(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        strictness: Strictness,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.with_reference_strictness(strictness, function, |emitter, function| {
            emitter.emit_global_property_write_checked(
                name,
                payload_local,
                tag_local,
                strictness,
                function,
            )
        })?;
        self.emit_propagate_current_completion_if_throw(function);
        Ok(())
    }
}
