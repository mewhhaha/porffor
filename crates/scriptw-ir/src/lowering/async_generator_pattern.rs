//! Mixed patterns retain the original References, operations and IteratorClose owner.
use super::pattern_target::{owned_pattern_binding, retain_pattern_value, PatternContinuation};
use super::*;
use crate::async_generator_source::{
    async_generator_pattern_default_states, AsyncGeneratorPatternSource,
};

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_async_generator_pattern_with_storage_names(
        &mut self,
        source: AsyncGeneratorPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let entry = self.async_generator_entry_state()?;
        match source {
            AsyncGeneratorPatternSource::Array(source) => {
                let states = source.states(entry)?;
                let mut prefix = Vec::new();
                let mut result = retain_pattern_value(
                    self,
                    &mut prefix,
                    "async.generator.array.pattern.raw.",
                    value,
                );
                let iterator = owned_pattern_binding(
                    self,
                    "async.generator.array.pattern.iterator.",
                    ValueInfo::undefined(),
                );
                prefix.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: iterator.name.clone(),
                    init: TypedExpr::undefined(),
                });
                let iterator =
                    ArrayIteratorStorageIr::new(iterator, &self.generated_owned_env_bindings)
                        .ok()?;
                self.set_async_generator_phase(states.body().entry());
                let statements = self.lower_resumable_array_pattern_elements(
                    source.pattern(),
                    PatternContinuation::AsyncGenerator,
                    &iterator,
                    mode,
                    storage_names,
                )?;
                if self.async_generator_entry_state()? != states.body().end() {
                    return None;
                }
                let exit = states.exit();
                let body = BlockIr {
                    statements,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                };
                let plan = AsyncGeneratorArrayDestructuringIr::new(
                    states,
                    result.clone(),
                    iterator,
                    body,
                    &self.generated_owned_env_bindings,
                )
                .ok()?;
                prefix.push(StatementIr::AsyncGeneratorArrayDestructuring(Box::new(
                    plan,
                )));
                self.set_async_generator_phase(exit);
                result.heap_shape = None;
                Some((prefix, result))
            }
            AsyncGeneratorPatternSource::Object(source) => {
                let end = source.end_state(entry)?;
                let result = self.lower_resumable_object_pattern_elements(
                    source.pattern(),
                    PatternContinuation::AsyncGenerator,
                    value,
                    mode,
                    storage_names,
                )?;
                if self.async_generator_entry_state()? != end {
                    return None;
                }
                Some(result)
            }
        }
    }

    pub(super) fn lower_async_generator_pattern_initializer(
        &mut self,
        pattern: &Pattern,
        initializer: &Expression,
        mode: BindingMode,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<Vec<StatementIr>> {
        self.async_generator_entry_state()?;
        if matches!(mode, BindingMode::Let | BindingMode::Const) {
            for bound in supported_bound_names(self.interner, &Binding::Pattern(pattern.clone()))? {
                if !self
                    .scopes
                    .last()
                    .is_some_and(|scope| scope.contains_key(&bound.source_name))
                {
                    self.declare_binding(
                        bound.source_name.clone(),
                        BindingInfo::tdz_placeholder(
                            mode,
                            TdzPlaceholderName::for_source_name(&bound.source_name),
                        ),
                    );
                }
            }
        }
        let source = AsyncGeneratorPatternSource::new(pattern)?;
        let (mut prefix, value) = self.lower_mixed_generator_value(initializer)?;
        let (body, _) = self.lower_staged_async_generator_pattern_with_storage_names(
            source,
            value,
            Some(mode),
            storage_names,
        )?;
        prefix.extend(body);
        Some(prefix)
    }

    pub(super) fn lower_async_generator_pattern_assignment(
        &mut self,
        pattern: &Pattern,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let (mut prefix, value) = self.lower_mixed_generator_value(rhs)?;
        if contains(pattern, ContainsSymbol::AwaitExpression)
            || contains(pattern, ContainsSymbol::YieldExpression)
        {
            let source = AsyncGeneratorPatternSource::new(pattern)?;
            let (statements, value) = self
                .lower_staged_async_generator_pattern_with_storage_names(
                    source, value, None, None,
                )?;
            prefix.extend(statements);
            Some((prefix, value))
        } else {
            // Eager patterns retain their original semantic owner after the
            // whole mixed RHS, including the exact whole assignment result.
            let mut value = self.lower_pattern_assign_value(pattern, value)?;
            value.heap_shape = None;
            Some((prefix, value))
        }
    }

    pub(super) fn apply_async_generator_pattern_default(
        &mut self,
        value: &TypedExpr,
        source: &Expression,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        let ExprIr::Identifier(name) = &value.expr else {
            return None;
        };
        let condition = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::StrictEquality {
                op: EqualityBinaryOp::StrictEqual,
                lhs: Box::new(value.clone()),
                rhs: Box::new(TypedExpr::undefined()),
            },
        );
        let before = self.capture_conditional_flow_facts();
        if !contains(source, ContainsSymbol::AwaitExpression)
            && !contains(source, ContainsSymbol::YieldExpression)
        {
            let value = self.lower_expression(source);
            let then_branch =
                StatementIr::Expression(self.lower_identifier_assign_value(name.clone(), value));
            let then_facts = self.capture_conditional_flow_facts();
            self.install_conditional_flow_facts(before.clone());
            self.merge_conditional_flow_facts(then_facts, before);
            statements.push(StatementIr::If {
                condition,
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(StatementIr::Empty)),
            });
            return Some(());
        }
        let states =
            async_generator_pattern_default_states(source, self.async_generator_entry_state()?)?;
        self.set_async_generator_phase(states.condition().entry());
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
            condition,
        );
        self.set_async_generator_phase(states.then_branch().entry());
        self.push_scope();
        let result = self.lower_mixed_generator_value(source);
        self.pop_scope();
        let (mut then_statements, then_value) = result?;
        then_statements.push(StatementIr::Expression(
            self.lower_identifier_assign_value(name.clone(), then_value),
        ));
        let then_branch = self
            .finish_async_generator_region(
                BlockIr {
                    statements: then_statements,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.then_branch(),
            )
            .ok()?;
        let then_facts = self.capture_conditional_flow_facts();
        self.install_conditional_flow_facts(before);
        self.set_async_generator_phase(states.else_branch().entry());
        let else_branch = self
            .finish_async_generator_region(
                BlockIr {
                    statements: Vec::new(),
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.else_branch(),
            )
            .ok()?;
        let else_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(then_facts, else_facts);
        let plan = AsyncGeneratorIfIr::new(states, condition, then_branch, else_branch).ok()?;
        self.set_async_generator_phase(plan.exit_state());
        statements.push(StatementIr::AsyncGeneratorIf(Box::new(plan)));
        Some(())
    }
}
