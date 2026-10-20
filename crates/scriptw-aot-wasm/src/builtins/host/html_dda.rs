use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_host_create_html_dda_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&HostBuiltinId::HTMLDDA.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing HTMLDDA host callable"))?;
        let schema = self.runtime_schema();
        let callable = schema
            .reserve_gc_local(f)
            .initialize(self.emit_function_value_payload(&meta, f)?, f);
        self.completion().initialize(f);
        self.completion()
            .value()
            .set_reference(&callable, schema, f);
        callable.clear(f);
        Ok(())
    }

    pub(crate) fn compile_host_html_dda_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.completion().initialize(f);
        self.completion().value().set_scalar(ScalarValue::Null, f);
        Ok(())
    }
}
