//! Optional constructor grammar hints across independent Function source units.

use super::dynamic_source::dynamic_source_kind_for_function_id;
use super::*;
use crate::prepared_source_cache::FunctionGlobalConstructorHints;

impl ScriptLowerer<'_> {
    pub(super) fn collect_function_global_hints(&self) -> FunctionGlobalConstructorHints {
        let mut hints = self.analysis.function_global_hints.clone();
        // A Function body never captures the caller's activation or eval cells.
        // This first slice only exports analyzed Script global object bindings.
        if !matches!(
            self.analysis.script_instantiation,
            ScriptInstantiation::FreshEntry
        ) || !self.script_variables_are_global()
        {
            return hints;
        }
        for (name, binding) in &self.var_bindings {
            if !binding.is_script_global {
                continue;
            }
            for candidate in self
                .finite_binding_source_candidates(name)
                .unwrap_or_default()
            {
                let kind = match candidate {
                    FiniteSourceValue::FunctionConstructor(kind) => Some(*kind),
                    FiniteSourceValue::Function(function_id) => {
                        match dynamic_source_kind_for_function_id(function_id) {
                            Some(DynamicSourceKind::Function(kind)) => Some(kind),
                            Some(
                                DynamicSourceKind::DirectEval
                                | DynamicSourceKind::IndirectEval
                                | DynamicSourceKind::RealmEvalScript
                                | DynamicSourceKind::ShadowRealmEvaluate,
                            )
                            | None => None,
                        }
                    }
                    FiniteSourceValue::Text(_)
                    | FiniteSourceValue::SupplementalText(_)
                    | FiniteSourceValue::ReturnedCallableSource(_)
                    | FiniteSourceValue::Record(_)
                    | FiniteSourceValue::Array(_) => None,
                };
                if let Some(kind) = kind {
                    hints.observe(name, kind);
                }
            }
        }
        hints
    }

    pub(super) fn register_function_global_hint_candidates(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) {
        let Expression::Identifier(identifier) = Self::unwrap_parenthesized_expr(callee) else {
            return;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        if self.lookup_binding(&name).is_some()
            || self.visible_function_names.contains_key(&name)
            || !self.with_environment_chain.is_empty()
            || self.uses_runtime_identifier_environment()
            || arguments
                .iter()
                .any(|argument| matches!(argument, Expression::Spread(_)))
        {
            return;
        }
        let kinds = self
            .analysis
            .function_global_hints
            .kinds(&name)
            .collect::<Vec<_>>();
        if kinds.is_empty() {
            return;
        }
        // Read existing scalar source hints only. Generic value discovery can
        // itself parse a returned Realm callable and discover host requests.
        let candidates = arguments
            .iter()
            .map(|argument| {
                let mut texts = Vec::new();
                if let Expression::Identifier(identifier) =
                    Self::unwrap_parenthesized_expr(argument)
                {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    texts.extend(
                        self.finite_binding_source_candidates(&name)
                            .unwrap_or_default()
                            .iter()
                            .filter_map(|candidate| candidate.text().map(str::to_owned)),
                    );
                }
                texts.extend(self.function_source_candidate(argument));
                texts
                    .into_iter()
                    .map(FiniteSourceValue::SupplementalText)
                    .collect()
            })
            .collect::<Vec<_>>();
        let tuples = Self::source_candidate_tuples(&candidates, true);
        for kind in kinds {
            for arguments in &tuples {
                if self
                    .analysis
                    .allocations
                    .prepared_sources
                    .admits_function_global_hint(kind, arguments)
                {
                    self.register_dynamic_function_source(DynamicFunctionSource {
                        admission: crate::prepared_function::DynamicFunctionSourceAdmission::RuntimeCandidate,
                        kind,
                        arguments: arguments.clone(),
                    });
                }
            }
        }
    }
}
