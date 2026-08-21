use super::*;

/// One identifier logical-assignment Reference located before RHS lowering.
/// A proven global value is owned here so RHS lowering cannot silently replace
/// the lhs metadata before the short-circuit read is emitted.
#[must_use = "a pre-RHS logical-assignment Reference must be consumed after RHS lowering"]
pub(super) struct LocatedIdentifierLogicalAssignment {
    reference: LocatedIdentifierReference,
    proven_global_value: Option<ValueInfo>,
}

impl LocatedIdentifierLogicalAssignment {
    pub(super) fn declarative_position(&self) -> Option<DeclarativeEnvironmentPosition> {
        self.reference.declarative_position()
    }

    pub(super) fn is_unproven_global(&self) -> bool {
        matches!(&self.reference, LocatedIdentifierReference::Unresolvable)
            && self.proven_global_value.is_none()
    }

    pub(super) fn reject_definite_tdz(self) -> Result<Self, TypedExpr> {
        let Self {
            reference,
            proven_global_value,
        } = self;
        match reference {
            LocatedIdentifierReference::Declarative {
                resolution: BindingResolution::Uninitialized(violation),
                ..
            } => Err(violation.into_throw()),
            reference => Ok(Self {
                reference,
                proven_global_value,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LogicalAssignmentReachability {
    Definite,
    WithEnvironmentFallback,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn locate_identifier_logical_assignment(
        &self,
        name: &str,
    ) -> LocatedIdentifierLogicalAssignment {
        let reference = self.locate_identifier_reference(name);
        let proven_global_value = matches!(&reference, LocatedIdentifierReference::Unresolvable)
            .then(|| self.lookup_global_property_info(name))
            .flatten()
            .filter(|info| info.proven_present)
            .map(|info| info.value_info.clone());
        LocatedIdentifierLogicalAssignment {
            reference,
            proven_global_value,
        }
    }

    pub(super) fn lower_global_object_environment_logical_assignment(
        &mut self,
        name: String,
        op: LogicalBinaryOp,
        rhs: TypedExpr,
    ) -> TypedExpr {
        if let Some(info) = self.global_properties.get_mut(&name) {
            info.value_info = unknown_runtime_value_info();
            info.proven_present = false;
        }
        let strictness = self.reference_strictness();
        GlobalObjectEnvironmentReferencePlan::new(self.global_this_info(), name, strictness)
            .logical_assignment(op, rhs)
    }

    pub(super) fn lower_located_identifier_logical_assignment(
        &mut self,
        name: String,
        op: LogicalBinaryOp,
        rhs: TypedExpr,
        located: LocatedIdentifierLogicalAssignment,
        reachability: LogicalAssignmentReachability,
    ) -> TypedExpr {
        let LocatedIdentifierLogicalAssignment {
            reference,
            proven_global_value,
        } = located;
        let binding = match reference {
            LocatedIdentifierReference::Declarative {
                resolution: BindingResolution::Uninitialized(violation),
                ..
            } => return violation.into_throw(),
            LocatedIdentifierReference::Declarative {
                resolution: BindingResolution::Initialized(binding),
                ..
            } => Some(binding),
            LocatedIdentifierReference::Unresolvable => None,
            LocatedIdentifierReference::Declarative {
                resolution: BindingResolution::Unresolvable,
                ..
            } => unreachable!("a declarative location cannot be unresolvable"),
        };

        let script_global_binding =
            self.is_script_global_var_name(&name) && !self.has_scope_binding(&name);
        let global_binding = binding.is_none() || script_global_binding;
        if global_binding && reachability == LogicalAssignmentReachability::WithEnvironmentFallback
        {
            if let Some(binding) = &binding {
                self.set_binding_value_info(&name, unknown_runtime_value_info());
                debug_assert_eq!(binding.mode, BindingMode::Var);
            }
            return self.lower_global_object_environment_logical_assignment(name, op, rhs);
        }

        let (lhs, write) = if !global_binding {
            let binding = binding.expect("a non-global located Reference must own a binding");
            let lhs_info = match reachability {
                LogicalAssignmentReachability::Definite => ValueInfo {
                    kind: binding.kind,
                    possible_kinds: binding.possible_kinds,
                    heap_shape: binding.heap_shape.clone(),
                    function_targets: binding.function_targets.clone(),
                },
                LogicalAssignmentReachability::WithEnvironmentFallback => {
                    unknown_runtime_value_info()
                }
            };
            let lhs =
                TypedExpr::from_info(lhs_info, ExprIr::Identifier(binding.storage_name.clone()));
            let write = if binding.mode == BindingMode::Const {
                self.immutable_binding_write(&binding.storage_name, rhs)
            } else {
                let result_info = match reachability {
                    LogicalAssignmentReachability::Definite => {
                        self.merge_value_infos(lhs.value_info(), rhs.value_info())
                    }
                    LogicalAssignmentReachability::WithEnvironmentFallback => {
                        unknown_runtime_value_info()
                    }
                };
                self.set_binding_value_info(&name, result_info);
                TypedExpr::from_info(
                    rhs.value_info(),
                    ExprIr::AssignIdentifier {
                        name: binding.storage_name,
                        value: Box::new(rhs),
                    },
                )
            };
            (lhs, write)
        } else {
            let global_info = binding
                .as_ref()
                .map(|binding| ValueInfo {
                    kind: binding.kind,
                    possible_kinds: binding.possible_kinds,
                    heap_shape: binding.heap_shape.clone(),
                    function_targets: binding.function_targets.clone(),
                })
                .or(proven_global_value)
                .unwrap_or_else(unknown_runtime_value_info);
            let lhs = TypedExpr::from_info(
                global_info,
                ExprIr::GlobalPropertyRead { name: name.clone() },
            );
            let result_info = self.merge_value_infos(lhs.value_info(), rhs.value_info());
            if binding.is_some() {
                self.set_binding_value_info(&name, result_info.clone());
            }
            if let Some(info) = self.global_properties.get_mut(&name) {
                info.value_info = unknown_runtime_value_info();
                info.proven_present = false;
            }
            let strictness = self.reference_strictness();
            let write = TypedExpr::from_info(
                rhs.value_info(),
                ExprIr::GlobalPropertyWrite {
                    name,
                    value: Box::new(rhs),
                    implicit: false,
                    strictness,
                },
            );
            (lhs, write)
        };

        let result_info = match reachability {
            LogicalAssignmentReachability::Definite => {
                self.merge_value_infos(lhs.value_info(), write.value_info())
            }
            LogicalAssignmentReachability::WithEnvironmentFallback => unknown_runtime_value_info(),
        };
        TypedExpr::from_info(
            result_info,
            ExprIr::LogicalShortCircuit {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(write),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};

    #[test]
    fn object_environment_logical_assignment_retains_pre_rhs_proven_global_info() {
        let source = parse("x = 1; x ||= (x = 'rhs');", ParseOptions::script())
            .expect("script should parse");
        let program = lower(&source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR should exist");
        let logical = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Expression(expression)
                    if matches!(&expression.expr, ExprIr::LogicalShortCircuit { .. }) =>
                {
                    Some(expression)
                }
                _ => None,
            })
            .expect("logical assignment should use short-circuit IR");
        let ExprIr::LogicalShortCircuit {
            op: LogicalBinaryOp::Or,
            lhs,
            rhs,
        } = &logical.expr
        else {
            panic!("expected ||= lowering");
        };
        assert_eq!(lhs.kind, ValueKind::Number);
        assert!(matches!(&lhs.expr, ExprIr::GlobalPropertyRead { name } if name == "x"));
        let ExprIr::GlobalPropertyWrite { name, value, .. } = &rhs.expr else {
            panic!("PutValue must be inside the taken RHS branch");
        };
        assert_eq!(name, "x");
        assert!(matches!(&value.expr, ExprIr::GlobalPropertyWrite { name, .. } if name == "x"));
    }
}
