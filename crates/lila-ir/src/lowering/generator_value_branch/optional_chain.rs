use super::super::async_expression_prefix::CompletedOptionalPropertyRead;
use super::super::suspended_call::{
    EvaluatedCallReference, EvaluatedInvocationArgument, InvocationSuspension,
};
use super::*;
use crate::async_generator_source::{
    AsyncGeneratorIfSourceStates, AsyncGeneratorOptionalChainStates,
};

enum OptionalOperandLayouts {
    Generator(GeneratorOptionalChainStates),
    Mixed(AsyncGeneratorOptionalChainStates),
}

impl OptionalOperandLayouts {
    fn exit(&self) -> u32 {
        match self {
            Self::Generator(states) => states.exit(),
            Self::Mixed(states) => states.exit(),
        }
    }

    fn finish(self) -> Option<u32> {
        match self {
            Self::Generator(states) => states.finish(),
            Self::Mixed(states) => states.finish(),
        }
    }
}

/// A false cell records that a real shorted link skipped the entire suffix.
/// The saved operand may be undefined on an ordinary link without clearing it.
struct GeneratorOptionalLive {
    name: String,
}

impl GeneratorOptionalLive {
    fn new(lowerer: &mut ScriptLowerer<'_>, prefix: &mut Vec<StatementIr>) -> Self {
        let name = lowerer.alloc_suspension_owned_binding(
            "generator.optional.live.",
            ValueInfo::new(ValueKind::Boolean),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: name.clone(),
            init: TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true)),
        });
        Self { name }
    }

    fn read(&self) -> TypedExpr {
        TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::Identifier(self.name.clone()),
        )
    }

    fn skipped(&self) -> TypedExpr {
        TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::StrictEquality {
                op: EqualityBinaryOp::StrictEqual,
                lhs: Box::new(self.read()),
                rhs: Box::new(TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Boolean),
                    ExprIr::Boolean(false),
                )),
            },
        )
    }

    fn guard(&self, statements: Vec<StatementIr>) -> StatementIr {
        StatementIr::If {
            condition: self.read(),
            then_branch: Box::new(StatementIr::LexicalBlock(statements)),
            else_branch: None,
        }
    }

    fn short_at(
        &self,
        lowerer: &mut ScriptLowerer<'_>,
        prefix: &mut Vec<StatementIr>,
        operand: &GeneratorOptionalOperand,
    ) {
        let condition = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::And,
                lhs: Box::new(self.read()),
                rhs: Box::new(nullish_condition(operand.read())),
            },
        );
        let disable = lowerer.lower_identifier_assign_value(
            self.name.clone(),
            TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(false)),
        );
        prefix.push(StatementIr::If {
            condition,
            then_branch: Box::new(StatementIr::Expression(disable)),
            else_branch: None,
        });
    }
}

/// A next Call retains a completed Reference; a next Property retains GetValue.
/// Only actual source/read/call consumers can choose the corresponding variant.
enum GeneratorOptionalOperand {
    Value(RetainedGeneratorValue),
    Call(EvaluatedCallReference),
}

impl GeneratorOptionalOperand {
    fn read(&self) -> TypedExpr {
        let mut value = match self {
            Self::Value(value) => value.read(),
            Self::Call(reference) => reference.callee_value(),
        };
        // A resumed caller or an intervening getter may change contents, while
        // the saved value and Reference receiver preserve their identities.
        value.heap_shape = None;
        value
    }

    fn from_read(
        lowerer: &mut ScriptLowerer<'_>,
        read: CompletedOptionalPropertyRead,
        next_is_call: bool,
    ) -> Self {
        if next_is_call {
            Self::Call(lowerer.capture_optional_chain_property_reference(read))
        } else {
            Self::retain_value(lowerer, read.into_value())
        }
    }

    fn from_call(lowerer: &mut ScriptLowerer<'_>, value: TypedExpr, next_is_call: bool) -> Self {
        if next_is_call {
            // A Call result has no surviving property receiver.
            Self::Call(lowerer.capture_optional_chain_call_result(value))
        } else {
            Self::retain_value(lowerer, value)
        }
    }

    fn retain_value(lowerer: &mut ScriptLowerer<'_>, value: TypedExpr) -> Self {
        let mut prefix = Vec::new();
        let retained = RetainedGeneratorValue::new(lowerer, &mut prefix, value);
        lowerer
            .async_expression_prefix
            .as_mut()
            .expect("optional chain owns a prefix")
            .extend(prefix);
        Self::Value(retained)
    }
}

/// Only the checked terminal-Property entry can supply a Reference destination.
/// The ordinary expression and constructor entries always publish a Value.
enum GeneratorOptionalChainDestination {
    Value,
    PropertyReference(CapturedCallReceiverIr),
    DeleteProperty,
}

impl ScriptLowerer<'_> {
    pub(in crate::lowering) fn lower_generator_optional_chain(
        &mut self,
        source: GeneratorOptionalChainSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_generator_optional_chain_to(source, GeneratorOptionalChainDestination::Value)
    }

    pub(in crate::lowering) fn lower_generator_grouped_optional_reference(
        &mut self,
        source: GeneratorGroupedOptionalReferenceSource<'_>,
        receiver: CapturedCallReceiverIr,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_generator_optional_chain_to(
            source.into_chain(),
            GeneratorOptionalChainDestination::PropertyReference(receiver),
        )
    }

    pub(in crate::lowering) fn lower_generator_optional_delete(
        &mut self,
        source: GeneratorOptionalDeleteSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let (chain, terminal) = source.into_parts();
        match terminal {
            GeneratorOptionalDeleteTerminal::Property => self.lower_generator_optional_chain_to(
                chain,
                GeneratorOptionalChainDestination::DeleteProperty,
            ),
            GeneratorOptionalDeleteTerminal::Value => {
                let (prefix, value) = self.lower_generator_optional_chain(chain)?;
                Some((
                    prefix,
                    TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::DeleteValue {
                            expr: Box::new(value),
                        },
                    ),
                ))
            }
        }
    }

    fn lower_generator_optional_chain_to(
        &mut self,
        source: GeneratorOptionalChainSource<'_>,
        destination: GeneratorOptionalChainDestination,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let mixed = source.is_mixed();
        if (mixed && self.async_generator_entry_state().is_none())
            || (!mixed
                && !matches!(
                    self.generator_value_branch_admission(),
                    GeneratorValueBranchAdmission::OrdinaryOutsideLoops
                ))
        {
            return None;
        }
        let delete_terminal = matches!(
            &destination,
            GeneratorOptionalChainDestination::DeleteProperty
        );
        let mut terminal_receiver = match destination {
            GeneratorOptionalChainDestination::Value
            | GeneratorOptionalChainDestination::DeleteProperty => None,
            GeneratorOptionalChainDestination::PropertyReference(receiver) => Some(receiver),
        };
        let mut prefix = Vec::new();
        let result = GeneratorValueResult::new(self, &mut prefix);
        let live = GeneratorOptionalLive::new(self, &mut prefix);
        if delete_terminal {
            prefix.push(StatementIr::Expression(self.lower_identifier_assign_value(
                result.name.clone(),
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true)),
            )));
        }
        let first_is_call = source.starts_with_call();
        let (initial, mut operand) = self.generator_optional_prefix(|lowerer| {
            if first_is_call {
                lowerer
                    .capture_optional_chain_target_reference(
                        source.base_source(),
                        if mixed {
                            InvocationSuspension::AsyncGenerator
                        } else {
                            InvocationSuspension::Yield
                        },
                    )
                    .map(GeneratorOptionalOperand::Call)
            } else {
                let (statements, value) =
                    lowerer.lower_staged_generator_expression(source.base_source())?;
                lowerer
                    .async_expression_prefix
                    .as_mut()
                    .expect("optional base owns a prefix")
                    .extend(statements);
                Some(GeneratorOptionalOperand::retain_value(lowerer, value))
            }
        })?;
        prefix.extend(initial);
        // The entire base completes before any optional link is guarded.
        let mut states = if mixed {
            OptionalOperandLayouts::Mixed(source.mixed_states(self.async_generator_entry_state()?)?)
        } else {
            OptionalOperandLayouts::Generator(source.states(self.plain_generator_entry_state()?)?)
        };
        let exit = states.exit();
        let (_, links) = source.into_parts();
        let mut links = links.into_iter().peekable();
        while let Some(link) = links.next() {
            if link.shorted() {
                live.short_at(self, &mut prefix, &operand);
            }
            let next_is_call = matches!(links.peek(), Some(GeneratorOptionalChainLink::Call(_)));
            operand = match (operand, link) {
                (
                    GeneratorOptionalOperand::Value(base),
                    GeneratorOptionalChainLink::Private { field, .. },
                ) => {
                    let private_name_id = self.current_private_name_id(field)?;
                    let receiver = if links.peek().is_none() {
                        terminal_receiver.take()
                    } else {
                        None
                    };
                    self.generator_optional_guarded(&live, &mut prefix, |lowerer| {
                        let mut target = base.into_value();
                        target.heap_shape = None;
                        let read = CompletedOptionalPropertyRead::from_evaluated_private(
                            lowerer,
                            target,
                            private_name_id,
                        );
                        Some(match receiver {
                            Some(receiver) => {
                                let value =
                                    lowerer.capture_optional_chain_property_callee(read, receiver);
                                GeneratorOptionalOperand::retain_value(lowerer, value)
                            }
                            None => {
                                GeneratorOptionalOperand::from_read(lowerer, read, next_is_call)
                            }
                        })
                    })?
                }
                (
                    GeneratorOptionalOperand::Value(base),
                    GeneratorOptionalChainLink::Property(link),
                ) => {
                    let (field, _) = link.into_parts();
                    let key = match field {
                        GeneratorOptionalKeySource::Constant(name) => PropertyKeyIr::StaticString(
                            self.interner.resolve_expect(name).to_string(),
                        ),
                        GeneratorOptionalKeySource::Computed(source) => {
                            let value = self.lower_generator_optional_operand(
                                &live,
                                &mut prefix,
                                source,
                                &mut states,
                            )?;
                            PropertyKeyIr::StringExpr(Box::new(value))
                        }
                    };
                    // Grouping retains only the final property's Reference.
                    // Earlier Gets still complete into their original values.
                    let receiver = if links.peek().is_none() {
                        terminal_receiver.take()
                    } else {
                        None
                    };
                    let terminal_delete = delete_terminal && links.peek().is_none();
                    self.generator_optional_guarded(&live, &mut prefix, |lowerer| {
                        if let PropertyKeyIr::StringExpr(value) = &key {
                            lowerer.record_possible_to_primitive_effects(&value.value_info());
                        }
                        let mut target = base.into_value();
                        target.heap_shape = None;
                        if terminal_delete {
                            let value = lowerer.lower_delete_property_from_evaluated(target, key);
                            return Some(GeneratorOptionalOperand::retain_value(lowerer, value));
                        }
                        let read =
                            CompletedOptionalPropertyRead::from_evaluated(lowerer, target, key);
                        Some(match receiver {
                            Some(receiver) => {
                                let callee =
                                    lowerer.capture_optional_chain_property_callee(read, receiver);
                                GeneratorOptionalOperand::retain_value(lowerer, callee)
                            }
                            None => {
                                GeneratorOptionalOperand::from_read(lowerer, read, next_is_call)
                            }
                        })
                    })?
                }
                (
                    GeneratorOptionalOperand::Call(reference),
                    GeneratorOptionalChainLink::Call(link),
                ) => {
                    let (source, operands, _) = link.into_parts();
                    let mut arguments: Vec<TypedExpr> = Vec::with_capacity(operands.len());
                    for argument in operands {
                        let epoch = self.intervening_effect_epoch;
                        let (source, spread) = match argument {
                            GeneratorOptionalArgumentSource::Value(value) => (value, false),
                            GeneratorOptionalArgumentSource::Spread(value) => (value, true),
                        };
                        let value = self.lower_generator_optional_operand(
                            &live,
                            &mut prefix,
                            source,
                            &mut states,
                        )?;
                        let argument =
                            self.generator_optional_guarded(&live, &mut prefix, |lowerer| {
                                Some(lowerer.capture_evaluated_invocation_argument(if spread {
                                    EvaluatedInvocationArgument::Spread(value)
                                } else {
                                    EvaluatedInvocationArgument::Value(value)
                                }))
                            })?;
                        if self.intervening_effect_epoch != epoch {
                            for previous in &mut arguments {
                                previous.heap_shape = None;
                            }
                        }
                        arguments.push(argument);
                    }
                    self.generator_optional_guarded(&live, &mut prefix, |lowerer| {
                        lowerer.intervening_effect_epoch =
                            lowerer.intervening_effect_epoch.saturating_add(1);
                        let value = lowerer.finish_suspended_call(
                            reference,
                            arguments,
                            CallCandidateSource::IndirectSyntax(source),
                        );
                        Some(GeneratorOptionalOperand::from_call(
                            lowerer,
                            value,
                            next_is_call,
                        ))
                    })?
                }
                (GeneratorOptionalOperand::Value(_), GeneratorOptionalChainLink::Call(_))
                | (GeneratorOptionalOperand::Call(_), GeneratorOptionalChainLink::Property(_)) => {
                    unreachable!("the checked next link owns its Value or Call Reference")
                }
                (GeneratorOptionalOperand::Call(_), GeneratorOptionalChainLink::Private { .. }) => {
                    unreachable!("a checked private property follows a completed Value");
                }
            };
        }
        if terminal_receiver.is_some() {
            unreachable!("the checked terminal Property consumes its receiver destination");
        }
        let GeneratorOptionalOperand::Value(value) = operand else {
            unreachable!("a terminal optional chain result is a Value");
        };
        let value = value.into_value();
        let mut info = if delete_terminal {
            ValueInfo::new(ValueKind::Boolean)
        } else {
            self.merge_value_infos(ValueInfo::undefined(), value.value_info())
        };
        info.heap_shape = None;
        self.generator_optional_guarded(&live, &mut prefix, |lowerer| {
            let write = lowerer.lower_identifier_assign_value(result.name.clone(), value);
            lowerer
                .async_expression_prefix
                .as_mut()
                .expect("optional chain owns a prefix")
                .push(StatementIr::Expression(write));
            Some(())
        })?;
        if states.finish()? != exit
            || self.current_generator_resume_state != Some(exit)
            || (mixed && self.current_async_resume_state != Some(exit))
        {
            return None;
        }
        self.set_binding_value_info(&result.name, info.clone());
        Some((
            prefix,
            TypedExpr::from_info(info, ExprIr::Identifier(result.name)),
        ))
    }

    fn lower_generator_optional_operand(
        &mut self,
        live: &GeneratorOptionalLive,
        prefix: &mut Vec<StatementIr>,
        source: GeneratorOptionalOperandSource<'_>,
        states: &mut OptionalOperandLayouts,
    ) -> Option<TypedExpr> {
        if !source.suspends() {
            return self.generator_optional_guarded(live, prefix, |lowerer| {
                let value = lowerer.lower_expression(source.source());
                Some(lowerer.pin_async_operand_before_suspension(
                    value,
                    true,
                    "generator.optional.operand.",
                ))
            });
        }
        let states = match states {
            OptionalOperandLayouts::Generator(states) => states.next()?,
            OptionalOperandLayouts::Mixed(states) => {
                return self.lower_mixed_optional_operand(live, prefix, source, states.next()?);
            }
        };
        if self.plain_generator_entry_state()? != states.entry() {
            return None;
        }
        let result = GeneratorValueResult::new(self, prefix);
        let skipped_value = RetainedGeneratorValue::new(self, prefix, TypedExpr::undefined());
        let before = self.capture_conditional_flow_facts();
        let skipped = CompleteGeneratorValueArm::lower(
            self,
            CompleteGeneratorValueArmSource::Retained(skipped_value),
            states.then_arm(),
            &result,
        )?;
        let skipped_facts = self.capture_conditional_flow_facts();
        self.install_conditional_flow_facts(before);
        let selected = CompleteGeneratorValueArm::lower(
            self,
            CompleteGeneratorValueArmSource::Expression(source.source()),
            states.else_arm(),
            &result,
        )?;
        let selected_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(skipped_facts, selected_facts);
        let mut info = self.merge_value_infos(skipped.info, selected.info);
        info.heap_shape = None;
        let branch = OrdinaryGeneratorIfIr::new(
            live.skipped(),
            states.entry(),
            skipped.region,
            selected.region,
            states.exit(),
        )
        .ok()?;
        prefix.push(StatementIr::OrdinaryGeneratorIf(Box::new(branch)));
        self.current_generator_resume_state = Some(states.exit());
        self.set_binding_value_info(&result.name, info.clone());
        Some(TypedExpr::from_info(info, ExprIr::Identifier(result.name)))
    }

    fn lower_mixed_optional_operand(
        &mut self,
        live: &GeneratorOptionalLive,
        prefix: &mut Vec<StatementIr>,
        source: GeneratorOptionalOperandSource<'_>,
        states: AsyncGeneratorIfSourceStates,
    ) -> Option<TypedExpr> {
        if self.async_generator_entry_state()? != states.condition().entry() {
            return None;
        }
        let result = GeneratorValueResult::new(self, prefix);
        let condition = AsyncGeneratorLoopExpressionIr::new(
            self.finish_async_generator_region(
                BlockIr {
                    statements: Vec::new(),
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.condition(),
            )
            .ok()?,
            live.skipped(),
        );
        let mut reference = None;
        let before = self.capture_conditional_flow_facts();
        self.set_async_generator_phase(states.then_branch().entry());
        let skipped = self.lower_mixed_value_arm(None, &result.name, &mut reference, None)?;
        let skipped = self
            .finish_async_generator_region(
                BlockIr {
                    statements: skipped,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.then_branch(),
            )
            .ok()?;
        let skipped_facts = self.capture_conditional_flow_facts();
        self.install_conditional_flow_facts(before);
        self.set_async_generator_phase(states.else_branch().entry());
        let selected =
            self.lower_mixed_value_arm(Some(source.source()), &result.name, &mut reference, None)?;
        let selected = self
            .finish_async_generator_region(
                BlockIr {
                    statements: selected,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.else_branch(),
            )
            .ok()?;
        let selected_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(skipped_facts, selected_facts);
        let branch = AsyncGeneratorIfIr::new(states, condition, skipped, selected).ok()?;
        self.set_async_generator_phase(branch.exit_state());
        prefix.push(StatementIr::AsyncGeneratorIf(Box::new(branch)));
        let mut info = unknown_runtime_value_info();
        info.heap_shape = None;
        self.set_binding_value_info(&result.name, info.clone());
        Some(TypedExpr::from_info(info, ExprIr::Identifier(result.name)))
    }

    /// Capture every emitted eager side effect into the selected operation,
    /// restoring the caller's prefix even when an existing producer refuses.
    fn generator_optional_prefix<T>(
        &mut self,
        lower: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<(Vec<StatementIr>, T)> {
        let enclosing = self.async_expression_prefix.replace(Vec::new());
        let value = lower(self);
        let prefix = std::mem::replace(&mut self.async_expression_prefix, enclosing)
            .expect("optional chain owns its eager prefix");
        value.map(|value| (prefix, value))
    }

    fn generator_optional_guarded<T>(
        &mut self,
        live: &GeneratorOptionalLive,
        prefix: &mut Vec<StatementIr>,
        lower: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<T> {
        let before = self.capture_conditional_flow_facts();
        self.push_scope();
        let lowered = self.generator_optional_prefix(lower);
        self.pop_scope();
        let selected = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, selected);
        let (guarded, value) = lowered?;
        if guarded
            .iter()
            .any(|statement| SynchronousLoopBodyIr::new(statement).is_err())
        {
            return None;
        }
        if !guarded.is_empty() {
            prefix.push(live.guard(guarded));
        }
        Some(value)
    }
}
