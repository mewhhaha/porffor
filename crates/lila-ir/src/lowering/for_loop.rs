use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_for_loop(&mut self, for_loop: &ForLoop) -> (StatementIr, ValueKind) {
        let async_disposable_head = Self::async_disposable_for_head(for_loop);
        if async_disposable_head.is_some() {
            let loop_has_suspension = Self::async_disposable_for_has_source_suspension(for_loop);
            if loop_has_suspension {
                self.unsupported("suspension inside an await using classic-for loop");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
        }
        let generator_entry_state = self.current_generator_resume_state;
        let loop_head_has_await = async_disposable_head.is_none()
            && for_loop
                .init()
                .is_some_and(|init| contains(init, ContainsSymbol::AwaitExpression))
            || for_loop
                .condition()
                .is_some_and(|condition| contains(condition, ContainsSymbol::AwaitExpression))
            || for_loop
                .final_expr()
                .is_some_and(|update| contains(update, ContainsSymbol::AwaitExpression));
        let resumable_await_loop = self.current_resumable_plan.is_some()
            && (contains(for_loop.body(), ContainsSymbol::AwaitExpression) || loop_head_has_await);
        if resumable_await_loop
            && (!simple_resumable_await_loop_body_is_supported(for_loop.body())
                || loop_head_has_await)
        {
            self.unsupported(
                "resumable async loop requires one direct body await and an eager loop head without break or continue",
            );
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let plain_async_entry_state = self.plain_async_entry_state();
        let plain_async_await_loop = plain_async_entry_state.is_some()
            && (contains(for_loop.body(), ContainsSymbol::AwaitExpression) || loop_head_has_await);
        if plain_async_await_loop
            && (loop_head_has_await
                || generator_loop_has_unsupported_control(for_loop.body(), false))
        {
            self.unsupported(
                "async loop with await requires an eager loop head without break or continue",
            );
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.push_scope();
        if let Some(ForLoopInitializer::Lexical(lexical)) = for_loop.init() {
            let declaration = lexical.declaration();
            let (mode, list) = match declaration {
                LexicalDeclaration::Let(list) => (BindingMode::Let, list),
                LexicalDeclaration::Const(list)
                | LexicalDeclaration::Using(list)
                | LexicalDeclaration::AwaitUsing(list) => (BindingMode::Const, list),
            };
            for variable in list.as_ref() {
                if let Some(bound_names) = supported_bound_names(self.interner, variable.binding())
                {
                    for bound in bound_names {
                        self.declare_binding(
                            bound.source_name.clone(),
                            BindingInfo::tdz_placeholder(
                                mode,
                                TdzPlaceholderName::for_source_name(&bound.source_name),
                            ),
                        );
                    }
                }
            }
        }
        let mut pending_async_disposable_init = None;
        let mut init = if let Some(list) = async_disposable_head {
            pending_async_disposable_init = self.lower_async_disposable_for_init(list);
            if pending_async_disposable_init.is_none() {
                self.pop_scope();
                return (StatementIr::Empty, ValueKind::Undefined);
            }
            None
        } else {
            for_loop.init().and_then(|init| self.lower_for_init(init))
        };
        let lexical_environment = self.lower_for_lexical_environment(for_loop, init.as_ref());
        if resumable_await_loop {
            if lexical_environment.is_some() {
                self.unsupported(
                    "resumable async loop with a captured per-iteration lexical environment",
                );
                self.pop_scope();
                return (StatementIr::Empty, ValueKind::Undefined);
            }
            match &init {
                Some(ForInitIr::Lexical { name, .. }) => {
                    self.add_suspension_owned_binding(name.clone());
                }
                Some(ForInitIr::LexicalBlock(bindings)) => {
                    for binding in bindings {
                        self.add_suspension_owned_binding(binding.name.clone());
                    }
                }
                Some(ForInitIr::Statements(_)) => {
                    // A pattern head binds names this bookkeeping cannot
                    // enumerate, and resuming into bindings the suspension does
                    // not own would read uninitialised storage.
                    self.unsupported("resumable async loop with a destructuring loop head");
                    self.pop_scope();
                    return (StatementIr::Empty, ValueKind::Undefined);
                }
                Some(ForInitIr::SyncDisposable(_)) => {
                    self.unsupported("resumable async loop with a synchronous using head");
                    self.pop_scope();
                    return (StatementIr::Empty, ValueKind::Undefined);
                }
                Some(ForInitIr::AsyncDisposable(_)) => {
                    unreachable!("pending async-disposable init is finalized after loop lowering")
                }
                Some(ForInitIr::Var(_)) | Some(ForInitIr::Expression(_)) | None => {}
            }
        }
        let after_init_vars = self.var_bindings.clone();
        let after_init_globals = self.global_properties.clone();
        let test = for_loop.condition().map(|expr| self.lower_expression(expr));
        let update = for_loop
            .final_expr()
            .map(|expr| self.lower_expression(expr));
        let (body, body_kind) = self.lower_loop_body(for_loop.body());
        if let Some(pending) = pending_async_disposable_init {
            init = Some(ForInitIr::AsyncDisposable(
                self.finish_async_disposable_for_init(pending),
            ));
        }
        self.pop_scope();
        let after_body_vars = self.var_bindings.clone();
        let after_body_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&after_init_vars, &after_body_vars);
        self.global_properties =
            self.merge_global_properties(&after_init_globals, &after_body_globals);
        if for_loop.init().is_none() {
            self.var_bindings = self.merge_var_bindings(&before_vars, &self.var_bindings.clone());
            self.global_properties =
                self.merge_global_properties(&before_globals, &self.global_properties.clone());
        }

        if let Some(entry_state) = generator_entry_state.or(if plain_async_await_loop {
            plain_async_entry_state
        } else {
            None
        }) {
            if lexical_environment.is_none() {
                if let Some((before_suspension, suspension_statement, after_suspension)) =
                    Self::split_resumable_loop_body(body.clone())
                {
                    if self.current_resumable_plan.is_some() {
                        match &init {
                            Some(ForInitIr::Lexical { name, .. }) => {
                                self.add_suspension_owned_binding(name.clone());
                            }
                            Some(ForInitIr::LexicalBlock(bindings)) => {
                                for binding in bindings {
                                    self.add_suspension_owned_binding(binding.name.clone());
                                }
                            }
                            Some(ForInitIr::Statements(_)) => {
                                self.unsupported(
                                    "resumable async loop with a destructuring loop head",
                                );
                                return (StatementIr::Empty, ValueKind::Undefined);
                            }
                            Some(ForInitIr::SyncDisposable(_)) => {
                                self.unsupported(
                                    "resumable async loop with a synchronous using head",
                                );
                                return (StatementIr::Empty, ValueKind::Undefined);
                            }
                            Some(ForInitIr::AsyncDisposable(_)) => unreachable!(
                                "pending async-disposable init is finalized after loop lowering"
                            ),
                            Some(ForInitIr::Var(_)) | Some(ForInitIr::Expression(_)) | None => {}
                        }
                        for statement in before_suspension.iter().chain(after_suspension.iter()) {
                            if let StatementIr::Lexical { name, .. } = statement {
                                self.add_suspension_owned_binding(name.clone());
                            }
                        }
                    }
                    let resume_state = match &suspension_statement {
                        StatementIr::GeneratorYield { resume_state, .. }
                        | StatementIr::AsyncAwait { resume_state, .. } => *resume_state,
                        _ => unreachable!(
                            "split resumable loop must return an await or yield statement"
                        ),
                    };
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
                            init,
                            test,
                            update,
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
        }
        if resumable_await_loop {
            self.unsupported("resumable async loop body did not lower to one direct await");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        if plain_async_await_loop {
            // Falling through would emit a straight-line `StatementIr::For`
            // holding a suspension: the driver re-enters the body from the top,
            // so the loop would restart at iteration zero and never suspend
            // again. A diagnostic is the only safe answer.
            self.unsupported("async loop body did not lower to one direct await");
            return (StatementIr::Empty, ValueKind::Undefined);
        }

        (
            StatementIr::For {
                init,
                test,
                update,
                body: Box::new(body),
                lexical_environment,
            },
            body_kind,
        )
    }
}
