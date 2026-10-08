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
            .unwrap_or_else(|error| panic!("Indian calendar control failed: {error}\n{source}"));
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
fn indian_projection_uses_the_complete_iso_date_on_every_carrier() {
    assert_modes(
        include_str!("fixtures/temporal_indian_calendar/projection.js"),
        "indian-projection:ok",
    );
}

#[test]
fn calendar_fields_regulate_before_iso_conversion_and_preserve_observation_order() {
    assert_modes(
        include_str!("fixtures/temporal_indian_calendar/fields.js"),
        "indian-fields:ok",
    );
}

#[test]
fn calendar_addition_and_difference_use_indian_months_and_virtual_anchors() {
    assert_modes(
        include_str!("fixtures/temporal_indian_calendar/arithmetic.js"),
        "indian-arithmetic:ok",
    );
}

#[test]
fn partial_dates_retain_calendar_reference_dates_through_all_conversions() {
    assert_modes(
        include_str!("fixtures/temporal_indian_calendar/partial_dates.js"),
        "indian-partial-dates:ok",
    );
}

#[test]
fn relative_duration_consumers_retain_plain_and_zoned_calendars() {
    assert_modes(
        include_str!("fixtures/temporal_indian_calendar/relative_duration.js"),
        "indian-relative-duration:ok",
    );
}

#[test]
fn calendar_fields_preserve_iso_limits_and_the_called_intrinsic_realm() {
    assert_modes(
        include_str!("fixtures/temporal_indian_calendar/limits_and_realms.js"),
        "indian-limits-and-realms:ok",
    );
}
