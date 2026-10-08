//! Complete plain Async phases feed the original shared classic-loop emitter.
use super::resumable_operand::ResumableOperandProtocol;
use super::*;
use crate::async_generator_source::{AsyncGeneratorLoopSourceStates, PlainAsyncClassicLoopSource};
use crate::generator_loop_control::GeneratorLoopControlError as Error;

impl ScriptLowerer<'_> {
    fn finish_plain_async_loop_region(
        &self,
        block: BlockIr,
        range: AsyncGeneratorSourceRange,
        execution: ResumableRegionProtocolIr,
    ) -> Result<ResumableRegionIr, Error> {
        let actual = self
            .resource_entry_state(execution)
            .ok_or(Error::ForeignContinuation)?;
        if actual != range.end() {
            return Err(Error::StateMismatch {
                expected: range.end(),
                actual,
            });
        }
        ResumableRegionIr::new(block, range, execution)
    }
    fn lower_plain_async_loop_expression(
        &mut self,
        source: Option<&Expression>,
        range: AsyncGeneratorSourceRange,
        is_test: bool,
        execution: ResumableRegionProtocolIr,
    ) -> Result<ResumableExpressionIr, Error> {
        self.set_resource_phase(execution, range.entry());
        let protocol = match execution {
            ResumableRegionProtocolIr::Generator => ResumableOperandProtocol::Generator,
            ResumableRegionProtocolIr::Async => ResumableOperandProtocol::Async,
            ResumableRegionProtocolIr::AsyncGenerator => return Err(Error::ForeignContinuation),
        };
        let (mut statements, mut value) = match source {
            Some(source) => protocol
                .lower(self, source)
                .ok_or(Error::ForeignContinuation)?,
            None if is_test => (
                vec![],
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true)),
            ),
            None => (vec![], TypedExpr::undefined()),
        };
        if !is_test {
            statements.push(StatementIr::Expression(value));
            value = TypedExpr::undefined();
        }
        value.heap_shape = None;
        let region = self.finish_plain_async_loop_region(
            BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            range,
            execution,
        )?;
        Ok(ResumableExpressionIr::new(region, value))
    }
    fn lower_plain_async_loop_body(
        &mut self,
        source: &Statement,
        range: AsyncGeneratorSourceRange,
        execution: ResumableRegionProtocolIr,
    ) -> Result<ResumableRegionIr, Error> {
        self.set_resource_phase(execution, range.entry());
        self.breakable_depth += 1;
        let (statement, kind) = self.lower_statement(source);
        self.breakable_depth -= 1;
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
        self.finish_plain_async_loop_region(block, range, execution)
    }
    pub(super) fn lower_plain_async_classic_loop(
        &mut self,
        source: PlainAsyncClassicLoopSource<'_>,
    ) -> (StatementIr, ValueKind) {
        let execution = source.execution();
        let states = self
            .resource_entry_state(execution)
            .and_then(|entry| source.plan(entry));
        let Some(states) = states else {
            self.unsupported("plain async classic loop has no complete source phase owner");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        self.loop_depth += 1;
        match execution {
            ResumableRegionProtocolIr::Generator => self.ordinary_generator_region_depth += 1,
            ResumableRegionProtocolIr::Async => self.plain_async_classic_depth += 1,
            ResumableRegionProtocolIr::AsyncGenerator => {
                unreachable!("closed plain classic source")
            }
        }
        let previous = std::mem::replace(
            &mut self.async_value_branch_context,
            AsyncValueBranchContext::ClassicLoop {
                loop_depth: self.loop_depth,
            },
        );
        let result = self.lower_plain_async_classic_phases(source, states);
        self.async_value_branch_context = previous;
        match execution {
            ResumableRegionProtocolIr::Generator => self.ordinary_generator_region_depth -= 1,
            ResumableRegionProtocolIr::Async => self.plain_async_classic_depth -= 1,
            ResumableRegionProtocolIr::AsyncGenerator => {
                unreachable!("closed plain classic source")
            }
        }
        self.loop_depth -= 1;
        self.pop_scope();
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        match result {
            Ok((plan, kind)) => {
                self.set_resource_phase(execution, plan.exit_state());
                (StatementIr::AsyncGeneratorLoop(Box::new(plan)), kind)
            }
            Err(error) => {
                self.unsupported_with_message(format!(
                    "unsupported in lila wasm-aot: invalid complete async classic loop: {error:?}"
                ));
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }
    fn lower_plain_async_classic_phases(
        &mut self,
        source: PlainAsyncClassicLoopSource<'_>,
        states: AsyncGeneratorLoopSourceStates,
    ) -> Result<(AsyncGeneratorLoopIr, ValueKind), Error> {
        let execution = states.execution();
        let capability = states
            .resource()
            .map(|_| self.allocate_mixed_resource_capability("resumable.classic.resource."));
        let (initialization, test, body, update, environment) = match source {
            PlainAsyncClassicLoopSource::For(source)
            | PlainAsyncClassicLoopSource::GeneratorResourceFor(source) => {
                let range = states.initialization().ok_or(Error::InvalidPhases)?;
                self.set_resource_phase(execution, range.entry());
                if let Some(ForLoopInitializer::Lexical(declaration)) = source.init() {
                    let mode = match declaration.declaration() {
                        LexicalDeclaration::Let(_) => BindingMode::Let,
                        LexicalDeclaration::Const(_)
                        | LexicalDeclaration::Using(_)
                        | LexicalDeclaration::AwaitUsing(_) => BindingMode::Const,
                    };
                    for variable in declaration.declaration().variable_list().as_ref() {
                        for name in supported_bound_names(self.interner, variable.binding())
                            .ok_or(Error::ForeignContinuation)?
                        {
                            self.declare_binding(
                                name.source_name.clone(),
                                BindingInfo::tdz_placeholder(
                                    mode,
                                    TdzPlaceholderName::for_source_name(&name.source_name),
                                ),
                            );
                        }
                    }
                }
                let (statements, head) = if let (Some(resource), Some(capability)) =
                    (states.resource(), capability.as_ref())
                {
                    let Some(ForLoopInitializer::Lexical(declaration)) = source.init() else {
                        return Err(Error::ForeignContinuation);
                    };
                    let variables = match declaration.declaration() {
                        LexicalDeclaration::Using(variables)
                        | LexicalDeclaration::AwaitUsing(variables) => variables.as_ref(),
                        _ => return Err(Error::ForeignContinuation),
                    };
                    let statements = self
                        .lower_mixed_classic_resource_initialization(
                            variables, resource, capability,
                        )
                        .ok_or(Error::ForeignContinuation)?;
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
                    self.finish_resumable_resource_region(resource, range, capability, block)
                        .ok_or(Error::ForeignContinuation)?
                } else {
                    self.finish_plain_async_loop_region(block, range, execution)?
                };
                let test = self.lower_plain_async_loop_expression(
                    source.condition(),
                    states.test(),
                    true,
                    execution,
                )?;
                let body =
                    self.lower_plain_async_loop_body(source.body(), states.body(), execution)?;
                let update = self.lower_plain_async_loop_expression(
                    source.final_expr(),
                    states.update().ok_or(Error::InvalidPhases)?,
                    false,
                    execution,
                )?;
                (Some(initialization), test, body, Some(update), environment)
            }
            PlainAsyncClassicLoopSource::While(source) => {
                let test = self.lower_plain_async_loop_expression(
                    Some(source.condition()),
                    states.test(),
                    true,
                    execution,
                )?;
                let body =
                    self.lower_plain_async_loop_body(source.body(), states.body(), execution)?;
                (None, test, body, None, None)
            }
            PlainAsyncClassicLoopSource::DoWhile(source) => {
                let body =
                    self.lower_plain_async_loop_body(source.body(), states.body(), execution)?;
                let test = self.lower_plain_async_loop_expression(
                    Some(source.cond()),
                    states.test(),
                    true,
                    execution,
                )?;
                (None, test, body, None, None)
            }
        };
        let kind = body.block().result_kind;
        let name =
            self.alloc_suspension_owned_binding("async.classic.value.", ValueInfo::undefined());
        let value = self
            .generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .ok_or(Error::InvalidPhases)?
            .clone();
        let resource = if let (Some(resource), Some(capability)) = (states.resource(), capability) {
            let regions = initialization
                .iter()
                .chain(std::iter::once(test.region()))
                .chain(std::iter::once(&body))
                .chain(update.iter().map(ResumableExpressionIr::region))
                .collect::<Vec<_>>();
            Some(
                AsyncGeneratorScopedResourceIr::new_resumable(
                    resource,
                    capability,
                    &regions,
                    &self.generated_owned_env_bindings,
                )
                .map_err(|_| Error::ForeignContinuation)?,
            )
        } else {
            None
        };
        let plan = match execution {
            ResumableRegionProtocolIr::Async => AsyncGeneratorLoopIr::new_plain_async(
                states,
                initialization,
                test,
                body,
                update,
                environment,
                value,
                resource,
                &self.generated_owned_env_bindings,
            )?,
            ResumableRegionProtocolIr::Generator => AsyncGeneratorLoopIr::new_generator_resource(
                states,
                initialization,
                test,
                body,
                update,
                environment,
                value,
                resource.ok_or(Error::InvalidPhases)?,
                &self.generated_owned_env_bindings,
            )?,
            ResumableRegionProtocolIr::AsyncGenerator => return Err(Error::ForeignContinuation),
        };
        Ok((plan, kind))
    }
}
