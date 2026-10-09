use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn suspended_for_in_initialization_preserves_cursor_reference_and_per_key_environments() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let source = include_str!("../fixtures/async_generator_for_in/initialization_lifecycle.js");
    for source in [source.to_owned(), format!("'use strict';\n{source}")] {
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
            .unwrap();
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine(
                "mixed-for-in-initialization:ok".into()
            )]
        );
    }
}
