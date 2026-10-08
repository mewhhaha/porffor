use super::*;
use crate::generator_loop_control::GeneratorLoopControlError;

// The actual For head initialization and original analyzed storage remain shared.
impl ScriptLowerer<'_> {
    pub(super) fn retain_generator_for_head_bindings(
        &mut self,
        head: &ForInitIr,
    ) -> Result<(), GeneratorLoopControlError> {
        if matches!(
            head,
            ForInitIr::SyncDisposable(_) | ForInitIr::AsyncDisposable(_)
        ) {
            return Err(GeneratorLoopControlError::ForeignContinuation);
        }
        Self::visit_for_head_lexical_bindings(head, &mut |mode, name| {
            if matches!(mode, BindingMode::Let | BindingMode::Const) {
                self.add_suspension_owned_binding(name.to_string());
            }
        });
        Ok(())
    }

    pub(super) fn lower_resumable_generator_for_init(
        &mut self,
        source: Option<&ForLoopInitializer>,
    ) -> Result<(Vec<StatementIr>, Option<ForInitIr>), GeneratorLoopControlError> {
        let Some(source) = source else {
            return Ok((Vec::new(), None));
        };
        match source {
            ForLoopInitializer::Expression(source) => {
                let (statement, _) = self.lower_expression_statement(source);
                let statements = match statement {
                    StatementIr::LexicalBlock(statements) => statements,
                    statement => vec![statement],
                };
                Ok((statements, None))
            }
            ForLoopInitializer::Var(source) => {
                let (statement, _) = self.lower_var_statement(source);
                let statements = match statement {
                    StatementIr::LexicalBlock(statements) => statements,
                    statement => vec![statement],
                };
                Ok((statements, None))
            }
            ForLoopInitializer::Lexical(source)
                if contains(source.declaration(), ContainsSymbol::YieldExpression)
                    || (self.current_async_resume_state.is_some()
                        && contains(source.declaration(), ContainsSymbol::AwaitExpression))
                    || source
                        .declaration()
                        .variable_list()
                        .as_ref()
                        .iter()
                        .any(|variable| matches!(variable.binding(), Binding::Pattern(_))) =>
            {
                let mode = match source.declaration() {
                    LexicalDeclaration::Let(_) => BindingMode::Let,
                    LexicalDeclaration::Const(_) => BindingMode::Const,
                    LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => {
                        return Err(GeneratorLoopControlError::ForeignContinuation)
                    }
                };
                let mut statements = Vec::new();
                let mut head = Vec::new();
                let pattern_head = source
                    .declaration()
                    .variable_list()
                    .as_ref()
                    .iter()
                    .any(|variable| matches!(variable.binding(), Binding::Pattern(_)));
                for variable in source.declaration().variable_list().as_ref() {
                    let identifier = match variable.binding() {
                        Binding::Identifier(identifier) => identifier,
                        Binding::Pattern(pattern) => {
                            if self.current_async_resume_state.is_none()
                                && contains(pattern, ContainsSymbol::AwaitExpression)
                            {
                                return Err(GeneratorLoopControlError::ForeignContinuation);
                            }
                            let initializer = variable
                                .init()
                                .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
                            let (pattern, initializer) =
                                if self.async_generator_entry_state().is_none()
                                    && (contains(initializer, ContainsSymbol::YieldExpression)
                                        || contains(pattern, ContainsSymbol::YieldExpression))
                                {
                                    GeneratorPatternInitializerSource::new(
                                        variable,
                                        self.generator_value_branch_admission(),
                                    )
                                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?
                                    .into_parts()
                                } else {
                                    (pattern, initializer)
                                };
                            // Analysis assigns each source BoundName its scoped
                            // head cell. Keep that exact map for every nested
                            // pattern leaf instead of allocating fallback names.
                            let storage_names =
                                supported_bound_names(self.interner, variable.binding())
                                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?
                                    .into_iter()
                                    .map(|bound| {
                                        let storage = scoped_lexical_binding_storage_name(
                                            &bound.source_name,
                                            bound.span,
                                        );
                                        (bound.source_name, storage)
                                    })
                                    .collect::<BTreeMap<_, _>>();
                            if self.async_generator_entry_state().is_some()
                                && (contains(pattern, ContainsSymbol::AwaitExpression)
                                    || contains(pattern, ContainsSymbol::YieldExpression))
                            {
                                statements.extend(
                                    self.lower_async_generator_pattern_initializer(
                                        pattern,
                                        initializer,
                                        mode,
                                        Some(&storage_names),
                                    )
                                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?,
                                );
                                continue;
                            }
                            let (prefix, value) =
                                super::resumable_operand::ResumableOperandProtocol::current(self)
                                    .and_then(|protocol| protocol.lower(self, initializer))
                                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
                            statements.extend(prefix);
                            if self.plain_async_entry_state().is_some()
                                && contains(pattern, ContainsSymbol::AwaitExpression)
                            {
                                let (initialization, _) = self
                                    .lower_staged_async_pattern_with_storage_names(
                                        AsyncPatternSource::new(pattern).ok_or(
                                            GeneratorLoopControlError::ForeignContinuation,
                                        )?,
                                        value,
                                        Some(mode),
                                        Some(&storage_names),
                                    )
                                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
                                statements.extend(initialization);
                                continue;
                            }
                            if contains(pattern, ContainsSymbol::YieldExpression) {
                                let (initialization, _) = match pattern {
                                    Pattern::Object(_) => self
                                        .lower_staged_generator_object_pattern_with_storage_names(
                                            GeneratorObjectPatternSource::new(pattern).ok_or(
                                                GeneratorLoopControlError::ForeignContinuation,
                                            )?,
                                            value,
                                            Some(mode),
                                            Some(&storage_names),
                                        ),
                                    Pattern::Array(_) => self
                                        .lower_staged_generator_array_pattern_with_storage_names(
                                            GeneratorArrayPatternSource::new(pattern).ok_or(
                                                GeneratorLoopControlError::ForeignContinuation,
                                            )?,
                                            value,
                                            Some(mode),
                                            Some(&storage_names),
                                        ),
                                }
                                .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
                                statements.extend(initialization);
                            } else {
                                statements.extend(self.lower_pattern_lexical_binding_from_value_with_storage_names(
                                    mode,
                                    pattern,
                                    value,
                                    Some(&storage_names),
                                )
                                .ok_or(GeneratorLoopControlError::ForeignContinuation)?);
                            }
                            continue;
                        }
                    };
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let (prefix, init) = match variable.init() {
                        Some(source) => {
                            super::resumable_operand::ResumableOperandProtocol::current(self)
                                .and_then(|protocol| protocol.lower(self, source))
                                .ok_or(GeneratorLoopControlError::ForeignContinuation)?
                        }
                        None => (Vec::new(), TypedExpr::undefined()),
                    };
                    statements.extend(prefix);
                    let storage_name =
                        scoped_lexical_binding_storage_name(&name, identifier.span());
                    self.declare_binding(
                        name.clone(),
                        BindingInfo {
                            mode,
                            storage_name: storage_name.clone(),
                            kind: init.kind,
                            possible_kinds: init.possible_kinds,
                            heap_shape: init.heap_shape.clone(),
                            function_targets: init.function_targets.clone(),
                            initialization: Initialization::Initialized,
                        },
                    );
                    self.static_to_string_regexp_object_bindings.remove(&name);
                    head.push(ForLexicalInitIr {
                        mode,
                        name: storage_name.clone(),
                        init: init.clone(),
                    });
                    statements.push(StatementIr::Lexical {
                        mode,
                        name: storage_name,
                        init,
                    });
                }
                let head = if pattern_head {
                    // These are the real initialization operations. A pattern
                    // does not invent scalar ForLexicalInitIr initializers.
                    ForInitIr::Statements(statements.clone())
                } else {
                    ForInitIr::LexicalBlock(head)
                };
                Ok((statements, Some(head)))
            }
            ForLoopInitializer::Lexical(_) => {
                let head = self
                    .lower_for_init(source)
                    .ok_or(GeneratorLoopControlError::ForeignContinuation)?;
                let statements = match &head {
                    ForInitIr::Lexical { mode, name, init } => vec![StatementIr::Lexical {
                        mode: *mode,
                        name: name.clone(),
                        init: init.clone(),
                    }],
                    ForInitIr::LexicalBlock(bindings) => bindings
                        .iter()
                        .map(|binding| StatementIr::Lexical {
                            mode: binding.mode,
                            name: binding.name.clone(),
                            init: binding.init.clone(),
                        })
                        .collect(),
                    ForInitIr::Statements(statements) => statements.clone(),
                    ForInitIr::Var(_)
                    | ForInitIr::Expression(_)
                    | ForInitIr::SyncDisposable(_)
                    | ForInitIr::AsyncDisposable(_) => {
                        return Err(GeneratorLoopControlError::ForeignContinuation)
                    }
                };
                Ok((statements, Some(head)))
            }
        }
    }
}
