use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn generator_object_patterns_preserve_keys_references_defaults_rest_and_completions() {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            "{directive}{}",
            include_str!("fixtures/generator_object_patterns/object_patterns.js")
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
            .expect("actual suspended object pattern executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(
                "generator-object-patterns:ok".into()
            )]
        );
    }
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            include_str!("fixtures/generator_object_patterns/with_references.js"),
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
        .expect("actual retained Object/Global References through pattern defaults");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine(
            "generator-object-pattern-with:ok".into()
        )]
    );
}
