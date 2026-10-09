use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn mixed_catch_patterns_keep_original_records_close_and_whole_completions() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for strict in [false, true] {
        let source = include_str!("../fixtures/async_generator_patterns/catch.js");
        let source = if strict {
            format!("'use strict';\n{source}")
        } else {
            source.to_string()
        };
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
            "strict={strict}: {:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine(
                "mixed-async-generator-catch-patterns:ok".into()
            )]
        );
    }
}
