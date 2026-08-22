use super::*;

pub(super) struct OrdinaryPropertyReferenceMetadata {
    base_value_info: ValueInfo,
    is_array_prototype_value: bool,
    has_array_prototype_shape: bool,
    static_setter: Option<FunctionId>,
}

impl<'a> ScriptLowerer<'a> {
    /// Lower the two evaluated operands and strictness which jointly identify
    /// one ordinary property Reference. The plan is non-cloneable; the cloned
    /// key is returned only for conservative shape invalidation.
    pub(super) fn lower_ordinary_property_reference_plan(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
    ) -> (
        OrdinaryPropertyReferencePlan,
        PropertyKeyIr,
        OrdinaryPropertyReferenceMetadata,
    ) {
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
        let static_setter = match &referenced_name {
            PropertyKeyIr::StaticString(name) => {
                match self.read_object_shape_property(&base_and_receiver, name) {
                    Some(ObjectShapeProperty::Accessor {
                        setter: Some(setter),
                        ..
                    }) => Some(setter.function_id),
                    Some(ObjectShapeProperty::Data(_))
                    | Some(ObjectShapeProperty::Accessor { setter: None, .. })
                    | None => None,
                }
            }
            PropertyKeyIr::StringExpr(_)
            | PropertyKeyIr::ArrayIndex(_)
            | PropertyKeyIr::ArrayLength => None,
        };
        let metadata = OrdinaryPropertyReferenceMetadata {
            base_value_info: base_and_receiver.value_info(),
            is_array_prototype_value: self.is_builtin_property_expr(
                &base_and_receiver,
                ARRAY_NAME,
                "prototype",
            ),
            has_array_prototype_shape: Self::has_array_prototype_shape(&base_and_receiver),
            static_setter,
        };
        let plan = OrdinaryPropertyReferencePlan::new(
            base_and_receiver,
            referenced_name.clone(),
            self.reference_strictness(),
        );
        (plan, referenced_name, metadata)
    }

    /// Lower a source-level plain assignment into one retained ordinary
    /// property Reference. Base and raw key are lowered before the RHS; the
    /// carrier leaves ToObject, ToPropertyKey, and Set to its backend consumer.
    pub(super) fn lower_ordinary_property_plain_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        rhs: &Expression,
    ) -> TypedExpr {
        let (plan, referenced_name, metadata) = self.lower_ordinary_property_reference_plan(access);
        let rhs_value = self.lower_expression(rhs);

        if metadata.is_array_prototype_value
            || metadata.has_array_prototype_shape
            || self.is_constructor_prototype_property_expr(
                &Expression::PropertyAccess(PropertyAccess::Simple(access.clone())),
                ARRAY_NAME,
                "toString",
            )
        {
            self.array_prototype_mutated = true;
        }
        let access_expression = Expression::PropertyAccess(PropertyAccess::Simple(access.clone()));
        if self.is_number_prototype_property_expr(&access_expression, "toString")
            && self.is_object_prototype_property_expr(rhs, "toString")
        {
            self.number_prototype_to_string_deleted = true;
        }
        if self.is_number_prototype_property_expr(&access_expression, "match") {
            self.number_prototype_match_is_string_match =
                self.is_string_prototype_property_expr(rhs, "match");
        }
        if self.is_number_prototype_property_expr(&access_expression, "split") {
            self.number_prototype_split_is_string_split =
                self.is_string_prototype_property_expr(rhs, "split");
        }
        if let PropertyKeyIr::StaticString(name) = &referenced_name {
            if self.is_global_this_expr(access.target()) {
                self.set_global_property_value_info_with_source(
                    name.clone(),
                    rhs_value.value_info(),
                    GlobalPropertySource::GlobalWrite,
                );
            }
        }
        if let Some(setter) = metadata.static_setter {
            self.merge_function_this_info(&setter, metadata.base_value_info);
        }

        self.update_written_shape(access.target(), &referenced_name, &rhs_value.value_info());
        plan.plain_assignment(rhs_value)
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
        let (plan, referenced_name, _) = self.lower_ordinary_property_reference_plan(access);
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

    fn returned_plain_assignment<'a>(
        script: &'a ScriptIr,
        function_name: &str,
    ) -> &'a OrdinaryPropertyAssignmentIr {
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
        let ExprIr::OrdinaryPropertyAssignment(assignment) = &value.expr else {
            panic!(
                "expected fused ordinary property plain assignment, got {:?}",
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

    #[test]
    fn ordinary_property_plain_assignment_retains_base_key_rhs_and_strictness() {
        let program = lower(
            r#"
            function computed(base, key, rhs) {
                "use strict";
                return base()[key()] = rhs();
            }
            function named(base, rhs) {
                return base.prop = rhs();
            }
            function nullBase(rhs) {
                let base = null;
                return base.prop = rhs();
            }
            function undefinedBase(rhs) {
                let base = undefined;
                return base.prop = rhs();
            }
            "#,
        );
        let script = program.script.as_ref().expect("script IR should exist");

        let computed = returned_plain_assignment(script, "computed");
        assert!(matches!(
            &computed.base_and_receiver().expr,
            ExprIr::CallIndirect { .. }
        ));
        assert!(matches!(
            computed.referenced_name(),
            PropertyKeyIr::StringExpr(key)
                if matches!(&key.expr, ExprIr::CallIndirect { .. })
        ));
        assert!(matches!(&computed.rhs().expr, ExprIr::CallIndirect { .. }));
        assert_eq!(computed.strictness(), Strictness::Strict);

        let named = returned_plain_assignment(script, "named");
        assert!(matches!(
            named.referenced_name(),
            PropertyKeyIr::StaticString(name) if name == "prop"
        ));
        assert!(matches!(&named.rhs().expr, ExprIr::CallIndirect { .. }));
        assert_eq!(named.strictness(), Strictness::Sloppy);

        let null_base = returned_plain_assignment(script, "nullBase");
        assert_eq!(null_base.base_and_receiver().kind, ValueKind::Null);
        assert!(matches!(&null_base.rhs().expr, ExprIr::CallIndirect { .. }));

        let undefined_base = returned_plain_assignment(script, "undefinedBase");
        assert_eq!(
            undefined_base.base_and_receiver().kind,
            ValueKind::Undefined
        );
        assert!(matches!(
            &undefined_base.rhs().expr,
            ExprIr::CallIndirect { .. }
        ));
    }
}
