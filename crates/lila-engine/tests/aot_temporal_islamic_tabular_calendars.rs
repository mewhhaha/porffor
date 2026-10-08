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
                panic!("Islamic tabular calendar control failed: {error}\n{source}")
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
fn islamic_projection_preserves_epochs_signed_eras_and_exact_year_boundaries() {
    assert_modes(
        include_str!("fixtures/temporal_islamic_tabular_calendars/projection.js"),
        "islamic-tabular-projection:ok",
    );
}

#[test]
fn islamic_fields_preserve_suitability_regulation_and_hook_order() {
    assert_modes(
        include_str!("fixtures/temporal_islamic_tabular_calendars/fields.js"),
        "islamic-tabular-fields:ok",
    );
}

#[test]
fn islamic_arithmetic_retains_lunar_lengths_and_virtual_anchors() {
    assert_modes(
        include_str!("fixtures/temporal_islamic_tabular_calendars/arithmetic.js"),
        "islamic-tabular-arithmetic:ok",
    );
}

#[test]
fn islamic_partial_dates_complete_their_latest_calendar_references() {
    assert_modes(
        include_str!("fixtures/temporal_islamic_tabular_calendars/partial_dates.js"),
        "islamic-tabular-partial-dates:ok",
    );
}

#[test]
fn islamic_relative_duration_uses_plain_and_zoned_calendar_owners() {
    assert_modes(
        include_str!("fixtures/temporal_islamic_tabular_calendars/relative_duration.js"),
        "islamic-tabular-relative-duration:ok",
    );
}

#[test]
fn islamic_limits_and_errors_use_carrier_policy_and_called_realm() {
    assert_modes(
        include_str!("fixtures/temporal_islamic_tabular_calendars/limits_and_realms.js"),
        "islamic-tabular-limits-and-realms:ok",
    );
}
