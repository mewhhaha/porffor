use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_if_statement(&mut self, if_statement: &If) -> (StatementIr, ValueKind) {
        let generator_entry_state = self.current_generator_resume_state;
        let (mut condition_prefix, condition) = if self.plain_async_entry_state().is_some() {
            self.lower_async_prefixed_expression(if_statement.cond())
                .unwrap_or_else(|| (Vec::new(), self.lower_expression(if_statement.cond())))
        } else {
            (Vec::new(), self.lower_expression(if_statement.cond()))
        };
        if condition_prefix.is_empty() {
            if let Some(value) = Self::static_bool_expr(&condition) {
                if value {
                    return self.lower_statement(if_statement.body());
                }
                return match if_statement.else_node() {
                    Some(else_node) => self.lower_statement(else_node),
                    None => (StatementIr::Empty, ValueKind::Undefined),
                };
            }
        }
        let async_entry_state = self.plain_async_entry_state();
        if let Some(entry_state) = async_entry_state {
            let Some(then_entry_state) = entry_state.checked_add(1) else {
                self.unsupported("async conditional continuation state overflow");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            self.current_async_resume_state = Some(then_entry_state);
        }
        let before = self.capture_conditional_flow_facts();
        let (then_branch, then_kind) = self.lower_statement(if_statement.body());
        let async_then_exit_state = self.plain_async_entry_state();
        if let Some(then_exit_state) = async_then_exit_state {
            let Some(else_entry_state) = then_exit_state.checked_add(1) else {
                self.unsupported("async conditional continuation state overflow");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            self.current_async_resume_state = Some(else_entry_state);
        }
        let then_facts = self.capture_conditional_flow_facts();
        let (else_branch, result_kind) = match if_statement.else_node() {
            Some(else_node) => {
                self.install_conditional_flow_facts(before);
                let (else_branch, else_kind) = self.lower_statement(else_node);
                let else_facts = self.capture_conditional_flow_facts();
                self.merge_conditional_flow_facts(then_facts, else_facts);
                let kind = if then_kind == else_kind {
                    then_kind
                } else if Self::statement_completes_by_throw(&then_branch) {
                    else_kind
                } else if Self::statement_completes_by_throw(&else_branch) {
                    then_kind
                } else {
                    self.merge_value_kinds(then_kind, else_kind)
                };
                (Some(Box::new(else_branch)), kind)
            }
            None => {
                self.merge_conditional_flow_facts(then_facts, before);
                (None, ValueKind::Undefined)
            }
        };

        if let Some(entry_state) = generator_entry_state {
            let (then_before_yield, then_yield_statement, then_after_yield) =
                Self::split_generator_if_branch(then_branch.clone());
            let (else_before_yield, else_yield_statement, else_after_yield) = else_branch
                .as_deref()
                .cloned()
                .map(Self::split_generator_if_branch)
                .unwrap_or_default();
            if then_yield_statement.is_some() || else_yield_statement.is_some() {
                let then_resume_state = then_yield_statement.as_ref().and_then(|statement| {
                    let StatementIr::GeneratorYield { resume_state, .. } = statement else {
                        return None;
                    };
                    Some(*resume_state)
                });
                let else_resume_state = else_yield_statement.as_ref().and_then(|statement| {
                    let StatementIr::GeneratorYield { resume_state, .. } = statement else {
                        return None;
                    };
                    Some(*resume_state)
                });
                let exit_state = self.current_generator_resume_state.unwrap_or(entry_state) + 1;
                if let Some(plan) = self.current_resumable_plan.as_mut() {
                    for suspension in plan
                        .suspension_points
                        .iter_mut()
                        .skip(self.next_resumable_suspension_index)
                    {
                        suspension.suspend_state += 1;
                        suspension.resume_state += 1;
                    }
                    plan.state_count += 1;
                    self.current_async_resume_state = Some(exit_state);
                }
                self.current_generator_resume_state = Some(exit_state);
                return (
                    StatementIr::GeneratorIf {
                        condition,
                        then_before_yield,
                        then_yield_statement: then_yield_statement.map(Box::new),
                        then_after_yield,
                        else_before_yield,
                        else_yield_statement: else_yield_statement.map(Box::new),
                        else_after_yield,
                        entry_state,
                        then_resume_state,
                        else_resume_state,
                        exit_state,
                    },
                    result_kind,
                );
            }
        }

        let async_plan = if let Some(entry_state) = async_entry_state {
            let then_exit_state = async_then_exit_state
                .expect("plain async then branch retains its continuation counter");
            let else_exit_state = self
                .plain_async_entry_state()
                .expect("plain async else branch retains its continuation counter");
            match AsyncFunctionIfPlanIr::new(entry_state, then_exit_state, else_exit_state) {
                Ok(plan) => {
                    self.current_async_resume_state =
                        Some(plan.map_or(entry_state, AsyncFunctionIfPlanIr::exit_state));
                    plan
                }
                Err(error) => {
                    self.unsupported_with_message(format!(
                        "unsupported in lila wasm-aot: invalid async conditional continuation plan: {error:?}",
                    ));
                    return (StatementIr::Empty, ValueKind::Undefined);
                }
            }
        } else {
            None
        };
        let statement = match async_plan {
            Some(plan) => StatementIr::AsyncFunctionIf {
                condition,
                then_branch: Box::new(then_branch),
                else_branch,
                plan,
            },
            None => StatementIr::If {
                condition,
                then_branch: Box::new(then_branch),
                else_branch,
            },
        };
        if condition_prefix.is_empty() {
            (statement, result_kind)
        } else {
            condition_prefix.push(statement);
            (StatementIr::LexicalBlock(condition_prefix), result_kind)
        }
    }

    fn split_generator_if_branch(
        branch: StatementIr,
    ) -> (Vec<StatementIr>, Option<StatementIr>, Vec<StatementIr>) {
        let statements = match branch {
            StatementIr::Block(block) if block.lexical_environment.is_none() => block.statements,
            statement => vec![statement],
        };
        let Some(yield_index) = statements
            .iter()
            .position(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
        else {
            return (statements, None, Vec::new());
        };
        let mut before_yield = statements;
        let after_yield = before_yield.split_off(yield_index + 1);
        let yield_statement = before_yield.pop();
        (before_yield, yield_statement, after_yield)
    }

    fn statement_completes_by_throw(statement: &StatementIr) -> bool {
        match statement {
            StatementIr::Throw(_) => true,
            StatementIr::Block(block) if block.statements.len() == 1 => {
                Self::statement_completes_by_throw(&block.statements[0])
            }
            _ => false,
        }
    }
}
