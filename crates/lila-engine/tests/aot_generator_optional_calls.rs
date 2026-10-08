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
            .expect("generator optional calls execute through Wasm AOT");
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
fn generator_optional_calls_and_grouped_values_keep_references_and_order_across_each_yield() {
    assert_modes(
        include_str!("fixtures/generator_optional_calls/references_and_order.js"),
        "generator-optional-references:ok",
    );
}

#[test]
fn generator_optional_calls_and_grouped_values_keep_suffix_and_ordinary_outer_order() {
    assert_modes(
        include_str!("fixtures/generator_optional_calls/skipping_and_suffixes.js"),
        "generator-optional-suffixes:ok",
    );
}

#[test]
fn generator_optional_call_and_grouped_value_abrupt_resumes_keep_identity_realm_and_finally() {
    assert_modes(
        include_str!("fixtures/generator_optional_calls/abrupt_and_realms.js"),
        "generator-optional-abrupt:ok",
    );
}
