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
            .expect("ordinary method invocation compiles and executes through Wasm AOT");
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
fn direct_array_syntax_retains_actual_methods_results_and_effects() {
    assert_modes(
        include_str!("fixtures/invocation_shortcut_retirement/arrays.js"),
        "invocation-shortcut-arrays:ok",
    );
}

#[test]
fn string_range_aliases_retain_raw_receiver_arguments_and_coercion_order() {
    assert_modes(
        include_str!("fixtures/invocation_shortcut_retirement/strings.js"),
        "invocation-shortcut-strings:ok",
    );
}
