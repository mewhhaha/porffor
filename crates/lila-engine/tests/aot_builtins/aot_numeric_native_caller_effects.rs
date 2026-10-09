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
                    can_block: true,
                    ..RunOptions::default()
                },
            )
            .expect("numeric caller effects fixture compiles and executes through Wasm AOT");
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
fn numeric_conversions_and_formatters_observe_hooks_and_retained_operands() {
    assert_modes(
        include_str!("../fixtures/numeric_native_caller_effects/conversions_and_formatting.js"),
        "numeric-conversions:ok",
    );
}

#[test]
fn math_iteration_and_atomics_preparation_invalidate_captured_facts() {
    assert_modes(
        include_str!("../fixtures/numeric_native_caller_effects/math_and_atomics.js"),
        "numeric-math-atomics:ok",
    );
}

#[test]
fn numeric_abrupt_cutoffs_and_native_errors_preserve_called_realms() {
    assert_modes(
        include_str!("../fixtures/numeric_native_caller_effects/abrupt_and_realms.js"),
        "numeric-abrupt-realms:ok",
    );
}
