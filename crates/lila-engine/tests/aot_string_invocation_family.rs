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
            .expect("String invocation fixture compiles and executes through Wasm AOT");
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
fn string_family_preserves_transferred_references_and_full_arguments() {
    assert_modes(
        include_str!("fixtures/string_invocation_family/references_and_arguments.js"),
        "string-invocation-references:ok",
    );
}

#[test]
fn string_family_retains_arbitrary_hook_results_and_coercion_effects() {
    assert_modes(
        include_str!("fixtures/string_invocation_family/hook_results_and_effects.js"),
        "string-invocation-hooks:ok",
    );
}

#[test]
fn string_family_preserves_original_abrupt_values_and_called_realms() {
    assert_modes(
        include_str!("fixtures/string_invocation_family/realms_and_abrupt.js"),
        "string-invocation-realms:ok",
    );
}
