use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_generator_pattern_assignment(
        &mut self,
        source: GeneratorPatternAssignmentSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let (pattern, rhs) = source.into_parts();
        // Destructuring assignment acquires its complete RHS before evaluating
        // any target Reference, computed key, Get, default or rest operation.
        let (mut prefix, value) = self.lower_staged_generator_expression(rhs)?;
        if contains(pattern, ContainsSymbol::YieldExpression) {
            let (pattern_prefix, value) = match pattern {
                Pattern::Object(_) => self.lower_staged_generator_object_pattern(
                    GeneratorObjectPatternSource::new(pattern)?,
                    value,
                    None,
                )?,
                Pattern::Array(_) => self.lower_staged_generator_array_pattern(
                    GeneratorArrayPatternSource::new(pattern)?,
                    value,
                    None,
                )?,
            };
            prefix.extend(pattern_prefix);
            return Some((prefix, value));
        }
        let assigned = self.lower_pattern_assign_value(pattern, value)?;
        let mut info = assigned.value_info();
        // Pattern target/key/default code may mutate the returned whole value.
        // Its kind and callable identity survive; its mutable shape does not.
        info.heap_shape = None;
        Some((prefix, TypedExpr::from_info(info, assigned.expr)))
    }
}
