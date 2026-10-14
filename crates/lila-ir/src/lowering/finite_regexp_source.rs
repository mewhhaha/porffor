//! Optional RegExp .source views; live construction and Get remain authoritative.

use super::finite_function_source::bounded_candidates;
use super::*;
use lila_front::{decode_js_string_utf16, encode_js_string_utf16};

impl ScriptLowerer<'_> {
    fn finite_regexp_pattern_is_undefined(&self, expression: &Expression) -> bool {
        match Self::unwrap_parenthesized_expr(expression) {
            Expression::Unary(unary) if unary.op() == UnaryOp::Void => true,
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                self.lookup_binding(&name).is_some_and(|binding| {
                    binding.possible_kinds == KindSet::from_kind(ValueKind::Undefined)
                }) || (name == "undefined" && self.identifier_resolves_to_intrinsic_global(&name))
            }
            _ => false,
        }
    }

    pub(super) fn finite_regexp_source_candidates(
        &self,
        expression: &Expression,
        callee_candidates: &[FiniteSourceValue],
    ) -> Vec<FiniteSourceValue> {
        let pattern_candidates = match Self::unwrap_parenthesized_expr(expression) {
            Expression::RegExpLiteral(regexp) => {
                vec![self.interned_runtime_string(regexp.pattern())]
            }
            expression @ (Expression::Call(_) | Expression::New(_)) => {
                let arguments = match expression {
                    Expression::Call(call) => call.args(),
                    Expression::New(construct) => construct.arguments(),
                    _ => unreachable!("call or construction matched above"),
                };
                let possible_regexp = callee_candidates.iter().any(|candidate| {
                    matches!(candidate,
                        FiniteSourceValue::Function(function_id)
                        if StandardBuiltinId::from_function_id(function_id)
                            == Some(StandardBuiltinId::RegExpConstructor))
                });
                if !possible_regexp {
                    return Vec::new();
                }
                match arguments.first() {
                    None => vec![String::new()],
                    Some(pattern) if self.finite_regexp_pattern_is_undefined(pattern) => {
                        vec![String::new()]
                    }
                    Some(pattern) if matches!(pattern, Expression::Spread(_)) => {
                        return Vec::new();
                    }
                    Some(pattern) => bounded_candidates(
                        self.function_source_value_candidates(pattern)
                            .into_iter()
                            .flat_map(|candidate| match candidate {
                                FiniteSourceValue::Text(text)
                                | FiniteSourceValue::SupplementalText(text) => vec![text],
                                FiniteSourceValue::Record(mut properties) => properties
                                    .remove("source")
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter_map(|value| value.text().map(str::to_owned))
                                    .collect(),
                                _ => Vec::new(),
                            })
                            .map(FiniteSourceValue::SupplementalText),
                    )
                    .into_iter()
                    .filter_map(|value| value.text().map(str::to_owned))
                    .collect(),
                }
            }
            _ => return Vec::new(),
        };
        let sources = bounded_candidates(pattern_candidates.into_iter().map(|pattern| {
            let escaped = escape_regexp_pattern_utf16(&decode_js_string_utf16(&pattern));
            FiniteSourceValue::SupplementalText(encode_js_string_utf16(&escaped))
        }));
        if sources.is_empty() {
            Vec::new()
        } else {
            // Every leaf is supplemental, so the record itself consumes only
            // spare candidate capacity. It proves no instance or getter identity.
            vec![FiniteSourceValue::Record(BTreeMap::from([(
                "source".to_string(),
                sources,
            )]))]
        }
    }
}

// Test-only work accounting measures each complete candidate-discovery tree,
// independent of wall time or the number of lowering retries.
#[cfg(test)]
thread_local! {
    static SOURCE_DISCOVERY_WORK: std::cell::Cell<(usize, usize, usize)> =
        const { std::cell::Cell::new((0, 0, 0)) };
}

#[cfg(test)]
pub(super) struct SourceCandidateDiscoveryWork;

#[cfg(test)]
pub(super) fn enter_source_candidate_discovery() -> SourceCandidateDiscoveryWork {
    SOURCE_DISCOVERY_WORK.with(|work| {
        let (depth, count, largest) = work.get();
        work.set((depth + 1, if depth == 0 { 1 } else { count + 1 }, largest));
    });
    SourceCandidateDiscoveryWork
}

#[cfg(test)]
impl Drop for SourceCandidateDiscoveryWork {
    fn drop(&mut self) {
        SOURCE_DISCOVERY_WORK.with(|work| {
            let (depth, count, largest) = work.get();
            let remaining = depth - 1;
            work.set((
                remaining,
                count,
                if remaining == 0 {
                    largest.max(count)
                } else {
                    largest
                },
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::SOURCE_DISCOVERY_WORK;

    #[test]
    fn ordinary_nested_callee_discovery_keeps_linear_work_and_no_source_authority() {
        for depth in [4, 8, 12] {
            for construct in [false, true] {
                SOURCE_DISCOVERY_WORK.with(|work| {
                    assert_eq!(work.get().0, 0, "no discovery is active at control entry");
                    work.set((0, 0, 0));
                });
                let callee = format!("unknown{}", "()".repeat(depth));
                let value = if construct {
                    format!("new ({callee})().source")
                } else {
                    callee
                };
                let source = format!("eval('/' + {value} + '/');");
                let parsed =
                    lila_front::parse(&source, lila_front::ParseOptions::script()).unwrap();
                let program = crate::lower(&parsed);
                assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
                assert!(
                    program.script.as_ref().unwrap().prepared_scripts.is_empty(),
                    "unknown callees cannot supply a prepared source: {source}"
                );
                SOURCE_DISCOVERY_WORK.with(|work| {
                    let (active, _, largest) = work.get();
                    assert_eq!(active, 0, "every discovery guard has completed");
                    assert!(largest >= depth + 1, "the actual nested callee was traversed");
                    assert!(largest <= 2 * depth + 8,
                        "duplicated callee discovery: depth={depth}, work={largest}, construct={construct}");
                });
            }
        }
    }
}
