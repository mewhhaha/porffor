use super::*;

impl ScriptLowerer<'_> {
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
                    let dispose_state = self
                        .current_async_resume_state
                        .expect("an async-dispose scope must have a plain async state")
                        .checked_add(1)
                        .expect("async-dispose state overflow");
                    let resume_state = dispose_state
                        .checked_add(1)
                        .expect("async-dispose state overflow");
                    let exit_state = resume_state
                        .checked_add(1)
                        .expect("async-dispose state overflow");
                    self.current_async_resume_state = Some(exit_state);
                    let finalizer = AsyncDisposableFinalizerPlanIr::new(
                        scope.entry_state,
                        dispose_state,
                        resume_state,
                        exit_state,
                    );
                    StatementIr::AsyncDisposableScope {
                        capability: AsyncFunctionAsyncDisposableCapabilityIr::new(
                            scope.binding_name,
                            finalizer,
                        ),
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

    /// Lowers one plain-async-function `await using` declaration into the
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

        match self.async_disposable_scope_owner() {
            AsyncDisposableScopeOwnerPlan::AsyncFunction => {}
            AsyncDisposableScopeOwnerPlan::Ordinary => {
                self.unsupported("await using declaration outside a plain async function");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::Generator => {
                self.unsupported("await using declaration in a plain generator");
                return None;
            }
            AsyncDisposableScopeOwnerPlan::AsyncGenerator => {
                self.unsupported("await using declaration in an async generator");
                return None;
            }
        }

        let entry_state = self
            .current_async_resume_state
            .expect("a plain async owner must publish its current resume state");
        let binding_name = self.alloc_suspension_owned_binding(
            "async.function.async.dispose.capability.",
            ValueInfo::new(ValueKind::Object),
        );

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
            entry_state,
            binding_name,
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
