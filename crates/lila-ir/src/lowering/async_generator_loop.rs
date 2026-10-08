use super::*;
use crate::async_generator_loop_control::AsyncGeneratorControlError;

impl ScriptLowerer<'_> {
    pub(super) fn set_async_generator_phase(&mut self, state: u32) {
        self.current_generator_resume_state = Some(state);
        self.current_async_resume_state = Some(state);
    }
    pub(super) fn finish_async_generator_region(
        &self,
        block: BlockIr,
        range: AsyncGeneratorSourceRange,
    ) -> Result<AsyncGeneratorLoopRegionIr, AsyncGeneratorControlError> {
        let actual = self
            .async_generator_entry_state()
            .ok_or(AsyncGeneratorControlError::ForeignContinuation)?;
        if actual != range.end() || self.current_async_resume_state != Some(actual) {
            return Err(AsyncGeneratorControlError::StateMismatch {
                expected: range.end(),
                actual,
            });
        }
        AsyncGeneratorLoopRegionIr::new(block, range)
    }
    pub(super) fn lower_async_generator_expression_region(
        &mut self,
        source: Option<&Expression>,
        range: AsyncGeneratorSourceRange,
        omitted_test: bool,
    ) -> Result<AsyncGeneratorLoopExpressionIr, AsyncGeneratorControlError> {
        self.set_async_generator_phase(range.entry());
        let (statements, mut value) = match source {
            Some(source) => self
                .lower_mixed_generator_value(source)
                .ok_or(AsyncGeneratorControlError::ForeignContinuation)?,
            None if omitted_test => (
                Vec::new(),
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true)),
            ),
            None => (Vec::new(), TypedExpr::undefined()),
        };
        value.heap_shape = None;
        let region = self.finish_async_generator_region(
            BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            range,
        )?;
        Ok(AsyncGeneratorLoopExpressionIr::new(region, value))
    }
    pub(super) fn lower_async_generator_body_region(
        &mut self,
        source: &Statement,
        range: AsyncGeneratorSourceRange,
    ) -> Result<AsyncGeneratorLoopRegionIr, AsyncGeneratorControlError> {
        self.set_async_generator_phase(range.entry());
        self.mixed_async_generator_region_depth += 1;
        self.breakable_depth += 1;
        self.push_scope();
        let (statement, kind) = self.lower_statement(source);
        self.pop_scope();
        self.breakable_depth -= 1;
        self.mixed_async_generator_region_depth -= 1;
        let block = match statement {
            StatementIr::Block(block) => block,
            StatementIr::LexicalBlock(statements) => BlockIr {
                statements,
                result_kind: kind,
                lexical_environment: None,
            },
            statement => BlockIr {
                statements: vec![statement],
                result_kind: kind,
                lexical_environment: None,
            },
        };
        self.finish_async_generator_region(block, range)
    }
    fn finish_mixed_loop(
        &mut self,
        result: Result<(StatementIr, ValueKind), AsyncGeneratorControlError>,
    ) -> (StatementIr, ValueKind) {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.unsupported_with_message(format!("unsupported in lila wasm-aot: invalid mixed async-generator classic region: {error:?}"));
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }
    fn mixed_loop_value_binding(&mut self) -> OwnedEnvBindingIr {
        let name = self
            .alloc_suspension_owned_binding("async.generator.loop.value.", ValueInfo::undefined());
        self.generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .expect("mixed loop V is an actual allocated cell")
            .clone()
    }
    pub(super) fn lower_async_generator_classic_for(
        &mut self,
        source: &ForLoop,
    ) -> (StatementIr, ValueKind) {
        let states = AsyncGeneratorClassicLoopSource::for_loop(source)
            .and_then(|source| source.states(self.async_generator_entry_state()?));
        let Some(states) = states else {
            self.unsupported(
                "async-generator classic For source has no complete mixed phase owner",
            );
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        self.loop_depth += 1;
        let result = (|| -> Result<_, AsyncGeneratorControlError> {
            let range = states.initialization().expect("checked For initialization");
            self.set_async_generator_phase(range.entry());
            if let Some(ForLoopInitializer::Lexical(source)) = source.init() {
                let mode = match source.declaration() {
                    LexicalDeclaration::Let(_) => BindingMode::Let,
                    LexicalDeclaration::Const(_)
                    | LexicalDeclaration::Using(_)
                    | LexicalDeclaration::AwaitUsing(_) => BindingMode::Const,
                };
                for variable in source.declaration().variable_list().as_ref() {
                    for bound in supported_bound_names(self.interner, variable.binding())
                        .ok_or(AsyncGeneratorControlError::ForeignContinuation)?
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
            let capability = states
                .resource()
                .map(|_| self.allocate_mixed_resource_capability("async.generator.for.resource."));
            let (statements, head) = if let (Some(resource), Some(capability)) =
                (states.resource(), capability.as_ref())
            {
                let Some(ForLoopInitializer::Lexical(declaration)) = source.init() else {
                    return Err(AsyncGeneratorControlError::ForeignContinuation);
                };
                let variables = match declaration.declaration() {
                    LexicalDeclaration::Using(list) | LexicalDeclaration::AwaitUsing(list) => {
                        list.as_ref()
                    }
                    _ => return Err(AsyncGeneratorControlError::ForeignContinuation),
                };
                let statements = self
                    .lower_mixed_classic_resource_initialization(variables, resource, capability)
                    .ok_or(AsyncGeneratorControlError::ForeignContinuation)?;
                let head = ForInitIr::Statements(statements.clone());
                (statements, Some(head))
            } else {
                self.lower_resumable_generator_for_init(source.init())?
            };
            let environment = self.lower_for_lexical_environment(source, head.as_ref());
            if environment.is_none() {
                if let Some(head) = &head {
                    self.retain_generator_for_head_bindings(head)?;
                }
            }
            let block = BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            };
            let initialization = if let (Some(resource), Some(capability)) =
                (states.resource(), capability.as_ref())
            {
                self.finish_mixed_resource_region(resource, range, capability, block)
                    .ok_or(AsyncGeneratorControlError::ForeignContinuation)?
            } else {
                self.finish_async_generator_region(block, range)?
            };
            let test = self.lower_async_generator_expression_region(
                source.condition(),
                states.test(),
                true,
            )?;
            let body = self.lower_async_generator_body_region(source.body(), states.body())?;
            let body_kind = body.block().result_kind;
            let update = self.lower_async_generator_discarded_region(
                source.final_expr(),
                states.update().expect("checked For update"),
            )?;
            // Update evaluates the complete value for its effects once, as the
            // original discarded head owner does. Its value never becomes V.
            let value = self.mixed_loop_value_binding();
            let resource =
                if let (Some(resource), Some(capability)) = (states.resource(), capability) {
                    Some(
                        AsyncGeneratorScopedResourceIr::new(
                            resource,
                            capability,
                            &[&initialization, test.region(), &body, update.region()],
                            &self.generated_owned_env_bindings,
                        )
                        .map_err(|_| AsyncGeneratorControlError::ForeignContinuation)?,
                    )
                } else {
                    None
                };
            let plan = AsyncGeneratorLoopIr::new(
                states,
                Some(initialization),
                test,
                body,
                Some(update),
                environment,
                value,
                resource,
                &self.generated_owned_env_bindings,
            )?;
            self.set_async_generator_phase(plan.exit_state());
            Ok((StatementIr::AsyncGeneratorLoop(Box::new(plan)), body_kind))
        })();
        self.loop_depth -= 1;
        self.pop_scope();
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        self.finish_mixed_loop(result)
    }
    fn lower_async_generator_discarded_region(
        &mut self,
        source: Option<&Expression>,
        range: AsyncGeneratorSourceRange,
    ) -> Result<AsyncGeneratorLoopExpressionIr, AsyncGeneratorControlError> {
        self.set_async_generator_phase(range.entry());
        let (mut statements, value) = match source {
            Some(source) => self
                .lower_mixed_generator_value(source)
                .ok_or(AsyncGeneratorControlError::ForeignContinuation)?,
            None => (Vec::new(), TypedExpr::undefined()),
        };
        statements.push(StatementIr::Expression(value));
        let region = self.finish_async_generator_region(
            BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            range,
        )?;
        Ok(AsyncGeneratorLoopExpressionIr::new(
            region,
            TypedExpr::undefined(),
        ))
    }
    pub(super) fn lower_async_generator_classic_while(
        &mut self,
        source: &WhileLoop,
    ) -> (StatementIr, ValueKind) {
        self.lower_mixed_non_for(
            AsyncGeneratorClassicLoopSource::while_loop(source),
            source.body(),
            source.condition(),
            false,
        )
    }
    pub(super) fn lower_async_generator_classic_do_while(
        &mut self,
        source: &DoWhileLoop,
    ) -> (StatementIr, ValueKind) {
        self.lower_mixed_non_for(
            AsyncGeneratorClassicLoopSource::do_while(source),
            source.body(),
            source.cond(),
            true,
        )
    }
    fn lower_mixed_non_for(
        &mut self,
        source: Option<AsyncGeneratorClassicLoopSource<'_>>,
        body_source: &Statement,
        test_source: &Expression,
        body_first: bool,
    ) -> (StatementIr, ValueKind) {
        let states = source.and_then(|source| source.states(self.async_generator_entry_state()?));
        let Some(states) = states else {
            self.unsupported(
                "async-generator classic loop source has no complete mixed phase owner",
            );
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        self.loop_depth += 1;
        let result = (|| -> Result<_, AsyncGeneratorControlError> {
            let (test, body) = if body_first {
                let body = self.lower_async_generator_body_region(body_source, states.body())?;
                let test = self.lower_async_generator_expression_region(
                    Some(test_source),
                    states.test(),
                    true,
                )?;
                (test, body)
            } else {
                let test = self.lower_async_generator_expression_region(
                    Some(test_source),
                    states.test(),
                    true,
                )?;
                let body = self.lower_async_generator_body_region(body_source, states.body())?;
                (test, body)
            };
            let kind = body.block().result_kind;
            let value = self.mixed_loop_value_binding();
            let plan = AsyncGeneratorLoopIr::new(
                states,
                None,
                test,
                body,
                None,
                None,
                value,
                None,
                &self.generated_owned_env_bindings,
            )?;
            self.set_async_generator_phase(plan.exit_state());
            Ok((StatementIr::AsyncGeneratorLoop(Box::new(plan)), kind))
        })();
        self.loop_depth -= 1;
        self.pop_scope();
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        self.finish_mixed_loop(result)
    }
}
