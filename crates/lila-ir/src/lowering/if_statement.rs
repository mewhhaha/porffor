mod generator;

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
        let generator_branch_yields =
            contains(if_statement.body(), ContainsSymbol::YieldExpression)
                || if_statement
                    .else_node()
                    .is_some_and(|branch| contains(branch, ContainsSymbol::YieldExpression));
        if condition_prefix.is_empty()
            && !(generator_entry_state.is_some() && generator_branch_yields)
        {
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
        let structured_generator_if = generator_entry_state.is_some()
            && generator_branch_yields
            && (self.generator_structured_depth > 0
                || simple_generator_if_branch_yield_count(if_statement.body()).is_none()
                || if_statement.else_node().is_some_and(|branch| {
                    simple_generator_if_branch_yield_count(branch).is_none()
                }));
        if structured_generator_if {
            self.current_generator_resume_state = Some(
                generator_entry_state.expect("structured generator branch has an entry state") + 1,
            );
        }
        if let Some(entry_state) = async_entry_state {
            let Some(then_entry_state) = entry_state.checked_add(1) else {
                self.unsupported("async conditional continuation state overflow");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            self.current_async_resume_state = Some(then_entry_state);
        }
        let before = self.capture_conditional_flow_facts();
        if structured_generator_if {
            self.generator_structured_depth += 1;
        }
        let (then_branch, then_kind) = self.lower_statement(if_statement.body());
        let generator_then_exit_state = structured_generator_if
            .then_some(self.current_generator_resume_state)
            .flatten();
        if let Some(then_exit_state) = generator_then_exit_state {
            self.current_generator_resume_state = Some(then_exit_state + 1);
        }
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
        if structured_generator_if {
            self.generator_structured_depth -= 1;
        }

        if let (Some(entry_state), Some(then_exit_state)) =
            (generator_entry_state, generator_then_exit_state)
        {
            let else_exit_state = self
                .current_generator_resume_state
                .expect("structured generator else retains a state");
            let Some(plan) = GeneratorStructuredIfPlanIr::new(
                entry_state,
                then_exit_state,
                else_exit_state,
                then_branch,
                else_branch,
            ) else {
                self.unsupported("generator branch continuation state overflow");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            self.current_generator_resume_state = Some(plan.exit_state());
            return (
                StatementIr::GeneratorStructuredIf { condition, plan },
                result_kind,
            );
        }

        if let Some(entry_state) = generator_entry_state {
            if let Some(generator_if) = self.finish_simple_generator_if(
                &condition,
                &then_branch,
                else_branch.as_deref(),
                result_kind,
                entry_state,
            ) {
                return generator_if;
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
