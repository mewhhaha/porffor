use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_instant_methods(source: &str) {
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
        .expect("Instant methods must compile and execute through Wasm AOT");
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
fn metadata_and_intrinsic_results() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/metadata_and_intrinsic_results.js"
    ));
}

#[test]
fn exact_arithmetic_and_representation_boundaries() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/exact_arithmetic_and_representation_boundaries.js"
    ));
}

#[test]
fn duration_conversion_order_and_branding() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/duration_conversion_order_and_branding.js"
    ));
}

#[test]
fn epoch_limits_and_one_nanosecond_overflow() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/epoch_limits_and_one_nanosecond_overflow.js"
    ));
}

#[test]
fn wide_duration_fields_are_canonical() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/wide_duration_fields_are_canonical.js"
    ));
}

#[test]
fn round_as_if_positive_and_global_half_even() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/round_as_if_positive_and_global_half_even.js"
    ));
}

#[test]
fn round_options_and_full_day_increment() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/round_options_and_full_day_increment.js"
    ));
}

#[test]
fn differences_balance_exact_seconds_and_subseconds() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/differences_balance_exact_seconds_and_subseconds.js"
    ));
}

#[test]
fn difference_rounding_and_since_inversion() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/difference_rounding_and_since_inversion.js"
    ));
}

#[test]
fn difference_option_order_and_canonical_conversion() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/difference_option_order_and_canonical_conversion.js"
    ));
}

#[test]
fn difference_duration_number_precision() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/difference_duration_number_precision.js"
    ));
}

#[test]
fn borrowed_methods_keep_their_intrinsic_realm() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/borrowed_methods_keep_their_intrinsic_realm.js"
    ));
}

#[test]
fn arguments_primitive_conversion() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/arguments_primitive_conversion.js"
    ));
}

#[test]
fn non_string_primitive_results() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/non_string_primitive_results.js"
    ));
}

#[test]
fn primitive_abrupt_identity_and_order() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/primitive_abrupt_identity_and_order.js"
    ));
}

#[test]
fn primitive_conversion_borrowed_realms() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/primitive_conversion_borrowed_realms.js"
    ));
}

#[test]
fn difference_unit_category_after_option_reads() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/difference_unit_category_after_option_reads.js"
    ));
}

#[test]
fn difference_option_abrupt_precedence() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/difference_option_abrupt_precedence.js"
    ));
}

#[test]
fn difference_option_snapshots_and_error_realm() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/difference_option_snapshots_and_error_realm.js"
    ));
}

#[test]
fn locale_metadata_and_branding() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/locale_metadata_and_branding.js"
    ));
}

#[test]
fn locale_intrinsic_defaults_and_exact_inputs() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/locale_intrinsic_defaults_and_exact_inputs.js"
    ));
}

#[test]
fn locale_options_and_abrupt_order() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/locale_options_and_abrupt_order.js"
    ));
}

#[test]
fn locale_called_function_realms() {
    assert_instant_methods(include_str!(
        "fixtures/temporal_instant_methods/locale_called_function_realms.js"
    ));
}
