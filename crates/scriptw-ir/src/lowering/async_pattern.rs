//! Checked plain async patterns retain original targets across their Await prefixes.

use super::awaited_while_condition::AsyncValueBranchContext;
use super::pattern_target::PatternContinuation;
use super::*;

#[path = "async_pattern/array.rs"]
mod array;
#[path = "async_pattern/object.rs"]
mod object;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_async_pattern(
        &mut self,
        source: AsyncPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_staged_async_pattern_with_storage_names(source, value, mode, None)
    }

    pub(super) fn lower_staged_async_pattern_with_storage_names(
        &mut self,
        source: AsyncPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.plain_async_entry_state()?;
        let enclosing = std::mem::replace(
            &mut self.async_value_branch_context,
            AsyncValueBranchContext::Pattern {
                loop_depth: self.loop_depth,
            },
        );
        let result = match source {
            AsyncPatternSource::Array(source) => self
                .lower_staged_async_array_pattern_with_storage_names(
                    source,
                    value,
                    mode,
                    storage_names,
                ),
            AsyncPatternSource::Object(source) => self
                .lower_staged_async_object_pattern_with_storage_names(
                    source,
                    value,
                    mode,
                    storage_names,
                ),
        };
        self.async_value_branch_context = enclosing;
        result
    }

    pub(super) fn lower_async_pattern_expression(
        &mut self,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.plain_async_entry_state()?;
        if contains(source, ContainsSymbol::YieldExpression) {
            return None;
        }
        if !contains(source, ContainsSymbol::AwaitExpression) {
            return Some((Vec::new(), self.lower_expression(source)));
        }
        let enclosing = std::mem::replace(
            &mut self.async_value_branch_context,
            AsyncValueBranchContext::Pattern {
                loop_depth: self.loop_depth,
            },
        );
        let result = self.lower_async_prefixed_expression(source);
        self.async_value_branch_context = enclosing;
        result
    }

    pub(super) fn lower_async_pattern_lexical_initializer(
        &mut self,
        mode: BindingMode,
        pattern: &Pattern,
        initializer: &Expression,
    ) -> Option<Vec<StatementIr>> {
        let source = AsyncPatternSource::new(pattern)?;
        let names = supported_bound_names(self.interner, &Binding::Pattern(pattern.clone()))?;
        for bound in names {
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
        let (mut prefix, value) = self.lower_async_pattern_expression(initializer)?;
        let (pattern_prefix, _) = self.lower_staged_async_pattern(source, value, Some(mode))?;
        prefix.extend(pattern_prefix);
        Some(prefix)
    }

    pub(super) fn lower_async_pattern_var_initializer(
        &mut self,
        pattern: &Pattern,
        initializer: &Expression,
    ) -> Option<Vec<StatementIr>> {
        let source = AsyncPatternSource::new(pattern)?;
        let (mut prefix, value) = self.lower_async_pattern_expression(initializer)?;
        let (pattern_prefix, _) =
            self.lower_staged_async_pattern(source, value, Some(BindingMode::Var))?;
        prefix.extend(pattern_prefix);
        Some(prefix)
    }

    pub(super) fn lower_staged_async_pattern_assignment(
        &mut self,
        pattern: &Pattern,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let source = AsyncPatternSource::new(pattern)?;
        let (mut prefix, value) = self.lower_async_pattern_expression(rhs)?;
        let (pattern_prefix, value) = self.lower_staged_async_pattern(source, value, None)?;
        prefix.extend(pattern_prefix);
        Some((prefix, value))
    }

    pub(super) fn apply_async_pattern_default(
        &mut self,
        value: &TypedExpr,
        default: &Expression,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        let ExprIr::Identifier(name) = &value.expr else {
            return None;
        };
        let states = AsyncPatternDefaultStates::new(default, self.plain_async_entry_state()?)?;
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.current_async_resume_state = Some(states.then_entry());
        self.push_scope();
        let then_result = self.lower_async_pattern_expression(default);
        self.pop_scope();
        let (mut then_prefix, then_value) = then_result?;
        if self.plain_async_entry_state()? != states.then_end() {
            return None;
        }
        then_prefix.push(StatementIr::Expression(
            self.lower_identifier_assign_value(name.clone(), then_value),
        ));
        let then_vars = self.var_bindings.clone();
        let then_globals = self.global_properties.clone();
        self.var_bindings = before_vars;
        self.global_properties = before_globals;
        self.var_bindings = self.merge_var_bindings(&then_vars, &self.var_bindings);
        self.global_properties =
            self.merge_global_properties(&then_globals, &self.global_properties);
        let condition = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::StrictEquality {
                op: EqualityBinaryOp::StrictEqual,
                lhs: Box::new(value.clone()),
                rhs: Box::new(TypedExpr::undefined()),
            },
        );
        let then_branch = Box::new(StatementIr::LexicalBlock(then_prefix));
        let else_branch = Some(Box::new(StatementIr::Empty));
        let statement = if states.resumable() {
            let plan =
                AsyncFunctionIfPlanIr::new(states.entry(), states.then_end(), states.else_entry())
                    .ok()??;
            if plan.exit_state() != states.exit() {
                return None;
            }
            StatementIr::AsyncFunctionIf {
                condition,
                then_branch,
                else_branch,
                plan,
            }
        } else {
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            }
        };
        statements.push(statement);
        self.current_async_resume_state = Some(states.exit());
        Some(())
    }
}
