use super::*;

#[test]
fn pinned_module_source_prototype_chain_matches_host_intrinsic_in_both_script_modes() {
    const BODY: &str = include_str!("../../../../test262/vendor/test262/test/staging/source-phase-imports/module-source-prototype-chain.js");
    const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
    const ASYNC: &str = include_str!("../../../../test262/vendor/test262/harness/asyncHelpers.js");
    lila_engine::configure_compilation_jobs(1).unwrap();
    for directive in ["", "\"use strict\";\n"] {
        let source = format!(
            "{directive}{STA}\n{ASSERT}\n{ASYNC}\nconst $262 = {{ AbstractModuleSource: __lilaGetAbstractModuleSource() }};\nfunction $DONE(error) {{ print(error === undefined ? 'source-prototype:ok' : 'source-prototype:fail'); }}\n{BODY}"
        );
        let graph = EmbeddedModuleGraph::try_new_typed(
            EmbeddedModuleEntryInput {
                goal: EmbeddedModuleGoal::Script,
                identity: "entry.js".into(),
                source: source.clone(),
                meta_url: "lila://entry.js".into(),
            },
            vec![EmbeddedModuleInput::ModuleSource(
                EmbeddedModuleSourceValueInput {
                    identity: "native/source".into(),
                    source_tag: "ModuleSource".into(),
                    meta_url: "lila://native/source".into(),
                },
            )],
            vec![EmbeddedModuleResolutionInput {
                referrer: EmbeddedModuleReferrer::Script("entry.js".into()),
                specifier: "<module source>".into(),
                attributes: Vec::new(),
                target: "native/source".into(),
            }],
        )
        .unwrap();
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    module_loading_policy: ModuleLoadingPolicy::Embedded(graph),
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap();
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "mode {directive:?}: {:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine("source-prototype:ok".into())],
            "mode {directive:?}"
        );
    }
}
