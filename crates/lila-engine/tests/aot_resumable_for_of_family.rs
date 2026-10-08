use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for prefix in ["", "\"use strict\";\n"] {
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{prefix}{source}"),
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
            .expect("compiled complete iterator family");
        assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(result.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            result.completion
        );
        assert_eq!(
            result.output_events,
            [HostOutputEvent::PrintLine("ok".into())]
        );
    }
}
#[test]
fn ordinary_generator_retains_head_target_and_unadopted_received_value() {
    observe(include_str!(
        "fixtures/resumable_for_of_family/generator.js"
    ));
}
#[test]
fn plain_async_iterator_owns_nested_next_initialization_body_and_close() {
    observe(include_str!("fixtures/resumable_for_of_family/async.js"));
}
#[test]
fn per_key_resources_dispose_before_advance_and_outer_iterator_close() {
    observe(include_str!(
        "fixtures/resumable_for_of_family/resources.js"
    ));
}
