use super::*;

/// The eager identifier compound-assignment domain. Logical assignments have
/// a distinct short-circuit lifecycle and cannot enter the consuming eager
/// Reference path by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EagerCompoundAssignmentOp {
    Arithmetic(ArithmeticOp),
    Bitwise(BitwiseOp),
}

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
        let bindings = WithEnvironmentCompoundAssignmentBindings::allocate(|prefix| {
            self.alloc_temp_binding_name(prefix)
        });
        let old_value = bindings.old_value();
        let applied = Self::apply_eager_compound_assignment(op, old_value, rhs);
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
                return self.lower_with_environment_global_eager_compound_assignment(name, op, rhs);
            }

            let lhs = TypedExpr::from_info(
                unknown_runtime_value_info(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let applied = Self::apply_eager_compound_assignment(op, lhs, rhs);
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

        self.lower_with_environment_global_eager_compound_assignment(name, op, rhs)
    }

    fn lower_with_environment_global_eager_compound_assignment(
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
        let lhs = TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::GlobalPropertyRead { name: name.clone() },
        );
        let applied = Self::apply_eager_compound_assignment(op, lhs, rhs);
        let strictness = self.reference_strictness();
        let write = TypedExpr::from_info(
            applied.value_info(),
            ExprIr::GlobalPropertyWrite {
                name: name.clone(),
                value: Box::new(applied),
                implicit: false,
                strictness,
            },
        );
        self.guard_with_environment_global_get_value(name, write)
    }

    /// The canonical dynamic operation shape used by both a selected Object
    /// Environment Record and its declarative/global fallback. This exhaustive
    /// match keeps logical assignment out and makes adding an eager operation a
    /// compile-time obligation.
    fn apply_eager_compound_assignment(
        op: EagerCompoundAssignmentOp,
        lhs: TypedExpr,
        rhs: TypedExpr,
    ) -> TypedExpr {
        match op {
            EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Add) => {
                let possible_kinds = KindSet::from_kind(ValueKind::String)
                    .union(KindSet::from_kind(ValueKind::Number))
                    .union(KindSet::from_kind(ValueKind::BigInt));
                TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::CoerciveAdd {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            EagerCompoundAssignmentOp::Arithmetic(arithmetic) => {
                let op = match arithmetic {
                    ArithmeticOp::Sub => ArithmeticBinaryOp::Sub,
                    ArithmeticOp::Mul => ArithmeticBinaryOp::Mul,
                    ArithmeticOp::Div => ArithmeticBinaryOp::Div,
                    ArithmeticOp::Mod => ArithmeticBinaryOp::Mod,
                    ArithmeticOp::Exp => ArithmeticBinaryOp::Exp,
                    ArithmeticOp::Add => unreachable!("addition has string-or-numeric semantics"),
                };
                let possible_kinds = KindSet::from_kind(ValueKind::Number)
                    .union(KindSet::from_kind(ValueKind::BigInt));
                TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::CoerciveBinaryNumber {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
            EagerCompoundAssignmentOp::Bitwise(bitwise) => {
                let op = match bitwise {
                    BitwiseOp::And => BitwiseBinaryOp::And,
                    BitwiseOp::Or => BitwiseBinaryOp::Or,
                    BitwiseOp::Xor => BitwiseBinaryOp::Xor,
                    BitwiseOp::Shl => BitwiseBinaryOp::Shl,
                    BitwiseOp::Shr => BitwiseBinaryOp::Shr,
                    BitwiseOp::UShr => BitwiseBinaryOp::UShr,
                };
                let possible_kinds = if matches!(bitwise, BitwiseOp::UShr) {
                    KindSet::from_kind(ValueKind::Number)
                } else {
                    KindSet::from_kind(ValueKind::Number)
                        .union(KindSet::from_kind(ValueKind::BigInt))
                };
                TypedExpr::from_info(
                    ValueInfo {
                        kind: possible_kinds.as_value_kind(),
                        possible_kinds,
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::BitwiseNumeric {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
        }
    }

    /// ResolveBinding may reach the global fallback only after observable
    /// `with` HasBinding/@@unscopables work. Recheck presence before GetValue so
    /// deletion throws and creation is admitted at run time.
    fn guard_with_environment_global_get_value(
        &self,
        name: String,
        present_value: TypedExpr,
    ) -> TypedExpr {
        let present = TypedExpr::spec_has_property(
            TypedExpr::from_info(
                self.global_this_info(),
                ExprIr::Identifier(GLOBAL_THIS_NAME.to_string()),
            ),
            TypedExpr::from_info(ValueInfo::new(ValueKind::String), ExprIr::String(name)),
        );
        let missing = TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                message: "unbound identifier in with scope",
            },
        );
        TypedExpr::from_info(
            present_value.value_info(),
            ExprIr::Conditional {
                condition: Box::new(present),
                then_expr: Box::new(present_value),
                else_expr: Box::new(missing),
            },
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
    fn with_environment_eager_compound_assignment_domain_is_twelve_closed_operations() {
        let arithmetic = [
            (ArithmeticOp::Sub, ArithmeticBinaryOp::Sub),
            (ArithmeticOp::Mul, ArithmeticBinaryOp::Mul),
            (ArithmeticOp::Div, ArithmeticBinaryOp::Div),
            (ArithmeticOp::Mod, ArithmeticBinaryOp::Mod),
            (ArithmeticOp::Exp, ArithmeticBinaryOp::Exp),
        ];
        let add = ScriptLowerer::apply_eager_compound_assignment(
            EagerCompoundAssignmentOp::Arithmetic(ArithmeticOp::Add),
            operand("old"),
            operand("rhs"),
        );
        assert!(matches!(add.expr, ExprIr::CoerciveAdd { .. }));
        for (source, expected) in arithmetic {
            let applied = ScriptLowerer::apply_eager_compound_assignment(
                EagerCompoundAssignmentOp::Arithmetic(source),
                operand("old"),
                operand("rhs"),
            );
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
            let applied = ScriptLowerer::apply_eager_compound_assignment(
                EagerCompoundAssignmentOp::Bitwise(source),
                operand("old"),
                operand("rhs"),
            );
            assert!(matches!(
                applied.expr,
                ExprIr::BitwiseNumeric { op, .. } if op == expected
            ));
        }
    }
}
