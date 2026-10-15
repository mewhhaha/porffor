//! Finite string and array views supply hints; their live operations still run.

use super::finite_function_source::bounded_candidates;
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn finite_template_source_candidates(
        &self,
        template: &TemplateLiteral,
    ) -> Vec<FiniteSourceValue> {
        if template
            .elements()
            .iter()
            .all(|element| matches!(element, TemplateElement::String(_)))
        {
            // Literal-only templates already have the original syntax proof.
            return Vec::new();
        }
        let parts = template
            .elements()
            .iter()
            .map(|element| match element {
                TemplateElement::String(symbol) => {
                    vec![FiniteSourceValue::Text(
                        self.interned_runtime_string(*symbol),
                    )]
                }
                TemplateElement::Expr(expression) => {
                    self.function_source_value_candidates(expression)
                }
            })
            .collect::<Vec<_>>();
        // This closed Cartesian product uses the existing 256-tuple budget.
        // Each substitution still evaluates and performs ToString at runtime.
        let expanded = Self::source_candidate_tuples(&parts, true);
        if expanded.is_empty() {
            return self.finite_correlated_template_source_hints(template, &parts);
        }
        // Preserve every fitting Cartesian source, including mixed text after
        // mutation; the additional search never narrows an admitted product.
        expanded
            .into_iter()
            .map(|parts| FiniteSourceValue::SupplementalText(parts.concat()))
            .collect()
    }

    pub(super) fn finite_string_call_source_candidates(
        &self,
        call: &boa_ast::expression::Call,
        callees: &[FiniteSourceValue],
        filter_receiver: Option<&[FiniteSourceValue]>,
    ) -> Vec<FiniteSourceValue> {
        let possible_string = callees.iter().any(|callee| {
            matches!(callee, FiniteSourceValue::Function(id)
                if StandardBuiltinId::from_function_id(id)
                    == Some(StandardBuiltinId::StringConstructor))
        }) || matches!(Self::unwrap_parenthesized_expr(call.function()),
            Expression::Identifier(identifier)
                if self.interner.resolve_expect(identifier.sym()).to_string() == "String"
                    && self.identifier_resolves_to_intrinsic_global("String"));
        if possible_string {
            let Some(argument) = call.args().first() else {
                return Vec::new();
            };
            if matches!(argument, Expression::Spread(_)) {
                return Vec::new();
            }
            return bounded_candidates(
                self.function_source_value_candidates(argument)
                    .into_iter()
                    .filter_map(|value| {
                        value
                            .text()
                            .map(|text| FiniteSourceValue::SupplementalText(text.to_owned()))
                    }),
            );
        }
        let Expression::PropertyAccess(PropertyAccess::Simple(access)) =
            Self::unwrap_parenthesized_expr(call.function())
        else {
            return Vec::new();
        };
        let name = self.finite_source_access_name(access);
        if name.as_deref() != Some("filter") {
            return Vec::new();
        }
        let Some(receiver) = filter_receiver else {
            return Vec::new();
        };
        let elements = bounded_candidates(
            receiver
                .iter()
                .cloned()
                .flat_map(|value| match value {
                    FiniteSourceValue::Array(elements) => elements,
                    FiniteSourceValue::Text(_)
                    | FiniteSourceValue::SupplementalText(_)
                    | FiniteSourceValue::ReturnedCallableSource(_)
                    | FiniteSourceValue::Function(_)
                    | FiniteSourceValue::FunctionConstructor(_)
                    | FiniteSourceValue::Record(_) => Vec::new(),
                })
                .map(FiniteSourceValue::supplemental),
        );
        if elements.is_empty() {
            return Vec::new();
        }
        // A filter can retain any input element. Neither its predicate nor its
        // result is evaluated here. An overwritten method, species constructor,
        // accessor, proxy or callback can produce other values; the original
        // Get/Call and the source dispatcher's exact live checks remain required.
        vec![FiniteSourceValue::Array(elements)]
    }
}
