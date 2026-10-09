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
                panic!("Umm al-Qura calendar control failed: {error}\n{source}")
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
fn umalqura_projection_preserves_full_table_signed_eras_and_civil_edges() {
    assert_modes(
        include_str!("../fixtures/temporal_umalqura_calendar/projection.js"),
        "umalqura-projection:ok",
    );
}

#[test]
fn umalqura_fields_preserve_suitability_regulation_and_hook_order() {
    assert_modes(
        include_str!("../fixtures/temporal_umalqura_calendar/fields.js"),
        "umalqura-fields:ok",
    );
}

#[test]
fn umalqura_arithmetic_retains_table_lengths_and_virtual_anchors() {
    assert_modes(
        include_str!("../fixtures/temporal_umalqura_calendar/arithmetic.js"),
        "umalqura-arithmetic:ok",
    );
}

#[test]
fn umalqura_partial_dates_complete_all_twelve_latest_day30_references() {
    assert_modes(
        include_str!("../fixtures/temporal_umalqura_calendar/partial_dates.js"),
        "umalqura-partial-dates:ok",
    );
}

#[test]
fn umalqura_relative_duration_uses_plain_and_zoned_calendar_owners() {
    assert_modes(
        include_str!("../fixtures/temporal_umalqura_calendar/relative_duration.js"),
        "umalqura-relative-duration:ok",
    );
}

#[test]
fn umalqura_limits_and_errors_use_carrier_policy_and_called_realm() {
    assert_modes(
        include_str!("../fixtures/temporal_umalqura_calendar/limits_and_realms.js"),
        "umalqura-limits-and-realms:ok",
    );
}
