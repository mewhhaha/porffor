use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_while_loop(&mut self, while_loop: &WhileLoop) -> (StatementIr, ValueKind) {
        let generator_entry_state = self.current_generator_resume_state;
        let plain_async_entry_state = self.plain_async_entry_state();
        let plain_async_await_loop = plain_async_entry_state.is_some()
            && (contains(while_loop.body(), ContainsSymbol::AwaitExpression)
                || contains(while_loop.condition(), ContainsSymbol::AwaitExpression));
        if plain_async_await_loop
            && (contains(while_loop.condition(), ContainsSymbol::AwaitExpression)
                || generator_loop_has_unsupported_control(while_loop.body(), false))
        {
            self.unsupported(
                "async loop with await requires an eager loop head without break or continue",
            );
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let condition = self.lower_expression(while_loop.condition());
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        let (body, body_kind) = self.lower_loop_body(while_loop.body());
        let after_vars = self.var_bindings.clone();
        let after_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        self.global_properties = self.merge_global_properties(&before_globals, &after_globals);
        if let Some(entry_state) = generator_entry_state.or(if plain_async_await_loop {
            plain_async_entry_state
        } else {
            None
        }) {
            if let Some((before_suspension, suspension_statement, after_suspension)) =
                Self::split_resumable_loop_body(body.clone())
            {
                let (StatementIr::GeneratorYield { resume_state, .. }
                | StatementIr::AsyncAwait { resume_state, .. }) = &suspension_statement
                else {
                    unreachable!("while-loop resumable statement must be a yield or await");
                };
                let resume_state = *resume_state;
                let exit_state = if self.current_resumable_plan.is_some() {
                    resume_state
                } else {
                    resume_state + 1
                };
                if generator_entry_state.is_some() {
                    self.current_generator_resume_state = Some(exit_state);
                } else {
                    self.current_async_resume_state = Some(exit_state);
                }
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
        if plain_async_await_loop {
            self.unsupported("async loop body did not lower to one direct await");
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
        // `StatementIr::DoWhile` has no resumable form: the async driver
        // re-enters the body from the top and the suspension, already past its
        // state guard, never fires again — the loop spins forever. Report it
        // rather than emit that.
        if self.plain_async_entry_state().is_some()
            && (contains(do_while.body(), ContainsSymbol::AwaitExpression)
                || contains(do_while.cond(), ContainsSymbol::AwaitExpression))
        {
            self.unsupported("await inside a do-while loop");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let (body, body_kind) = self.lower_loop_body(do_while.body());
        let condition = self.lower_expression(do_while.cond());
        (
            StatementIr::DoWhile {
                body: Box::new(body),
                condition,
            },
            body_kind,
        )
    }
}
