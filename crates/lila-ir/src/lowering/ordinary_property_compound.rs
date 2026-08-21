use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Lower the two evaluated operands and strictness which jointly identify
    /// one ordinary property Reference. The plan is non-cloneable; the cloned
    /// key is returned only for conservative shape invalidation.
    pub(super) fn lower_ordinary_property_reference_plan(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
    ) -> (OrdinaryPropertyReferencePlan, PropertyKeyIr) {
        let base_and_receiver = Box::new(self.lower_property_target(access.target()));
        let referenced_name = match access.field() {
            PropertyAccessField::Const(name) => {
                let name = self.interner.resolve_expect(name.sym()).to_string();
                if base_and_receiver.kind == ValueKind::Array && name == "length" {
                    PropertyKeyIr::ArrayLength
                } else {
                    PropertyKeyIr::StaticString(name)
                }
            }
            PropertyAccessField::Expr(expression) => self
                .lower_static_property_key(expression)
                .unwrap_or_else(|| {
                    PropertyKeyIr::StringExpr(Box::new(self.lower_expression(expression)))
                }),
        };
        let plan = OrdinaryPropertyReferencePlan::new(
            base_and_receiver,
            referenced_name.clone(),
            self.reference_strictness(),
        );
        (plan, referenced_name)
    }

    /// Lower one ordinary property Reference directly into its fused eager
    /// mutation carrier. The base and raw computed-key expression are lowered
    /// before the RHS; their runtime GetValue/PutValue staging remains owned by
    /// the carrier's single backend consumer.
    pub(super) fn lower_ordinary_property_eager_compound_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        op: EagerCompoundAssignmentOp,
        rhs: &Expression,
    ) -> TypedExpr {
        let (plan, referenced_name) = self.lower_ordinary_property_reference_plan(access);
        let rhs = self.lower_expression(rhs);
        let old_value_binding = self.alloc_temp_binding_name("ordinary.property.compound.old.");
        let result = plan.eager_compound_assignment(old_value_binding, op, rhs);
        self.update_written_shape(access.target(), &referenced_name, &result.value_info());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};

    fn lower(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        crate::lower(&source)
    }

    fn returned_assignment<'a>(
        script: &'a ScriptIr,
        function_name: &str,
    ) -> &'a OrdinaryPropertyEagerCompoundAssignmentIr {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing function {function_name}"));
        let StatementIr::Return(value) = function
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("function should return its assignment")
        else {
            unreachable!("selected statement is a return")
        };
        let ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) = &value.expr else {
            panic!(
                "expected fused ordinary property assignment, got {:?}",
                value.expr
            );
        };
        assignment
    }

    fn applied_lhs(result: &TypedExpr) -> &TypedExpr {
        match &result.expr {
            ExprIr::CoerciveAdd { lhs, .. }
            | ExprIr::CoerciveBinaryNumber { lhs, .. }
            | ExprIr::BitwiseNumeric { lhs, .. } => lhs,
            other => panic!("unexpected eager operation {other:?}"),
        }
    }

    #[test]
    fn ordinary_property_eager_compound_assignment_owns_one_reference() {
        let program = lower(
            r#"
            function add(base, key, rhs) { "use strict"; return base[key] += rhs; }
            function multiply(base, key, rhs) { "use strict"; return base[key] *= rhs; }
            function xor(base, key, rhs) { "use strict"; return base[key] ^= rhs; }
            function exponentiate(base, key, rhs) { "use strict"; return base[key] **= rhs; }
            "#,
        );
        let script = program.script.as_ref().expect("script IR should exist");

        for name in ["add", "multiply", "xor", "exponentiate"] {
            let assignment = returned_assignment(&script, name);
            assert!(matches!(
                &assignment.base_and_receiver().expr,
                ExprIr::Identifier(_)
            ));
            assert!(matches!(
                assignment.referenced_name(),
                PropertyKeyIr::StringExpr(key)
                    if matches!(&key.expr, ExprIr::Identifier(_))
            ));
            assert_eq!(assignment.strictness(), Strictness::Strict);
            assert!(assignment
                .old_value_binding()
                .starts_with("$ordinary.property.compound.old."));
            assert!(matches!(
                &applied_lhs(assignment.result()).expr,
                ExprIr::Identifier(binding) if binding == assignment.old_value_binding()
            ));
        }

        assert!(matches!(
            &returned_assignment(&script, "add").result().expr,
            ExprIr::CoerciveAdd { .. }
        ));
        assert!(matches!(
            &returned_assignment(&script, "multiply").result().expr,
            ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Mul,
                ..
            }
        ));
        assert!(matches!(
            &returned_assignment(&script, "xor").result().expr,
            ExprIr::BitwiseNumeric {
                op: BitwiseBinaryOp::Xor,
                ..
            }
        ));
        assert!(matches!(
            &returned_assignment(&script, "exponentiate").result().expr,
            ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Exp,
                ..
            }
        ));
    }
}
