use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn destructuring_binding_target(
        &self,
        mode: BindingMode,
        source_name: String,
        storage_name: String,
    ) -> DestructuringTargetIr {
        if mode == BindingMode::Var && self.borrows_direct_eval_variable_environment() {
            DestructuringTargetIr::AssignmentIdentifier(IdentifierWriteReferenceIr::environment(
                source_name,
                self.reference_strictness(),
            ))
        } else {
            DestructuringTargetIr::Binding {
                mode,
                name: storage_name,
            }
        }
    }

    pub(super) fn uses_runtime_identifier_environment(&self) -> bool {
        let owner = &self.analysis.owner_plans[&self.current_owner_id];
        self.analysis.environment_plans[&owner.activation_environment_id].eval_visible
    }

    pub(super) fn environment_identifier(
        &mut self,
        name: String,
        operation: EnvironmentIdentifierOperationIr,
    ) -> TypedExpr {
        if let Some(host) = self.host_surface_policy.resolve_global(&name) {
            self.used_host_builtins.insert(host);
        }
        self.invalidate_unknown_user_code_effects();
        TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::EnvironmentIdentifier(Box::new(EnvironmentIdentifierIr {
                name,
                strictness: self.reference_strictness(),
                operation,
            })),
        )
    }

    pub(super) fn lower_environment_identifier_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Option<TypedExpr> {
        if !self.uses_runtime_identifier_environment() {
            return None;
        }
        let Expression::Identifier(identifier) = Self::unwrap_parenthesized_expr(callee) else {
            return None;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let direct_eval = (name == "eval").then(|| self.direct_eval_context());
        if let Some(context) = &direct_eval {
            self.register_direct_eval_source(context, arguments);
        }
        let args = self
            .lower_call_args_expanding_spread(arguments)
            .into_arguments_without_predecessor();
        Some(self.environment_identifier(
            name,
            EnvironmentIdentifierOperationIr::Call { args, direct_eval },
        ))
    }

    pub(super) fn lower_environment_identifier_assignment(
        &mut self,
        name: String,
        operation: AssignOp,
        rhs: &Expression,
    ) -> TypedExpr {
        let value = Box::new(self.lower_expression(rhs));
        use EnvironmentCompoundOperationIr as Compound;
        use EnvironmentIdentifierOperationIr as Identifier;
        let operation = match operation {
            AssignOp::Assign => Identifier::Assign { value },
            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
                Identifier::LogicalCompound {
                    operation: match operation {
                        AssignOp::BoolAnd => LogicalBinaryOp::And,
                        AssignOp::BoolOr => LogicalBinaryOp::Or,
                        AssignOp::Coalesce => LogicalBinaryOp::Coalesce,
                        _ => unreachable!(),
                    },
                    rhs: value,
                }
            }
            operation => Identifier::EagerCompound {
                operation: match operation {
                    AssignOp::Add => Compound::Add,
                    AssignOp::Sub => Compound::Arithmetic(ArithmeticBinaryOp::Sub),
                    AssignOp::Mul => Compound::Arithmetic(ArithmeticBinaryOp::Mul),
                    AssignOp::Div => Compound::Arithmetic(ArithmeticBinaryOp::Div),
                    AssignOp::Mod => Compound::Arithmetic(ArithmeticBinaryOp::Mod),
                    AssignOp::Exp => Compound::Arithmetic(ArithmeticBinaryOp::Exp),
                    AssignOp::And => Compound::Bitwise(BitwiseBinaryOp::And),
                    AssignOp::Or => Compound::Bitwise(BitwiseBinaryOp::Or),
                    AssignOp::Xor => Compound::Bitwise(BitwiseBinaryOp::Xor),
                    AssignOp::Shl => Compound::Bitwise(BitwiseBinaryOp::Shl),
                    AssignOp::Shr => Compound::Bitwise(BitwiseBinaryOp::Shr),
                    AssignOp::Ushr => Compound::Bitwise(BitwiseBinaryOp::UShr),
                    AssignOp::Assign
                    | AssignOp::BoolAnd
                    | AssignOp::BoolOr
                    | AssignOp::Coalesce => unreachable!(),
                },
                rhs: value,
            },
        };
        self.environment_identifier(name, operation)
    }
}
