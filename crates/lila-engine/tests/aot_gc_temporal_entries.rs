use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_temporal_gc(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &script,
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
            .unwrap_or_else(|error| panic!("Temporal GC control must execute: {error}\n{script}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine("temporal-gc:ok".into())],
            "{script}"
        );
    }
}

#[test]
fn branded_records_and_duration_conversion_preserve_values_and_observable_order() {
    assert_temporal_gc(include_str!(
        "fixtures/temporal_gc/records_duration_order.js"
    ));
}

#[test]
fn negative_epochs_and_utf16_parsers_preserve_exact_values_and_abrupt_prefixes() {
    assert_temporal_gc(include_str!("fixtures/temporal_gc/instant_epoch_utf16.js"));
}

#[test]
fn zoned_provider_transitions_and_borrowed_realms_retain_their_owners() {
    assert_temporal_gc(include_str!(
        "fixtures/temporal_gc/zoned_transitions_realms.js"
    ));
}
