use super::*;

/// A lowering-only statement-list result. Resource declarations remain in
/// this private domain until their containing list finalizes the suffix.
pub(super) enum LoweredStatementListItemIr {
    Statement {
        statement: StatementIr,
        result_kind: ValueKind,
    },
    SyncDisposableScope {
        execution: SyncDisposableScopeExecutionIr,
        resources: SyncDisposableResourcesIr,
    },
    AsyncDisposableScope(PendingAsyncDisposableScopeIr),
}

impl LoweredStatementListItemIr {
    pub(super) fn statement(statement: StatementIr, result_kind: ValueKind) -> Self {
        Self::Statement {
            statement,
            result_kind,
        }
    }
}

/// Lowering-only classification of the resource-bearing for-of heads.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LoweredForOfHeadKind {
    Assignment,
    SyncDisposable,
    AsyncDisposable,
}

/// Lowering-only half of an admitted async `await using` scope.
///
/// The declaration fixes acquisition entry and capability storage immediately;
/// the finalizer states can be minted only after the remaining statement-list
/// suffix has allocated all of its source suspension states.
#[must_use = "a pending async-dispose scope must be finalized around its suffix"]
pub(super) struct PendingAsyncDisposableScopeIr {
    execution: PendingAsyncDisposableScopeExecutionIr,
    resources: AsyncDisposableResourcesIr,
}

/// Lowering-only ownership of an admitted classic-for `await using` head.
///
/// The head is lowered before the test, update, and body. Consuming this value
/// afterward is the only route to the public complete initializer, so a caller
/// cannot publish finalizer states before the entire loop region is known.
#[must_use = "a pending async-disposable classic-for initializer must be finalized"]
pub(super) struct PendingAsyncDisposableForInitIr {
    entry_state: u32,
    binding_name: String,
    resources: AsyncDisposableResourcesIr,
}

/// Lowering-only ownership of an admitted plain-async `for-of` resource head.
///
/// Iterator storage and capability identity are fixed before the body is
/// lowered. Consuming this carrier afterward is the only way to mint the
/// public head with its complete finalizer state plan.
#[must_use = "a pending async-disposable for-of head must be finalized after its body"]
pub(super) struct PendingAsyncDisposableForOfHeadIr {
    entry_state: u32,
    binding_name: String,
    capability_binding_name: String,
    record: IteratorRecordIr,
}

/// The private pre-finalizer owner proof for one async-dispose scope.
///
/// Keeping both admitted owners named prevents async-generator storage from
/// being finalized as a plain-async capability by convention.
#[must_use = "an async-dispose execution owner must be finalized into public IR"]
enum PendingAsyncDisposableScopeExecutionIr {
    AsyncFunction {
        entry_state: u32,
        binding_name: String,
    },
    AsyncGenerator {
        entry_state: u32,
        binding_name: String,
    },
}

impl PendingAsyncDisposableScopeExecutionIr {
    fn entry_state(&self) -> u32 {
        match self {
            Self::AsyncFunction { entry_state, .. } | Self::AsyncGenerator { entry_state, .. } => {
                *entry_state
            }
        }
    }

    fn finalize(
        self,
        finalizer: AsyncDisposableFinalizerPlanIr,
    ) -> AsyncDisposableScopeExecutionIr {
        match self {
            Self::AsyncFunction { binding_name, .. } => {
                AsyncDisposableScopeExecutionIr::AsyncFunction(
                    AsyncFunctionAsyncDisposableCapabilityIr::new(binding_name, finalizer),
                )
            }
            Self::AsyncGenerator { binding_name, .. } => {
                AsyncDisposableScopeExecutionIr::AsyncGenerator(
                    AsyncGeneratorAsyncDisposableCapabilityIr::new(binding_name, finalizer),
                )
            }
        }
    }
}

enum LoweredDisposableScopeIr {
    Sync {
        execution: SyncDisposableScopeExecutionIr,
        resources: SyncDisposableResourcesIr,
    },
    Async(PendingAsyncDisposableScopeIr),
}

impl ScriptLowerer<'_> {
    /// Admits only the bounded plain-async, synchronous-iterator identifier
    /// source form. Keeping the rejection matrix here leaves the main for-of
    /// classifier as a small exhaustive shape decision.
    pub(super) fn admit_async_disposable_for_of_head(
        &mut self,
        for_of: &ForOfLoop,
        binding: &Binding,
    ) -> Option<String> {
        if for_of.r#await() {
            self.unsupported("await using declaration in for-await-of");
            return None;
        }
        if contains(for_of.iterable(), ContainsSymbol::AwaitExpression)
            || contains(for_of.iterable(), ContainsSymbol::YieldExpression)
            || contains(for_of.body(), ContainsSymbol::AwaitExpression)
            || contains(for_of.body(), ContainsSymbol::YieldExpression)
        {
            self.unsupported("source suspension in await using for-of loop");
            return None;
        }
        let Binding::Identifier(identifier) = binding else {
            self.unsupported("await using declaration binding pattern in for-of");
            return None;
        };
        Some(self.interner.resolve_expect(identifier.sym()).to_string())
    }

    pub(super) fn async_disposable_for_head(for_loop: &ForLoop) -> Option<&[Variable]> {
        match for_loop.init() {
            Some(ForLoopInitializer::Lexical(lexical)) => match lexical.declaration() {
                LexicalDeclaration::AwaitUsing(list) => Some(list.as_ref()),
                LexicalDeclaration::Let(_)
                | LexicalDeclaration::Const(_)
                | LexicalDeclaration::Using(_) => None,
            },
            Some(ForLoopInitializer::Expression(_) | ForLoopInitializer::Var(_)) | None => None,
        }
    }

    pub(super) fn async_disposable_for_has_source_suspension(for_loop: &ForLoop) -> bool {
        [for_loop.condition(), for_loop.final_expr()]
            .into_iter()
            .flatten()
            .any(|expression| {
                contains(expression, ContainsSymbol::AwaitExpression)
                    || contains(expression, ContainsSymbol::YieldExpression)
            })
            || contains(for_loop.body(), ContainsSymbol::AwaitExpression)
            || contains(for_loop.body(), ContainsSymbol::YieldExpression)
    }

    /// Replaces each declaration marker with a scope over the remaining suffix.
    ///
    /// A resource is live only if evaluation reached its declaration. Nesting
    /// the suffix makes that reachability structural and composes reverse
    /// disposal order across interleaved synchronous and asynchronous scopes.
    pub(super) fn finish_disposable_scopes(
        &mut self,
        items: Vec<LoweredStatementListItemIr>,
    ) -> BlockIr {
        let mut segments = Vec::new();
        let mut current = Vec::new();
        let mut current_kind = ValueKind::Undefined;
        for item in items {
            match item {
                LoweredStatementListItemIr::Statement {
                    statement,
                    result_kind,
                } => {
                    current.push(statement);
                    current_kind = result_kind;
                }
                LoweredStatementListItemIr::SyncDisposableScope {
                    execution,
                    resources,
                } => {
                    segments.push((
                        current,
                        LoweredDisposableScopeIr::Sync {
                            execution,
                            resources,
                        },
                    ));
                    current = Vec::new();
                    current_kind = ValueKind::Undefined;
                }
                LoweredStatementListItemIr::AsyncDisposableScope(scope) => {
                    segments.push((current, LoweredDisposableScopeIr::Async(scope)));
                    current = Vec::new();
                    current_kind = ValueKind::Undefined;
                }
            }
        }

        if segments.is_empty() {
            return BlockIr {
                statements: current,
                result_kind: current_kind,
                lexical_environment: None,
            };
        }

        let mut suffix = BlockIr {
            statements: current,
            result_kind: current_kind,
            lexical_environment: None,
        };
        for (mut prefix, scope) in segments.into_iter().rev() {
            let result_kind = suffix.result_kind;
            let statement = match scope {
                LoweredDisposableScopeIr::Sync {
                    execution,
                    resources,
                } => StatementIr::SyncDisposableScope {
                    execution,
                    resources,
                    body: suffix,
                },
                LoweredDisposableScopeIr::Async(scope) => {
                    let finalizer =
                        self.allocate_async_disposable_finalizer(scope.execution.entry_state());
                    StatementIr::AsyncDisposableScope {
                        execution: scope.execution.finalize(finalizer),
                        resources: scope.resources,
                        body: suffix,
                    }
                }
            };
            prefix.push(statement);
            suffix = BlockIr {
                statements: prefix,
                result_kind,
                lexical_environment: None,
            };
        }
        suffix
    }

    fn allocate_async_disposable_finalizer(
        &mut self,
        entry_state: u32,
    ) -> AsyncDisposableFinalizerPlanIr {
        let dispose_state = self
            .current_async_resume_state
            .expect("an async-dispose scope must have an async resume state")
            .checked_add(1)
            .expect("async-dispose state overflow");
        let resume_state = dispose_state
            .checked_add(1)
            .expect("async-dispose state overflow");
        let exit_state = resume_state
            .checked_add(1)
            .expect("async-dispose state overflow");
        self.current_async_resume_state = Some(exit_state);
        if self.current_resumable_plan.is_some() {
            self.current_generator_resume_state = Some(exit_state);
        }
        AsyncDisposableFinalizerPlanIr::new(entry_state, dispose_state, resume_state, exit_state)
    }

    /// Lowers the resource side of an admitted classic-for `await using` head.
    /// Finalizer states are deliberately absent until [`Self::finish_async_disposable_for_init`].
    pub(super) fn lower_async_disposable_for_init(
        &mut self,
        list: &[Variable],
    ) -> Option<PendingAsyncDisposableForInitIr> {
        if list.is_empty() {
            self.unsupported("empty await using declaration");
            return None;
        }
        if list
            .iter()
            .any(|variable| !matches!(variable.binding(), Binding::Identifier(_)))
        {
            self.unsupported("await using classic-for binding pattern");
            return None;
        }
        if list.iter().any(|variable| variable.init().is_none()) {
            self.unsupported("await using declaration without initializer");
            return None;
        }
        if list.iter().filter_map(Variable::init).any(|initializer| {
            contains(initializer, ContainsSymbol::AwaitExpression)
                || contains(initializer, ContainsSymbol::YieldExpression)
        }) {
            self.unsupported("suspension inside an await using classic-for initializer");
            return None;
        }
        match self.async_disposable_scope_owner() {
            AsyncDisposableScopeOwnerPlan::AsyncFunction => {}
            AsyncDisposableScopeOwnerPlan::AsyncGenerator => {
                self.unsupported("await using classic-for head in an async generator");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::Generator => {
                self.unsupported("await using classic-for head in a plain generator");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::Ordinary => {
                self.unsupported("await using classic-for head outside a plain async function");
                return None;
            }
        }

        let entry_state = self
            .current_async_resume_state
            .expect("an admitted plain async owner must publish its current resume state");
        let binding_name = self.alloc_suspension_owned_binding(
            "async.function.for.await.dispose.capability.",
            ValueInfo::new(ValueKind::Object),
        );
        let mut resources = Vec::with_capacity(list.len());
        for variable in list {
            let Binding::Identifier(identifier) = variable.binding() else {
                unreachable!("binding patterns were rejected before lowering")
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            let initializer = variable
                .init()
                .expect("await using initializers were validated before lowering");
            let init = self.lower_expression(initializer);
            self.static_iterator_binding_values.remove(&name);
            self.static_string_bindings.remove(&name);
            self.static_to_string_regexp_object_bindings.remove(&name);
            let storage_name = scoped_lexical_binding_storage_name(&name, identifier.span());
            let initialized =
                InitializedBinding::without_creation(name, BindingMode::Const, storage_name, init);
            resources.push(initialized.into_async_disposable_resource(self));
        }
        let mut resources = resources.into_iter();
        let first = resources
            .next()
            .expect("a parsed await using BindingList is non-empty");
        Some(PendingAsyncDisposableForInitIr {
            entry_state,
            binding_name,
            resources: AsyncDisposableResourcesIr::new(first, resources.collect()),
        })
    }

    pub(super) fn finish_async_disposable_for_init(
        &mut self,
        pending: PendingAsyncDisposableForInitIr,
    ) -> AsyncDisposableForInitIr {
        let finalizer = self.allocate_async_disposable_finalizer(pending.entry_state);
        AsyncDisposableForInitIr::new(
            AsyncFunctionAsyncDisposableCapabilityIr::new(pending.binding_name, finalizer),
            pending.resources,
        )
    }

    /// Fixes every activation-owned role of a plain-async `for-of` resource
    /// head before its body is lowered, but deliberately leaves the finalizer
    /// states absent until the body has consumed any admitted source states.
    pub(super) fn begin_async_disposable_for_of_head(
        &mut self,
        binding_name: String,
    ) -> Option<PendingAsyncDisposableForOfHeadIr> {
        match self.async_disposable_scope_owner() {
            AsyncDisposableScopeOwnerPlan::AsyncFunction => {}
            AsyncDisposableScopeOwnerPlan::AsyncGenerator => {
                self.unsupported("await using for-of head in an async generator");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::Generator => {
                self.unsupported("await using for-of head in a plain generator");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::Ordinary => {
                self.unsupported("await using for-of head outside a plain async function");
                return None;
            }
        }

        let entry_state = self
            .current_async_resume_state
            .expect("an admitted plain async owner must publish its current resume state");
        let capability_binding_name = self.alloc_suspension_owned_binding(
            "async.function.forof.await.dispose.capability.",
            ValueInfo::new(ValueKind::Object),
        );
        let record = IteratorRecordIr::new(
            self.alloc_iterator_slot(),
            self.alloc_next_method_slot(),
            self.alloc_done_slot(),
        );
        Some(PendingAsyncDisposableForOfHeadIr {
            entry_state,
            binding_name,
            capability_binding_name,
            record,
        })
    }

    pub(super) fn begin_async_disposable_for_of_if_needed(
        &mut self,
        head_kind: LoweredForOfHeadKind,
        binding_name: &str,
    ) -> Result<Option<PendingAsyncDisposableForOfHeadIr>, ()> {
        match head_kind {
            LoweredForOfHeadKind::Assignment | LoweredForOfHeadKind::SyncDisposable => Ok(None),
            LoweredForOfHeadKind::AsyncDisposable => self
                .begin_async_disposable_for_of_head(binding_name.to_string())
                .map(Some)
                .ok_or(()),
        }
    }

    /// Completes the repeating per-iteration finalizer only after the loop body
    /// has been lowered.
    pub(super) fn finish_async_disposable_for_of_head(
        &mut self,
        pending: PendingAsyncDisposableForOfHeadIr,
    ) -> AsyncDisposableForOfHeadIr {
        let finalizer = self.allocate_async_disposable_finalizer(pending.entry_state);
        AsyncDisposableForOfHeadIr::new(
            pending.binding_name,
            AsyncFunctionAsyncDisposableForOfCapabilityIr::new(
                pending.capability_binding_name,
                finalizer,
            ),
            pending.record,
        )
    }

    pub(super) fn async_disposable_for_of_statement(
        head: AsyncDisposableForOfHeadIr,
        iterable: TypedExpr,
        body: StatementIr,
        lexical_environment: Option<ForInOfEnvironmentIr>,
    ) -> (StatementIr, IteratorProtocolWitness) {
        let protocol = IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL;
        (
            StatementIr::ForOfIterator {
                head: ForOfIteratorHeadIr::AsyncDisposable(head),
                iterable,
                body: Box::new(body),
                lexical_environment,
            },
            protocol,
        )
    }

    /// Lowers one admitted async-owner `await using` declaration into the
    /// pending half of a dedicated async-dispose scope.
    ///
    /// Finalizer states are allocated only after the suffix has consumed all
    /// source Await states. The returned carrier cannot escape statement-list
    /// finalization.
    pub(super) fn lower_await_using_declaration(
        &mut self,
        list: &[Variable],
        scope: &mut LexicalScopeInstantiation,
    ) -> Option<PendingAsyncDisposableScopeIr> {
        if list.is_empty() {
            self.unsupported("empty await using declaration");
            return None;
        }
        if list
            .iter()
            .any(|variable| !matches!(variable.binding(), Binding::Identifier(_)))
        {
            self.unsupported("await using declaration binding pattern");
            return None;
        }
        if list.iter().any(|variable| variable.init().is_none()) {
            self.unsupported("await using declaration without initializer");
            return None;
        }
        if list.iter().filter_map(Variable::init).any(|initializer| {
            contains(initializer, ContainsSymbol::AwaitExpression)
                || contains(initializer, ContainsSymbol::YieldExpression)
        }) {
            self.unsupported("suspension inside an await using initializer");
            return None;
        }

        let owner = self.async_disposable_scope_owner();
        match owner {
            AsyncDisposableScopeOwnerPlan::AsyncFunction
            | AsyncDisposableScopeOwnerPlan::AsyncGenerator => {}
            AsyncDisposableScopeOwnerPlan::Ordinary => {
                self.unsupported("await using declaration outside an async function or generator");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::Generator => {
                self.unsupported("await using declaration in a plain generator");
                return None;
            }
        }

        let entry_state = self
            .current_async_resume_state
            .expect("an admitted async owner must publish its current resume state");
        let execution = match owner {
            AsyncDisposableScopeOwnerPlan::AsyncFunction => {
                PendingAsyncDisposableScopeExecutionIr::AsyncFunction {
                    entry_state,
                    binding_name: self.alloc_suspension_owned_binding(
                        "async.function.async.dispose.capability.",
                        ValueInfo::new(ValueKind::Object),
                    ),
                }
            }
            AsyncDisposableScopeOwnerPlan::AsyncGenerator => {
                PendingAsyncDisposableScopeExecutionIr::AsyncGenerator {
                    entry_state,
                    binding_name: self.alloc_suspension_owned_binding(
                        "async.generator.await.dispose.capability.",
                        ValueInfo::new(ValueKind::Object),
                    ),
                }
            }
            AsyncDisposableScopeOwnerPlan::Ordinary | AsyncDisposableScopeOwnerPlan::Generator => {
                unreachable!("unsupported async-dispose owners returned before allocation")
            }
        };

        let mut resources = Vec::with_capacity(list.len());
        for variable in list {
            let Binding::Identifier(identifier) = variable.binding() else {
                unreachable!("binding patterns were rejected before lowering")
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            let pending = scope.take(&name);
            let initializer = variable
                .init()
                .expect("await using initializers were validated before lowering");
            let init = self.lower_expression(initializer);
            self.static_iterator_binding_values.remove(&name);
            self.static_string_bindings.remove(&name);
            self.static_to_string_regexp_object_bindings.remove(&name);
            let init = LoweredInitializer::evaluated(init);
            let initialized = match pending {
                Some(pending) => pending.initialize(init),
                None => {
                    let storage_name = self.direct_lexical_storage_name(&name, identifier.span());
                    InitializedBinding::without_creation(
                        name.clone(),
                        BindingMode::Const,
                        storage_name,
                        init.into_expr(),
                    )
                }
            };
            resources.push(initialized.into_async_disposable_resource(self));
        }

        let mut resources = resources.into_iter();
        let first = resources
            .next()
            .expect("a parsed await using BindingList is non-empty");
        Some(PendingAsyncDisposableScopeIr {
            execution,
            resources: AsyncDisposableResourcesIr::new(first, resources.collect()),
        })
    }

    fn async_disposable_scope_owner(&self) -> AsyncDisposableScopeOwnerPlan {
        self.current_function_id
            .as_ref()
            .and_then(|function_id| self.analysis.function_plans.get(function_id))
            .map_or(AsyncDisposableScopeOwnerPlan::Ordinary, |function| {
                function.async_disposable_scope_owner()
            })
    }
}
