use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_var_statement(
        &mut self,
        declaration: &VarDeclaration,
    ) -> (StatementIr, ValueKind) {
        let mut statements = Vec::new();
        let mut declarators = Vec::new();

        for variable in declaration.0.as_ref() {
            match variable.binding() {
                Binding::Identifier(_)
                    if self.async_generator_entry_state().is_some()
                        && variable.init().is_some_and(|source| {
                            contains(source, ContainsSymbol::AwaitExpression)
                                || contains(source, ContainsSymbol::YieldExpression)
                        }) =>
                {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let Some(prefix) = self.lower_mixed_var_identifier_initializer(variable) else {
                        self.unsupported("mixed async-generator var initializer");
                        return (StatementIr::Empty, ValueKind::Undefined);
                    };
                    statements.extend(prefix);
                }
                Binding::Identifier(identifier)
                    if self.plain_async_entry_state().is_some()
                        && !self.with_environment_chain.is_empty()
                        && variable.init().is_some_and(|source| {
                            contains(source, ContainsSymbol::AwaitExpression)
                        }) =>
                {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    statements.push(StatementIr::Var(vec![VarDeclaratorIr {
                        name: name.clone(),
                        init: None,
                    }]));
                    let Some((prefix, value)) = self.lower_async_identifier_initializer(
                        name,
                        variable.init().expect("checked awaited initializer"),
                    ) else {
                        self.unsupported(
                            "awaited var initializer requires its original With Reference owner",
                        );
                        return (StatementIr::Empty, ValueKind::Undefined);
                    };
                    statements.extend(prefix);
                    statements.push(StatementIr::DeclarationEvaluation(value));
                }
                Binding::Identifier(identifier)
                    if self.current_async_resume_state.is_some()
                        && matches!(variable.init(), Some(Expression::Await(_))) =>
                {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let Some(Expression::Await(await_expression)) = variable.init() else {
                        unreachable!()
                    };
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let (awaited_prefix, awaited_value) = if contains(
                        await_expression.target(),
                        ContainsSymbol::AwaitExpression,
                    ) {
                        let Some(staged) =
                            self.lower_async_prefixed_expression(await_expression.target())
                        else {
                            self.unsupported("conditionally reached or mixed suspension in async var await target");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        staged
                    } else {
                        (Vec::new(), self.lower_expression(await_expression.target()))
                    };
                    statements.extend(awaited_prefix);
                    self.static_boolean_bindings.remove(&name);
                    self.static_to_string_regexp_object_bindings.remove(&name);
                    self.set_binding_value_info(
                        &name,
                        ValueInfo {
                            kind: ValueKind::Dynamic,
                            possible_kinds: KindSet::all_runtime_tags(),
                            heap_shape: None,
                            function_targets: FunctionTargetKnowledge::unknown(),
                        },
                    );
                    statements.push(StatementIr::Var(vec![VarDeclaratorIr {
                        name: name.clone(),
                        init: None,
                    }]));
                    let (await_statement, _) = self.lower_linear_async_await_value(
                        awaited_value,
                        AsyncResumeModeIr::AssignIdentifier(name),
                    );
                    statements.push(await_statement);
                }
                Binding::Identifier(identifier)
                    if variable
                        .init()
                        .is_some_and(|init| self.async_initializer_is_stageable(init)) =>
                {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    statements.push(StatementIr::Var(vec![VarDeclaratorIr {
                        name: name.clone(),
                        init: None,
                    }]));
                    let (prefix, value) = self
                        .lower_async_prefixed_expression(
                            variable
                                .init()
                                .expect("guarded async var initializer must exist"),
                        )
                        .expect("stageable async var initializer must stage");
                    statements.extend(prefix);
                    self.static_boolean_bindings.remove(&name);
                    self.static_to_string_regexp_object_bindings.remove(&name);
                    self.set_binding_value_info(&name, value.value_info());
                    statements.push(StatementIr::DeclarationEvaluation(
                        self.lower_identifier_assign_value(name, value),
                    ));
                }
                Binding::Identifier(_)
                    if self.current_async_resume_state.is_some()
                        && variable
                            .init()
                            .is_some_and(|init| self.has_branch_sensitive_await(init)) =>
                {
                    self.unsupported("async var initializer branch-sensitive await expression");
                    return (StatementIr::Empty, ValueKind::Undefined);
                }
                Binding::Identifier(identifier)
                    if self.current_generator_resume_state.is_some()
                        && matches!(variable.init(), Some(Expression::Yield(_))) =>
                {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let Some(Expression::Yield(yield_expression)) = variable.init() else {
                        unreachable!()
                    };
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    self.set_binding_value_info(
                        &name,
                        ValueInfo {
                            kind: ValueKind::Dynamic,
                            possible_kinds: KindSet::all_runtime_tags(),
                            heap_shape: None,
                            function_targets: FunctionTargetKnowledge::unknown(),
                        },
                    );
                    statements.push(StatementIr::Var(vec![VarDeclaratorIr {
                        name: name.clone(),
                        init: None,
                    }]));
                    let (yield_statement, _) = self.lower_linear_generator_yield(
                        yield_expression.target(),
                        yield_expression.delegate(),
                        GeneratorResumeModeIr::AssignIdentifier(name),
                    );
                    statements.push(yield_statement);
                }
                Binding::Identifier(identifier)
                    if self.plain_generator_entry_state().is_some()
                        && (self.loop_depth == 0 || self.ordinary_generator_region_depth > 0)
                        && variable.init().is_some_and(|init| {
                            contains(init, ContainsSymbol::YieldExpression)
                        })
                        && self.with_environment_chain.is_empty() =>
                {
                    // A generator's own `var` always exists in its function
                    // environment, so even eval-visible (runtime) resolution
                    // finds the same binding before and after the initializer.
                    let init = variable.init().expect("guarded generator var initializer");
                    if GeneratorExpressionSourcePlan::new(
                        init,
                        self.generator_value_branch_admission(),
                    )
                    .is_none()
                    {
                        self.unsupported("generator var initializer suspension expression");
                        return (StatementIr::Empty, ValueKind::Undefined);
                    }
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    statements.push(StatementIr::Var(vec![VarDeclaratorIr {
                        name: name.clone(),
                        init: None,
                    }]));
                    let Some((prefix, value)) = self.lower_staged_generator_expression(init) else {
                        self.unsupported("generator var initializer suspension expression");
                        return (StatementIr::Empty, ValueKind::Undefined);
                    };
                    statements.extend(prefix);
                    self.static_boolean_bindings.remove(&name);
                    self.static_to_string_regexp_object_bindings.remove(&name);
                    statements.push(StatementIr::DeclarationEvaluation(
                        self.lower_identifier_assign_value(name, value),
                    ));
                }
                Binding::Identifier(identifier)
                    if !self.with_environment_chain.is_empty() && variable.init().is_some() =>
                {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    if !self.borrows_direct_eval_variable_environment() {
                        statements
                            .push(StatementIr::Var(vec![VarDeclaratorIr { name, init: None }]));
                    }
                    // VariableDeclaration resolves its reference before evaluating
                    // the initializer, just as an identifier assignment does.
                    let initializer = self.lower_assign(
                        AssignOp::Assign,
                        &AssignTarget::Identifier(*identifier),
                        variable.init().expect("guarded var initializer must exist"),
                    );
                    statements.push(StatementIr::DeclarationEvaluation(initializer));
                }
                Binding::Identifier(_) => {
                    if let Some(declarator) = self.lower_var_declarator(variable) {
                        if self.borrows_direct_eval_variable_environment() {
                            if let Some(value) = declarator.init {
                                statements.push(StatementIr::DeclarationEvaluation(
                                    self.environment_identifier(
                                        declarator.name,
                                        EnvironmentIdentifierOperationIr::Assign {
                                            value: Box::new(value),
                                        },
                                    ),
                                ));
                            }
                        } else {
                            declarators.push(declarator);
                        }
                    }
                }
                Binding::Pattern(pattern) => {
                    if !declarators.is_empty() {
                        statements.push(StatementIr::Var(std::mem::take(&mut declarators)));
                    }
                    if self.async_generator_entry_state().is_some()
                        && (contains(pattern, ContainsSymbol::AwaitExpression)
                            || contains(pattern, ContainsSymbol::YieldExpression))
                    {
                        let Some(init) = variable.init() else {
                            self.unsupported("destructuring binding without initializer");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        let Some(prefix) = self.lower_async_generator_pattern_initializer(
                            pattern,
                            init,
                            BindingMode::Var,
                            None,
                        ) else {
                            self.unsupported(
                                "mixed async-generator var pattern continuation ownership",
                            );
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        statements.push(StatementIr::LexicalBlock(prefix));
                        continue;
                    }
                    if self.async_generator_entry_state().is_some()
                        && variable.init().is_some_and(|source| {
                            contains(source, ContainsSymbol::AwaitExpression)
                                || contains(source, ContainsSymbol::YieldExpression)
                        })
                    {
                        if contains(pattern, ContainsSymbol::AwaitExpression)
                            || contains(pattern, ContainsSymbol::YieldExpression)
                        {
                            self.unsupported("mixed async-generator pattern-owned suspension");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        }
                        let Some((mut prefix, value)) = self.lower_mixed_generator_value(
                            variable.init().expect("checked mixed pattern initializer"),
                        ) else {
                            self.unsupported("mixed async-generator var pattern initializer");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        let Some(bindings) =
                            self.lower_pattern_var_binding_from_value(pattern, value)
                        else {
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        prefix.extend(bindings);
                        statements.push(StatementIr::LexicalBlock(prefix));
                        continue;
                    }
                    if self.plain_async_entry_state().is_some()
                        && contains(pattern, ContainsSymbol::AwaitExpression)
                    {
                        let Some(initializer) = variable.init() else {
                            self.unsupported("destructuring binding without initializer");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        // The staged pattern only assigns through retained
                        // references, so each bound name is declared first,
                        // as the identifier forms above do.
                        let Some(names) = supported_bound_names(
                            self.interner,
                            &Binding::Pattern(pattern.clone()),
                        ) else {
                            self.unsupported("async var pattern bound names");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        let Some(bindings) =
                            self.lower_async_pattern_var_initializer(pattern, initializer)
                        else {
                            self.unsupported("async var pattern continuation ownership");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        statements.push(StatementIr::Var(
                            names
                                .into_iter()
                                .map(|bound| VarDeclaratorIr {
                                    name: bound.source_name,
                                    init: None,
                                })
                                .collect(),
                        ));
                        statements.push(StatementIr::LexicalBlock(bindings));
                        continue;
                    }
                    if matches!(
                        self.generator_value_branch_admission(),
                        GeneratorValueBranchAdmission::OrdinaryOutsideLoops
                    ) && (variable
                        .init()
                        .is_some_and(|init| contains(init, ContainsSymbol::YieldExpression))
                        || contains(pattern, ContainsSymbol::YieldExpression))
                    {
                        let Some(source) = GeneratorPatternInitializerSource::new(
                            variable,
                            self.generator_value_branch_admission(),
                        ) else {
                            self.unsupported("generator binding pattern suspension requires its own continuation");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        let Some(bindings) = self.lower_generator_var_pattern_initializer(source)
                        else {
                            self.unsupported("generator pattern initializer suspension expression");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        statements.push(StatementIr::LexicalBlock(bindings));
                        continue;
                    }
                    if let Some(bindings) = self.lower_pattern_var_binding(pattern, variable.init())
                    {
                        statements.push(StatementIr::LexicalBlock(bindings));
                    }
                }
            }
        }

        if !declarators.is_empty() {
            statements.push(StatementIr::Var(declarators));
        }

        let statement = if statements.len() == 1 {
            statements.remove(0)
        } else {
            StatementIr::LexicalBlock(statements)
        };
        (statement, ValueKind::Undefined)
    }

    pub(super) fn lower_var_init(&mut self, declaration: &VarDeclaration) -> Option<ForInitIr> {
        // Borrowed eval variables, patterns and with-environment references
        // need declaration lowering: VarDeclaratorIr only writes owned storage.
        // Hoisting still belongs to the enclosing variable environment.
        if self.borrows_direct_eval_variable_environment()
            || !self.with_environment_chain.is_empty()
            || declaration
                .0
                .as_ref()
                .iter()
                .any(|variable| matches!(variable.binding(), Binding::Pattern(_)))
        {
            let (statement, _) = self.lower_var_statement(declaration);
            return Some(ForInitIr::Statements(vec![statement]));
        }
        Some(ForInitIr::Var(self.lower_var_declarators(declaration)))
    }

    fn lower_var_declarators(&mut self, declaration: &VarDeclaration) -> Vec<VarDeclaratorIr> {
        let mut declarators = Vec::with_capacity(declaration.0.as_ref().len());
        for variable in declaration.0.as_ref() {
            if let Some(declarator) = self.lower_var_declarator(variable) {
                declarators.push(declarator);
            }
        }
        declarators
    }

    pub(super) fn lower_mixed_var_identifier_initializer(
        &mut self,
        variable: &Variable,
    ) -> Option<Vec<StatementIr>> {
        self.async_generator_entry_state()?;
        self.lower_resumable_var_identifier_initializer(
            variable,
            super::resumable_operand::ResumableOperandProtocol::Mixed,
        )
    }

    pub(super) fn lower_resumable_var_identifier_initializer(
        &mut self,
        variable: &Variable,
        protocol: super::resumable_operand::ResumableOperandProtocol,
    ) -> Option<Vec<StatementIr>> {
        protocol.entry_state(self)?;
        let Binding::Identifier(identifier) = variable.binding() else {
            return None;
        };
        let initializer = variable.init()?;
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let mut statements = vec![StatementIr::Var(vec![VarDeclaratorIr {
            name: name.clone(),
            init: None,
        }])];
        let mut prefix = Vec::new();
        let reference = super::generator_identifier_reference::RetainedGeneratorIdentifierTarget::capture_write_only(self, &mut prefix, name.clone());
        let (rhs, value) = protocol.lower(self, initializer)?;
        prefix.extend(rhs);
        let value = reference.put_value(self, value);
        statements.extend(prefix);
        self.static_boolean_bindings.remove(&name);
        self.static_to_string_regexp_object_bindings.remove(&name);
        self.set_binding_value_info(&name, value.value_info());
        statements.push(StatementIr::DeclarationEvaluation(value));
        Some(statements)
    }

    pub(super) fn lower_var_declarator(&mut self, variable: &Variable) -> Option<VarDeclaratorIr> {
        let Binding::Identifier(identifier) = variable.binding() else {
            self.unsupported("destructuring var declaration");
            return None;
        };

        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        if variable
            .init()
            .is_some_and(|expression| self.is_constructor_prototype_expr(expression, ARRAY_NAME))
        {
            self.array_prototype_mutated = true;
        }
        let static_to_string_regexp_object = variable
            .init()
            .is_some_and(|expression| self.static_to_string_returns_regexp_object_expr(expression));
        let static_string_value = variable
            .init()
            .and_then(|expression| self.static_string_expression(expression));
        let source_candidate = variable
            .init()
            .map(|expression| self.function_source_value_candidates(expression));
        let init = variable
            .init()
            .map(|expression| self.lower_expression(expression));
        if let Some(expression) = variable.init() {
            if let Some(value) = self.static_boolean_receiver_value(expression) {
                self.static_boolean_bindings.insert(name.clone(), value);
            } else {
                self.static_boolean_bindings.remove(&name);
            }
            if static_to_string_regexp_object {
                self.static_to_string_regexp_object_bindings
                    .insert(name.clone());
            } else {
                self.static_to_string_regexp_object_bindings.remove(&name);
            }
        } else {
            self.static_boolean_bindings.remove(&name);
            self.static_to_string_regexp_object_bindings.remove(&name);
        }
        let static_boolean_value = self.static_boolean_bindings.get(&name).copied();
        let static_to_string_regexp_object =
            self.static_to_string_regexp_object_bindings.contains(&name);
        if let Some(init) = &init {
            self.set_binding_value_info(&name, init.value_info());
            let binding = self.lookup_binding(&name).unwrap_or_else(|| {
                panic!(
                    "var initializer target `{name}` must remain declared while installing facts"
                )
            });
            if let Some(value) = static_boolean_value {
                self.static_boolean_bindings.insert(name.clone(), value);
            }
            if let Some(candidate) = source_candidate {
                self.function_source_binding_candidates
                    .insert(binding.storage_name.clone(), candidate);
            }
            if let Some(value) = static_string_value {
                self.static_string_bindings.insert(&binding, value);
            } else {
                self.static_string_bindings.remove(&binding);
            }
            if static_to_string_regexp_object {
                self.static_to_string_regexp_object_bindings
                    .insert(name.clone());
            }
        } else if let Some(binding) = self.lookup_binding(&name) {
            self.static_string_bindings.remove(&binding);
        }
        Some(VarDeclaratorIr { name, init })
    }
}
