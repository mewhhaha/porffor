use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(in crate::lowering) fn lower_expression_statement(
        &mut self,
        expression: &Expression,
    ) -> (StatementIr, ValueKind) {
        if self.async_generator_entry_state().is_some()
            && (contains(expression, ContainsSymbol::AwaitExpression)
                || contains(expression, ContainsSymbol::YieldExpression))
        {
            let Some((mut prefix, value)) = self.lower_mixed_generator_value(expression) else {
                self.unsupported(
                    "async-generator expression suspension requires its checked mixed value owner",
                );
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            let kind = value.kind;
            prefix.push(StatementIr::Expression(value));
            return (StatementIr::LexicalBlock(prefix), kind);
        }
        if self.plain_async_entry_state().is_some() {
            if let Expression::Assign(assignment) = Self::unwrap_parenthesized_expr(expression) {
                if matches!(assignment.lhs(), AssignTarget::Identifier(_))
                    && contains(assignment.rhs(), ContainsSymbol::AwaitExpression)
                {
                    let Some((mut statements, value)) =
                        self.lower_async_prefixed_expression(expression)
                    else {
                        self.unsupported(
                            "awaited Identifier assignment requires its original Reference owner",
                        );
                        return (StatementIr::Empty, ValueKind::Undefined);
                    };
                    statements.push(StatementIr::Expression(value));
                    return (StatementIr::LexicalBlock(statements), ValueKind::Undefined);
                }
            }
            if let Expression::Assign(assignment) = Self::unwrap_parenthesized_expr(expression) {
                if assignment.op() == AssignOp::Assign {
                    if let AssignTarget::Pattern(pattern) = assignment.lhs() {
                        if contains(pattern, ContainsSymbol::AwaitExpression) {
                            let Some((mut statements, value)) = self
                                .lower_staged_async_pattern_assignment(pattern, assignment.rhs())
                            else {
                                self.unsupported("async pattern assignment continuation ownership");
                                return (StatementIr::Empty, ValueKind::Undefined);
                            };
                            let kind = value.kind;
                            statements.push(StatementIr::Expression(value));
                            return (StatementIr::LexicalBlock(statements), kind);
                        }
                    }
                }
            }
        }
        match expression {
            Expression::Await(await_expression) if self.current_async_resume_state.is_some() => {
                self.lower_linear_async_await(await_expression.target(), AsyncResumeModeIr::Ignore)
            }
            Expression::Assign(assignment)
                if self.current_async_resume_state.is_some()
                    && assignment.op() == AssignOp::Assign
                    && !matches!(
                        assignment.lhs(),
                        AssignTarget::Access(PropertyAccess::Simple(_))
                    )
                    && matches!(assignment.rhs(), Expression::Await(_)) =>
            {
                let Expression::Await(await_expression) = assignment.rhs() else {
                    unreachable!()
                };
                let AssignTarget::Identifier(identifier) = assignment.lhs() else {
                    self.unsupported("async await assignment target");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let mode = if let Some(binding) = self.lookup_binding(&name) {
                    AsyncResumeModeIr::AssignIdentifier(binding.storage_name)
                } else {
                    if self.uses_runtime_identifier_environment()
                        || !self.with_environment_chain.is_empty()
                    {
                        self.unsupported(
                            "suspended assignment through a runtime identifier environment",
                        );
                        return (StatementIr::Empty, ValueKind::Undefined);
                    }
                    AsyncResumeModeIr::AssignGlobal {
                        name,
                        strictness: self.reference_strictness(),
                    }
                };
                self.lower_linear_async_await(await_expression.target(), mode)
            }
            Expression::Yield(yield_expression)
                if self.current_resumable_plan.is_some()
                    && yield_expression.target().is_some_and(|target| {
                        contains(target, ContainsSymbol::AwaitExpression)
                    }) =>
            {
                let target = yield_expression
                    .target()
                    .expect("guarded yielded await target");
                let Some((mut statements, value)) = self.lower_async_prefixed_expression(target)
                else {
                    self.unsupported(
                        "conditionally reached or mixed suspension in async-generator yield target",
                    );
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                let (yield_statement, kind) = self.lower_linear_generator_yield_value(
                    value,
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::Ignore,
                );
                statements.push(yield_statement);
                (StatementIr::LexicalBlock(statements), kind)
            }
            expression
                if self.current_async_resume_state.is_some()
                    && contains(expression, ContainsSymbol::AwaitExpression) =>
            {
                if contains_async_property_assignment(expression) {
                    let Some((mut statements, value)) =
                        self.lower_async_prefixed_expression(expression)
                    else {
                        self.unsupported("conditionally reached or mixed suspension in async property assignment");
                        return (StatementIr::Empty, ValueKind::Undefined);
                    };
                    statements.push(StatementIr::Expression(value));
                    return (StatementIr::LexicalBlock(statements), ValueKind::Undefined);
                }
                let Some((mut statements, value)) =
                    self.lower_async_prefixed_expression(expression)
                else {
                    self.unsupported(
                        "conditionally reached or mixed suspension in async expression statement",
                    );
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                statements.push(StatementIr::Expression(value));
                (StatementIr::LexicalBlock(statements), ValueKind::Undefined)
            }
            Expression::Assign(assignment)
                if matches!(
                    self.generator_value_branch_admission(),
                    GeneratorValueBranchAdmission::OrdinaryOutsideLoops
                ) && assignment.op() == AssignOp::Assign
                    && matches!(assignment.lhs(), AssignTarget::Identifier(_))
                    && contains(assignment.rhs(), ContainsSymbol::YieldExpression) =>
            {
                let AssignTarget::Identifier(identifier) = assignment.lhs() else {
                    unreachable!("guarded plain Identifier target")
                };
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let Some((mut statements, value)) =
                    self.lower_staged_generator_identifier_assignment(name, assignment.rhs())
                else {
                    self.unsupported("plain generator Identifier assignment suspension");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                let kind = value.kind;
                statements.push(StatementIr::Expression(value));
                (StatementIr::LexicalBlock(statements), kind)
            }
            Expression::Assign(assignment)
                if GeneratorPatternAssignmentSource::new(
                    assignment,
                    self.generator_value_branch_admission(),
                )
                .is_some() =>
            {
                let source = GeneratorPatternAssignmentSource::new(
                    assignment,
                    self.generator_value_branch_admission(),
                )
                .expect("guarded ordinary eager assignment pattern");
                let Some((mut statements, value)) =
                    self.lower_staged_generator_pattern_assignment(source)
                else {
                    self.unsupported("ordinary generator pattern assignment suspension");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                let kind = value.kind;
                statements.push(StatementIr::Expression(value));
                (StatementIr::LexicalBlock(statements), kind)
            }
            Expression::Assign(assignment)
                if self.current_generator_resume_state.is_some()
                    && assignment.op() == AssignOp::Assign
                    && matches!(assignment.rhs(), Expression::TemplateLiteral(template) if contains(template, ContainsSymbol::YieldExpression)) =>
            {
                let AssignTarget::Identifier(identifier) = assignment.lhs() else {
                    self.unsupported("generator template assignment target");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                let Expression::TemplateLiteral(template) = assignment.rhs() else {
                    unreachable!()
                };
                let target_name = self.interner.resolve_expect(identifier.sym()).to_string();
                let Some(statements) =
                    self.lower_generator_template_assignment(target_name, template)
                else {
                    self.unsupported("generator template interpolation suspension");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                (StatementIr::LexicalBlock(statements), ValueKind::String)
            }
            expression
                if self.current_generator_resume_state.is_some()
                    && !matches!(expression, Expression::Yield(_))
                    && !matches!(
                        expression,
                        Expression::Assign(assignment)
                            if assignment.op() == AssignOp::Assign
                                && matches!(assignment.rhs(), Expression::Yield(_))
                                && !contains(assignment.lhs(), ContainsSymbol::YieldExpression)
                    )
                    && contains(expression, ContainsSymbol::YieldExpression) =>
            {
                let Some(mut statements) = self.lower_discarded_generator_expression(expression)
                else {
                    self.unsupported("discarded generator expression suspension");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                if statements.len() == 1 {
                    return (statements.remove(0), ValueKind::Undefined);
                }
                (StatementIr::LexicalBlock(statements), ValueKind::Undefined)
            }
            Expression::Yield(yield_expression)
                if self.current_generator_resume_state.is_some()
                    && yield_expression.target().is_some_and(|target| {
                        contains(target, ContainsSymbol::YieldExpression)
                    }) =>
            {
                let Some((mut statements, value)) = yield_expression
                    .target()
                    .and_then(|target| self.lower_staged_generator_expression(target))
                else {
                    self.unsupported("generator expression suspension");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                let (yield_statement, kind) = self.lower_linear_generator_yield_value(
                    value,
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::Ignore,
                );
                statements.push(yield_statement);
                (StatementIr::LexicalBlock(statements), kind)
            }
            Expression::Yield(yield_expression)
                if self.current_generator_resume_state.is_some() =>
            {
                self.lower_linear_generator_yield(
                    yield_expression.target(),
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::Ignore,
                )
            }
            Expression::Assign(assignment)
                if self.current_generator_resume_state.is_some()
                    && assignment.op() == AssignOp::Assign
                    && matches!(assignment.rhs(), Expression::Yield(_)) =>
            {
                let Expression::Yield(yield_expression) = assignment.rhs() else {
                    unreachable!()
                };
                let resume_mode = match assignment.lhs() {
                    AssignTarget::Identifier(identifier) => {
                        let name = self.interner.resolve_expect(identifier.sym()).to_string();
                        if let Some(binding) = self.lookup_binding(&name) {
                            GeneratorResumeModeIr::AssignIdentifier(binding.storage_name)
                        } else {
                            if self.uses_runtime_identifier_environment()
                                || !self.with_environment_chain.is_empty()
                            {
                                self.unsupported(
                                    "suspended assignment through a runtime identifier environment",
                                );
                                return (StatementIr::Empty, ValueKind::Undefined);
                            }
                            GeneratorResumeModeIr::AssignGlobal {
                                name,
                                strictness: self.reference_strictness(),
                            }
                        }
                    }
                    AssignTarget::Access(PropertyAccess::Simple(access)) => {
                        self.record_caller_flow_invalidation();
                        let (plan, key, _) = self.lower_ordinary_property_reference_plan(access);
                        self.update_written_shape(
                            access.target(),
                            &key,
                            &ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: FunctionTargetKnowledge::unknown(),
                            },
                        );
                        GeneratorResumeModeIr::AssignProperty(plan.suspended_assignment())
                    }
                    AssignTarget::Access(PropertyAccess::Private(_) | PropertyAccess::Super(_))
                    | AssignTarget::Pattern(_)
                    | AssignTarget::WebCompatCall(_) => {
                        self.unsupported("generator yield assignment target");
                        return (StatementIr::Empty, ValueKind::Undefined);
                    }
                };
                self.lower_linear_generator_yield(
                    yield_expression.target(),
                    yield_expression.delegate(),
                    resume_mode,
                )
            }
            expression => {
                let lowered = self.lower_expression(expression);
                let kind = lowered.kind;
                (StatementIr::Expression(lowered), kind)
            }
        }
    }
}
