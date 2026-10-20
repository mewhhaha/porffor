use super::*;

impl ScriptLowerer<'_> {
    /// The operand has completed GetValue before this coercion consumer.
    pub(super) fn combine_unary_value(
        &mut self,
        op: UnaryOp,
        lowered_target: TypedExpr,
    ) -> TypedExpr {
        match op {
            UnaryOp::Plus => {
                self.record_possible_to_primitive_effects(&lowered_target.value_info());
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Number,
                        possible_kinds: KindSet::from_kind(ValueKind::Number),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::UnaryPlus {
                        expr: Box::new(lowered_target),
                    },
                )
            }
            UnaryOp::Minus => {
                if let ExprIr::BigInt(bits) = &lowered_target.expr {
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::BigInt),
                        ExprIr::BigInt(bits.negated()),
                    );
                }
                let primitive = self.to_primitive_info(&lowered_target, ToPrimitiveHint::Number);
                let (has_number, has_bigint) = numeric_domain(primitive.as_ref());
                let mut result_kinds = KindSet::EMPTY;
                if has_number {
                    result_kinds = result_kinds.union(KindSet::from_kind(ValueKind::Number));
                }
                if has_bigint {
                    result_kinds = result_kinds.union(KindSet::from_kind(ValueKind::BigInt));
                }
                if result_kinds == KindSet::EMPTY {
                    result_kinds = KindSet::from_kind(ValueKind::Number);
                }
                TypedExpr::from_info(
                    ValueInfo {
                        kind: result_kinds.as_value_kind(),
                        possible_kinds: result_kinds,
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::UnaryMinusNumeric {
                        expr: Box::new(lowered_target),
                    },
                )
            }
            UnaryOp::Not => TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Boolean,
                    possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                    heap_shape: None,
                    function_targets: FunctionTargetKnowledge::none(),
                },
                ExprIr::LogicalNot {
                    expr: Box::new(lowered_target),
                },
            ),
            UnaryOp::TypeOf => TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::TypeOf {
                    expr: Box::new(lowered_target),
                },
            ),
            UnaryOp::Void => TypedExpr::from_info(
                ValueInfo::undefined(),
                ExprIr::Void {
                    expr: Box::new(lowered_target),
                },
            ),
            UnaryOp::Delete => unreachable!(),
            UnaryOp::Tilde => {
                if let ExprIr::BigInt(value) = &lowered_target.expr {
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::BigInt),
                        ExprIr::BigInt(value.complemented()),
                    );
                }
                self.combine_unary_bitwise(UnaryBitwiseOp::Complement, lowered_target)
            }
        }
    }

    /// Both whole operands precede comparison conversion or property hooks.
    pub(super) fn combine_relational(
        &mut self,
        relational: RelationalOp,
        lhs: TypedExpr,
        rhs: TypedExpr,
    ) -> TypedExpr {
        match relational {
            RelationalOp::LessThan
            | RelationalOp::LessThanOrEqual
            | RelationalOp::GreaterThan
            | RelationalOp::GreaterThanOrEqual => {
                let op = match relational {
                    RelationalOp::LessThan => RelationalBinaryOp::LessThan,
                    RelationalOp::LessThanOrEqual => RelationalBinaryOp::LessThanOrEqual,
                    RelationalOp::GreaterThan => RelationalBinaryOp::GreaterThan,
                    RelationalOp::GreaterThanOrEqual => RelationalBinaryOp::GreaterThanOrEqual,
                    _ => unreachable!(),
                };
                if self
                    .to_primitive_info(&lhs, ToPrimitiveHint::Number)
                    .is_some()
                    && self
                        .to_primitive_info(&rhs, ToPrimitiveHint::Number)
                        .is_some()
                {
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::CompareValue {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                    );
                }
                if lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                    && rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
                {
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::CompareValue {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                    );
                }
                let lhs = match self.coerce_expr_to_number(lhs.clone()) {
                    Some(lhs) => lhs,
                    None => {
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::Boolean),
                            ExprIr::CompareValue {
                                op,
                                lhs: Box::new(lhs),
                                rhs: Box::new(rhs),
                            },
                        );
                    }
                };
                let rhs = match self.coerce_expr_to_number(rhs.clone()) {
                    Some(rhs) => rhs,
                    None => {
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::Boolean),
                            ExprIr::CompareValue {
                                op,
                                lhs: Box::new(lhs),
                                rhs: Box::new(rhs),
                            },
                        );
                    }
                };
                if lhs.kind != ValueKind::Number || rhs.kind != ValueKind::Number {
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::CompareValue {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                    );
                }
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Boolean,
                        possible_kinds: KindSet::from_kind(ValueKind::Boolean),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::CompareNumber {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            RelationalOp::StrictEqual | RelationalOp::StrictNotEqual => {
                let equality = TypedExpr::spec_strict_equality_comparison(lhs, rhs);
                if matches!(relational, RelationalOp::StrictNotEqual) {
                    TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::LogicalNot {
                            expr: Box::new(equality),
                        },
                    )
                } else {
                    equality
                }
            }
            RelationalOp::Equal | RelationalOp::NotEqual => {
                self.record_possible_to_primitive_effects(&lhs.value_info());
                self.record_possible_to_primitive_effects(&rhs.value_info());
                let equality = TypedExpr::spec_is_loosely_equal(lhs, rhs);
                if matches!(relational, RelationalOp::NotEqual) {
                    TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::LogicalNot {
                            expr: Box::new(equality),
                        },
                    )
                } else {
                    equality
                }
            }
            RelationalOp::In => {
                self.invalidate_unknown_user_code_effects();
                // HasProperty's semantic operands are (object, key), but `in`
                // evaluates the key expression before the object expression.
                // Retain its value before evaluating the object; ToPropertyKey
                // still belongs to HasProperty, after the object type check.
                let key_name = self.alloc_temp_binding_name("in.key.");
                let key =
                    TypedExpr::from_info(lhs.value_info(), ExprIr::Identifier(key_name.clone()));
                let body = TypedExpr::spec_has_property(rhs, key);
                TypedExpr::from_info(
                    body.value_info(),
                    ExprIr::MaterializeBinding {
                        name: key_name,
                        value: Box::new(lhs),
                        body: Box::new(body),
                    },
                )
            }
            RelationalOp::InstanceOf => {
                if let Some(function_id) = self.resolve_single_function_target(&rhs) {
                    let Some(signature) = self.function_signatures.get(&function_id) else {
                        return self.unsupported_expr("unsupported comparison operator");
                    };
                    if !signature.callable && !signature.protocol.is_constructable() {
                        return self.unsupported_expr("unsupported comparison operator");
                    }
                }
                self.invalidate_unknown_user_code_effects();
                TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Boolean),
                    ExprIr::InstanceOf {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
        }
    }
}
