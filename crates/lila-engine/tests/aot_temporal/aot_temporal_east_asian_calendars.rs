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
                panic!("East Asian calendar control failed: {error}\n{source}")
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
fn east_asian_projection_preserves_observations_and_retained_model_joins() {
    assert_modes(
        include_str!("../fixtures/temporal_east_asian_calendars/projection.js"),
        "east-asian-projection:ok",
    );
}

#[test]
fn east_asian_fields_preserve_code_agreement_overflow_and_hook_order() {
    assert_modes(
        include_str!("../fixtures/temporal_east_asian_calendars/fields.js"),
        "east-asian-fields:ok",
    );
}

#[test]
fn east_asian_arithmetic_uses_real_serials_and_both_virtual_comparisons() {
    assert_modes(
        include_str!("../fixtures/temporal_east_asian_calendars/arithmetic.js"),
        "east-asian-arithmetic:ok",
    );
}

#[test]
fn east_asian_partial_dates_use_complete_reference_table_and_year_codes() {
    assert_modes(
        include_str!("../fixtures/temporal_east_asian_calendars/partial_dates.js"),
        "east-asian-partial-dates:ok",
    );
}

#[test]
fn east_asian_duration_only_source_keeps_plain_and_zoned_calendar_paths() {
    assert_modes(
        include_str!("../fixtures/temporal_east_asian_calendars/relative_duration.js"),
        "east-asian-relative-duration:ok",
    );
}

#[test]
fn east_asian_limits_and_errors_use_native_year_and_called_realm() {
    assert_modes(
        include_str!("../fixtures/temporal_east_asian_calendars/limits_and_realms.js"),
        "east-asian-limits-and-realms:ok",
    );
}
