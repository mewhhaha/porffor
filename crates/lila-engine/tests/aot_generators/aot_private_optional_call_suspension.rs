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
            .expect("private optional suspension executes through Wasm AOT");
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
fn awaited_private_optional_references_keep_effect_order_lazy_arguments_and_live_roots() {
    assert_modes(
        include_str!("../fixtures/private_optional_suspension/awaited.js"),
        "private-optional-await:ok",
    );
}

#[test]
fn yielded_private_optional_references_keep_receiver_and_whole_abrupt_completion() {
    assert_modes(
        include_str!("../fixtures/private_optional_suspension/yielded.js"),
        "private-optional-yield:ok",
    );
}

#[test]
fn grouped_private_optional_calls_publish_the_actual_terminal_reference_or_value() {
    assert_modes(
        include_str!("../fixtures/private_optional_suspension/grouped.js"),
        "private-optional-grouped:ok",
    );
}
