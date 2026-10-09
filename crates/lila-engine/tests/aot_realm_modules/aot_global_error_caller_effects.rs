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
            .expect("Global/Error effects fixture compiles and executes through Wasm AOT");
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
fn uri_and_annex_b_globals_observe_coercion_and_invalidate_caller_facts() {
    assert_modes(
        include_str!("../fixtures/global_error_caller_effects/globals_and_coercion.js"),
        "global-error-globals:ok",
    );
}

#[test]
fn error_families_observe_their_actual_prefix_and_retained_name_order() {
    assert_modes(
        include_str!("../fixtures/global_error_caller_effects/errors_and_ordering.js"),
        "global-error-errors:ok",
    );
}

#[test]
fn global_and_error_effects_preserve_abrupt_identity_and_called_realms() {
    assert_modes(
        include_str!("../fixtures/global_error_caller_effects/abrupt_and_realms.js"),
        "global-error-abrupt:ok",
    );
}
