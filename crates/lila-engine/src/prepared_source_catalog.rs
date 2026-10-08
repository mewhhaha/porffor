//! Resolve finite prepared-source module requests before artifact identity is
//! computed. Each retry reuses retained syntax, and the accepted IR is the IR
//! later emitted rather than a discarded discovery product.

use super::*;

pub(super) fn prepare_source_catalog(
    source: &ParsedSource,
    prelude: Option<&lila_front::ParsedScript>,
    modules: &mut PreparedModuleCompilation,
    options: &CompileOptions,
) -> Result<ProgramIr, EngineError> {
    let session = lila_ir::LoweringSession::default();
    loop {
        let ir = match &*modules {
            PreparedModuleCompilation::Embedded { graph, .. } => {
                session.lower_complete_catalog(graph, options.host_surface_policy, prelude)
            }
            PreparedModuleCompilation::Ambient { graph, .. } => match (source, graph, prelude) {
                (_, Some(graph), prelude) => {
                    session.lower_loaded_graph(graph, options.host_surface_policy, prelude)
                }
                (ParsedSource::Module(source), None, prelude) => session.lower_loaded_graph(
                    &lila_ir::ModuleGraphSources::single(source),
                    options.host_surface_policy,
                    prelude,
                ),
                (ParsedSource::Script(_), None, None) => {
                    session.lower_source(source, options.host_surface_policy)
                }
                (ParsedSource::Script(_), None, Some(_)) => {
                    unreachable!("preparation rejects a Module prelude for a Script entry")
                }
            },
        };
        let PreparedModuleCompilation::Ambient { policy, graph } = modules else {
            // Complete catalogs have their exact Realm-origin resolutions
            // already. Finite strings cannot grant ambient loading authority.
            return Ok(ir);
        };
        let requests = session
            .realm_module_requests()
            .into_iter()
            .filter(|request| {
                graph
                    .as_ref()
                    .is_none_or(|graph| !graph.realm_requests.contains_key(request))
            })
            .collect::<Vec<_>>();
        if requests.is_empty() {
            return Ok(ir);
        }
        let Some(loader) = configured_module_loader(options, *policy) else {
            // Source candidates never grant loading authority or force their
            // speculative call to execute. Without a host loader the actual
            // native import owns asynchronous rejection through its existing
            // no-catalog path; an ordinary user method remains an ordinary call.
            return Ok(ir);
        };
        if graph.is_none() {
            let locator = options.filename.as_deref().unwrap_or("<entry>");
            let loaded = match source {
                ParsedSource::Script(source) => {
                    module_loader::load_module_graph_from_parsed_script(
                        locator,
                        source.clone(),
                        loader.as_ref(),
                    )
                }
                ParsedSource::Module(source) => module_loader::load_module_graph_from_parsed(
                    locator,
                    source.clone(),
                    loader.as_ref(),
                ),
            }
            .map_err(|error| EngineError::new(error.to_string()))?;
            *graph = Some(loaded);
        }
        let graph = graph
            .as_mut()
            .expect("prepared request has its retained graph");
        module_loader::extend_module_graph_realm_requests(graph, &requests, loader.as_ref())
            .map_err(|error| EngineError::new(error.to_string()))?;
        if requests
            .iter()
            .any(|request| !graph.realm_requests.contains_key(request))
        {
            return Err(EngineError::new(
                "module loader omitted a prepared Realm-request resolution",
            ));
        }
        // Newly loaded modules can themselves contain finite prepared source.
        // Reuse all old parse products while closing those requests as well.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct ModuleFiles(PathBuf);

    impl ModuleFiles {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "lila-prepared-module-catalog-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn write(&self, name: &str, source: &str) {
            std::fs::write(self.0.join(name), source).unwrap();
        }

        fn options(&self) -> CompileOptions {
            CompileOptions {
                filename: Some(self.0.join("entry.js").to_str().unwrap().into()),
                module_root: Some(self.0.to_str().unwrap().into()),
                ..CompileOptions::default()
            }
        }
    }

    impl Drop for ModuleFiles {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn prepared_requests_close_before_artifact_identity_and_keep_the_final_ir() {
        let files = ModuleFiles::new();
        let source = r#"
new ShadowRealm().evaluate('void new ShadowRealm().importValue("./first.js", "value")');
Function('void new ShadowRealm().importValue("./function.js", "value")')();
new ShadowRealm().evaluate('new.target');
true;
"#;
        files.write("entry.js", source);
        files.write(
            "first.js",
            r#"
new ShadowRealm().evaluate('void new ShadowRealm().importValue("./second.js", "value")');
export const value = 1;
"#,
        );
        files.write("second.js", "export const value = 2;");
        files.write("function.js", "export const value = 3;");
        let options = files.options();
        let engine = Engine::new(RealmBuilder::new().build());
        let prepared = engine
            .prepare_compilation(source, ParseGoal::Script, &options)
            .unwrap();
        let graph = prepared
            .modules
            .graph()
            .expect("finite sources own a module catalog");
        for name in ["./first.js", "./second.js", "./function.js"] {
            assert!(
                matches!(
                    graph
                        .realm_requests
                        .get(&lila_ir::ModuleRequestKeyIr::plain(name)),
                    Some(lila_ir::RealmModuleResolutionIr::Loaded(_))
                ),
                "{name}"
            );
        }
        assert_eq!(graph.modules.len(), 4);
        assert!(prepared
            .ir
            .script
            .as_ref()
            .unwrap()
            .prepared_scripts
            .iter()
            .any(|script| {
                script.kind == lila_ir::PreparedScriptKind::ShadowRealmEvaluate
                    && script.source == "new.target"
                    && matches!(
                        script.outcome,
                        lila_ir::PreparedScriptOutcome::DeferredSyntaxError { .. }
                    )
            }));
        let first_key = program_cache_key(source, ParseGoal::Script, &options, Some(graph));
        let final_ir = prepared.ir.clone();
        let unit = engine.compile_prepared_on_current_thread(prepared).unwrap();
        assert_eq!(unit.ir, final_ir);

        files.write("second.js", "export const value = 20;");
        let changed = engine
            .prepare_compilation(source, ParseGoal::Script, &options)
            .unwrap();
        let changed_key =
            program_cache_key(source, ParseGoal::Script, &options, changed.modules.graph());
        assert_ne!(first_key, changed_key);
    }

    #[test]
    fn prepared_requests_keep_reject_all_authority_and_hash_rejections() {
        let files = ModuleFiles::new();
        let source = r#"new ShadowRealm().evaluate('void new ShadowRealm().importValue("./present.js", "value")');"#;
        files.write("entry.js", source);
        files.write("present.js", "export const value = 42;");
        let mut options = files.options();
        options.module_loading_policy = ModuleLoadingPolicy::RejectAll;
        let engine = Engine::new(RealmBuilder::new().build());
        let prepared = engine
            .prepare_compilation(source, ParseGoal::Script, &options)
            .unwrap();
        let graph = prepared.modules.graph().unwrap();
        let request = lila_ir::ModuleRequestKeyIr::plain("./present.js");
        assert!(matches!(
            graph.realm_requests.get(&request),
            Some(lila_ir::RealmModuleResolutionIr::Rejected(_))
        ));
        assert_eq!(graph.modules.len(), 1);
        let mut changed = graph.clone();
        changed.realm_requests.insert(
            request,
            lila_ir::RealmModuleResolutionIr::Rejected("different host failure".into()),
        );
        assert_ne!(module_graph_digest(graph), module_graph_digest(&changed));
    }
}
