use super::*;
use crate::generator_loop_control::{GeneratorLoopControlError, GeneratorLoopSourceRange};
use crate::generator_loop_source::{classic_generator_if_states, ClassicGeneratorLoopSource};

impl ScriptLowerer<'_> {
    pub(super) fn lower_ordinary_generator_for(
        &mut self,
        source: &ForLoop,
    ) -> (StatementIr, ValueKind) {
        let Some(states) = ClassicGeneratorLoopSource::For(source).plan(
            self.plain_generator_entry_state()
                .expect("ordinary generator loop entry"),
        ) else {
            self.unsupported("ordinary generator classic-for source has no owned phase plan");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        self.ordinary_generator_region_depth += 1;
        self.loop_depth += 1;
        let lowered = (|| -> Result<_, GeneratorLoopControlError> {
            let range = states
                .initialization
                .expect("For source has initialization");
            self.current_generator_resume_state = Some(range.entry);
            if let Some(ForLoopInitializer::Lexical(source)) = source.init() {
                let mode = match source.declaration() {
                    LexicalDeclaration::Let(_) => BindingMode::Let,
                    LexicalDeclaration::Const(_) => BindingMode::Const,
                    LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => {
                        return Err(GeneratorLoopControlError::ForeignContinuation)
                    }
                };
                for variable in source.declaration().variable_list().as_ref() {
                    for bound in supported_bound_names(self.interner, variable.binding())
                        .ok_or(GeneratorLoopControlError::ForeignContinuation)?
                    {
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
            let (initialization_statements, head) =
                self.lower_resumable_generator_for_init(source.init())?;
            let lexical_environment = self.lower_for_lexical_environment(source, head.as_ref());
            // Every carried storage-only lexical survives suspension; captured
            // head bindings instead retain the actual For Environment Record.
            if lexical_environment.is_none() {
                if let Some(head) = &head {
                    self.retain_generator_for_head_bindings(head)?;
                }
            }
            let initialization = self.finish_generator_loop_region(
                initialization_statements,
                ValueKind::Undefined,
                None,
                range,
            )?;
            let test =
                self.lower_generator_loop_expression(source.condition(), states.test, true)?;
            let body = self.lower_generator_loop_body(source.body(), states.body)?;
            let body_kind = body.block().result_kind;
            let update = self.lower_generator_loop_discarded_expression(
                source.final_expr(),
                states.update.expect("For source has update"),
            )?;
            let value_binding = self.allocate_generator_loop_value_binding();
            let plan = OrdinaryGeneratorLoopIr::new(
                states,
                Some(initialization),
                test,
                body,
                Some(update),
                lexical_environment,
                value_binding,
            )?;
            self.current_generator_resume_state = Some(plan.exit_state());
            Ok((
                StatementIr::OrdinaryGeneratorLoop(Box::new(plan)),
                body_kind,
            ))
        })();
        self.loop_depth -= 1;
        self.ordinary_generator_region_depth -= 1;
        self.pop_scope();
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        self.finish_generator_loop_lowering(lowered)
    }

    pub(super) fn lower_ordinary_generator_while(
        &mut self,
        source: &WhileLoop,
    ) -> (StatementIr, ValueKind) {
        self.lower_ordinary_generator_non_for(
            ClassicGeneratorLoopSource::While(source),
            source.body(),
            source.condition(),
            false,
        )
    }

    pub(super) fn lower_ordinary_generator_do_while(
        &mut self,
        source: &DoWhileLoop,
    ) -> (StatementIr, ValueKind) {
        self.lower_ordinary_generator_non_for(
            ClassicGeneratorLoopSource::DoWhile(source),
            source.body(),
            source.cond(),
            true,
        )
    }

    fn lower_ordinary_generator_non_for(
        &mut self,
        source: ClassicGeneratorLoopSource<'_>,
        body_source: &Statement,
        test_source: &Expression,
        body_first: bool,
    ) -> (StatementIr, ValueKind) {
        let Some(states) = source.plan(
            self.plain_generator_entry_state()
                .expect("ordinary generator loop entry"),
        ) else {
            self.unsupported("ordinary generator classic loop source has no owned phase plan");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        self.ordinary_generator_region_depth += 1;
        self.loop_depth += 1;
        let lowered = (|| -> Result<_, GeneratorLoopControlError> {
            let (test, body) = if body_first {
                let body = self.lower_generator_loop_body(body_source, states.body)?;
                let test =
                    self.lower_generator_loop_expression(Some(test_source), states.test, true)?;
                (test, body)
            } else {
                let test =
                    self.lower_generator_loop_expression(Some(test_source), states.test, true)?;
                let body = self.lower_generator_loop_body(body_source, states.body)?;
                (test, body)
            };
            let body_kind = body.block().result_kind;
            let value_binding = self.allocate_generator_loop_value_binding();
            let plan =
                OrdinaryGeneratorLoopIr::new(states, None, test, body, None, None, value_binding)?;
            self.current_generator_resume_state = Some(plan.exit_state());
            Ok((
                StatementIr::OrdinaryGeneratorLoop(Box::new(plan)),
                body_kind,
            ))
        })();
        self.loop_depth -= 1;
        self.ordinary_generator_region_depth -= 1;
        self.pop_scope();
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        self.finish_generator_loop_lowering(lowered)
    }

    fn finish_generator_loop_lowering(
        &mut self,
        lowered: Result<(StatementIr, ValueKind), GeneratorLoopControlError>,
    ) -> (StatementIr, ValueKind) {
        match lowered {
            Ok(lowered) => lowered,
            Err(error) => {
                self.unsupported_with_message(format!("unsupported in lila wasm-aot: invalid ordinary generator loop control plan: {error:?}"));
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn allocate_generator_loop_value_binding(&mut self) -> OwnedEnvBindingIr {
        let name =
            self.alloc_suspension_owned_binding("generator.loop.value.", ValueInfo::undefined());
        self.generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .expect("loop completion value owns an allocated activation cell")
            .clone()
    }

    fn finish_generator_loop_region(
        &self,
        statements: Vec<StatementIr>,
        result_kind: ValueKind,
        lexical_environment: Option<LexicalEnvironmentIr>,
        range: GeneratorLoopSourceRange,
    ) -> Result<GeneratorLoopRegionIr, GeneratorLoopControlError> {
        let actual = self
            .current_generator_resume_state
            .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
        if actual != range.end {
            return Err(GeneratorLoopControlError::StateMismatch {
                expected: range.end,
                actual,
            });
        }
        GeneratorLoopRegionIr::new(
            BlockIr {
                statements,
                result_kind,
                lexical_environment,
            },
            range,
        )
    }

    fn lower_generator_loop_expression(
        &mut self,
        source: Option<&Expression>,
        range: GeneratorLoopSourceRange,
        missing_is_true: bool,
    ) -> Result<GeneratorLoopExpressionIr, GeneratorLoopControlError> {
        self.current_generator_resume_state = Some(range.entry);
        let (prefix, value) = match source {
            Some(source) => self
                .lower_staged_generator_expression(source)
                .ok_or(GeneratorLoopControlError::ForeignContinuation)?,
            None if missing_is_true => (
                Vec::new(),
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true)),
            ),
            None => (Vec::new(), TypedExpr::undefined()),
        };
        let region =
            self.finish_generator_loop_region(prefix, ValueKind::Undefined, None, range)?;
        Ok(GeneratorLoopExpressionIr::new(region, value))
    }

    fn lower_generator_loop_body(
        &mut self,
        source: &Statement,
        range: GeneratorLoopSourceRange,
    ) -> Result<GeneratorLoopRegionIr, GeneratorLoopControlError> {
        self.current_generator_resume_state = Some(range.entry);
        self.breakable_depth += 1;
        self.push_scope();
        let (statement, kind) = self.lower_statement(source);
        self.pop_scope();
        self.breakable_depth -= 1;
        match statement {
            StatementIr::Block(block) => self.finish_generator_loop_region(
                block.statements,
                block.result_kind,
                block.lexical_environment,
                range,
            ),
            statement => self.finish_generator_loop_region(vec![statement], kind, None, range),
        }
    }

    fn lower_generator_loop_discarded_expression(
        &mut self,
        source: Option<&Expression>,
        range: GeneratorLoopSourceRange,
    ) -> Result<GeneratorLoopExpressionIr, GeneratorLoopControlError> {
        self.current_generator_resume_state = Some(range.entry);
        let statements = source
            .map(|source| {
                let (statement, _) = self.lower_expression_statement(source);
                match statement {
                    StatementIr::LexicalBlock(statements) => statements,
                    statement => vec![statement],
                }
            })
            .unwrap_or_default();
        let region =
            self.finish_generator_loop_region(statements, ValueKind::Undefined, None, range)?;
        Ok(GeneratorLoopExpressionIr::new(
            region,
            TypedExpr::undefined(),
        ))
    }

    pub(super) fn lower_ordinary_generator_loop_if(
        &mut self,
        source: &If,
    ) -> (StatementIr, ValueKind) {
        let lowered = (|| -> Result<_, GeneratorLoopControlError> {
            let states = classic_generator_if_states(
                source,
                self.plain_generator_entry_state()
                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?,
            )
            .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
            let (prefix, condition) = self
                .lower_staged_generator_expression(source.cond())
                .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
            let mut prefix = self
                .finish_generator_loop_region(prefix, ValueKind::Undefined, None, states.condition)?
                .into_block()
                .statements;
            self.current_generator_resume_state = Some(states.then_branch.entry);
            let before = self.capture_conditional_flow_facts();
            self.push_scope();
            let (then_statement, then_kind) = self.lower_statement(source.body());
            self.pop_scope();
            let then_end = self
                .plain_generator_entry_state()
                .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
            if then_end != states.then_branch.end {
                return Err(GeneratorLoopControlError::StateMismatch {
                    expected: states.then_branch.end,
                    actual: then_end,
                });
            }
            let then_facts = self.capture_conditional_flow_facts();
            self.install_conditional_flow_facts(before);
            self.current_generator_resume_state = Some(states.else_branch.entry);
            let (else_statement, else_kind) = source
                .else_node()
                .map(|source| {
                    self.push_scope();
                    let lowered = self.lower_statement(source);
                    self.pop_scope();
                    lowered
                })
                .unwrap_or((StatementIr::Empty, ValueKind::Undefined));
            let else_end = self
                .plain_generator_entry_state()
                .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
            if else_end != states.else_branch.end {
                return Err(GeneratorLoopControlError::StateMismatch {
                    expected: states.else_branch.end,
                    actual: else_end,
                });
            }
            let else_facts = self.capture_conditional_flow_facts();
            self.merge_conditional_flow_facts(then_facts, else_facts);
            let block = |statement, result_kind| match statement {
                StatementIr::Block(block) => block,
                statement => BlockIr {
                    statements: vec![statement],
                    result_kind,
                    lexical_environment: None,
                },
            };
            let then_branch =
                GeneratorLoopRegionIr::new(block(then_statement, then_kind), states.then_branch)?;
            let else_branch =
                GeneratorLoopRegionIr::new(block(else_statement, else_kind), states.else_branch)?;
            let plan = OrdinaryGeneratorIfIr::new(
                condition,
                states.condition.end,
                then_branch,
                else_branch,
                states.exit,
            )?;
            self.current_generator_resume_state = Some(states.exit);
            prefix.push(StatementIr::OrdinaryGeneratorIf(Box::new(plan)));
            Ok((
                StatementIr::LexicalBlock(prefix),
                self.merge_value_kinds(then_kind, else_kind),
            ))
        })();
        self.finish_generator_loop_lowering(lowered)
    }
}
