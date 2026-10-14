use super::*;

impl ScriptLowerer<'_> {
    /// HasName examines the source grammar, not a parser-inferred display label.
    /// Arrows cannot declare a name; the other callable forms can bind one.
    pub(super) fn is_anonymous_field_function_definition(source: &Expression) -> bool {
        match Self::unwrap_parenthesized_expr(source) {
            Expression::ArrowFunction(_) | Expression::AsyncArrowFunction(_) => true,
            Expression::FunctionExpression(function) => !function.has_binding_identifier(),
            Expression::GeneratorExpression(function) => !function.has_binding_identifier(),
            Expression::AsyncFunctionExpression(function) => !function.has_binding_identifier(),
            Expression::AsyncGeneratorExpression(function) => !function.has_binding_identifier(),
            // Class NamedEvaluation has its own creation-time name protocol.
            // A sequence or reference expression is not a function definition.
            _ => false,
        }
    }
}
