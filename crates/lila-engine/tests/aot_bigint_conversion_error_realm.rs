use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
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
            .expect("BigInt conversion error Realms execute through Wasm AOT");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observation.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observation.completion
        );
        assert_eq!(
            observation.output_events,
            vec![HostOutputEvent::PrintLine(expected.to_owned())],
            "{source}"
        );
    }
}

#[test]
fn borrowed_bigint_uses_intrinsic_error_realms_after_primitive_conversion() {
    assert_trace(
        include_str!("fixtures/bigint_conversion_error_realm/borrowed_bigint.js"),
        "bigint:ok",
    );
}

#[test]
fn borrowed_bigint_setters_keep_realm_order_and_abrupt_identity() {
    assert_trace(
        include_str!("fixtures/bigint_conversion_error_realm/borrowed_data_view.js"),
        "dataview:ok",
    );
}
