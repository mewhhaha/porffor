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
            .expect("optional calls with staged awaits execute through Wasm AOT");
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
fn optional_calls_retain_original_callee_receiver_and_argument_order() {
    assert_modes(
        include_str!("fixtures/optional_call_await/references_and_order.js"),
        "optional-call-references:ok",
    );
}

#[test]
fn optional_calls_guard_the_complete_suffix_and_retain_each_call_result() {
    assert_modes(
        include_str!("fixtures/optional_call_await/skipping_and_suffixes.js"),
        "optional-call-suffixes:ok",
    );
}

#[test]
fn optional_call_abrupt_completions_preserve_realm_and_finally_order() {
    assert_modes(
        include_str!("fixtures/optional_call_await/abrupt_and_realms.js"),
        "optional-call-abrupt:ok",
    );
}
