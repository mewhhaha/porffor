use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn suspended_pattern_initializers_preserve_tdz_get_order_iterator_close_and_abrupt_values() {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            "{directive}{}",
            include_str!("fixtures/generator_pattern_initializers/initialization.js")
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
            .expect("actual Wasm AOT pattern initializer");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(
                "generator-pattern-initializers:ok".into()
            )]
        );
    }
}
