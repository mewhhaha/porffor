use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_relative_bag(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "\"use strict\";\n"] {
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
                    timeout_ms: Some(120_000),
                    ..RunOptions::default()
                },
            )
            .expect("relativeTo bag must execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
    }
}

#[test]
fn relative_bag_reads_once_in_calendar_sensitive_alphabetical_order() {
    assert_relative_bag(include_str!("../fixtures/temporal_relative_bag/order.js"));
}

#[test]
fn relative_bag_preserves_era_calendar_zone_and_exact_fold_origin() {
    assert_relative_bag(include_str!(
        "../fixtures/temporal_relative_bag/era_and_dst.js"
    ));
}

#[test]
fn relative_bag_offset_and_zone_abrupt_completions_stop_later_reads() {
    assert_relative_bag(include_str!(
        "../fixtures/temporal_relative_bag/abrupt_offset.js"
    ));
}
