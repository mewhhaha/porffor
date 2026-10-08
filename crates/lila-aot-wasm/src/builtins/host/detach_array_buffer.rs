use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_host_detach_array_buffer_builtin(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let source = s.reserve_value_local(f);
        let key = s.reserve_value_local(f);
        let result = s.reserve_completion(f);
        self.emit_builtin_arg_to_value(0, &source, f);
        self.emit_builtin_arg_to_value(1, &key, f);
        self.emit_detach_array_buffer(&source, &key, &result, f)?;
        self.completion().copy_from(&result, f);
        result.clear(f);
        key.clear(f);
        source.clear(f);
        Ok(())
    }
}
