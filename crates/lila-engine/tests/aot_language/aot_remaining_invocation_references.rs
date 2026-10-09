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
            .expect("remaining invocation fixture compiles and executes through Wasm AOT");
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
fn factories_and_iterator_next_preserve_acquired_references_and_actual_results() {
    assert_modes(
        include_str!("../fixtures/remaining_invocation_references/factories_and_iteration.js"),
        "remaining-invocation-factories:ok",
    );
}

#[test]
fn literal_native_calls_and_forwarding_retain_operands_raw_this_and_effects() {
    assert_modes(
        include_str!("../fixtures/remaining_invocation_references/literals_and_forwarding.js"),
        "remaining-invocation-forwarding:ok",
    );
}

#[test]
fn remaining_invocation_owners_preserve_abrupt_values_and_called_realms() {
    assert_modes(
        include_str!("../fixtures/remaining_invocation_references/abrupt_and_realms.js"),
        "remaining-invocation-abrupt:ok",
    );
}
