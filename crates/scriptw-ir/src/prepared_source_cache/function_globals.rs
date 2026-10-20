//! Canonical constructor grammars attached to an actual prepared Function parse.
//! No parent IR identities, runtime values or Realm assumptions cross this seam.

use super::*;
use boa_ast::{
    visitor::{VisitWith, Visitor},
    Expression,
};
use scriptw_front::{ParseDiagnosticKind, ParsedScriptIdentity};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FunctionGlobalConstructorHints {
    names: BTreeMap<String, BTreeSet<DynamicFunctionKind>>,
}

impl FunctionGlobalConstructorHints {
    pub(crate) fn observe(&mut self, name: &str, kind: DynamicFunctionKind) {
        if self.names.values().map(BTreeSet::len).sum::<usize>() < MAX_SOURCE_CANDIDATES {
            self.names.entry(name.to_string()).or_default().insert(kind);
        }
    }

    pub(crate) fn kinds(&self, name: &str) -> impl Iterator<Item = DynamicFunctionKind> + '_ {
        self.names.get(name).into_iter().flatten().copied()
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        for (name, kinds) in &other.names {
            for kind in kinds {
                self.observe(name, *kind);
            }
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct FunctionGlobalHintOwners {
    owners: Vec<(ParsedScriptIdentity, FunctionGlobalConstructorHints)>,
}

// This deliberately small supplemental domain cannot discover nested source,
// private eval contexts, literal imports or host Realm module requests.
const MAX_GLOBAL_HINT_SOURCE_BYTES: usize = 64 * 1024;

struct ResourceFreeFunction;

impl<'ast> Visitor<'ast> for ResourceFreeFunction {
    type BreakTy = ();

    fn visit_expression(&mut self, expression: &'ast Expression) -> ControlFlow<()> {
        if matches!(
            expression,
            Expression::Call(_)
                | Expression::SuperCall(_)
                | Expression::New(_)
                | Expression::ImportCall(_)
                | Expression::TaggedTemplate(_)
        ) || matches!(expression, Expression::Optional(optional)
            if optional.chain().iter().any(|operation|
                matches!(operation.kind(), boa_ast::expression::OptionalOperationKind::Call { .. })))
        {
            return ControlFlow::Break(());
        }
        expression.visit_with(self)
    }
}

impl PreparedSourceCache {
    pub(crate) fn record_function_global_hints(
        &self,
        parsed: &ParsedScript,
        hints: &FunctionGlobalConstructorHints,
    ) {
        let identity = parsed.syntax_identity();
        let mut state = self.0.borrow_mut();
        let owners = &mut state.function_globals.owners;
        if let Some((_, known)) = owners.iter_mut().find(|(owner, _)| *owner == identity) {
            known.merge(hints);
        } else {
            owners.push((identity, hints.clone()));
        }
    }

    pub(crate) fn function_global_hints(
        &self,
        parsed: &ParsedScript,
    ) -> FunctionGlobalConstructorHints {
        let identity = parsed.syntax_identity();
        self.0
            .borrow()
            .function_globals
            .owners
            .iter()
            .find(|(owner, _)| *owner == identity)
            .map(|(_, hints)| hints.clone())
            .unwrap_or_default()
    }

    pub(crate) fn admits_function_global_hint(
        &self,
        kind: DynamicFunctionKind,
        arguments: &[String],
    ) -> bool {
        if arguments.iter().fold(0usize, |bytes, argument| {
            bytes.saturating_add(argument.len())
        }) > MAX_GLOBAL_HINT_SOURCE_BYTES
        {
            return false;
        }
        let parsed = self.parse_syntax(PreparedSourceInput::Function(
            kind,
            arguments
                .iter()
                .map(|argument| crate::decode_js_string_utf16(argument))
                .collect(),
        ));
        match parsed {
            Ok(parsed) => parsed.with_compiler_session(|script, _| {
                ResourceFreeFunction.visit_script(script).is_continue()
            }),
            Err(error) => error.diagnostic().kind() == ParseDiagnosticKind::MalformedJavaScript,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_global_hints_keep_exact_parse_owner_grammar_and_256_cap() {
        let cache = PreparedSourceCache::default();
        let ordinary = cache
            .parse_function(DynamicFunctionKind::Ordinary, &[])
            .unwrap();
        let generator = cache
            .parse_function(DynamicFunctionKind::Generator, &[])
            .unwrap();
        let separate =
            scriptw_front::prepare_dynamic_function_utf16(FunctionParseKind::Ordinary, &[])
                .unwrap();
        let mut hints = FunctionGlobalConstructorHints::default();
        for index in 0..256 {
            hints.observe(&format!("alias{index}"), DynamicFunctionKind::Generator);
        }
        hints.observe("overflow", DynamicFunctionKind::Async);
        cache.record_function_global_hints(&ordinary, &hints);
        let retained = cache.function_global_hints(&ordinary.clone());
        assert_eq!(retained.names.len(), 256);
        assert_eq!(
            retained.kinds("alias0").collect::<Vec<_>>(),
            [DynamicFunctionKind::Generator]
        );
        assert_eq!(retained.kinds("overflow").count(), 0);
        assert!(cache.function_global_hints(&generator).names.is_empty());
        assert!(cache.function_global_hints(&separate).names.is_empty());
    }

    #[test]
    fn hint_parses_never_request_modules_and_normal_admission_promotes_cached_syntax() {
        let cache = PreparedSourceCache::default();
        assert!(
            cache.admits_function_global_hint(DynamicFunctionKind::Generator, &["yield 1;".into()])
        );
        assert!(cache.admits_function_global_hint(
            DynamicFunctionKind::Generator,
            &["yield".into(), "return yield;".into()]
        ));
        for body in [
            "return realm.importValue('./hint-only.js', 'value');",
            "return import('./dynamic-only.js');",
            "return eval('x');",
            "return new Function('return 1;');",
            "return class extends Function { constructor() { super(\"return realm.importValue('./super-only.js', 'value');\"); } };",
            "return tag`text`;",
            "return realm?.importValue?.('./optional-only.js', 'value');",
        ] {
            assert!(
                !cache.admits_function_global_hint(DynamicFunctionKind::Ordinary, &[body.into()])
            );
        }
        assert!(cache.realm_module_requests().is_empty());
        cache
            .parse_function(
                DynamicFunctionKind::Ordinary,
                &["return realm.importValue('./hint-only.js', 'value');".into()],
            )
            .unwrap();
        assert_eq!(
            cache.realm_module_requests(),
            [ModuleRequestKeyIr::plain("./hint-only.js")]
        );
    }
}
