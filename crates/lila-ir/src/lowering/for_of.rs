mod async_function;
mod generator;
mod protocol;

use self::generator::GeneratorForOfHeadSource;
use self::protocol::ForOfLoweringIr;
use super::async_disposable::LoweredForOfHeadKind;
use super::*;
use crate::resumable_for_of_control::resumable_sync_for_of_body_has_local_control_owners;

enum ForOfBareIdentifierHead {
    Absent,
    AssignmentTarget { source_name: String },
    BorrowedVar { source_name: String },
}

struct LexicalForOfPatternBinding {
    source_name: String,
    iteration_storage_name: String,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_for_of_loop(&mut self, for_of: &ForOfLoop) -> (StatementIr, ValueKind) {
        if self
            .analysis
            .complete_for_of_owners
            .contains_key(&(for_of as *const ForOfLoop as usize))
        {
            return self.lower_for_of_loop_region(for_of);
        }
        if matches!(for_of.initializer(), IterableLoopInitializer::Using(_)) {
            let Some(region) =
                super::synchronous_resource_loop::SynchronousResourceLoop::iterator(for_of)
            else {
                self.unsupported("suspension inside a synchronous resource loop");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            return region.lower(self);
        }
        self.lower_for_of_loop_region(for_of)
    }

    pub(super) fn lower_for_of_loop_region(
        &mut self,
        for_of: &ForOfLoop,
    ) -> (StatementIr, ValueKind) {
        if let Some(owner) = self
            .analysis
            .complete_for_of_owners
            .get(&(for_of as *const ForOfLoop as usize))
            .copied()
        {
            return self.lower_async_generator_for_of(for_of, owner);
        }
        self.lower_for_of_head(for_of).into_statement_and_kind()
    }

    fn lower_for_of_head(&mut self, for_of: &ForOfLoop) -> ForOfLoweringIr {
        let uses_unified_resumable_plan = for_of.r#await() && self.current_resumable_plan.is_some();
        if for_of.r#await()
            && !uses_unified_resumable_plan
            && self.current_async_resume_state.is_none()
        {
            self.unsupported("for-await-of outside async function");
            return ForOfLoweringIr::no_iteration();
        }
        let plain_async_for_await_body = self.plain_async_entry_state().is_some()
            && for_of.r#await()
            && contains(for_of.body(), ContainsSymbol::AwaitExpression);
        if for_of.r#await()
            && contains(for_of.body(), ContainsSymbol::AwaitExpression)
            && !plain_async_for_await_body
        {
            self.unsupported("explicit await in for-await-of body");
            return ForOfLoweringIr::no_iteration();
        }
        // Only a plain Generator owns the new Yield continuation region.
        let generator_entry_state = self.plain_generator_entry_state().filter(|_| {
            !for_of.r#await() && contains(for_of.body(), ContainsSymbol::YieldExpression)
        });
        if generator_entry_state.is_some() && !generator_for_of_source_shape_is_supported(for_of) {
            self.unsupported("generator for-of requires an eager identifier binding, lexical-pattern binding, identifier assignment, or ordinary-property assignment head and a body without nested resumable loops or foreign branch owners");
            return ForOfLoweringIr::no_iteration();
        }
        if plain_async_for_await_body {
            let eager_binding = match for_of.initializer() {
                IterableLoopInitializer::Var(variable) => {
                    matches!(variable.binding(), Binding::Identifier(_))
                        && !self.borrows_direct_eval_variable_environment()
                }
                IterableLoopInitializer::Let(Binding::Identifier(_))
                | IterableLoopInitializer::Const(Binding::Identifier(_)) => true,
                _ => false,
            };
            if !eager_binding
                || contains(for_of.initializer(), ContainsSymbol::AwaitExpression)
                || contains(for_of.iterable(), ContainsSymbol::AwaitExpression)
                || !resumable_sync_for_of_body_has_local_control_owners(for_of.body())
            {
                self.unsupported("plain async for-await-of with a body await requires an eager identifier binding and iterable, and local body control owners");
                return ForOfLoweringIr::no_iteration();
            }
        }
        // A body await must retain the Iterator Record across driver returns.
        let plain_async_await_body = self.plain_async_entry_state().is_some()
            && !for_of.r#await()
            && contains(for_of.body(), ContainsSymbol::AwaitExpression);
        if plain_async_await_body
            && (!resumable_sync_for_of_body_has_local_control_owners(for_of.body())
                || contains(for_of.iterable(), ContainsSymbol::AwaitExpression))
        {
            self.unsupported(
                "async for-of with await requires an eager iterable and a body without foreign branch owners",
            );
            return ForOfLoweringIr::no_iteration();
        }
        if plain_async_await_body && contains(for_of.initializer(), ContainsSymbol::AwaitExpression)
        {
            self.unsupported("async for-of with a body await requires an eager assignment target");
            return ForOfLoweringIr::no_iteration();
        }
        if let IterableLoopInitializer::WebCompatCall(call) = for_of.initializer() {
            // The invalid Reference throws after iterable evaluation, before GetIterator.
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
                Binding::Identifier(identifier)
                    if self.borrows_direct_eval_variable_environment() =>
                {
                    bare_identifier_head = ForOfBareIdentifierHead::BorrowedVar {
                        source_name: self.interner.resolve_expect(identifier.sym()).to_string(),
                    };
                    (
                        LoweredForOfHeadKind::Assignment,
                        BindingMode::Let,
                        self.alloc_temp_binding_name("forof.var"),
                    )
                }
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
                    self.alloc_temp_binding_name(if generator_entry_state.is_some() {
                        "forof.assignment"
                    } else {
                        "forof"
                    }),
                )
            }
            IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                pattern_initializer = Some((BindingMode::Const, pattern.clone()));
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name(if generator_entry_state.is_some() {
                        "forof.assignment"
                    } else {
                        "forof"
                    }),
                )
            }
            IterableLoopInitializer::Using(Binding::Identifier(identifier)) => {
                if for_of.r#await() {
                    self.unsupported("using declaration in for-await-of");
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
            // Re-evaluate the property Reference on each iteration, after
            // the iterator value has been stored in a private temporary.
            IterableLoopInitializer::Access(
                access @ (PropertyAccess::Simple(_)
                | PropertyAccess::Private(_)
                | PropertyAccess::Super(_)),
            ) => {
                access_initializer = Some(access.clone());
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name(if generator_entry_state.is_some() {
                        "forof.assignment"
                    } else {
                        "forof.access"
                    }),
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
            | (None, None, ForOfBareIdentifierHead::BorrowedVar { .. })
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
        // Allocate next()'s suspension after the iterable's awaits and before
        // any body suspension, matching their evaluation order.
        let async_entry_state = (for_of.r#await() && !uses_unified_resumable_plan)
            .then_some(self.current_async_resume_state)
            .flatten();
        if for_of.r#await() {
            // GetIterator tries @@asyncIterator before wrapping @@iterator.
            for key in [WellKnownSymbol::AsyncIterator, WellKnownSymbol::Iterator] {
                let function_targets = self
                    .optional_chain_well_known_symbol_property_info(&iterable.value_info(), key)
                    .function_targets
                    .known_targets()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
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
        // Iterator methods can enter user code before the head and body run.
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
                    | ForOfBareIdentifierHead::BorrowedVar { .. }
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
        let mut pattern_prefix = if let ForOfBareIdentifierHead::BorrowedVar { source_name } =
            &bare_identifier_head
        {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            vec![StatementIr::DeclarationEvaluation(
                self.environment_identifier(
                    source_name.clone(),
                    EnvironmentIdentifierOperationIr::Assign {
                        value: Box::new(value),
                    },
                ),
            )]
        } else if let ForOfBareIdentifierHead::AssignmentTarget { source_name } =
            &bare_identifier_head
        {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let assignment = self.lower_bare_iteration_head_write(source_name.clone(), value);
            vec![StatementIr::DeclarationEvaluation(assignment)]
        } else if let Some(access) = access_initializer.as_ref() {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let assignment = match (generator_entry_state, access) {
                (Some(_), PropertyAccess::Simple(access)) => {
                    let (plan, referenced_name, metadata) =
                        self.lower_ordinary_property_reference_plan(access);
                    self.complete_ordinary_property_plain_assignment(
                        access,
                        plan,
                        referenced_name,
                        metadata,
                        value,
                        false,
                    )
                }
                _ => self.lower_property_assign_value(access, value),
            };
            vec![StatementIr::DeclarationEvaluation(assignment)]
        } else if let Some(pattern) = assignment_pattern_initializer.as_ref() {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let Some(assign) = self.lower_pattern_assign_value(pattern, value) else {
                self.pop_scope();
                return ForOfLoweringIr::no_iteration();
            };
            vec![StatementIr::DeclarationEvaluation(assign)]
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
                let lowered = self.lower_pattern_lexical_binding_from_value_with_storage_names(
                    *pattern_mode,
                    pattern,
                    init,
                    Some(&storage_names),
                );
                let Some(prefix) = lowered else {
                    self.pop_scope();
                    return ForOfLoweringIr::no_iteration();
                };
                prefix
            }
        } else {
            Vec::new()
        };
        // Validate the actual lexical head before lowering any body states.
        let generator_pattern = if generator_entry_state.is_some() {
            if let (
                Some((pattern_mode @ (BindingMode::Let | BindingMode::Const), _)),
                Some(bindings),
            ) = (
                pattern_initializer.as_ref(),
                lexical_pattern_bindings.as_ref(),
            ) {
                match ValidatedResumableSyncForOfLexicalPatternIr::new(
                    *pattern_mode,
                    storage_name.clone(),
                    bindings
                        .iter()
                        .map(|binding| binding.iteration_storage_name.clone())
                        .collect(),
                    bindings
                        .iter()
                        .map(|binding| {
                            TdzPlaceholderName::for_source_name(&binding.source_name).into_string()
                        })
                        .collect(),
                    pattern_prefix.clone(),
                    lexical_environment.clone(),
                ) {
                    Ok(pattern) => Some(pattern),
                    Err(error) => {
                        self.pop_scope();
                        self.unsupported(&format!(
                            "invalid generator lexical-pattern head: {error:?}"
                        ));
                        return ForOfLoweringIr::no_iteration();
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        if plain_async_for_await_body {
            let Some(value_resume_state) = async_entry_state.and_then(|state| state.checked_add(1))
            else {
                self.pop_scope();
                self.unsupported("plain async for-await-of value resume state overflows");
                return ForOfLoweringIr::no_iteration();
            };
            self.current_async_resume_state = Some(value_resume_state);
        }
        let plain_async_entry_state = self.plain_async_entry_state();
        // This body's existing iterator continuation is a separate owner.
        // An enclosing complete region must not allocate eager child phases
        // that its source walk intentionally does not count inside ForOf.
        let enclosing_complete_depths = (
            self.ordinary_generator_region_depth,
            self.ordinary_generator_switch_depth,
            self.ordinary_generator_for_in_depth,
        );
        self.ordinary_generator_region_depth = 0;
        self.ordinary_generator_switch_depth = 0;
        self.ordinary_generator_for_in_depth = 0;
        let enclosing_mixed_domain = self.async_generator_source_domain;
        self.async_generator_source_domain = AsyncGeneratorSourceDomain::ForeignIteratorBody;
        let (mut body, body_kind) = self.lower_loop_body(for_of.body());
        self.async_generator_source_domain = enclosing_mixed_domain;
        (
            self.ordinary_generator_region_depth,
            self.ordinary_generator_switch_depth,
            self.ordinary_generator_for_in_depth,
        ) = enclosing_complete_depths;
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
        if let Some(entry_state) = generator_entry_state {
            let source = if let Some(pattern) = generator_pattern {
                GeneratorForOfHeadSource::LexicalPattern(pattern)
            } else {
                match &bare_identifier_head {
                    ForOfBareIdentifierHead::AssignmentTarget { source_name } => {
                        GeneratorForOfHeadSource::IdentifierAssignment {
                            source_name,
                            value_name: storage_name,
                            head_environment: lexical_environment,
                        }
                    }
                    ForOfBareIdentifierHead::Absent if access_initializer.is_some() => {
                        GeneratorForOfHeadSource::OrdinaryProperty {
                            value_name: storage_name,
                            head_environment: lexical_environment,
                        }
                    }
                    ForOfBareIdentifierHead::Absent
                    | ForOfBareIdentifierHead::BorrowedVar { .. } => {
                        GeneratorForOfHeadSource::Binding {
                            source_name: &name,
                            binding: ForOfAssignmentIr {
                                mode,
                                name: storage_name,
                            },
                            head_environment: lexical_environment,
                        }
                    }
                }
            };
            return self.lower_generator_for_of_iterator(
                source,
                iterable,
                body,
                body_kind,
                entry_state,
            );
        }

        if plain_async_for_await_body {
            let head = AsyncFunctionForOfIteratorHeadIr::Binding {
                source_name: name,
                binding: ForOfAssignmentIr {
                    mode,
                    name: storage_name,
                },
            };
            return self.lower_plain_async_for_await_with_body_await(
                head,
                iterable,
                body,
                body_kind,
                async_entry_state.expect("a plain async for-await head owns next entry"),
                lexical_environment,
            );
        }
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
                AsyncFunctionForOfIteratorHeadIr::Binding {
                    source_name: name.clone(),
                    binding: ForOfAssignmentIr {
                        mode,
                        name: storage_name,
                    },
                }
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
                        // Allocate these slots in protocol order so their generated
                        // names remain consistent with the resume plan.
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
