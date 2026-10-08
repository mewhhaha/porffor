//! Compilation-local syntax owners shared by finite-source discovery and the
//! final lowering pass. A cache entry includes its exact grammar/context and
//! retains both successful parse products and deferred failures.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use lila_front::{
    DirectEvalParseContext, EvalParseContext, FunctionParseKind, ParseError, ParseOptions,
    ParsedScript, ParsedSource,
};

use crate::{DynamicFunctionKind, ModuleRequestKeyIr, PreparedScriptKind};

#[derive(Debug, Clone, PartialEq, Eq)]
enum PreparedSourceInput {
    RealmScript(String),
    IndirectEval(String),
    ShadowRealmEvaluate(String),
    DirectEval(String, DirectEvalParseContext),
    Function(DynamicFunctionKind, Vec<String>),
}

#[derive(Debug, Default)]
struct PreparedSourceCacheState {
    parsed: Vec<(PreparedSourceInput, Result<ParsedScript, ParseError>)>,
    realm_requests: BTreeSet<ModuleRequestKeyIr>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PreparedSourceCache(Rc<RefCell<PreparedSourceCacheState>>);

impl PreparedSourceCache {
    pub(crate) fn parse_script(
        &self,
        kind: &PreparedScriptKind,
        source: &str,
    ) -> Result<ParsedScript, ParseError> {
        let input = match kind {
            PreparedScriptKind::RealmScript => PreparedSourceInput::RealmScript(source.to_owned()),
            PreparedScriptKind::IndirectEval => {
                PreparedSourceInput::IndirectEval(source.to_owned())
            }
            PreparedScriptKind::ShadowRealmEvaluate => {
                PreparedSourceInput::ShadowRealmEvaluate(source.to_owned())
            }
            PreparedScriptKind::DirectEval(context) => {
                PreparedSourceInput::DirectEval(source.to_owned(), context.parse_context())
            }
        };
        self.parse(input)
    }

    pub(crate) fn parse_function(
        &self,
        kind: DynamicFunctionKind,
        arguments: &[String],
    ) -> Result<ParsedScript, ParseError> {
        self.parse(PreparedSourceInput::Function(kind, arguments.to_vec()))
    }

    fn parse(&self, input: PreparedSourceInput) -> Result<ParsedScript, ParseError> {
        if let Some((_, parsed)) = self
            .0
            .borrow()
            .parsed
            .iter()
            .find(|(candidate, _)| candidate == &input)
        {
            return parsed.clone();
        }
        let parsed = match &input {
            PreparedSourceInput::RealmScript(source) => {
                lila_front::parse(source.clone(), ParseOptions::script()).map(|parsed| {
                    let ParsedSource::Script(parsed) = parsed else {
                        unreachable!("Script goal produces a Script")
                    };
                    parsed
                })
            }
            PreparedSourceInput::IndirectEval(source)
            | PreparedSourceInput::ShadowRealmEvaluate(source) => {
                lila_front::prepare_eval_source(source.clone(), &EvalParseContext::Indirect)
            }
            PreparedSourceInput::DirectEval(source, context) => lila_front::prepare_eval_source(
                source.clone(),
                &EvalParseContext::Direct(context.clone()),
            ),
            PreparedSourceInput::Function(kind, arguments) => {
                let kind = match kind {
                    DynamicFunctionKind::Ordinary => FunctionParseKind::Ordinary,
                    DynamicFunctionKind::Generator => FunctionParseKind::Generator,
                    DynamicFunctionKind::Async => FunctionParseKind::Async,
                    DynamicFunctionKind::AsyncGenerator => FunctionParseKind::AsyncGenerator,
                };
                lila_front::prepare_dynamic_function(kind, arguments)
            }
        };
        let requests = parsed
            .as_ref()
            .map(crate::modules::scan_script_realm_module_requests)
            .unwrap_or_default();
        let mut state = self.0.borrow_mut();
        state.realm_requests.extend(requests);
        state.parsed.push((input, parsed.clone()));
        parsed
    }

    pub(crate) fn realm_module_requests(&self) -> Vec<ModuleRequestKeyIr> {
        self.0.borrow().realm_requests.iter().cloned().collect()
    }

    pub(crate) fn record_realm_module_requests(
        &self,
        requests: impl IntoIterator<Item = ModuleRequestKeyIr>,
    ) {
        self.0.borrow_mut().realm_requests.extend(requests);
    }
}

/// One host compilation's finite-source parsing and request-discovery owner.
/// Lowering retries allocate fresh IR identities while retaining the exact
/// prepared syntax products. This owner is local to preparation, not a shared
/// process-wide artifact or parser cache.
#[derive(Debug, Default)]
pub struct LoweringSession {
    pub(crate) prepared_sources: PreparedSourceCache,
}

impl LoweringSession {
    /// Potential Realm-origin imports found in successfully parsed finite
    /// sources, including nested sources and Function constructor bodies.
    pub fn realm_module_requests(&self) -> Vec<ModuleRequestKeyIr> {
        self.prepared_sources.realm_module_requests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_parse_identity_and_rejections_keep_the_exact_grammar_context() {
        let cache = PreparedSourceCache::default();
        let input = PreparedSourceInput::ShadowRealmEvaluate("1 + 2".into());
        let first = cache.parse(input.clone()).unwrap();
        let again = cache.parse(input).unwrap();
        assert_eq!(first.syntax_identity(), again.syntax_identity());
        let other_kind = cache
            .parse(PreparedSourceInput::RealmScript("1 + 2".into()))
            .unwrap();
        assert_ne!(first.syntax_identity(), other_kind.syntax_identity());

        let context = |invocation| DirectEvalParseContext {
            strict_caller: false,
            invocation,
            private_names: Vec::new(),
        };
        let outside = PreparedSourceInput::DirectEval(
            "new.target".into(),
            context(lila_front::EvalInvocationContext::Script),
        );
        let rejected = cache.parse(outside.clone()).unwrap_err();
        let count = cache.0.borrow().parsed.len();
        assert_eq!(cache.parse(outside).unwrap_err(), rejected);
        assert_eq!(cache.0.borrow().parsed.len(), count);
        assert!(cache
            .parse(PreparedSourceInput::DirectEval(
                "new.target".into(),
                context(lila_front::EvalInvocationContext::Function),
            ))
            .is_ok());
        assert!(cache
            .parse_function(DynamicFunctionKind::Ordinary, &["yield 1;".into()])
            .is_err());
        assert!(cache
            .parse_function(DynamicFunctionKind::Generator, &["yield 1;".into()])
            .is_ok());
    }

    #[test]
    fn nested_and_function_sources_discover_requests_without_reparsing_on_retry() {
        let source = lila_front::parse(
            r#"
const realm = new ShadowRealm();
const evaluate = realm.evaluate;
evaluate.call(realm, `new ShadowRealm().evaluate('void new ShadowRealm().importValue("./nested.js", "value")')`);
Function('void new ShadowRealm().importValue("./function.js", "value")')();
"#,
            ParseOptions::script(),
        )
        .unwrap();
        let session = LoweringSession::default();
        let program = session.lower_source(&source, crate::HostSurfacePolicy::default());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert_eq!(
            session.realm_module_requests(),
            [
                ModuleRequestKeyIr::plain("./function.js"),
                ModuleRequestKeyIr::plain("./nested.js"),
            ]
        );
        let identities = || {
            session
                .prepared_sources
                .0
                .borrow()
                .parsed
                .iter()
                .map(|(_, parsed)| {
                    parsed
                        .as_ref()
                        .map(ParsedScript::syntax_identity)
                        .map_err(Clone::clone)
                })
                .collect::<Vec<_>>()
        };
        let first = identities();
        let retried = session.lower_source(&source, crate::HostSurfacePolicy::default());
        assert!(retried.is_wasm_supported(), "{:?}", retried.diagnostics);
        assert_eq!(identities(), first);
    }

    #[test]
    fn realm_import_aliases_forwarding_optional_calls_and_finite_names_are_candidates() {
        let source = lila_front::parse(
            r#"
const realm = new ShadowRealm();
const load = realm.importValue;
const specifier = './' + 'alias.js';
const requestKey = 'importValue', callKey = 'call', applyKey = 'apply';
load.call(realm, specifier, 'value');
Reflect.apply(load, realm, ['./reflected.js', 'value']);
realm['import' + 'Value']('./computed.js', 'value');
realm.importValue?.('./optional.js', 'value');
realm?.importValue('./optional-receiver.js', 'value');
realm?.['import' + 'Value']?.('./optional-computed.js', 'value');
load?.call(realm, './optional-call-target.js', 'value');
load.call?.(realm, './optional-call-method.js', 'value');
load?.apply(realm, ['./optional-apply-target.js', 'value']);
load.apply?.(realm, ['./optional-apply-method.js', 'value']);
realm?.importValue.call(realm, './optional-direct-call-target.js', 'value');
realm.importValue.call?.(realm, './optional-direct-call-method.js', 'value');
realm?.importValue.apply(realm, ['./optional-direct-apply-target.js', 'value']);
realm.importValue.apply?.(realm, ['./optional-direct-apply-method.js', 'value']);
realm[requestKey]('./named-key.js', 'value');
realm?.[requestKey]('./optional-key.js', 'value');
realm?.[requestKey][callKey](realm, './optional-key-call-target.js', 'value');
realm[requestKey][callKey]?.(realm, './optional-key-call-method.js', 'value');
load?.[applyKey](realm, ['./optional-key-apply.js', 'value']);
"#,
            ParseOptions::script(),
        )
        .unwrap();
        let session = LoweringSession::default();
        let program = session.lower_source(&source, crate::HostSurfacePolicy::default());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert_eq!(
            session.realm_module_requests(),
            [
                ModuleRequestKeyIr::plain("./alias.js"),
                ModuleRequestKeyIr::plain("./computed.js"),
                ModuleRequestKeyIr::plain("./named-key.js"),
                ModuleRequestKeyIr::plain("./optional-apply-method.js"),
                ModuleRequestKeyIr::plain("./optional-apply-target.js"),
                ModuleRequestKeyIr::plain("./optional-call-method.js"),
                ModuleRequestKeyIr::plain("./optional-call-target.js"),
                ModuleRequestKeyIr::plain("./optional-computed.js"),
                ModuleRequestKeyIr::plain("./optional-direct-apply-method.js"),
                ModuleRequestKeyIr::plain("./optional-direct-apply-target.js"),
                ModuleRequestKeyIr::plain("./optional-direct-call-method.js"),
                ModuleRequestKeyIr::plain("./optional-direct-call-target.js"),
                ModuleRequestKeyIr::plain("./optional-key-apply.js"),
                ModuleRequestKeyIr::plain("./optional-key-call-method.js"),
                ModuleRequestKeyIr::plain("./optional-key-call-target.js"),
                ModuleRequestKeyIr::plain("./optional-key.js"),
                ModuleRequestKeyIr::plain("./optional-receiver.js"),
                ModuleRequestKeyIr::plain("./optional.js"),
                ModuleRequestKeyIr::plain("./reflected.js"),
            ]
        );
    }

    #[test]
    fn optional_source_projection_retains_original_function_parameter_ownership() {
        let source = lila_front::parse(
            r#"
const realm = new ShadowRealm();
((source) => realm.evaluate(source))?.('6 * 7');
(function (source) { return realm.evaluate(source); })?.('7 * 8');
({ invoke(source) { return realm.evaluate(source); } })?.invoke('8 * 9');
"#,
            ParseOptions::script(),
        )
        .unwrap();
        let program =
            LoweringSession::default().lower_source(&source, crate::HostSurfacePolicy::default());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        for source in ["6 * 7", "7 * 8", "8 * 9"] {
            assert!(
                script.prepared_scripts.iter().any(|prepared| {
                    prepared.kind == PreparedScriptKind::ShadowRealmEvaluate
                        && prepared.source == source
                        && matches!(
                            prepared.outcome,
                            crate::PreparedScriptOutcome::Executable(_)
                        )
                }),
                "original function parameter lost source candidate {source}"
            );
        }
    }
}
