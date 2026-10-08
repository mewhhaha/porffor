use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_host_modes(source: &str, expected: &[&str]) {
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
            .expect("finite native host fixture executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).into()))
                .collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn numeric_hosts_consume_exact_utf16_prefixes_and_ordered_whole_arguments() {
    assert_host_modes(
        include_str!("fixtures/native_host_values/numeric_utf16.js"),
        &["native-host-numeric:ok"],
    );
}
#[test]
fn native_host_coercions_preserve_mutations_abrupt_identity_and_cutoffs() {
    assert_host_modes(
        include_str!("fixtures/native_host_values/coercion_and_abrupt.js"),
        &["native-host-print true", "native-host-coercion:ok"],
    );
}
#[test]
fn created_realms_html_dda_and_assert_throws_retain_gc_identity() {
    assert_host_modes(
        include_str!("fixtures/native_host_values/created_realms_and_identity.js"),
        &["native-host-realms:ok"],
    );
}
