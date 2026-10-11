use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_proxy_execution_realm_type_error(
        &mut self,
        message: RuntimeErrorMessage,
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Dispatch helpers receive the selected caller execution Realm explicitly.
        // A Promise job's Realm can differ from the runner's lexical Environment.
        self.emit_throw_runtime_error(
            lila_ir::NativeErrorKind::TypeError,
            message,
            result,
            function,
        )
    }
}
