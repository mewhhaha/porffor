use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
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
            .expect("grouped optional awaited References and completed Call values execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn grouped_calls_keep_original_function_receiver_and_ordered_outer_arguments() {
    assert_modes(
        include_str!("../fixtures/grouped_optional_reference_await/calls.js"),
        "grouped-optional-calls:ok",
    );
}

#[test]
fn grouped_outer_calls_evaluate_nullish_arguments_and_preserve_abrupt_identity() {
    assert_modes(
        include_str!("../fixtures/grouped_optional_reference_await/abrupt.js"),
        "grouped-optional-abrupt:ok",
    );
}

#[test]
fn grouped_tags_keep_original_receiver_and_cached_frozen_template_objects() {
    assert_modes(
        include_str!("../fixtures/grouped_optional_reference_await/templates.js"),
        "grouped-optional-templates:ok",
    );
}
