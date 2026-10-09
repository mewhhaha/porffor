use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_named_conversion(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
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
            .unwrap_or_else(|error| panic!("named conversion must execute: {error}\n{script}"));
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            observation.output_events,
            vec![HostOutputEvent::PrintLine("ok".into())],
            "{script}"
        );
    }
}

#[test]
fn instant_conversion_keeps_exact_epoch_identifier_and_iso_calendar() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/instant_epoch_identity_and_slots.js"
    ));
}

#[test]
fn plain_date_start_of_day_is_distinct_from_compatible_midnight() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/plain_date_start_of_day.js"
    ));
}

#[test]
fn plain_date_one_argument_preserves_conversion_order_and_calendar() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/plain_date_argument_order_and_calendar.js"
    ));
}

#[test]
fn plain_date_time_retains_all_disambiguation_policies_and_calendar() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/plain_date_time_disambiguation.js"
    ));
}

#[test]
fn plain_date_time_options_precede_epoch_validation_and_keep_abrupt_identity() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/plain_date_time_options_and_range.js"
    ));
}

#[test]
fn conversions_keep_half_hour_transitions_and_exact_instant_boundaries() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/half_hour_and_boundaries.js"
    ));
}

#[test]
fn instant_string_projects_rounded_epoch_and_preserves_option_error_order() {
    assert_named_conversion(include_str!(
        "../fixtures/temporal_named_conversions/instant_string_named_rounding_and_order.js"
    ));
}
