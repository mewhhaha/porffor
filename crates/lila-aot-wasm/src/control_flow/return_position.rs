use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn compile_return_position_expr(
        &mut self,
        expression: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = self.runtime_schema().reserve_value_local(function);
        self.compile_expr_to_value(expression, &value, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        self.completion().value().copy_from(&value, function);
        value.clear(function);
        self.emit_prepare_fresh_return(function)?;
        self.set_completion_kind(CompletionKind::Return, function);
        self.emit_derived_constructor_body_result(function)?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
