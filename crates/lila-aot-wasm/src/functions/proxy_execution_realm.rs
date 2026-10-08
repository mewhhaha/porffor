use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_proxy_execution_realm_type_error(
        &mut self,
        message: RuntimeErrorMessage,
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The common error producer consumes actual callable captures or the
        // helper's validated caller environment, then the real dynamic Realm.
        self.emit_throw_runtime_error(
            lila_ir::NativeErrorKind::TypeError,
            message,
            result,
            function,
        )
    }
}
