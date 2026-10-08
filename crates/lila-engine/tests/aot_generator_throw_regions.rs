use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn generator_throw_regions_preserve_operand_order_gc_close_and_whole_completions() {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            "{directive}{}",
            include_str!("fixtures/generator_throw_regions/throws.js")
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
            .expect("complete Throw operands compile and execute through actual Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(outcome.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            outcome.completion
        );
        assert_eq!(
            outcome.output_events,
            vec![HostOutputEvent::PrintLine(
                "generator-throw-regions:ok".into()
            )]
        );
    }
}
