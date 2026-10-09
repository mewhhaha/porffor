use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
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
            .expect("indexed-collection invocation compiles and executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.to_owned())],
            "{source}"
        );
    }
}

#[test]
fn indexed_collection_aliases_retain_reference_and_real_arguments() {
    assert_modes(
        include_str!("../fixtures/indexed_collection_invocation/references_and_arguments.js"),
        "indexed-collection-reference:ok",
    );
}

#[test]
fn indexed_collection_results_and_effects_remain_runtime_truthful() {
    assert_modes(
        include_str!("../fixtures/indexed_collection_invocation/results_and_effects.js"),
        "indexed-collection-results:ok",
    );
}

#[test]
fn indexed_collection_calls_preserve_realm_and_original_abrupt_completion() {
    assert_modes(
        include_str!("../fixtures/indexed_collection_invocation/realms_and_abrupt.js"),
        "indexed-collection-abrupt:ok",
    );
}
