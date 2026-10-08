use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_with_scoped_identifier_eager_compound_assignment(
        &mut self,
        name: String,
        op: EagerCompoundAssignmentOp,
        rhs: TypedExpr,
        objects: SelectedWithEnvironmentObjects,
        fallback: LocatedIdentifierReference,
    ) -> TypedExpr {
        let plan = self.with_environment_reference_plan(name.clone(), objects);
        let fallback = self.lower_located_identifier_eager_compound_assignment(
            name,
            op,
            rhs.clone(),
            fallback,
        );
        let bindings = EagerCompoundAssignmentBindings::allocate(|prefix| {
            self.alloc_temp_binding_name(prefix)
        });
        let old_value = bindings.old_value();
        let applied = op.apply(old_value, rhs);
        plan.compound_assignment(bindings.seal(applied), fallback)
    }

    /// Lower the already-located declarative/global fallback of a run-time
    /// Object Environment Record selection. The selected `with` observation
    /// can mutate the fallback before this branch runs, so reads and emitted
    /// coercions are deliberately Dynamic and mutable metadata is invalidated.
    fn lower_located_identifier_eager_compound_assignment(
        &mut self,
        name: String,
        op: EagerCompoundAssignmentOp,
        rhs: TypedExpr,
        reference: LocatedIdentifierReference,
    ) -> TypedExpr {
        let binding = match reference {
            LocatedIdentifierReference::Declarative { resolution, .. } => match resolution {
                BindingResolution::Uninitialized(violation) => return violation.into_throw(),
                BindingResolution::Initialized(binding) => Some(binding),
                BindingResolution::Unresolvable => {
                    unreachable!("a declarative location cannot be unresolvable")
                }
            },
            LocatedIdentifierReference::Unresolvable => None,
        };

        if let Some(binding) = binding {
            let storage_name = binding.storage_name.clone();
            if self.is_unshadowed_script_global_binding(&name) {
                self.widen_binding_for_possible_replacement(&name);
                return self.lower_global_identifier_eager_compound_assignment(name, op, rhs);
            }

            let lhs = TypedExpr::from_info(
                unknown_runtime_value_info(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let applied = op.apply(lhs, rhs);
            if binding.mode == BindingMode::Const {
                return self.immutable_binding_write(&storage_name, applied);
            }

            self.widen_binding_for_possible_replacement(&name);
            return TypedExpr::from_info(
                applied.value_info(),
                ExprIr::AssignIdentifier {
                    name: storage_name,
                    value: Box::new(applied),
                },
            );
        }

        self.lower_global_identifier_eager_compound_assignment(name, op, rhs)
    }

    pub(super) fn lower_global_identifier_eager_compound_assignment(
        &mut self,
        name: String,
        op: EagerCompoundAssignmentOp,
        rhs: TypedExpr,
    ) -> TypedExpr {
        // The operator may call ToPrimitive/ToNumeric after the RHS. Do
        // not publish object-property facts: this Global Record can delegate
        // to a lexical installed by Get, the RHS, or coercion.
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        let operation = op.environment_operation();
        let numeric =
            KindSet::from_kind(ValueKind::Number).union(KindSet::from_kind(ValueKind::BigInt));
        let possible_kinds = match operation {
            EnvironmentCompoundOperationIr::Add => {
                numeric.union(KindSet::from_kind(ValueKind::String))
            }
            EnvironmentCompoundOperationIr::Arithmetic(_) => numeric,
            EnvironmentCompoundOperationIr::Bitwise(BitwiseBinaryOp::UShr) => {
                KindSet::from_kind(ValueKind::Number)
            }
            EnvironmentCompoundOperationIr::Bitwise(
                BitwiseBinaryOp::And
                | BitwiseBinaryOp::Or
                | BitwiseBinaryOp::Xor
                | BitwiseBinaryOp::Shl
                | BitwiseBinaryOp::Shr,
            ) => numeric,
        };
        TypedExpr::from_info(
            ValueInfo {
                kind: possible_kinds.as_value_kind(),
                possible_kinds,
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            },
            ExprIr::EnvironmentIdentifier(Box::new(EnvironmentIdentifierIr::global(
                name,
                self.reference_strictness(),
                EnvironmentIdentifierOperationIr::EagerCompound {
                    operation,
                    rhs: Box::new(rhs),
                },
            ))),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operand(name: &str) -> TypedExpr {
        TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::Identifier(name.to_string()),
        )
    }

    #[test]
    fn eager_compound_assignment_domain_is_twelve_closed_operations() {
        let arithmetic = [
            (ArithmeticOp::Sub, ArithmeticBinaryOp::Sub),
            (ArithmeticOp::Mul, ArithmeticBinaryOp::Mul),
            (ArithmeticOp::Div, ArithmeticBinaryOp::Div),
            (ArithmeticOp::Mod, ArithmeticBinaryOp::Mod),
            (ArithmeticOp::Exp, ArithmeticBinaryOp::Exp),
        ];
        let add = EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Add)
            .apply(operand("old"), operand("rhs"));
        assert!(matches!(add.expr, ExprIr::CoerciveAdd { .. }));
        for (source, expected) in arithmetic {
            let applied =
                EagerCompoundAssignmentOp::Arithmetic(source).apply(operand("old"), operand("rhs"));
            assert!(matches!(
                applied.expr,
                ExprIr::CoerciveBinaryNumber { op, .. } if op == expected
            ));
        }

        let bitwise = [
            (BitwiseOp::And, BitwiseBinaryOp::And),
            (BitwiseOp::Or, BitwiseBinaryOp::Or),
            (BitwiseOp::Xor, BitwiseBinaryOp::Xor),
            (BitwiseOp::Shl, BitwiseBinaryOp::Shl),
            (BitwiseOp::Shr, BitwiseBinaryOp::Shr),
            (BitwiseOp::UShr, BitwiseBinaryOp::UShr),
        ];
        for (source, expected) in bitwise {
            let applied =
                EagerCompoundAssignmentOp::Bitwise(source).apply(operand("old"), operand("rhs"));
            assert!(matches!(
                applied.expr,
                ExprIr::BitwiseNumeric { op, .. } if op == expected
            ));
        }
    }
}
