use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn compile_return_position_expr(
        &mut self,
        value: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match &value.expr {
            ExprIr::CallIndirect {
                direct_eval,
                callee,
                this_arg,
                args,
                static_regexp_compilation: None,
            } if self.emit_tail_indirect_call(
                callee,
                this_arg.as_deref(),
                args,
                direct_eval.as_ref(),
                function,
            )? => {}
            ExprIr::EnvironmentIdentifier(identifier) => {
                self.compile_environment_identifier_to_locals(
                    identifier,
                    &crate::functions::CallContinuation::Return,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_from_result_locals(function);
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expr_to_locals(
                    condition,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_truthy_tagged_i32(self.result_tag_local, self.result_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_return_position_expr(then_expr, function)?;
                function.instruction(&Instruction::Else);
                self.compile_return_position_expr(else_expr, function)?;
                function.instruction(&Instruction::End);
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                match op {
                    LogicalBinaryOp::Coalesce => {
                        self.compile_nullish_tagged_i32(self.result_tag_local, function)?;
                    }
                    LogicalBinaryOp::And | LogicalBinaryOp::Or => {
                        self.compile_truthy_tagged_i32(
                            self.result_tag_local,
                            self.result_local,
                            function,
                        )?;
                    }
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And | LogicalBinaryOp::Coalesce => {
                        self.compile_return_position_expr(rhs, function)?;
                    }
                    LogicalBinaryOp::Or => self.emit_return_from_result_locals(function),
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And | LogicalBinaryOp::Coalesce => {
                        self.emit_return_from_result_locals(function);
                    }
                    LogicalBinaryOp::Or => {
                        self.compile_return_position_expr(rhs, function)?;
                    }
                }
                function.instruction(&Instruction::End);
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_return_position_expr(rhs, function)?;
            }
            _ => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_from_result_locals(function);
            }
        }
        Ok(())
    }

    fn emit_return_from_result_locals(&self, function: &mut Function) {
        self.set_completion_kind(CompletionKind::Return, function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
    }
}
