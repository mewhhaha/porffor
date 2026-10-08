use super::*;

impl ScriptLowerer<'_> {
    /// Var bookkeeping must keep its activation/captured declarative position.
    /// Creating a block binding here would incorrectly cut off the enclosing
    /// with chain for the next occurrence of the same target name.
    pub(super) fn record_destructuring_binding(
        &mut self,
        source_name: String,
        info: BindingInfo,
        target: &DestructuringTargetIr,
    ) {
        match info.mode {
            BindingMode::Var => {
                let value_info = match target {
                    // The selected Object Environment may receive the value
                    // while the fallback variable keeps its previous value.
                    // Getter/default/eval effects can also change that fallback;
                    // no precise rest Array/Object fact survives this Reference.
                    DestructuringTargetIr::ResolvedVarBinding { .. }
                    | DestructuringTargetIr::AssignmentIdentifier(_) => {
                        unknown_runtime_value_info()
                    }
                    DestructuringTargetIr::Binding { .. }
                    | DestructuringTargetIr::AssignmentProperty { .. }
                    | DestructuringTargetIr::AssignmentPrivate { .. }
                    | DestructuringTargetIr::AssignmentSuper { .. }
                    | DestructuringTargetIr::NestedArray(_)
                    | DestructuringTargetIr::NestedObject(_) => ValueInfo {
                        kind: info.kind,
                        possible_kinds: info.possible_kinds,
                        heap_shape: info.heap_shape,
                        function_targets: info.function_targets,
                    },
                };
                self.set_binding_value_info(&source_name, value_info);
            }
            BindingMode::Let | BindingMode::Const => self.declare_binding(source_name, info),
        }
    }

    pub(super) fn destructuring_binding_target(
        &self,
        mode: BindingMode,
        source_name: String,
        storage_name: String,
    ) -> DestructuringTargetIr {
        if mode == BindingMode::Var {
            if self.borrows_direct_eval_variable_environment() {
                return DestructuringTargetIr::AssignmentIdentifier(
                    IdentifierWriteReferenceIr::environment(
                        source_name,
                        self.reference_strictness(),
                    ),
                );
            }
            if self.uses_runtime_identifier_environment() {
                return DestructuringTargetIr::ResolvedVarBinding {
                    name: storage_name,
                    reference: IdentifierWriteReferenceIr::environment(
                        source_name,
                        self.reference_strictness(),
                    ),
                };
            }
            let fallback = self.locate_identifier_reference(&source_name);
            if let Some(objects) = self
                .with_environment_chain
                .select_preceding(fallback.declarative_position())
            {
                let reference = objects
                    .into_reference_plan(source_name, self.reference_strictness())
                    .deferred_var_write(storage_name.clone());
                return DestructuringTargetIr::ResolvedVarBinding {
                    name: storage_name,
                    reference,
                };
            }
        }
        DestructuringTargetIr::Binding {
            mode,
            name: storage_name,
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
            ExprIr::EnvironmentIdentifier(Box::new(EnvironmentIdentifierIr::current(
                name,
                self.reference_strictness(),
                operation,
            ))),
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
        if self
            .analysis
            .module_execution
            .is_private_dispatcher_name(&name)
        {
            return None;
        }
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
