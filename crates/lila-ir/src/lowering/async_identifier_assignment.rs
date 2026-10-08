//! One original Identifier Reference is captured before an awaited RHS.

use super::generator_identifier_reference::RetainedGeneratorIdentifierTarget;
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_async_identifier_assignment(
        &mut self,
        op: AssignOp,
        name: String,
        rhs: &Expression,
    ) -> Option<TypedExpr> {
        if self.plain_async_entry_state().is_none()
            || self.async_expression_prefix.is_none()
            || !contains(rhs, ContainsSymbol::AwaitExpression)
            || contains(rhs, ContainsSymbol::YieldExpression)
        {
            return None;
        }
        let mut capture = Vec::new();
        if op == AssignOp::Assign {
            let target =
                RetainedGeneratorIdentifierTarget::capture_write_only(self, &mut capture, name);
            self.async_expression_prefix.as_mut()?.extend(capture);
            let value = self.lower_expression(rhs);
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
            return Some(target.put_value(self, value));
        }
        let operation = match op {
            AssignOp::Add => EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Add),
            AssignOp::Sub => EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Sub),
            AssignOp::Mul => EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Mul),
            AssignOp::Div => EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Div),
            AssignOp::Mod => EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Mod),
            AssignOp::Exp => EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Exp),
            AssignOp::And => EagerCompoundAssignmentOp::Bitwise(BitwiseOp::And),
            AssignOp::Or => EagerCompoundAssignmentOp::Bitwise(BitwiseOp::Or),
            AssignOp::Xor => EagerCompoundAssignmentOp::Bitwise(BitwiseOp::Xor),
            AssignOp::Shl => EagerCompoundAssignmentOp::Bitwise(BitwiseOp::Shl),
            AssignOp::Shr => EagerCompoundAssignmentOp::Bitwise(BitwiseOp::Shr),
            AssignOp::Ushr => EagerCompoundAssignmentOp::Bitwise(BitwiseOp::UShr),
            AssignOp::Assign | AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
                return None
            }
        };
        let reference = RetainedGeneratorIdentifierReference::capture(self, &mut capture, name);
        self.async_expression_prefix.as_mut()?.extend(capture);
        let value = self.lower_expression(rhs);
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        let old = reference.old_value();
        let value = match operation {
            EagerCompoundAssignmentOp::Arithmetic(op) => self.combine_arithmetic(op, old, value),
            EagerCompoundAssignmentOp::Bitwise(op) => {
                let op = match op {
                    BitwiseOp::And => BitwiseBinaryOp::And,
                    BitwiseOp::Or => BitwiseBinaryOp::Or,
                    BitwiseOp::Xor => BitwiseBinaryOp::Xor,
                    BitwiseOp::Shl => BitwiseBinaryOp::Shl,
                    BitwiseOp::Shr => BitwiseBinaryOp::Shr,
                    BitwiseOp::UShr => BitwiseBinaryOp::UShr,
                };
                self.combine_bitwise(op, old, value)
            }
        };
        Some(reference.put_value(self, value))
    }

    /// A var initializer is a write through its original resolved Reference,
    /// while its declaration and hoisting still belong to the variable record.
    pub(super) fn lower_async_identifier_initializer(
        &mut self,
        name: String,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let entry = self.plain_async_entry_state()?;
        CheckedAsyncPrefixSource::new(rhs, self)?;
        let previous = self.async_expression_prefix.replace(Vec::new());
        let value = self.lower_async_identifier_assignment(AssignOp::Assign, name, rhs);
        let prefix = std::mem::replace(&mut self.async_expression_prefix, previous)?;
        let value = value?;
        (crate::async_switch::sequence_exit(&prefix, entry).ok() == self.plain_async_entry_state())
            .then_some((prefix, value))
    }
}
