use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn generator_staged_selectors_and_compound_identifiers_preserve_values_references_and_abrupts() {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            "{directive}{}",
            include_str!("fixtures/generator_staged_operands/selectors_and_compounds.js")
        );
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("staged operands execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(
                "generator-staged-operands:ok".into()
            )],
            "{source}"
        );
    }
}
