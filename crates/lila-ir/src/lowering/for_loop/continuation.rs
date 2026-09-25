use super::*;

/// The lowered classic-for head and body at the point continuation ownership is chosen.
/// This finalizes exactly one structured generator, legacy resumable, or ordinary loop.
pub(super) struct PreparedClassicForContinuation {
    pub(super) init: Option<ForInitIr>,
    pub(super) test: Option<TypedExpr>,
    pub(super) update: Option<TypedExpr>,
    pub(super) body: StatementIr,
    pub(super) body_kind: ValueKind,
    pub(super) lexical_environment: Option<ForLexicalEnvironmentIr>,
    pub(super) generator_entry_state: Option<u32>,
    pub(super) structured_body_exit_state: Option<u32>,
    pub(super) plain_async_entry_state: Option<u32>,
    pub(super) plain_async_await_loop: bool,
    pub(super) resumable_await_loop: bool,
    pub(super) source_body_has_yield: bool,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn finish_classic_for_continuation(
        &mut self,
        prepared: PreparedClassicForContinuation,
    ) -> (StatementIr, ValueKind) {
        let PreparedClassicForContinuation {
            init,
            test,
            update,
            body,
            body_kind,
            lexical_environment,
            generator_entry_state,
            structured_body_exit_state,
            plain_async_entry_state,
            plain_async_await_loop,
            resumable_await_loop,
            source_body_has_yield,
        } = prepared;
        if let (Some(entry_state), Some(body_exit_state)) =
            (generator_entry_state, structured_body_exit_state)
        {
            if lexical_environment.is_some() {
                self.unsupported("generator loop with a captured lexical head");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
            let Some(plan) = GeneratorStructuredLoopPlanIr::new(entry_state, body_exit_state, body)
            else {
                self.unsupported("generator loop continuation state overflow");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            self.current_generator_resume_state = Some(plan.exit_state());
            return (
                StatementIr::GeneratorStructuredLoop {
                    init,
                    test,
                    update,
                    plan,
                },
                body_kind,
            );
        }

        if let Some(entry_state) = generator_entry_state.or(if plain_async_await_loop {
            plain_async_entry_state
        } else {
            None
        }) {
            if lexical_environment.is_none() {
                if let Some((
                    before_suspension,
                    suspension_statement,
                    after_suspension,
                    resume_state,
                )) = Self::split_resumable_loop_body(
                    body.clone(),
                    generator_entry_state.is_some() && self.current_resumable_plan.is_none(),
                ) {
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
        if generator_entry_state.is_some() && source_body_has_yield {
            self.unsupported("generator loop body has no reentrant suspension segment");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        if resumable_await_loop {
            self.unsupported("resumable async loop body did not lower to a direct await sequence");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        if plain_async_await_loop {
            // Falling through would emit a straight-line `StatementIr::For`
            // holding a suspension: the driver re-enters the body from the top,
            // so the loop would restart at iteration zero and never suspend
            // again. A diagnostic is the only safe answer.
            self.unsupported("async loop body did not lower to a direct await sequence");
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
