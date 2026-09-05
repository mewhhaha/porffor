mod protocol;

use self::protocol::ForOfLoweringIr;
use super::async_disposable::LoweredForOfHeadKind;
use super::*;

enum ForOfBareIdentifierHead {
    Absent,
    AssignmentTarget { source_name: String },
}

struct LexicalForOfPatternBinding {
    source_name: String,
    iteration_storage_name: String,
}

impl<'a> ScriptLowerer<'a> {
    /// Builds the activation-backed synchronous Iterator Record walk for an
    /// ordinary `for-of` whose body directly awaits in a plain async function.
    fn lower_async_function_for_of_iterator_with_body_await(
        &mut self,
        head: AsyncFunctionForOfIteratorHeadIr,
        iterable: TypedExpr,
        body: StatementIr,
        body_kind: ValueKind,
        entry_state: Option<u32>,
        head_environment: Option<ForInOfEnvironmentIr>,
    ) -> ForOfLoweringIr {
        let Some(entry_state) = entry_state else {
            self.unsupported(
                "async for-of with a body await requires a plain async function body with a \
                 resumable entry state, and this body has none",
            );
            return ForOfLoweringIr::no_iteration();
        };
        let Some((before_suspension, suspension_statement, after_suspension)) =
            Self::split_resumable_loop_body(body)
        else {
            self.unsupported("async for-of body did not lower to one direct await");
            return ForOfLoweringIr::no_iteration();
        };
        let record = IteratorRecordIr::new(
            self.alloc_iterator_slot(),
            self.alloc_next_method_slot(),
            self.alloc_done_slot(),
        );
        let plan = match AsyncFunctionForOfIteratorPlanIr::new(
            head,
            record,
            head_environment,
            before_suspension,
            suspension_statement,
            after_suspension,
            entry_state,
        ) {
            Ok(plan) => plan,
            Err(AsyncFunctionForOfIteratorPlanError::AwaitStatementRequired)
            | Err(AsyncFunctionForOfIteratorPlanError::AdditionalDirectSuspension) => {
                self.unsupported("async for-of body did not lower to one direct await");
                return ForOfLoweringIr::no_iteration();
            }
            Err(AsyncFunctionForOfIteratorPlanError::AwaitStateMismatch {
                entry_state,
                suspend_state,
                resume_state,
            }) => {
                self.unsupported(&format!(
                    "async for-of await state mismatch: entry {entry_state}, suspend \
                     {suspend_state}, resume {resume_state}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
            Err(AsyncFunctionForOfIteratorPlanError::ExitStateOverflow { resume_state }) => {
                self.unsupported(&format!(
                    "async for-of exit state overflows after resume state {resume_state}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
            Err(AsyncFunctionForOfIteratorPlanError::CapturedTdzEnvironment {
                tdz_placeholder_names,
            }) => {
                self.unsupported(&format!(
                    "async for-of with a body await cannot materialize the head's captured TDZ \
                     environment for {tdz_placeholder_names:?}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
            Err(
                error @ (AsyncFunctionForOfIteratorPlanError::BindingHeadEnvironmentRequired {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::VarBindingHasHeadEnvironment { .. }
                | AsyncFunctionForOfIteratorPlanError::SingleBindingTdzNameCount { .. }
                | AsyncFunctionForOfIteratorPlanError::SingleBindingIterationNamesMismatch {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::PreparedAssignmentHasHeadEnvironment {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternMode { .. }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternHeadEnvironmentRequired {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternNameCountMismatch { .. }
                | AsyncFunctionForOfIteratorPlanError::DuplicateTdzPlaceholderName { .. }
                | AsyncFunctionForOfIteratorPlanError::DuplicateLexicalPatternIterationStorageName {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternTdzNamesMismatch { .. }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternIterationNamesMismatch {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::EmptyLexicalPatternHasIterationEnvironment {
                    ..
                }
                | AsyncFunctionForOfIteratorPlanError::LexicalPatternValueNameCollision { .. }
                | AsyncFunctionForOfIteratorPlanError::InvalidEnvironmentLayout(_)
                | AsyncFunctionForOfIteratorPlanError::InvalidLexicalPatternInitialization(_)),
            ) => {
                self.unsupported(&format!(
                    "invalid resumable async for-of head invariant: {error:?}"
                ));
                return ForOfLoweringIr::no_iteration();
            }
        };
        match plan.value_storage() {
            AsyncFunctionForOfIteratorValueStorageIr::Activation(binding) => {
                self.add_suspension_owned_binding(binding.name.clone());
            }
            AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(_)
            | AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { .. } => {}
        }
        self.current_async_resume_state = Some(plan.exit_state());
        ForOfLoweringIr::async_function_iterator(iterable, plan, body_kind)
    }

    /// Lowers a `for`-`of` head.
    ///
    /// Thin wrapper: the witness [`lower_for_of_head`] produced has done its
    /// work by the time control returns here (every path out of that function
    /// had to name one), and no emitter may read it.
    ///
    /// [`lower_for_of_head`]: Self::lower_for_of_head
    pub(super) fn lower_for_of_loop(&mut self, for_of: &ForOfLoop) -> (StatementIr, ValueKind) {
        self.lower_for_of_head(for_of).into_statement_and_kind()
    }

    /// Every path out of this function returns a [`ForOfLoweringIr`]. The
    /// generic statement carries its witness in the head, while
    /// `AsyncFunctionForOfIterator` can only be built through the constructor
    /// that selects its dedicated resumable-sync witness.
    fn lower_for_of_head(&mut self, for_of: &ForOfLoop) -> ForOfLoweringIr {
        let uses_unified_resumable_plan = for_of.r#await() && self.current_resumable_plan.is_some();
        if for_of.r#await()
            && !uses_unified_resumable_plan
            && self.current_async_resume_state.is_none()
        {
            self.unsupported("for-await-of outside async function");
            return ForOfLoweringIr::no_iteration();
        }
        if for_of.r#await() && contains(for_of.body(), ContainsSymbol::AwaitExpression) {
            self.unsupported("explicit await in for-await-of body");
            return ForOfLoweringIr::no_iteration();
        }
        // A plain `for (x of …)` whose body awaits needs an Iterator Record and
        // loop state that survive the async driver's return to the job queue.
        let plain_async_await_body = self.plain_async_entry_state().is_some()
            && !for_of.r#await()
            && contains(for_of.body(), ContainsSymbol::AwaitExpression);
        if plain_async_await_body
            && (generator_loop_has_unsupported_control(for_of.body(), false)
                || contains(for_of.iterable(), ContainsSymbol::AwaitExpression))
        {
            self.unsupported(
                "async for-of with await requires an eager iterable and a body without break or continue",
            );
            return ForOfLoweringIr::no_iteration();
        }
        if plain_async_await_body && contains(for_of.initializer(), ContainsSymbol::AwaitExpression)
        {
            self.unsupported("async for-of with a body await requires an eager assignment target");
            return ForOfLoweringIr::no_iteration();
        }
        if let IterableLoopInitializer::WebCompatCall(call) = for_of.initializer() {
            // The head evaluates the iterable and then throws a ReferenceError,
            // so no 7.4 operation ever runs: the loop is gone, not specialized.
            return ForOfLoweringIr::new(
                StatementIr::Expression(
                    self.lower_web_compat_loop_assignment_target(call, for_of.iterable()),
                ),
                ValueKind::Undefined,
                IteratorProtocolWitness::NO_ITERATION,
            );
        }
        let mut bare_identifier_head = ForOfBareIdentifierHead::Absent;
        let mut pattern_initializer: Option<(BindingMode, Pattern)> = None;
        let mut assignment_pattern_initializer: Option<Pattern> = None;
        let mut access_initializer: Option<PropertyAccess> = None;
        let (head_kind, mode, name) = match for_of.initializer() {
            IterableLoopInitializer::Identifier(identifier) => {
                bare_identifier_head = ForOfBareIdentifierHead::AssignmentTarget {
                    source_name: self.interner.resolve_expect(identifier.sym()).to_string(),
                };
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof.assignment"),
                )
            }
            IterableLoopInitializer::Var(variable) => match variable.binding() {
                Binding::Identifier(identifier) => (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Var,
                    self.interner.resolve_expect(identifier.sym()).to_string(),
                ),
                Binding::Pattern(pattern) => {
                    pattern_initializer = Some((BindingMode::Var, pattern.clone()));
                    (
                        LoweredForOfHeadKind::Assignment,
                        BindingMode::Let,
                        self.alloc_temp_binding_name("forof"),
                    )
                }
            },
            IterableLoopInitializer::Let(Binding::Identifier(identifier)) => (
                LoweredForOfHeadKind::Assignment,
                BindingMode::Let,
                self.interner.resolve_expect(identifier.sym()).to_string(),
            ),
            IterableLoopInitializer::Const(Binding::Identifier(identifier)) => (
                LoweredForOfHeadKind::Assignment,
                BindingMode::Const,
                self.interner.resolve_expect(identifier.sym()).to_string(),
            ),
            IterableLoopInitializer::Let(Binding::Pattern(pattern)) => {
                pattern_initializer = Some((BindingMode::Let, pattern.clone()));
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof"),
                )
            }
            IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                pattern_initializer = Some((BindingMode::Const, pattern.clone()));
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof"),
                )
            }
            IterableLoopInitializer::Using(Binding::Identifier(identifier)) => {
                if for_of.r#await() {
                    self.unsupported("using declaration in for-await-of");
                    return ForOfLoweringIr::no_iteration();
                }
                if self.root_this_binding == RootThisBinding::Undefined {
                    self.unsupported("using declaration in a module");
                    return ForOfLoweringIr::no_iteration();
                }
                if self.current_generator_resume_state.is_some()
                    || self.current_async_resume_state.is_some()
                    || self.current_resumable_plan.is_some()
                {
                    self.unsupported("using declaration in a generator or async function");
                    return ForOfLoweringIr::no_iteration();
                }
                (
                    LoweredForOfHeadKind::SyncDisposable,
                    BindingMode::Const,
                    self.interner.resolve_expect(identifier.sym()).to_string(),
                )
            }
            IterableLoopInitializer::Using(Binding::Pattern(_)) => {
                self.unsupported("using declaration binding pattern in for-of");
                return ForOfLoweringIr::no_iteration();
            }
            IterableLoopInitializer::AwaitUsing(binding) => {
                let Some(name) = self.admit_async_disposable_for_of_head(for_of, binding) else {
                    return ForOfLoweringIr::no_iteration();
                };
                (
                    LoweredForOfHeadKind::AsyncDisposable,
                    BindingMode::Const,
                    name,
                )
            }
            IterableLoopInitializer::Pattern(pattern) => {
                assignment_pattern_initializer = Some(pattern.clone());
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof"),
                )
            }
            // `for (obj.key of …)` and `for (this.#field of …)` both assign to a
            // reference that the spec re-evaluates on every iteration, so the
            // element lands in a temporary and the body prefix performs the store.
            IterableLoopInitializer::Access(
                access @ (PropertyAccess::Simple(_) | PropertyAccess::Private(_)),
            ) => {
                access_initializer = Some(access.clone());
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof.access"),
                )
            }
            _ => {
                self.unsupported("for-of initializer");
                return ForOfLoweringIr::no_iteration();
            }
        };
        let lexical_pattern_bindings = match pattern_initializer.as_ref() {
            Some((BindingMode::Let | BindingMode::Const, pattern)) => {
                let binding = Binding::Pattern(pattern.clone());
                let Some(bound_names) = supported_bound_names(self.interner, &binding) else {
                    self.unsupported("for-of initializer");
                    return ForOfLoweringIr::no_iteration();
                };
                Some(
                    bound_names
                        .into_iter()
                        .map(|bound| LexicalForOfPatternBinding {
                            iteration_storage_name: for_of_loop_binding_storage_name(
                                for_of,
                                &bound.source_name,
                            ),
                            source_name: bound.source_name,
                        })
                        .collect::<Vec<_>>(),
                )
            }
            Some((BindingMode::Var, _)) | None => None,
        };
        let resumable_sync_head_is_assignment = match head_kind {
            LoweredForOfHeadKind::Assignment => true,
            LoweredForOfHeadKind::SyncDisposable | LoweredForOfHeadKind::AsyncDisposable => false,
        };
        if plain_async_await_body && !resumable_sync_head_is_assignment {
            self.unsupported("async for-of with a body await requires an assignment head");
            return ForOfLoweringIr::no_iteration();
        }
        let lexical_environment =
            self.lower_for_in_of_environment(for_of as *const ForOfLoop as usize);
        let iterable = match (
            pattern_initializer.as_ref(),
            assignment_pattern_initializer.as_ref(),
            &bare_identifier_head,
        ) {
            (None, None, ForOfBareIdentifierHead::Absent) if access_initializer.is_none() => {
                self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())
            }
            (None, None, ForOfBareIdentifierHead::AssignmentTarget { .. })
            | (None, None, ForOfBareIdentifierHead::Absent) => {
                self.lower_expression(for_of.iterable())
            }
            (Some((BindingMode::Var, _)), _, _) | (None, Some(_), _) => {
                self.lower_expression(for_of.iterable())
            }
            (Some((pattern_mode @ (BindingMode::Let | BindingMode::Const), _)), None, _) => {
                self.push_scope();
                for bound in lexical_pattern_bindings
                    .as_ref()
                    .expect("lexical pattern bindings must be classified once")
                {
                    self.declare_binding(
                        bound.source_name.clone(),
                        BindingInfo::tdz_placeholder(
                            *pattern_mode,
                            TdzPlaceholderName::for_source_name(&bound.source_name),
                        ),
                    );
                }
                let iterable = self.lower_expression(for_of.iterable());
                self.pop_scope();
                iterable
            }
            (Some(_), Some(_), _) => {
                unreachable!("loop head cannot be binding and assignment")
            }
        };
        let async_generator_next_suspension = uses_unified_resumable_plan
            .then(|| self.take_resumable_suspension(ResumableSuspensionKindIr::ForAwaitNext))
            .flatten();
        // The loop's own first suspension is the `await` on `next()`, which
        // the spec reaches once the iterable has been evaluated and before the
        // body runs. Claiming the entry state here puts it in that same order:
        // after any `await` staged out of the loop head, which has already
        // consumed states, and before the body allocates its own.
        let async_entry_state = (for_of.r#await() && !uses_unified_resumable_plan)
            .then_some(self.current_async_resume_state)
            .flatten();
        if for_of.r#await() {
            // 7.4.3 GetIterator tries `@@asyncIterator` first, then falls back
            // to `@@iterator` wrapped as an async iterator. The order is the
            // spec obligation.
            for key in [WellKnownSymbol::AsyncIterator, WellKnownSymbol::Iterator] {
                let function_targets = self
                    .optional_chain_well_known_symbol_property_info(&iterable.value_info(), key)
                    .map(|method| {
                        method
                            .function_targets
                            .known_targets()
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for function_id in function_targets {
                    let fallback = self
                        .function_signature_for_current_flow(&function_id)
                        .map(|signature| signature.this_info.clone())
                        .unwrap_or_else(|| ValueInfo::new(ValueKind::Dynamic));
                    let this_info = self.explicit_this_info_for_function_target(
                        &function_id,
                        &iterable,
                        fallback,
                    );
                    self.merge_function_this_info(&function_id, this_info);
                }
            }
        }
        // Every admitted head below emits generic iterator operations.
        // `@@iterator`, `next` and abrupt-close `return` can enter user code
        // before the per-iteration head and body run.
        self.invalidate_unknown_user_code_effects();
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.push_scope();
        // A generic iterator can yield values unrelated to the iterable's
        // inferred shape, including through an own `@@iterator` method.
        let element_info = ValueInfo {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::all_runtime_tags(),
            heap_shape: None,
            function_targets: FunctionTargetKnowledge::unknown(),
        };
        let storage_name = if mode == BindingMode::Var
            || pattern_initializer.is_some()
            || assignment_pattern_initializer.is_some()
            || access_initializer.is_some()
            || matches!(
                &bare_identifier_head,
                ForOfBareIdentifierHead::AssignmentTarget { .. }
            ) {
            name.clone()
        } else {
            for_of_loop_binding_storage_name(for_of, &name)
        };
        let Ok(pending_async_disposable_head) =
            self.begin_async_disposable_for_of_if_needed(head_kind, &storage_name)
        else {
            self.pop_scope();
            return ForOfLoweringIr::no_iteration();
        };
        self.declare_binding(
            name.clone(),
            BindingInfo {
                mode,
                storage_name: storage_name.clone(),
                kind: element_info.kind,
                possible_kinds: element_info.possible_kinds,
                heap_shape: element_info.heap_shape.clone(),
                function_targets: element_info.function_targets.clone(),
                initialization: Initialization::Initialized,
            },
        );
        let mut pattern_prefix = if let ForOfBareIdentifierHead::AssignmentTarget { source_name } =
            &bare_identifier_head
        {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let reference = self.locate_identifier_reference(source_name);
            let selected = self
                .with_environment_chain
                .select_preceding(reference.declarative_position());
            let assignment = if let Some(objects) = selected {
                self.lower_with_scoped_identifier_write(
                    source_name.clone(),
                    value,
                    objects,
                    reference,
                )
            } else {
                self.lower_located_identifier_assign_value(source_name.clone(), value, reference)
            };
            vec![StatementIr::Expression(assignment)]
        } else if let Some(access) = access_initializer.as_ref() {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let access = access.clone();
            vec![StatementIr::Expression(
                self.lower_property_assign_value(&access, value),
            )]
        } else if let Some(pattern) = assignment_pattern_initializer.as_ref() {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let Some(assign) = self.lower_pattern_assign_value(pattern, value) else {
                self.pop_scope();
                return ForOfLoweringIr::no_iteration();
            };
            vec![StatementIr::Expression(assign)]
        } else if let Some((pattern_mode, pattern)) = pattern_initializer.as_ref() {
            let init = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            if *pattern_mode == BindingMode::Var {
                let Some(prefix) = self.lower_pattern_var_binding_from_value(pattern, init) else {
                    self.pop_scope();
                    return ForOfLoweringIr::no_iteration();
                };
                prefix
            } else {
                let bindings = lexical_pattern_bindings
                    .as_ref()
                    .expect("lexical pattern bindings must be classified once");
                let storage_names = bindings
                    .iter()
                    .map(|binding| {
                        (
                            binding.source_name.clone(),
                            binding.iteration_storage_name.clone(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                for binding in bindings {
                    self.declare_binding(
                        binding.source_name.clone(),
                        BindingInfo {
                            mode: *pattern_mode,
                            storage_name: binding.iteration_storage_name.clone(),
                            kind: element_info.kind,
                            possible_kinds: element_info.possible_kinds,
                            heap_shape: element_info.heap_shape.clone(),
                            function_targets: element_info.function_targets.clone(),
                            initialization: Initialization::Uninitialized(
                                UninitializedStorage::Allocated,
                            ),
                        },
                    );
                }
                let Some(prefix) = self
                    .lower_pattern_lexical_binding_from_value_with_storage_names(
                        *pattern_mode,
                        pattern,
                        init,
                        Some(&storage_names),
                    )
                else {
                    self.pop_scope();
                    return ForOfLoweringIr::no_iteration();
                };
                prefix
            }
        } else {
            Vec::new()
        };
        let plain_async_entry_state = self.plain_async_entry_state();
        let (mut body, body_kind) = self.lower_loop_body(for_of.body());
        let async_disposable_head = pending_async_disposable_head
            .map(|pending| self.finish_async_disposable_for_of_head(pending));
        let async_generator_close_suspension = uses_unified_resumable_plan
            .then(|| self.take_resumable_suspension(ResumableSuspensionKindIr::ForAwaitClose))
            .flatten();
        let lexical_pattern_initialization =
            if plain_async_await_body && lexical_pattern_bindings.is_some() {
                std::mem::take(&mut pattern_prefix)
            } else {
                Vec::new()
            };
        if !pattern_prefix.is_empty() {
            pattern_prefix.push(body);
            body = StatementIr::Block(BlockIr {
                result_kind: body_kind,
                statements: pattern_prefix,
                lexical_environment: None,
            });
        }
        self.pop_scope();
        let after_vars = self.var_bindings.clone();
        let after_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        self.global_properties = self.merge_global_properties(&before_globals, &after_globals);
        if plain_async_await_body {
            let head = if let (
                Some((pattern_mode @ (BindingMode::Let | BindingMode::Const), _)),
                Some(bindings),
            ) = (
                pattern_initializer.as_ref(),
                lexical_pattern_bindings.as_ref(),
            ) {
                AsyncFunctionForOfIteratorHeadIr::LexicalPattern {
                    mode: *pattern_mode,
                    value_name: storage_name,
                    iteration_storage_names: bindings
                        .iter()
                        .map(|binding| binding.iteration_storage_name.clone())
                        .collect(),
                    tdz_placeholder_names: bindings
                        .iter()
                        .map(|binding| {
                            TdzPlaceholderName::for_source_name(&binding.source_name).into_string()
                        })
                        .collect(),
                    initialization: lexical_pattern_initialization,
                }
            } else if !matches!(bare_identifier_head, ForOfBareIdentifierHead::Absent)
                || pattern_initializer.is_some()
                || assignment_pattern_initializer.is_some()
                || access_initializer.is_some()
            {
                AsyncFunctionForOfIteratorHeadIr::PreparedAssignment {
                    value_name: storage_name,
                }
            } else {
                AsyncFunctionForOfIteratorHeadIr::Binding(ForOfAssignmentIr {
                    mode,
                    name: storage_name,
                })
            };
            return self.lower_async_function_for_of_iterator_with_body_await(
                head,
                iterable,
                body,
                body_kind,
                plain_async_entry_state,
                lexical_environment,
            );
        }
        // Every ordinary path uses the generic iterator protocol. That includes
        // strings, whose `@@iterator` and iterator `next` methods are mutable,
        // and non-iterable primitives. `for (x of 37)` has to reach
        // `GetIterator`, which throws a TypeError at runtime.
        let (statement, protocol) = match head_kind {
            LoweredForOfHeadKind::SyncDisposable => {
                let protocol = IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL;
                (
                    StatementIr::ForOfIterator {
                        head: ForOfIteratorHeadIr::SyncDisposable(SyncDisposableForOfHeadIr::new(
                            storage_name,
                        )),
                        iterable,
                        body: Box::new(body),
                        lexical_environment,
                    },
                    protocol,
                )
            }
            LoweredForOfHeadKind::AsyncDisposable => Self::async_disposable_for_of_statement(
                async_disposable_head.expect("an admitted async-disposable head must be finalized"),
                iterable,
                body,
                lexical_environment,
            ),
            LoweredForOfHeadKind::Assignment => {
                let async_states = if uses_unified_resumable_plan {
                    async_generator_next_suspension
                        .zip(async_generator_close_suspension)
                        .map(|(next, close)| {
                            (
                                next.suspend_state,
                                next.resume_state,
                                close.suspend_state,
                                close.resume_state,
                            )
                        })
                } else {
                    async_entry_state.map(|entry_state| {
                        let value_resume_state = entry_state + 1;
                        let close_resume_state = entry_state + 2;
                        let exit_state = entry_state + 3;
                        self.current_async_resume_state = Some(exit_state);
                        (
                            entry_state,
                            value_resume_state,
                            close_resume_state,
                            exit_state,
                        )
                    })
                };
                let async_plan = async_states.map(
                    |(entry_state, value_resume_state, close_resume_state, exit_state)| {
                        // An uncaptured lexical head has no materialized iteration
                        // environment. Analysis therefore does not place its alias
                        // in the root activation, but the body can still read it on
                        // a later resume. Retain that cell alongside the Iterator
                        // Record, without duplicating a captured iteration cell.
                        if lexical_environment
                            .as_ref()
                            .and_then(|environment| environment.iteration_environment.as_ref())
                            .is_none()
                        {
                            self.add_suspension_owned_binding(storage_name.clone());
                        }
                        // Allocation order is load-bearing: `alloc_temp_binding_name`
                        // numbers bindings as it hands them out, so these five calls
                        // must stay in this sequence for the emitted names to be the
                        // ones they were before the Iterator Record retrofit.
                        let iterator = self.alloc_iterator_slot();
                        let next_method = self.alloc_next_method_slot();
                        let async_iterator_binding = self.alloc_suspension_owned_binding(
                            "async.forof.async_iterator.",
                            ValueInfo::new(ValueKind::Boolean),
                        );
                        let done = self.alloc_done_slot();
                        let close_on_rejection_binding = self.alloc_suspension_owned_binding(
                            "async.forof.close_on_rejection.",
                            ValueInfo::new(ValueKind::Boolean),
                        );
                        AsyncForOfIteratorPlanIr {
                            entry_state,
                            value_resume_state,
                            close_resume_state,
                            exit_state,
                            record: IteratorRecordIr::new(iterator, next_method, done),
                            async_iterator_binding,
                            close_on_rejection_binding,
                        }
                    },
                );
                let protocol = if async_plan.is_some() {
                    IteratorProtocolWitness::ASYNC_ITERATOR_PROTOCOL
                } else {
                    IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL
                };
                (
                    StatementIr::ForOfIterator {
                        head: ForOfIteratorHeadIr::Assignment {
                            binding: ForOfAssignmentIr {
                                mode,
                                name: storage_name,
                            },
                            async_plan,
                            protocol,
                        },
                        iterable,
                        body: Box::new(body),
                        lexical_environment,
                    },
                    protocol,
                )
            }
        };
        ForOfLoweringIr::new(statement, body_kind, protocol)
    }
}
