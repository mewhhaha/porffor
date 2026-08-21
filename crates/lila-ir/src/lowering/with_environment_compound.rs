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
            if self.is_script_global_var_name(&name) && !self.has_scope_binding(&name) {
                self.set_binding_value_info(&name, unknown_runtime_value_info());
                return self
                    .lower_global_object_environment_eager_compound_assignment(name, op, rhs);
            }

            let lhs = TypedExpr::from_info(
                unknown_runtime_value_info(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let applied = op.apply(lhs, rhs);
            if binding.mode == BindingMode::Const {
                return self.immutable_binding_write(&storage_name, applied);
            }

            self.set_binding_value_info(&name, unknown_runtime_value_info());
            return TypedExpr::from_info(
                applied.value_info(),
                ExprIr::AssignIdentifier {
                    name: storage_name,
                    value: Box::new(applied),
                },
            );
        }

        self.lower_global_object_environment_eager_compound_assignment(name, op, rhs)
    }

    pub(super) fn lower_global_object_environment_eager_compound_assignment(
        &mut self,
        name: String,
        op: EagerCompoundAssignmentOp,
        rhs: TypedExpr,
    ) -> TypedExpr {
        if let Some(info) = self.global_properties.get_mut(&name) {
            info.value_info = unknown_runtime_value_info();
            if info.configurable {
                info.proven_present = false;
            }
        }
        let bindings = EagerCompoundAssignmentBindings::allocate(|prefix| {
            self.alloc_temp_binding_name(prefix)
        });
        let applied = op.apply(bindings.old_value(), rhs);
        let strictness = self.reference_strictness();
        GlobalObjectEnvironmentReferencePlan::new(self.global_this_info(), name, strictness)
            .compound_assignment(bindings.seal(applied))
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
