use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn mixed_async_generator_with_preserves_original_records_references_and_completions() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for (source, marker) in [
        (
            include_str!("fixtures/async_generator_with/environments.js"),
            "mixed-async-generator-with-environments:ok",
        ),
        (
            include_str!("fixtures/async_generator_with/references_and_completions.js"),
            "mixed-async-generator-with-references:ok",
        ),
    ] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                source,
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
            .unwrap();
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine(marker.into())]
        );
    }
}
