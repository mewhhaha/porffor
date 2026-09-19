use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_duration_fields(source: &str) {
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
        .expect("canonical Duration fields must execute through Wasm AOT");
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
fn storage_and_sign() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/storage_and_sign.js"
    ));
}

#[test]
fn coercion_order() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/coercion_order.js"
    ));
}

#[test]
fn exact_wide_normalization() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/exact_wide_normalization.js"
    ));
}

#[test]
fn exact_balance_and_total() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/exact_balance_and_total.js"
    ));
}

#[test]
fn canonical_bounds() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/canonical_bounds.js"
    ));
}

#[test]
fn calendar_consumers() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/calendar_consumers.js"
    ));
}

#[test]
fn arguments_objects() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/arguments_objects.js"
    ));
}

#[test]
fn intrinsic_realms() {
    assert_duration_fields(include_str!(
        "fixtures/temporal_duration_wide_fields/intrinsic_realms.js"
    ));
}
