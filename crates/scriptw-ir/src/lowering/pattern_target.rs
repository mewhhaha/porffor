//! Original pattern target selection and Put semantics shared by checked continuations.

use super::generator_identifier_reference::RetainedGeneratorIdentifierTarget;
use super::*;
use crate::ir::reference::CapturedSuperPropertyReference;

#[derive(Clone, Copy)]
pub(super) enum PatternContinuation {
    Generator,
    Async,
    AsyncGenerator,
}

pub(super) enum RetainedPatternTarget<'ast> {
    Identifier(RetainedGeneratorIdentifierTarget),
    Binding {
        mode: BindingMode,
        source_name: String,
        storage_name: String,
    },
    Member(DestructuringTargetIr),
    Super(CapturedSuperPropertyReference),
    Nested {
        pattern: &'ast Pattern,
        mode: Option<BindingMode>,
        storage_names: Option<BTreeMap<String, String>>,
    },
}

pub(super) fn owned_pattern_binding(
    lowerer: &mut ScriptLowerer<'_>,
    hint: &str,
    mut info: ValueInfo,
) -> OwnedEnvBindingIr {
    info.heap_shape = None;
    let name = lowerer.alloc_suspension_owned_binding(hint, info);
    lowerer
        .generated_owned_env_bindings
        .iter()
        .find(|binding| binding.name == name)
        .expect("an allocated suspension cell has its exact owned inventory row")
        .clone()
}

pub(super) fn retain_pattern_value(
    lowerer: &mut ScriptLowerer<'_>,
    prefix: &mut Vec<StatementIr>,
    hint: &str,
    value: TypedExpr,
) -> TypedExpr {
    let mut info = value.value_info();
    info.heap_shape = None;
    let binding = owned_pattern_binding(lowerer, hint, info.clone());
    prefix.push(StatementIr::Lexical {
        mode: BindingMode::Let,
        name: binding.name.clone(),
        init: value,
    });
    TypedExpr::from_info(info, ExprIr::Identifier(binding.name))
}

impl ScriptLowerer<'_> {
    pub(super) fn retain_pattern_identifier(
        &mut self,
        ident: boa_ast::expression::Identifier,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
        statements: &mut Vec<StatementIr>,
    ) -> RetainedPatternTarget<'static> {
        let name = self.interner.resolve_expect(ident.sym()).to_string();
        match mode {
            None | Some(BindingMode::Var) => RetainedPatternTarget::Identifier(
                RetainedGeneratorIdentifierTarget::capture_write_only(self, statements, name),
            ),
            Some(mode @ (BindingMode::Let | BindingMode::Const)) => {
                let storage_name = storage_names
                    .and_then(|names| names.get(&name))
                    .cloned()
                    .unwrap_or_else(|| self.direct_lexical_storage_name(&name, ident.span()));
                RetainedPatternTarget::Binding {
                    mode,
                    source_name: name,
                    storage_name,
                }
            }
        }
    }
    pub(super) fn retain_pattern_member(
        &mut self,
        execution: PatternContinuation,
        access: &PropertyAccess,
        statements: &mut Vec<StatementIr>,
    ) -> Option<RetainedPatternTarget<'static>> {
        let target = match access {
            PropertyAccess::Simple(access) => {
                let (prefix, base) = self.lower_pattern_expression(execution, access.target())?;
                statements.extend(prefix);
                let target =
                    retain_pattern_value(self, statements, "generator.pattern.target.", base);
                let key = match access.field() {
                    PropertyAccessField::Const(name) => DestructuringPropertyKeyIr::Static(
                        self.interner.resolve_expect(name.sym()).to_string(),
                    ),
                    PropertyAccessField::Expr(source) => {
                        let (prefix, raw) = self.lower_pattern_expression(execution, source)?;
                        statements.extend(prefix);
                        // This is an actual Reference key, not PropertyName.
                        // PutValue alone will coerce it after Get/default.
                        let raw = retain_pattern_value(
                            self,
                            statements,
                            "generator.pattern.target.key.",
                            raw,
                        );
                        DestructuringPropertyKeyIr::Computed(raw)
                    }
                };
                DestructuringTargetIr::AssignmentProperty {
                    target,
                    key,
                    strictness: self.reference_strictness(),
                }
            }
            PropertyAccess::Private(access) => {
                let private_name_id = self.current_private_name_id(access.field())?;
                let (prefix, base) = self.lower_pattern_expression(execution, access.target())?;
                statements.extend(prefix);
                DestructuringTargetIr::AssignmentPrivate {
                    target: retain_pattern_value(
                        self,
                        statements,
                        "generator.pattern.target.",
                        base,
                    ),
                    private_name_id,
                }
            }
            PropertyAccess::Super(access) => {
                let (reference, capture) = self.capture_resumable_super_reference(
                    access,
                    SuperPropertyCaptureMode::WriteOnly,
                    statements,
                )?;
                // Evaluating the target owns the original receiver/base/raw
                // name before IteratorValue, GetV or a lazy default runs.
                statements.push(StatementIr::Expression(capture));
                return Some(RetainedPatternTarget::Super(reference));
            }
        };
        Some(RetainedPatternTarget::Member(target))
    }
    pub(super) fn put_pattern_target(
        &mut self,
        execution: PatternContinuation,
        target: RetainedPatternTarget<'_>,
        value: TypedExpr,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        let declaration = matches!(
            &target,
            RetainedPatternTarget::Binding { .. }
                | RetainedPatternTarget::Nested {
                    mode: Some(BindingMode::Let | BindingMode::Const),
                    ..
                }
        );
        let target = match target {
            RetainedPatternTarget::Identifier(target) => {
                statements.push(StatementIr::Expression(target.put_value(self, value)));
                return Some(());
            }
            RetainedPatternTarget::Super(reference) => {
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                self.record_caller_flow_invalidation();
                let mut put = reference.write(value);
                put.heap_shape = None;
                statements.push(StatementIr::Expression(put));
                return Some(());
            }
            RetainedPatternTarget::Binding {
                mode,
                source_name,
                storage_name,
            } => {
                let target = DestructuringTargetIr::Binding {
                    mode,
                    name: storage_name.clone(),
                };
                self.record_destructuring_binding(
                    source_name,
                    BindingInfo {
                        mode,
                        storage_name,
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::unknown(),
                        initialization: Initialization::Initialized,
                    },
                    &target,
                );
                target
            }
            RetainedPatternTarget::Member(target) => target,
            RetainedPatternTarget::Nested {
                pattern,
                mode,
                storage_names,
            } => match execution {
                PatternContinuation::Generator => match pattern {
                    Pattern::Object(_) => {
                        let (prefix, _) = self
                            .lower_staged_generator_object_pattern_with_storage_names(
                                GeneratorObjectPatternSource::new(pattern)?,
                                value,
                                mode,
                                storage_names.as_ref(),
                            )?;
                        statements.extend(prefix);
                        return Some(());
                    }
                    Pattern::Array(_) if contains(pattern, ContainsSymbol::YieldExpression) => {
                        let (prefix, _) = self
                            .lower_staged_generator_array_pattern_with_storage_names(
                                GeneratorArrayPatternSource::new(pattern)?,
                                value,
                                mode,
                                storage_names.as_ref(),
                            )?;
                        statements.extend(prefix);
                        return Some(());
                    }
                    Pattern::Array(pattern) => {
                        self.invalidate_unknown_user_code_effects();
                        let pattern = match mode {
                            None => {
                                let outer =
                                    std::mem::take(&mut self.pending_super_destructuring_slots);
                                let lowered =
                                    self.lower_array_assignment_pattern(pattern.bindings());
                                let slots = std::mem::replace(
                                    &mut self.pending_super_destructuring_slots,
                                    outer,
                                );
                                statements.extend(slots.into_iter().map(|name| {
                                    StatementIr::Lexical {
                                        mode: BindingMode::Let,
                                        name,
                                        init: TypedExpr::undefined(),
                                    }
                                }));
                                lowered?
                            }
                            Some(mode) => self.lower_array_binding_pattern(
                                mode,
                                pattern.bindings(),
                                storage_names.as_ref(),
                            )?,
                        };
                        DestructuringTargetIr::NestedArray(Box::new(pattern))
                    }
                },
                PatternContinuation::Async => {
                    let (prefix, _) = self.lower_staged_async_pattern_with_storage_names(
                        AsyncPatternSource::new(pattern)?,
                        value,
                        mode,
                        storage_names.as_ref(),
                    )?;
                    statements.extend(prefix);
                    return Some(());
                }
                PatternContinuation::AsyncGenerator => {
                    let (prefix, _) = self
                        .lower_staged_async_generator_pattern_with_storage_names(
                            crate::async_generator_source::AsyncGeneratorPatternSource::new(
                                pattern,
                            )?,
                            value,
                            mode,
                            storage_names.as_ref(),
                        )?;
                    statements.extend(prefix);
                    return Some(());
                }
            },
        };
        self.invalidate_unknown_user_code_effects();
        let put = ObjectDestructuringOperationIr::put_target(
            target,
            value,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        let value = put.into_expr();
        statements.push(if declaration {
            StatementIr::DeclarationEvaluation(value)
        } else {
            StatementIr::Expression(value)
        });
        Some(())
    }

    pub(super) fn lower_pattern_expression(
        &mut self,
        execution: PatternContinuation,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        match execution {
            PatternContinuation::Generator => self.lower_staged_generator_expression(source),
            PatternContinuation::Async => self.lower_async_pattern_expression(source),
            PatternContinuation::AsyncGenerator => self.lower_mixed_generator_value(source),
        }
    }
}
