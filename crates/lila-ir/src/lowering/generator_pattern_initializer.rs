use super::*;

impl ScriptLowerer<'_> {
    /// Lexical TDZ ownership precedes every initializer suspension; the shared
    /// pattern consumer initializes the same source bindings after Normal.
    pub(super) fn lower_generator_lexical_pattern_initializer(
        &mut self,
        mode: BindingMode,
        source: GeneratorPatternInitializerSource<'_>,
    ) -> Option<Vec<StatementIr>> {
        let (pattern, initializer) = source.into_parts();
        let binding = Binding::Pattern(pattern.clone());
        let bound_names = supported_bound_names(self.interner, &binding)?;
        for bound in bound_names {
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
        let (mut statements, value) = self.lower_staged_generator_expression(initializer)?;
        if contains(pattern, ContainsSymbol::YieldExpression) {
            let (prefix, _) = match pattern {
                Pattern::Object(_) => self.lower_staged_generator_object_pattern(
                    GeneratorObjectPatternSource::new(pattern)?,
                    value,
                    Some(mode),
                )?,
                Pattern::Array(_) => self.lower_staged_generator_array_pattern(
                    GeneratorArrayPatternSource::new(pattern)?,
                    value,
                    Some(mode),
                )?,
            };
            statements.extend(prefix);
            return Some(statements);
        }
        statements.extend(self.lower_pattern_lexical_binding_from_value(mode, pattern, value)?);
        Some(statements)
    }

    /// Var names already belong to the enclosing variable-instantiation owner.
    pub(super) fn lower_generator_var_pattern_initializer(
        &mut self,
        source: GeneratorPatternInitializerSource<'_>,
    ) -> Option<Vec<StatementIr>> {
        let (pattern, initializer) = source.into_parts();
        let (mut statements, value) = self.lower_staged_generator_expression(initializer)?;
        if contains(pattern, ContainsSymbol::YieldExpression) {
            let (prefix, _) = match pattern {
                Pattern::Object(_) => self.lower_staged_generator_object_pattern(
                    GeneratorObjectPatternSource::new(pattern)?,
                    value,
                    Some(BindingMode::Var),
                )?,
                Pattern::Array(_) => self.lower_staged_generator_array_pattern(
                    GeneratorArrayPatternSource::new(pattern)?,
                    value,
                    Some(BindingMode::Var),
                )?,
            };
            statements.extend(prefix);
            return Some(statements);
        }
        statements.extend(self.lower_pattern_var_binding_from_value(pattern, value)?);
        Some(statements)
    }
}
