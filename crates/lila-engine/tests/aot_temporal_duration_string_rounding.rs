use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_duration_string_rounding(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("Duration rounding and totals must execute through Wasm AOT");
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("ok".into())],
        "{source}"
    );
}

#[test]
fn carries() {
    assert_duration_string_rounding(include_str!(
        "fixtures/temporal_duration_string_rounding/carries.js"
    ));
}

#[test]
fn limits_and_options() {
    assert_duration_string_rounding(include_str!(
        "fixtures/temporal_duration_string_rounding/limits_and_options.js"
    ));
}

#[test]
fn exact_hour_totals() {
    assert_duration_string_rounding(include_str!(
        "fixtures/temporal_duration_string_rounding/exact_hour_totals.js"
    ));
}
