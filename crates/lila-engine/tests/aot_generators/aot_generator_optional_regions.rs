use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn optional_regions_preserve_reference_order_gc_and_whole_abrupt_completions() {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            "{directive}{}",
            include_str!("../fixtures/generator_optional_regions/regions.js")
        );
        let outcome = Engine::new(RealmBuilder::new().build())
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
            .expect("actual optional regions execute through Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(outcome.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            outcome.completion
        );
        assert_eq!(
            outcome.output_events,
            vec![HostOutputEvent::PrintLine(
                "generator-optional-regions:ok".into()
            )]
        );
    }
}
