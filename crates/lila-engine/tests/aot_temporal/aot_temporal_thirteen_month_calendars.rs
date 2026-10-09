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
            .unwrap_or_else(|error| {
                panic!("Thirteen-month calendar control failed: {error}\n{source}")
            });
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
fn thirteen_month_projection_preserves_epochs_signed_eras_and_leap_edges() {
    assert_modes(
        include_str!("../fixtures/temporal_thirteen_month_calendars/projection.js"),
        "thirteen-month-projection:ok",
    );
}

#[test]
fn thirteen_month_fields_preserve_suitability_regulation_and_hook_order() {
    assert_modes(
        include_str!("../fixtures/temporal_thirteen_month_calendars/fields.js"),
        "thirteen-month-fields:ok",
    );
}

#[test]
fn thirteen_month_arithmetic_retains_calendar_counts_and_virtual_anchors() {
    assert_modes(
        include_str!("../fixtures/temporal_thirteen_month_calendars/arithmetic.js"),
        "thirteen-month-arithmetic:ok",
    );
}

#[test]
fn thirteen_month_partial_dates_complete_their_calendar_references() {
    assert_modes(
        include_str!("../fixtures/temporal_thirteen_month_calendars/partial_dates.js"),
        "thirteen-month-partial-dates:ok",
    );
}

#[test]
fn thirteen_month_relative_duration_uses_plain_and_zoned_calendar_owners() {
    assert_modes(
        include_str!("../fixtures/temporal_thirteen_month_calendars/relative_duration.js"),
        "thirteen-month-relative-duration:ok",
    );
}

#[test]
fn thirteen_month_limits_and_errors_use_carrier_policy_and_called_realm() {
    assert_modes(
        include_str!("../fixtures/temporal_thirteen_month_calendars/limits_and_realms.js"),
        "thirteen-month-limits-and-realms:ok",
    );
}
