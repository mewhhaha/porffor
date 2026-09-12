use super::*;

impl ScriptLowerer<'_> {
    fn retain_generator_operand(
        &mut self,
        statements: &mut Vec<StatementIr>,
        value: TypedExpr,
        hint: &str,
    ) -> TypedExpr {
        let name = self.alloc_suspension_owned_binding(hint, value.value_info());
        statements.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: name.clone(),
            init: value,
        });
        self.lower_identifier_name(name, false)
    }

    pub(super) fn lower_staged_generator_property(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
    ) -> Option<(Vec<StatementIr>, TypedExpr, TypedExpr)> {
        let (mut statements, target) = self.lower_staged_generator_expression(access.target())?;
        let target = self.retain_generator_operand(&mut statements, target, "generator.receiver.");
        let key = match access.field() {
            PropertyAccessField::Const(field) => {
                PropertyKeyIr::StaticString(self.interner.resolve_expect(field.sym()).to_string())
            }
            PropertyAccessField::Expr(expression) => {
                let (prefix, key) = self.lower_staged_generator_expression(expression)?;
                statements.extend(prefix);
                PropertyKeyIr::StringExpr(Box::new(key))
            }
        };
        // Keep the ordinary property Reference operation: computed key
        // evaluation, the nullish check, ToPropertyKey and Get retain their
        // specified order, and a later Call retains the original receiver.
        self.invalidate_unknown_user_code_effects();
        let value = TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::PropertyRead {
                target: Box::new(target.clone()),
                key,
            },
        );
        Some((statements, target, value))
    }

    pub(super) fn lower_staged_generator_call(
        &mut self,
        source_callee: &Expression,
        source_arguments: &[Expression],
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if source_arguments
            .iter()
            .any(|argument| matches!(argument, Expression::Spread(_)))
        {
            return None;
        }
        let callee = Self::unwrap_parenthesized_expr(source_callee);
        let (mut statements, receiver, callee) = match callee {
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                self.lower_staged_generator_property(access)?
            }
            Expression::PropertyAccess(_) | Expression::Optional(_) | Expression::SuperCall(_) => {
                return None;
            }
            Expression::Identifier(identifier)
                if self.interner.resolve_expect(identifier.sym()).to_string() == "eval"
                    || self.uses_runtime_identifier_environment()
                    || !self.with_environment_chain.is_empty() =>
            {
                // These calls need the environment Reference or direct-eval
                // identity dispatch across the suspension, not just GetValue.
                return None;
            }
            _ => {
                let (statements, value) = self.lower_staged_generator_expression(callee)?;
                (statements, TypedExpr::undefined(), value)
            }
        };
        // GetValue of the callee precedes every argument, including arguments
        // that yield. Saving only a property receiver would repeat its Get
        // after resumption and could call a replacement method instead.
        let callee = self.retain_generator_operand(&mut statements, callee, "generator.callee.");
        let mut arguments = Vec::with_capacity(source_arguments.len());
        for argument in source_arguments {
            let (prefix, value) = self.lower_staged_generator_expression(argument)?;
            statements.extend(prefix);
            arguments.push(self.retain_generator_operand(
                &mut statements,
                value,
                "generator.argument.",
            ));
        }
        self.invalidate_unknown_user_code_effects();
        Some((
            statements,
            TypedExpr::spec_call(callee, receiver, arguments),
        ))
    }

    pub(super) fn lower_staged_generator_property_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let (mut statements, target) = self.lower_staged_generator_expression(access.target())?;
        let target = self.retain_generator_operand(&mut statements, target, "generator.receiver.");
        let key = match access.field() {
            PropertyAccessField::Const(field) => {
                PropertyKeyIr::StaticString(self.interner.resolve_expect(field.sym()).to_string())
            }
            PropertyAccessField::Expr(expression) => {
                let (prefix, key) = self.lower_staged_generator_expression(expression)?;
                statements.extend(prefix);
                let key = self.retain_generator_operand(&mut statements, key, "generator.key.");
                PropertyKeyIr::StringExpr(Box::new(key))
            }
        };
        let (prefix, value) = self.lower_staged_generator_expression(rhs)?;
        statements.extend(prefix);

        // A plain assignment retains the raw Reference. Its existing consumer
        // evaluates the RHS before ToObject, ToPropertyKey and Set; using the
        // property-read staging path here would also introduce an unwanted Get.
        let (_, mut possible_setters) = self.possible_unknown_accessor_functions();
        possible_setters.extend_known(self.dynamically_installed_setters.iter().cloned());
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        let reference =
            OrdinaryPropertyReferencePlan::new(Box::new(target), key, self.reference_strictness());
        Some((
            statements,
            reference.plain_assignment(value, possible_setters),
        ))
    }
}
