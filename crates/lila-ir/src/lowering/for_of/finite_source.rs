//! Optional source hints follow array binding cells after real destructuring.

use super::super::finite_function_source::bounded_candidates;
use super::*;
use boa_ast::pattern::ArrayPatternElement;

fn array_member_candidates(candidates: &[FiniteSourceValue]) -> Vec<FiniteSourceValue> {
    bounded_candidates(candidates.iter().flat_map(|candidate| match candidate {
        FiniteSourceValue::Array(elements) => elements.clone(),
        FiniteSourceValue::Text(_)
        | FiniteSourceValue::SupplementalText(_)
        | FiniteSourceValue::Function(_)
        | FiniteSourceValue::ReturnedCallableSource(_)
        | FiniteSourceValue::FunctionConstructor(_)
        | FiniteSourceValue::Record(_) => Vec::new(),
    }))
}

impl ScriptLowerer<'_> {
    pub(super) fn register_for_of_array_pattern_source_candidates(
        &mut self,
        mode: BindingMode,
        pattern: &Pattern,
        candidates: &[FiniteSourceValue],
    ) {
        // A sloppy eval var pattern writes the caller's live variable record;
        // it does not declare these names in this prepared unit's own record.
        if mode == BindingMode::Var && self.borrows_direct_eval_variable_environment() {
            return;
        }
        self.project_for_of_array_pattern_source_candidates(pattern, candidates);
    }

    fn project_for_of_array_pattern_source_candidates(
        &mut self,
        pattern: &Pattern,
        candidates: &[FiniteSourceValue],
    ) {
        let Pattern::Array(pattern) = pattern else {
            return;
        };
        // Array candidates are unordered possible members, not positional
        // tuples. Any member may supply any leaf at this pattern depth. These
        // bounded supersets only prepare sources; the live iterator, defaults,
        // binding writes and exact runtime source admission remain unchanged.
        let members = array_member_candidates(candidates);
        for element in pattern.bindings() {
            match element {
                ArrayPatternElement::SingleName {
                    ident,
                    default_init,
                } => {
                    let values = bounded_candidates(
                        members.iter().cloned().chain(
                            default_init
                                .as_ref()
                                .into_iter()
                                .flat_map(|default| self.function_source_value_candidates(default)),
                        ),
                    );
                    let name = self.interner.resolve_expect(ident.sym()).to_string();
                    self.merge_for_of_pattern_binding_source_candidates(&name, values);
                }
                ArrayPatternElement::Pattern {
                    pattern,
                    default_init,
                } => {
                    let values = bounded_candidates(
                        members.iter().cloned().chain(
                            default_init
                                .as_ref()
                                .into_iter()
                                .flat_map(|default| self.function_source_value_candidates(default)),
                        ),
                    );
                    self.project_for_of_array_pattern_source_candidates(pattern, &values);
                }
                ArrayPatternElement::SingleNameRest { ident } => {
                    if !members.is_empty() {
                        let name = self.interner.resolve_expect(ident.sym()).to_string();
                        self.merge_for_of_pattern_binding_source_candidates(
                            &name,
                            vec![FiniteSourceValue::Array(members.clone())],
                        );
                    }
                }
                ArrayPatternElement::PatternRest { pattern } => {
                    self.project_for_of_array_pattern_source_candidates(
                        pattern,
                        &[FiniteSourceValue::Array(members.clone())],
                    );
                }
                ArrayPatternElement::Elision
                | ArrayPatternElement::PropertyAccess { .. }
                | ArrayPatternElement::PropertyAccessRest { .. } => {}
            }
        }
    }

    fn merge_for_of_pattern_binding_source_candidates(
        &mut self,
        name: &str,
        candidates: Vec<FiniteSourceValue>,
    ) {
        if candidates.is_empty() {
            return;
        }
        let reference = self.locate_identifier_reference(name);
        if self
            .with_environment_chain
            .select_preceding(reference.declarative_position())
            .is_some()
        {
            return;
        }
        // The loop may execute zero times, and later iterations may retain
        // earlier writes. Merge against the same analyzed cell's existing
        // hints instead of replacing them with the possible iteration values.
        let previous = self
            .finite_binding_source_candidates(name)
            .unwrap_or_default();
        let candidates = bounded_candidates(
            previous
                .iter()
                .cloned()
                .chain(candidates.into_iter().map(FiniteSourceValue::supplemental)),
        );
        // Non-text candidates cannot carry supplemental priority. An overflow
        // must still leave the already retained source set intact.
        if candidates.is_empty() && !previous.is_empty() {
            return;
        }
        self.set_finite_source_binding_candidates(name, candidates);
    }
}
