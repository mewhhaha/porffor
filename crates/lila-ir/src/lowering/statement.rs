use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_statement(&mut self, statement: &Statement) -> (StatementIr, ValueKind) {
        match statement {
            Statement::Expression(Expression::Await(await_expression))
                if self.current_async_resume_state.is_some() =>
            {
                self.lower_linear_async_await(await_expression.target(), AsyncResumeModeIr::Ignore)
            }
            Statement::Expression(Expression::Assign(assignment))
                if self.current_async_resume_state.is_some()
                    && assignment.op() == AssignOp::Assign
                    && matches!(assignment.rhs(), Expression::Await(_)) =>
            {
                let Expression::Await(await_expression) = assignment.rhs() else {
                    unreachable!()
                };
                let AssignTarget::Identifier(identifier) = assignment.lhs() else {
                    self.unsupported("async await assignment target");
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                self.lower_linear_async_await(
                    await_expression.target(),
                    AsyncResumeModeIr::AssignIdentifier(
                        self.interner.resolve_expect(identifier.sym()).to_string(),
                    ),
                )
            }
            Statement::Expression(Expression::Yield(yield_expression))
                if self.current_resumable_plan.is_some()
                    && yield_expression.target().is_some_and(|target| {
                        contains(target, ContainsSymbol::AwaitExpression)
                    }) =>
            {
                let saved = self.async_expression_prefix.replace(Vec::new());
                let (yield_statement, kind) = self.lower_linear_generator_yield(
                    yield_expression.target(),
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::Ignore,
                );
                let mut statements = std::mem::replace(&mut self.async_expression_prefix, saved)
                    .expect("async-generator expression lowering must retain its statement prefix");
                statements.push(yield_statement);
                (StatementIr::LexicalBlock(statements), kind)
            }
            Statement::Expression(expression)
                if self.current_async_resume_state.is_some()
                    && contains(expression, ContainsSymbol::AwaitExpression) =>
            {
                // An expression statement discards its value and has always
                // hoisted every `await` it contains, including ones only some
                // paths reach, so it keeps that reach here. That hoist is
                // wrong for `f(cond && await p)` — the suspension happens even
                // when `cond` is falsy — and fixing it needs a resumable
                // branch rather than a flat prefix, so it stays a known
                // mis-evaluation instead of becoming a new refusal.
                let saved = self.async_expression_prefix.replace(Vec::new());
                let value = self.lower_expression(expression);
                let mut statements = std::mem::replace(&mut self.async_expression_prefix, saved)
                    .expect("async expression lowering must retain its statement prefix");
                statements.push(StatementIr::Expression(value));
                (StatementIr::LexicalBlock(statements), ValueKind::Undefined)
            }
            Statement::Expression(Expression::Assign(assignment))
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
            Statement::Expression(expression)
                if self.current_generator_resume_state.is_some()
                    && !matches!(expression, Expression::Yield(_))
                    && !matches!(
                        expression,
                        Expression::Assign(assignment)
                            if assignment.op() == AssignOp::Assign
                                && matches!(assignment.rhs(), Expression::Yield(_))
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
            Statement::Expression(Expression::Yield(yield_expression))
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
            Statement::Expression(Expression::Yield(yield_expression))
                if self.current_generator_resume_state.is_some() =>
            {
                self.lower_linear_generator_yield(
                    yield_expression.target(),
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::Ignore,
                )
            }
            Statement::Expression(Expression::Assign(assignment))
                if self.current_generator_resume_state.is_some()
                    && assignment.op() == AssignOp::Assign
                    && matches!(assignment.rhs(), Expression::Yield(_)) =>
            {
                let Expression::Yield(yield_expression) = assignment.rhs() else {
                    unreachable!()
                };
                let resume_mode = match assignment.lhs() {
                    AssignTarget::Identifier(identifier) => {
                        GeneratorResumeModeIr::AssignIdentifier(
                            self.interner.resolve_expect(identifier.sym()).to_string(),
                        )
                    }
                    AssignTarget::Access(access) => {
                        let received_value = TypedExpr::from_info(
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            },
                            ExprIr::Undefined,
                        );
                        let assignment = self.lower_property_assign_value(access, received_value);
                        let ExprIr::PropertyWrite {
                            target,
                            key,
                            value: _,
                            strictness,
                        } = assignment.expr
                        else {
                            self.unsupported("generator yield assignment target");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        GeneratorResumeModeIr::AssignProperty(
                            SuspendedPropertyReferenceIr::ordinary(target, key, strictness),
                        )
                    }
                    _ => {
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
            Statement::Expression(expression) => {
                let lowered = self.lower_expression(expression);
                let kind = lowered.kind;
                (StatementIr::Expression(lowered), kind)
            }
            Statement::Empty => (StatementIr::Empty, ValueKind::Undefined),
            Statement::Block(block) => {
                // The Block's declarative Environment Record (14.3.1.2 step 1)
                // is pushed and popped by `lower_block`'s
                // `LexicalScopeInstantiation`; pushing a second, empty frame
                // here would only give the sweep somewhere else it could have
                // landed.
                let block_ir = self.lower_block(block);
                let kind = block_ir.result_kind;
                (StatementIr::Block(block_ir), kind)
            }
            // Statement heads that are evaluated exactly once stage their
            // suspensions ahead of the statement. The guards check that the
            // head is the *only* suspending part, because the prefix runs
            // unconditionally and once: an `await` from a loop body or a
            // branch hoisted out here would run at the wrong time, or only
            // once for a body that runs many times.
            Statement::If(if_statement)
                if self.head_await_is_stageable(
                    if_statement.cond(),
                    [Some(if_statement.body()), if_statement.else_node()],
                ) =>
            {
                self.lower_with_async_head_prefix(|this| this.lower_if_statement(if_statement))
            }
            Statement::If(if_statement) => self.lower_if_statement(if_statement),
            Statement::WhileLoop(while_loop) => self.lower_while_loop(while_loop),
            Statement::DoWhileLoop(do_while) => self.lower_do_while_loop(do_while),
            Statement::ForLoop(for_loop) => self.lower_for_loop(for_loop),
            Statement::ForOfLoop(for_of)
                if self.head_await_is_stageable(for_of.iterable(), [Some(for_of.body())])
                    && !contains(for_of.initializer(), ContainsSymbol::AwaitExpression) =>
            {
                self.lower_with_async_head_prefix(|this| this.lower_for_of_loop(for_of))
            }
            Statement::ForOfLoop(for_of) => self.lower_for_of_loop(for_of),
            Statement::Switch(switch)
                if self.head_await_is_stageable(switch.val(), [])
                    && !switch
                        .cases()
                        .iter()
                        .any(|case| contains(case, ContainsSymbol::AwaitExpression)) =>
            {
                self.lower_with_async_head_prefix(|this| this.lower_switch(switch))
            }
            Statement::Switch(switch) => self.lower_switch(switch),
            Statement::Labelled(labelled) => self.lower_labelled(labelled),
            Statement::Break(brk) => self.lower_break(brk),
            Statement::Continue(cont) => self.lower_continue(cont),
            Statement::Debugger => (StatementIr::Debugger, ValueKind::Undefined),
            Statement::Throw(throw) if self.head_await_is_stageable(throw.target(), []) => {
                self.lower_with_async_head_prefix(|this| this.lower_throw(throw))
            }
            Statement::Throw(throw) => self.lower_throw(throw),
            Statement::Try(try_statement) => self.lower_try(try_statement),
            Statement::Var(var) => self.lower_var_statement(var),
            Statement::Return(ret) => self.lower_return(ret),
            Statement::ForInLoop(for_in)
                if self.head_await_is_stageable(for_in.target(), [Some(for_in.body())])
                    && !contains(for_in.initializer(), ContainsSymbol::AwaitExpression) =>
            {
                self.lower_with_async_head_prefix(|this| this.lower_for_in_loop(for_in))
            }
            Statement::ForInLoop(for_in) => self.lower_for_in_loop(for_in),
            Statement::With(with) => self.lower_with_statement(with),
        }
    }
}
