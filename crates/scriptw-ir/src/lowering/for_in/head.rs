//! The original eager ForIn head is shared by ordinary and resumed enumeration.

use super::*;
use crate::lowering::pattern_target::PatternContinuation;

pub(in crate::lowering) struct ForInIterationHead {
    mode: BindingMode,
    name: String,
    storage_name: String,
    destination: ForInDestination,
}

enum ForInDestination {
    Binding,
    BorrowedVar(String),
    Identifier(String),
    Property(PropertyAccess),
    AssignmentPattern(Pattern),
    BindingPattern {
        mode: BindingMode,
        pattern: Pattern,
        storage_names: BTreeMap<String, String>,
    },
}

enum ForInLexicalMode {
    Let,
    Const,
}

impl ForInIterationHead {
    pub(in crate::lowering) const fn mode(&self) -> BindingMode {
        self.mode
    }
    pub(in crate::lowering) fn storage_name(&self) -> &str {
        &self.storage_name
    }
}

impl ScriptLowerer<'_> {
    pub(in crate::lowering) fn prepare_for_in_iteration_head(
        &mut self,
        source: &boa_ast::statement::iteration::ForInLoop,
    ) -> Option<ForInIterationHead> {
        let (mode, name, destination) = match source.initializer() {
            IterableLoopInitializer::Identifier(identifier) => (
                BindingMode::Let,
                self.alloc_temp_binding_name("forin.assignment"),
                ForInDestination::Identifier(
                    self.interner.resolve_expect(identifier.sym()).to_string(),
                ),
            ),
            IterableLoopInitializer::Var(variable) => match variable.binding() {
                Binding::Identifier(identifier) => {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    if self.borrows_direct_eval_variable_environment() {
                        (
                            BindingMode::Let,
                            self.alloc_temp_binding_name("forin.var"),
                            ForInDestination::BorrowedVar(name),
                        )
                    } else {
                        (BindingMode::Var, name, ForInDestination::Binding)
                    }
                }
                Binding::Pattern(pattern) => (
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forin"),
                    ForInDestination::BindingPattern {
                        mode: BindingMode::Var,
                        pattern: pattern.clone(),
                        storage_names: BTreeMap::new(),
                    },
                ),
            },
            IterableLoopInitializer::Let(binding) => {
                self.prepare_for_in_lexical_head(source, binding, ForInLexicalMode::Let)?
            }
            IterableLoopInitializer::Const(binding) => {
                self.prepare_for_in_lexical_head(source, binding, ForInLexicalMode::Const)?
            }
            IterableLoopInitializer::Pattern(pattern) => (
                BindingMode::Let,
                self.alloc_temp_binding_name("forin"),
                ForInDestination::AssignmentPattern(pattern.clone()),
            ),
            IterableLoopInitializer::Access(access) => (
                BindingMode::Let,
                self.alloc_temp_binding_name("forin.access"),
                ForInDestination::Property(access.clone()),
            ),
            IterableLoopInitializer::Using(_)
            | IterableLoopInitializer::AwaitUsing(_)
            | IterableLoopInitializer::WebCompatCall(_) => {
                self.unsupported("for-in initializer");
                return None;
            }
        };
        let storage_name = if matches!(destination, ForInDestination::Binding)
            && matches!(mode, BindingMode::Let | BindingMode::Const)
        {
            for_in_loop_binding_storage_name(source, &name)
        } else {
            name.clone()
        };
        Some(ForInIterationHead {
            mode,
            name,
            storage_name,
            destination,
        })
    }

    fn prepare_for_in_lexical_head(
        &mut self,
        source: &boa_ast::statement::iteration::ForInLoop,
        binding: &Binding,
        mode: ForInLexicalMode,
    ) -> Option<(BindingMode, String, ForInDestination)> {
        let mode = match mode {
            ForInLexicalMode::Let => BindingMode::Let,
            ForInLexicalMode::Const => BindingMode::Const,
        };
        Some(match binding {
            Binding::Identifier(identifier) => (
                mode,
                self.interner.resolve_expect(identifier.sym()).to_string(),
                ForInDestination::Binding,
            ),
            Binding::Pattern(pattern) => {
                let Some(bound_names) = supported_bound_names(self.interner, binding) else {
                    self.unsupported("for-in initializer");
                    return None;
                };
                let storage_names = bound_names
                    .into_iter()
                    .map(|bound| {
                        let storage = for_in_loop_binding_storage_name(source, &bound.source_name);
                        (bound.source_name, storage)
                    })
                    .collect();
                (
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forin"),
                    ForInDestination::BindingPattern {
                        mode,
                        pattern: pattern.clone(),
                        storage_names,
                    },
                )
            }
        })
    }

    pub(in crate::lowering) fn lower_for_in_head_target<R>(
        &mut self,
        head: &ForInIterationHead,
        source: &Expression,
        lower: impl FnOnce(&mut Self, &Expression) -> R,
    ) -> R {
        let names = match &head.destination {
            ForInDestination::Binding
                if matches!(head.mode, BindingMode::Let | BindingMode::Const) =>
            {
                vec![(head.mode, head.name.clone())]
            }
            ForInDestination::BindingPattern {
                mode,
                storage_names,
                ..
            } if matches!(mode, BindingMode::Let | BindingMode::Const) => storage_names
                .keys()
                .map(|name| (*mode, name.clone()))
                .collect(),
            ForInDestination::Binding
            | ForInDestination::BorrowedVar(_)
            | ForInDestination::Identifier(_)
            | ForInDestination::Property(_)
            | ForInDestination::AssignmentPattern(_)
            | ForInDestination::BindingPattern { .. } => Vec::new(),
        };
        if names.is_empty() {
            return lower(self, source);
        }
        self.push_scope();
        for (mode, source_name) in names {
            self.declare_binding(
                source_name.clone(),
                BindingInfo::tdz_placeholder(
                    mode,
                    TdzPlaceholderName::for_source_name(&source_name),
                ),
            );
        }
        let value = lower(self, source);
        self.pop_scope();
        value
    }

    /// The caller owns the original loop scope. None preserves the original
    /// native key publication; Some uses the completed owner's retained key.
    pub(in crate::lowering) fn lower_for_in_iteration_initialization(
        &mut self,
        head: &ForInIterationHead,
        captured_key: Option<TypedExpr>,
    ) -> Option<Vec<StatementIr>> {
        self.lower_for_in_iteration_initialization_with_evidence(head, captured_key)
            .map(|(statements, _)| statements)
    }

    pub(in crate::lowering) fn lower_for_in_iteration_initialization_with_evidence(
        &mut self,
        head: &ForInIterationHead,
        captured_key: Option<TypedExpr>,
    ) -> Option<(
        Vec<StatementIr>,
        Option<crate::reference::IgnoredIterationIdentifierWriteIr>,
    )> {
        self.lower_for_in_iteration_initialization_with_continuation(head, captured_key, None)
    }

    pub(in crate::lowering) fn lower_for_in_iteration_initialization_with_continuation(
        &mut self,
        head: &ForInIterationHead,
        captured_key: Option<TypedExpr>,
        continuation: Option<PatternContinuation>,
    ) -> Option<(
        Vec<StatementIr>,
        Option<crate::reference::IgnoredIterationIdentifierWriteIr>,
    )> {
        let info = ValueInfo::new(ValueKind::String);
        if matches!(head.destination, ForInDestination::Binding) || captured_key.is_none() {
            if head.mode == BindingMode::Var {
                self.set_binding_value_info(&head.name, info.clone());
            }
            self.declare_binding(
                head.name.clone(),
                BindingInfo {
                    mode: head.mode,
                    storage_name: head.storage_name.clone(),
                    kind: info.kind,
                    possible_kinds: info.possible_kinds,
                    heap_shape: info.heap_shape.clone(),
                    function_targets: info.function_targets.clone(),
                    initialization: Initialization::Initialized,
                },
            );
        }
        let explicit_key = captured_key.is_some();
        let key = captured_key.unwrap_or_else(|| {
            TypedExpr::from_info(info, ExprIr::Identifier(head.storage_name.clone()))
        });
        let mut ignored = None;
        let statements = match &head.destination {
            ForInDestination::Binding if explicit_key => {
                Some(vec![if head.mode == BindingMode::Var {
                    StatementIr::Var(vec![VarDeclaratorIr {
                        name: head.storage_name.clone(),
                        init: Some(key),
                    }])
                } else {
                    StatementIr::Lexical {
                        mode: head.mode,
                        name: head.storage_name.clone(),
                        init: key,
                    }
                }])
            }
            ForInDestination::Binding => Some(Vec::new()),
            ForInDestination::BorrowedVar(name) => Some(vec![StatementIr::DeclarationEvaluation(
                self.environment_identifier(
                    name.clone(),
                    EnvironmentIdentifierOperationIr::Assign {
                        value: Box::new(key),
                    },
                ),
            )]),
            ForInDestination::Identifier(name) => {
                let write = self.lower_bare_iteration_head_write_with_evidence(name.clone(), key);
                ignored = write.ignored;
                Some(vec![StatementIr::DeclarationEvaluation(write.value)])
            }
            ForInDestination::Property(access)
                if continuation.is_some()
                    && (contains(access, ContainsSymbol::YieldExpression)
                        || contains(access, ContainsSymbol::AwaitExpression)) =>
            {
                let execution = continuation?;
                let mut statements = Vec::new();
                let reference = self.retain_pattern_member(execution, access, &mut statements)?;
                self.put_pattern_target(execution, reference, key, &mut statements)?;
                Some(statements)
            }
            ForInDestination::Property(PropertyAccess::Super(access)) => {
                let write = self.lower_super_property_assign_value(access, key)?;
                Some(vec![StatementIr::DeclarationEvaluation(write)])
            }
            ForInDestination::Property(
                access @ (PropertyAccess::Simple(_) | PropertyAccess::Private(_)),
            ) => Some(vec![StatementIr::DeclarationEvaluation(
                self.lower_property_assign_value(access, key),
            )]),
            ForInDestination::AssignmentPattern(pattern) => {
                self.lower_for_in_pattern_initialization(pattern, key, None, None, continuation)
            }
            ForInDestination::BindingPattern {
                mode: BindingMode::Var,
                pattern,
                ..
            } => self.lower_for_in_pattern_initialization(
                pattern,
                key,
                Some(BindingMode::Var),
                None,
                continuation,
            ),
            ForInDestination::BindingPattern {
                mode,
                pattern,
                storage_names,
            } => self.lower_for_in_pattern_initialization(
                pattern,
                key,
                Some(*mode),
                Some(storage_names),
                continuation,
            ),
        }?;
        Some((statements, ignored))
    }

    fn lower_for_in_pattern_initialization(
        &mut self,
        pattern: &Pattern,
        key: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
        continuation: Option<PatternContinuation>,
    ) -> Option<Vec<StatementIr>> {
        let suspended = contains(pattern, ContainsSymbol::YieldExpression)
            || contains(pattern, ContainsSymbol::AwaitExpression);
        if let Some(execution) = continuation.filter(|_| suspended) {
            if let Some(mode @ (BindingMode::Let | BindingMode::Const)) = mode {
                for bound in
                    supported_bound_names(self.interner, &Binding::Pattern(pattern.clone()))?
                {
                    let storage_name = storage_names?.get(&bound.source_name)?.clone();
                    self.declare_binding(
                        bound.source_name,
                        BindingInfo {
                            mode,
                            storage_name,
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
            }
            let (statements, _) = match execution {
                PatternContinuation::Generator => match pattern {
                    Pattern::Array(_) => self
                        .lower_staged_generator_array_pattern_with_storage_names(
                            GeneratorArrayPatternSource::new(pattern)?,
                            key,
                            mode,
                            storage_names,
                        ),
                    Pattern::Object(_) => self
                        .lower_staged_generator_object_pattern_with_storage_names(
                            GeneratorObjectPatternSource::new(pattern)?,
                            key,
                            mode,
                            storage_names,
                        ),
                },
                PatternContinuation::Async => self.lower_staged_async_pattern_with_storage_names(
                    AsyncPatternSource::new(pattern)?,
                    key,
                    mode,
                    storage_names,
                ),
                PatternContinuation::AsyncGenerator => self
                    .lower_staged_async_generator_pattern_with_storage_names(
                        crate::async_generator_source::AsyncGeneratorPatternSource::new(pattern)?,
                        key,
                        mode,
                        storage_names,
                    ),
            }?;
            return Some(statements);
        }
        match mode {
            None => Some(vec![StatementIr::DeclarationEvaluation(
                self.lower_pattern_assign_value(pattern, key)?,
            )]),
            Some(BindingMode::Var) => self.lower_pattern_var_binding_from_value(pattern, key),
            Some(mode @ (BindingMode::Let | BindingMode::Const)) => self
                .lower_pattern_lexical_binding_from_value_with_storage_names(
                    mode,
                    pattern,
                    key,
                    storage_names,
                ),
        }
    }
}
