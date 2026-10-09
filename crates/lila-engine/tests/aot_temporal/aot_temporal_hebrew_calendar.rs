use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
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
            .unwrap_or_else(|error| panic!("Hebrew calendar control failed: {error}\n{source}"));
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
fn hebrew_projection_preserves_native_year_month_codes_and_all_year_lengths() {
    assert_modes(
        include_str!("../fixtures/temporal_hebrew_calendar/projection.js"),
        "hebrew-projection:ok",
    );
}

#[test]
fn hebrew_fields_preserve_suitability_original_code_overflow_and_hook_order() {
    assert_modes(
        include_str!("../fixtures/temporal_hebrew_calendar/fields.js"),
        "hebrew-fields:ok",
    );
}

#[test]
fn hebrew_arithmetic_preserves_year_code_month_serials_and_virtual_anchors() {
    assert_modes(
        include_str!("../fixtures/temporal_hebrew_calendar/arithmetic.js"),
        "hebrew-arithmetic:ok",
    );
}

#[test]
fn hebrew_partial_dates_resolve_codes_in_supplied_and_reference_years() {
    assert_modes(
        include_str!("../fixtures/temporal_hebrew_calendar/partial_dates.js"),
        "hebrew-partial-dates:ok",
    );
}

#[test]
fn hebrew_relative_duration_uses_plain_and_zoned_calendar_owners() {
    assert_modes(
        include_str!("../fixtures/temporal_hebrew_calendar/relative_duration.js"),
        "hebrew-relative-duration:ok",
    );
}

#[test]
fn hebrew_limits_and_errors_use_carrier_policy_and_called_realm() {
    assert_modes(
        include_str!("../fixtures/temporal_hebrew_calendar/limits_and_realms.js"),
        "hebrew-limits-and-realms:ok",
    );
}
