use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_while_loop(&mut self, while_loop: &WhileLoop) -> (StatementIr, ValueKind) {
        if self.async_generator_entry_state().is_some() {
            return self.lower_async_generator_classic_while(while_loop);
        }
        if self.plain_generator_entry_state().is_some() {
            return self.lower_ordinary_generator_while(while_loop);
        }
        if let Some(entry_state) = self.plain_async_entry_state() {
            if crate::async_with_source::plain_async_while_uses_eager_body(while_loop) {
                return self.lower_awaited_while_condition(while_loop, entry_state);
            }
            return self.lower_plain_async_classic_loop(
                crate::async_generator_source::PlainAsyncClassicLoopSource::While(while_loop),
            );
        }
        let generator_entry_state = self.current_generator_resume_state;
        self.loop_depth += 1;
        let condition = self.lower_expression(while_loop.condition());
        self.loop_depth -= 1;
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        let (body, body_kind) = self.lower_loop_body(while_loop.body());
        let after_vars = self.var_bindings.clone();
        let after_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        self.global_properties = self.merge_global_properties(&before_globals, &after_globals);
        if let Some(entry_state) = generator_entry_state {
            if let Some((before_suspension, suspension_statement, after_suspension, resume_state)) =
                Self::split_resumable_loop_body(
                    body.clone(),
                    generator_entry_state.is_some() && self.current_resumable_plan.is_none(),
                )
            {
                let exit_state = if self.current_resumable_plan.is_some() {
                    resume_state
                } else {
                    resume_state + 1
                };
                self.current_generator_resume_state = Some(exit_state);
                return (
                    StatementIr::GeneratorLoop {
                        init: None,
                        test: Some(condition),
                        update: None,
                        iteration_environment: ResumableLoopIterationEnvironmentIr::StorageOnly,
                        before_suspension,
                        suspension_statement: Box::new(suspension_statement),
                        after_suspension,
                        entry_state,
                        resume_state,
                        exit_state,
                    },
                    body_kind,
                );
            }
        }
        if generator_entry_state.is_some()
            && contains(while_loop.body(), ContainsSymbol::YieldExpression)
        {
            self.unsupported("generator loop body has no reentrant suspension segment");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        (
            StatementIr::While {
                condition,
                body: Box::new(body),
            },
            body_kind,
        )
    }

    pub(super) fn lower_do_while_loop(
        &mut self,
        do_while: &DoWhileLoop,
    ) -> (StatementIr, ValueKind) {
        if self.async_generator_entry_state().is_some() {
            return self.lower_async_generator_classic_do_while(do_while);
        }
        if self.plain_generator_entry_state().is_some() {
            return self.lower_ordinary_generator_do_while(do_while);
        }
        if self.plain_async_entry_state().is_some() {
            return self.lower_plain_async_classic_loop(
                crate::async_generator_source::PlainAsyncClassicLoopSource::DoWhile(do_while),
            );
        }
        let (body, body_kind) = self.lower_loop_body(do_while.body());
        self.loop_depth += 1;
        let condition = self.lower_expression(do_while.cond());
        self.loop_depth -= 1;
        (
            StatementIr::DoWhile {
                body: Box::new(body),
                condition,
            },
            body_kind,
        )
    }
}
