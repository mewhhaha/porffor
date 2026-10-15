//! Argument-source requests belong to retained parsed syntax, never a guessed
//! runtime callable identity or a FunctionId from another analysis.

use super::*;
use boa_ast::{Expression, Statement, StatementListItem};
use lila_front::ParsedScriptIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ReturnedCallableSourceId(u32);

#[derive(Debug)]
struct ReturnedCallableSource {
    parsed: ParsedScriptIdentity,
    function_key: String,
    arguments: Vec<BTreeSet<String>>,
}

#[derive(Debug, Default)]
pub(super) struct ReturnedCallableSources {
    owners: Vec<ReturnedCallableSource>,
}

impl PreparedSourceCache {
    pub(crate) fn returned_shadow_callable_source(
        &self,
        source: &str,
    ) -> Option<ReturnedCallableSourceId> {
        let parsed = self
            .parse_script(&PreparedScriptKind::ShadowRealmEvaluate, source)
            .ok()?;
        let (function_key, fixed_parameters) = parsed.with_compiler_session(|script, _| {
            let StatementListItem::Statement(statement) =
                script.statements().statements().last()?
            else {
                return None;
            };
            let Statement::Expression(expression) = statement.as_ref() else {
                return None;
            };
            let mut expression = expression;
            while let Expression::Parenthesized(parenthesized) = expression {
                expression = parenthesized.expression();
            }
            let (key, parameters) = match expression {
                Expression::FunctionExpression(function) => (
                    crate::lowering_helpers::function_expression_key(function),
                    function.parameters(),
                ),
                Expression::ArrowFunction(function) => (
                    crate::lowering_helpers::arrow_function_key(function),
                    function.parameters(),
                ),
                _ => return None,
            };
            Some((
                key,
                parameters
                    .as_ref()
                    .iter()
                    .take_while(|parameter| !parameter.is_rest_param())
                    .count(),
            ))
        })?;
        let identity = parsed.syntax_identity();
        let mut state = self.0.borrow_mut();
        let owners = &mut state.returned_callables.owners;
        if let Some(index) = owners
            .iter()
            .position(|owner| owner.parsed == identity && owner.function_key == function_key)
        {
            return Some(ReturnedCallableSourceId(
                u32::try_from(index).expect("prepared callable source space exhausted"),
            ));
        }
        let id = ReturnedCallableSourceId(
            u32::try_from(owners.len()).expect("prepared callable source space exhausted"),
        );
        owners.push(ReturnedCallableSource {
            parsed: identity,
            function_key,
            arguments: vec![BTreeSet::new(); fixed_parameters],
        });
        Some(id)
    }

    pub(crate) fn observe_returned_callable_arguments(
        &self,
        source: ReturnedCallableSourceId,
        arguments: &[Vec<String>],
    ) {
        let mut state = self.0.borrow_mut();
        let owner = &mut state.returned_callables.owners[source.0 as usize];
        for (known, candidates) in owner.arguments.iter_mut().zip(arguments) {
            for candidate in candidates {
                // Additions are monotone and bounded. Later optional requests
                // cannot evict an already available text or any primary hint.
                if known.len() == MAX_SOURCE_CANDIDATES {
                    break;
                }
                known.insert(candidate.clone());
            }
        }
    }

    pub(crate) fn returned_callable_parameter_hints(
        &self,
        parsed: &ParsedScript,
    ) -> Vec<(String, usize, Vec<String>)> {
        let identity = parsed.syntax_identity();
        let state = self.0.borrow();
        state
            .returned_callables
            .owners
            .iter()
            .filter(|owner| owner.parsed == identity)
            .flat_map(|owner| {
                owner
                    .arguments
                    .iter()
                    .enumerate()
                    .filter(|(_, values)| !values.is_empty())
                    .map(|(index, values)| {
                        (
                            owner.function_key.clone(),
                            index,
                            values.iter().cloned().collect(),
                        )
                    })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returned_callable_requests_keep_exact_parse_owner_and_grammar() {
        let cache = PreparedSourceCache::default();
        // Both callable sites have the same span and parameter spelling.
        let left_source = "1; (text) => text";
        let right_source = "2; (text) => text";
        let left = cache.returned_shadow_callable_source(left_source).unwrap();
        let right = cache.returned_shadow_callable_source(right_source).unwrap();
        assert_ne!(left, right);
        assert_eq!(
            cache.returned_shadow_callable_source(left_source),
            Some(left)
        );
        cache.observe_returned_callable_arguments(left, &[vec!["17;".into()]]);
        cache.observe_returned_callable_arguments(right, &[vec!["19;".into()]]);
        let parsed_left = cache
            .parse_script(&PreparedScriptKind::ShadowRealmEvaluate, left_source)
            .unwrap();
        let parsed_right = cache
            .parse_script(&PreparedScriptKind::ShadowRealmEvaluate, right_source)
            .unwrap();
        let left_hints = cache.returned_callable_parameter_hints(&parsed_left);
        let right_hints = cache.returned_callable_parameter_hints(&parsed_right);
        assert_eq!(left_hints.len(), 1);
        assert_eq!(right_hints.len(), 1);
        assert_eq!(left_hints[0].0, right_hints[0].0, "same AST site key");
        assert_eq!(left_hints[0].1, 0);
        assert_eq!(left_hints[0].2, ["17;"]);
        assert_eq!(right_hints[0].2, ["19;"]);
        for kind in [
            PreparedScriptKind::IndirectEval,
            PreparedScriptKind::RealmScript,
        ] {
            let other = cache.parse_script(&kind, left_source).unwrap();
            assert!(cache.returned_callable_parameter_hints(&other).is_empty());
        }
        let independently_parsed = lila_front::parse(left_source, ParseOptions::script()).unwrap();
        assert!(cache
            .returned_callable_parameter_hints(independently_parsed.as_script().unwrap())
            .is_empty());
        assert!(PreparedSourceCache::default()
            .returned_callable_parameter_hints(&parsed_left)
            .is_empty());
    }

    #[test]
    fn returned_callable_summary_accepts_only_terminal_ordinary_or_arrow_literals() {
        let cache = PreparedSourceCache::default();
        for source in [
            "(function(text) { return text; })",
            "(((text) => text))",
            "const captured = 1; (text, other = captured, ...rest) => text",
        ] {
            assert!(
                cache.returned_shadow_callable_source(source).is_some(),
                "{source}"
            );
        }
        for source in [
            "const fn = text => text; fn",
            "true ? (text => text) : (text => text)",
            "(() => text => text)()",
            "if (true) { text => text; }",
            "(async function(text) { return text; })",
            "(function*(text) { yield text; })",
            "async text => text",
            "(text => text); let later;",
            "let = ;",
        ] {
            assert_eq!(
                cache.returned_shadow_callable_source(source),
                None,
                "{source}"
            );
        }
    }

    #[test]
    fn returned_callable_arguments_are_monotone_capped_and_stop_before_rest() {
        let cache = PreparedSourceCache::default();
        let source = "(first, second = 'default', ...rest) => first";
        let owner = cache.returned_shadow_callable_source(source).unwrap();
        let first = (0..MAX_SOURCE_CANDIDATES)
            .map(|index| format!("{index};"))
            .collect::<Vec<_>>();
        cache.observe_returned_callable_arguments(owner, &[first.clone(), vec!["second;".into()]]);
        cache.observe_returned_callable_arguments(
            owner,
            &[
                vec!["late;".into(), "0;".into()],
                vec!["later-second;".into()],
                vec!["must-not-be-a-rest-request;".into()],
            ],
        );
        let parsed = cache
            .parse_script(&PreparedScriptKind::ShadowRealmEvaluate, source)
            .unwrap();
        let hints = cache.returned_callable_parameter_hints(&parsed);
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[0].1, 0);
        assert_eq!(hints[0].2.len(), MAX_SOURCE_CANDIDATES);
        for text in first {
            assert!(hints[0].2.contains(&text));
        }
        assert!(!hints[0].2.iter().any(|text| text == "late;"));
        assert_eq!(hints[1].1, 1);
        assert_eq!(hints[1].2, ["later-second;", "second;"]);
    }
}
