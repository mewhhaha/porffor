//! A real source initializer alone mints the complete per-key region.
use super::pattern_target::{owned_pattern_binding, PatternContinuation};
use super::*;
use crate::analysis::CompleteResumableForOfOwner;
use crate::async_generator_source::{
    AsyncGeneratorForOfSource, AsyncGeneratorForOfSourceIdentity, AsyncGeneratorForOfSourceStates,
    AsyncGeneratorPatternSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedAsyncGeneratorForOfInitializer {
    source: AsyncGeneratorForOfSourceIdentity,
    incoming_binding: OwnedEnvBindingIr,
    region: ResumableRegionIr,
    environment: Option<ForInOfEnvironmentIr>,
    mode: BindingMode,
    resource: Option<AsyncGeneratorForOfResourceIr>,
}
impl CheckedAsyncGeneratorForOfInitializer {
    pub(crate) fn region(&self) -> &ResumableRegionIr {
        &self.region
    }
    pub(crate) fn environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.environment.as_ref()
    }
    pub(crate) fn mode(&self) -> BindingMode {
        self.mode
    }
    pub(crate) fn incoming_binding(&self) -> &OwnedEnvBindingIr {
        &self.incoming_binding
    }
    pub(crate) fn resource(&self) -> Option<&AsyncGeneratorForOfResourceIr> {
        self.resource.as_ref()
    }
    pub(crate) fn matches_source(&self, states: &AsyncGeneratorForOfSourceStates) -> bool {
        self.source == states.identity()
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_async_generator_for_of(
        &mut self,
        source: &ForOfLoop,
        owner: CompleteResumableForOfOwner,
    ) -> (StatementIr, ValueKind) {
        let Some(source) = owner.checked_source(source) else {
            self.unsupported("iterator source identity differs from analysis");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        self.lower_complete_resumable_for_of(source)
    }

    pub(super) fn lower_complete_resumable_for_of(
        &mut self,
        source: AsyncGeneratorForOfSource<'_>,
    ) -> (StatementIr, ValueKind) {
        let before = self.capture_conditional_flow_facts();
        let execution = source.execution();
        let previous_generator_depth = self.ordinary_generator_region_depth;
        let previous_async_context = self.async_value_branch_context;
        if execution == ResumableRegionProtocolIr::Generator {
            self.ordinary_generator_region_depth += 1;
        }
        if execution == ResumableRegionProtocolIr::Async {
            self.async_value_branch_context = AsyncValueBranchContext::ForOf {
                loop_depth: self.loop_depth,
            };
        }
        self.push_scope();
        let result = (|| {
            let entry = self.complete_for_of_entry(execution)?;
            let states = source.states(entry)?;
            let ast = source.source();
            let environment = self.lower_for_in_of_environment(ast as *const ForOfLoop as usize);
            self.set_complete_for_of_phase(execution, states.head().entry());
            self.push_scope();
            let head = (|| {
                if matches!(states.head_mode(), BindingMode::Let | BindingMode::Const) {
                    let binding = match ast.initializer() {
                        IterableLoopInitializer::Let(binding)
                        | IterableLoopInitializer::Const(binding)
                        | IterableLoopInitializer::Using(binding)
                        | IterableLoopInitializer::AwaitUsing(binding) => binding,
                        _ => return None,
                    };
                    for bound in supported_bound_names(self.interner, binding)? {
                        self.declare_binding(
                            bound.source_name.clone(),
                            BindingInfo::tdz_placeholder(
                                states.head_mode(),
                                TdzPlaceholderName::for_source_name(&bound.source_name),
                            ),
                        );
                    }
                }
                self.lower_complete_for_of_operand(execution, ast.iterable())
            })();
            self.pop_scope();
            let (mut prefix, mut raw) = head?;
            raw.heap_shape = None;
            let info = raw.value_info();
            let head_binding =
                owned_pattern_binding(self, "async.generator.forof.head.", info.clone());
            prefix.push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: head_binding.name.clone(),
                init: raw,
            });
            let head = ResumableExpressionIr::new(
                ResumableRegionIr::new(
                    BlockIr {
                        statements: prefix,
                        result_kind: ValueKind::Undefined,
                        lexical_environment: None,
                    },
                    states.head(),
                    execution,
                )
                .ok()?,
                TypedExpr::from_info(info, ExprIr::Identifier(head_binding.name.clone())),
            );
            let incoming_binding = owned_pattern_binding(
                self,
                "async.generator.forof.incoming.",
                ValueInfo::new(ValueKind::Dynamic),
            );
            let value_binding =
                owned_pattern_binding(self, "async.generator.forof.value.", ValueInfo::undefined());
            self.invalidate_unknown_user_code_effects();
            if let AsyncGeneratorIteratorProtocolIr::Awaited {
                next_suspend_state,
                next_resume_state,
                ..
            } = states.protocol()
            {
                self.set_complete_for_of_phase(execution, next_suspend_state);
                if execution == ResumableRegionProtocolIr::AsyncGenerator {
                    let point =
                        self.take_resumable_suspension(ResumableSuspensionKindIr::ForAwaitNext)?;
                    if point.suspend_state != next_suspend_state
                        || point.resume_state != next_resume_state
                    {
                        return None;
                    }
                }
            }
            if execution == ResumableRegionProtocolIr::Async {
                self.plain_async_for_of_depth += 1;
            }
            let initializer = self.lower_checked_async_generator_for_of_initializer(
                source,
                &states,
                incoming_binding.clone(),
                environment,
            );
            if execution == ResumableRegionProtocolIr::Async {
                self.plain_async_for_of_depth -= 1;
            }
            let initializer = initializer?;
            self.set_complete_for_of_phase(execution, states.body().entry());
            if execution == ResumableRegionProtocolIr::AsyncGenerator {
                self.mixed_async_generator_region_depth += 1;
            }
            if execution == ResumableRegionProtocolIr::Async {
                self.async_value_branch_context = AsyncValueBranchContext::ForOf {
                    loop_depth: self.loop_depth + 1,
                };
            }
            if execution == ResumableRegionProtocolIr::Async {
                self.plain_async_for_of_depth += 1;
            }
            let (body, kind) = self.lower_loop_body(ast.body());
            if execution == ResumableRegionProtocolIr::Async {
                self.plain_async_for_of_depth -= 1;
            }
            if execution == ResumableRegionProtocolIr::AsyncGenerator {
                self.mixed_async_generator_region_depth -= 1;
            }
            // The original source Block keeps its own nested lexical record.
            let body = ResumableRegionIr::new(
                BlockIr {
                    statements: vec![body],
                    result_kind: kind,
                    lexical_environment: None,
                },
                states.body(),
                execution,
            )
            .ok()?;
            if let AsyncGeneratorIteratorProtocolIr::Awaited {
                close_suspend_state,
                close_resume_state,
                ..
            } = states.protocol()
            {
                self.set_complete_for_of_phase(execution, close_suspend_state);
                if execution == ResumableRegionProtocolIr::AsyncGenerator {
                    let point =
                        self.take_resumable_suspension(ResumableSuspensionKindIr::ForAwaitClose)?;
                    if point.suspend_state != close_suspend_state
                        || point.resume_state != close_resume_state
                    {
                        return None;
                    }
                }
            }
            let exit = states.exit();
            let plan = AsyncGeneratorForOfIr::new(
                states,
                head,
                head_binding,
                incoming_binding,
                value_binding,
                initializer,
                body,
                &self.generated_owned_env_bindings,
            )
            .ok()?;
            self.set_complete_for_of_phase(execution, exit);
            Some((StatementIr::AsyncGeneratorForOf(Box::new(plan)), kind))
        })();
        self.pop_scope();
        self.ordinary_generator_region_depth = previous_generator_depth;
        self.async_value_branch_context = previous_async_context;
        let after = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(before, after);
        result.unwrap_or_else(|| {
            self.unsupported("async-generator iterator differs from its checked source, original initializer, or lexical environment");
            (StatementIr::Empty, ValueKind::Undefined)
        })
    }

    fn lower_checked_async_generator_for_of_initializer(
        &mut self,
        checked: AsyncGeneratorForOfSource<'_>,
        states: &AsyncGeneratorForOfSourceStates,
        incoming_binding: OwnedEnvBindingIr,
        environment: Option<ForInOfEnvironmentIr>,
    ) -> Option<CheckedAsyncGeneratorForOfInitializer> {
        let source = checked.source();
        if states.identity() != AsyncGeneratorForOfSourceIdentity::from_source(source) {
            return None;
        }
        let execution = states.execution();
        self.set_complete_for_of_phase(execution, states.initialization().entry());
        let value = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Dynamic),
            ExprIr::Identifier(incoming_binding.name.clone()),
        );
        if let Some(resource_states) = states.resource() {
            let capability = owned_pattern_binding(
                self,
                "async.generator.forof.resource.",
                ValueInfo {
                    kind: ValueKind::Object,
                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                    heap_shape: None,
                    function_targets: FunctionTargetKnowledge::none(),
                },
            );
            let registration = self.lower_async_generator_for_of_resource_registration(
                source,
                resource_states.registration(),
                &capability,
                value,
            )?;
            let (resource, region) = AsyncGeneratorForOfResourceIr::for_iteration(
                resource_states,
                states.initialization(),
                capability,
                &incoming_binding,
                registration,
                &self.generated_owned_env_bindings,
            )
            .ok()?;
            let region = ResumableRegionIr::from_checked_iteration_resource(
                region,
                &resource,
                resource_states,
                execution,
            )?;
            return Some(CheckedAsyncGeneratorForOfInitializer {
                source: states.identity(),
                incoming_binding,
                region,
                environment,
                mode: states.head_mode(),
                resource: Some(resource),
            });
        }
        let mut statements = Vec::new();
        match source.initializer() {
            IterableLoopInitializer::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                statements.push(StatementIr::DeclarationEvaluation(
                    self.lower_bare_iteration_head_write(name, value),
                ));
            }
            IterableLoopInitializer::Access(access) => {
                let continuation = Self::complete_for_of_pattern_continuation(execution);
                let target = self.retain_pattern_member(continuation, access, &mut statements)?;
                self.put_pattern_target(continuation, target, value, &mut statements)?;
            }
            IterableLoopInitializer::Pattern(pattern) => {
                statements.extend(
                    self.lower_resumable_for_of_pattern(execution, pattern, value, None, None)?,
                );
            }
            IterableLoopInitializer::Var(variable) if variable.init().is_none() => {
                match variable.binding() {
                    Binding::Identifier(identifier) => {
                        let name = self.interner.resolve_expect(identifier.sym()).to_string();
                        if self.borrows_direct_eval_variable_environment() {
                            statements.push(StatementIr::DeclarationEvaluation(
                                self.environment_identifier(
                                    name,
                                    EnvironmentIdentifierOperationIr::Assign {
                                        value: Box::new(value),
                                    },
                                ),
                            ));
                        } else {
                            self.set_binding_value_info(&name, value.value_info());
                            self.declare_binding(
                                name.clone(),
                                BindingInfo {
                                    mode: BindingMode::Var,
                                    storage_name: name.clone(),
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: FunctionTargetKnowledge::unknown(),
                                    initialization: Initialization::Initialized,
                                },
                            );
                            statements.push(StatementIr::Var(vec![VarDeclaratorIr {
                                name,
                                init: Some(value),
                            }]));
                        }
                    }
                    Binding::Pattern(pattern) => {
                        statements.extend(self.lower_resumable_for_of_pattern(
                            execution,
                            pattern,
                            value,
                            Some(BindingMode::Var),
                            None,
                        )?)
                    }
                }
            }
            IterableLoopInitializer::Let(binding) | IterableLoopInitializer::Const(binding) => {
                let mode = states.head_mode();
                let names = supported_bound_names(self.interner, binding)?
                    .into_iter()
                    .map(|bound| {
                        let storage = for_of_loop_binding_storage_name(source, &bound.source_name);
                        (bound.source_name, storage)
                    })
                    .collect::<BTreeMap<_, _>>();
                for (name, storage) in &names {
                    self.declare_binding(
                        name.clone(),
                        BindingInfo {
                            mode,
                            storage_name: storage.clone(),
                            kind: ValueKind::Dynamic,
                            possible_kinds: KindSet::all_runtime_tags(),
                            heap_shape: None,
                            function_targets: FunctionTargetKnowledge::unknown(),
                            initialization: Initialization::Uninitialized(
                                UninitializedStorage::Allocated,
                            ),
                        },
                    );
                }
                match binding {
                    Binding::Identifier(identifier) => {
                        let name = self.interner.resolve_expect(identifier.sym()).to_string();
                        let storage_name = names.get(&name)?.clone();
                        self.declare_binding(
                            name,
                            BindingInfo {
                                mode,
                                storage_name: storage_name.clone(),
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: FunctionTargetKnowledge::unknown(),
                                initialization: Initialization::Initialized,
                            },
                        );
                        statements.push(StatementIr::Lexical {
                            mode,
                            name: storage_name,
                            init: value,
                        });
                    }
                    Binding::Pattern(pattern) => {
                        statements.extend(self.lower_resumable_for_of_pattern(
                            execution,
                            pattern,
                            value,
                            Some(mode),
                            Some(&names),
                        )?)
                    }
                }
            }
            IterableLoopInitializer::Var(_)
            | IterableLoopInitializer::Using(_)
            | IterableLoopInitializer::AwaitUsing(_)
            | IterableLoopInitializer::WebCompatCall(_) => return None,
        }
        let region = ResumableRegionIr::new(
            BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            states.initialization(),
            execution,
        )
        .ok()?;
        Some(CheckedAsyncGeneratorForOfInitializer {
            source: states.identity(),
            incoming_binding,
            region,
            environment,
            mode: states.head_mode(),
            resource: None,
        })
    }

    fn lower_resumable_for_of_pattern(
        &mut self,
        execution: ResumableRegionProtocolIr,
        pattern: &Pattern,
        value: TypedExpr,
        mode: Option<BindingMode>,
        names: Option<&BTreeMap<String, String>>,
    ) -> Option<Vec<StatementIr>> {
        if contains(pattern, ContainsSymbol::AwaitExpression)
            || contains(pattern, ContainsSymbol::YieldExpression)
        {
            match execution {
                ResumableRegionProtocolIr::Generator => match pattern {
                    Pattern::Array(_) => self
                        .lower_staged_generator_array_pattern_with_storage_names(
                            GeneratorArrayPatternSource::new(pattern)?,
                            value,
                            mode,
                            names,
                        ),
                    Pattern::Object(_) => self
                        .lower_staged_generator_object_pattern_with_storage_names(
                            GeneratorObjectPatternSource::new(pattern)?,
                            value,
                            mode,
                            names,
                        ),
                },
                ResumableRegionProtocolIr::Async => self
                    .lower_staged_async_pattern_with_storage_names(
                        AsyncPatternSource::new(pattern)?,
                        value,
                        mode,
                        names,
                    ),
                ResumableRegionProtocolIr::AsyncGenerator => self
                    .lower_staged_async_generator_pattern_with_storage_names(
                        AsyncGeneratorPatternSource::new(pattern)?,
                        value,
                        mode,
                        names,
                    ),
            }
            .map(|(statements, _)| statements)
        } else {
            match mode {
                None => Some(vec![StatementIr::DeclarationEvaluation(
                    self.lower_pattern_assign_value(pattern, value)?,
                )]),
                Some(BindingMode::Var) => self.lower_pattern_var_binding_from_value(pattern, value),
                Some(mode @ (BindingMode::Let | BindingMode::Const)) => self
                    .lower_pattern_lexical_binding_from_value_with_storage_names(
                        mode, pattern, value, names,
                    ),
            }
        }
    }

    fn complete_for_of_pattern_continuation(
        execution: ResumableRegionProtocolIr,
    ) -> PatternContinuation {
        match execution {
            ResumableRegionProtocolIr::Generator => PatternContinuation::Generator,
            ResumableRegionProtocolIr::Async => PatternContinuation::Async,
            ResumableRegionProtocolIr::AsyncGenerator => PatternContinuation::AsyncGenerator,
        }
    }
    fn complete_for_of_entry(&self, execution: ResumableRegionProtocolIr) -> Option<u32> {
        match execution {
            ResumableRegionProtocolIr::Generator => self.plain_generator_entry_state(),
            ResumableRegionProtocolIr::Async => self.plain_async_entry_state(),
            ResumableRegionProtocolIr::AsyncGenerator => self.async_generator_entry_state(),
        }
    }
    fn set_complete_for_of_phase(&mut self, execution: ResumableRegionProtocolIr, state: u32) {
        match execution {
            ResumableRegionProtocolIr::Generator => {
                self.current_generator_resume_state = Some(state)
            }
            ResumableRegionProtocolIr::Async => self.current_async_resume_state = Some(state),
            ResumableRegionProtocolIr::AsyncGenerator => self.set_async_generator_phase(state),
        }
    }
    fn lower_complete_for_of_operand(
        &mut self,
        execution: ResumableRegionProtocolIr,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = match execution {
            ResumableRegionProtocolIr::Generator => {
                super::resumable_operand::ResumableOperandProtocol::Generator
            }
            ResumableRegionProtocolIr::Async => {
                super::resumable_operand::ResumableOperandProtocol::Async
            }
            ResumableRegionProtocolIr::AsyncGenerator => {
                super::resumable_operand::ResumableOperandProtocol::Mixed
            }
        };
        protocol.lower(self, source)
    }
}
