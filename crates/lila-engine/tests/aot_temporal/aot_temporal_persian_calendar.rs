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
            .unwrap_or_else(|error| panic!("Persian calendar control failed: {error}\n{source}"));
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
fn persian_projection_preserves_signed_years_and_the_pinned_corrections() {
    assert_modes(
        include_str!("../fixtures/temporal_persian_calendar/projection.js"),
        "persian-projection:ok",
    );
}

#[test]
fn persian_fields_regulate_before_conversion_and_retain_hook_order() {
    assert_modes(
        include_str!("../fixtures/temporal_persian_calendar/fields.js"),
        "persian-fields:ok",
    );
}

#[test]
fn persian_arithmetic_uses_calendar_months_and_virtual_anchors() {
    assert_modes(
        include_str!("../fixtures/temporal_persian_calendar/arithmetic.js"),
        "persian-arithmetic:ok",
    );
}

#[test]
fn persian_partial_dates_use_calendar_reference_dates() {
    assert_modes(
        include_str!("../fixtures/temporal_persian_calendar/partial_dates.js"),
        "persian-partial-dates:ok",
    );
}

#[test]
fn persian_relative_duration_preserves_plain_and_zoned_calendar_owners() {
    assert_modes(
        include_str!("../fixtures/temporal_persian_calendar/relative_duration.js"),
        "persian-relative-duration:ok",
    );
}

#[test]
fn persian_limits_and_errors_follow_the_completed_carrier_and_called_realm() {
    assert_modes(
        include_str!("../fixtures/temporal_persian_calendar/limits_and_realms.js"),
        "persian-limits-and-realms:ok",
    );
}
