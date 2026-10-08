use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_relative_duration(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("relative duration operations must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn calendar_rounding_keeps_larger_fields_when_selected_unit_is_zero() {
    assert_relative_duration(include_str!(
        "fixtures/temporal_duration_relative/calendar_rounding.js"
    ));
}

#[test]
fn annotated_plain_dates_and_fixed_zones_resolve_calendar_totals_and_comparisons() {
    assert_relative_duration(include_str!(
        "fixtures/temporal_duration_relative/totals_and_comparisons.js"
    ));
}

#[test]
fn relative_conversion_preserves_option_order_slots_and_abrupt_identity() {
    assert_relative_duration(include_str!(
        "fixtures/temporal_duration_relative/observation.js"
    ));
}

#[test]
fn relative_arithmetic_preserves_nanoseconds_and_rejects_epoch_overflow() {
    assert_relative_duration(include_str!(
        "fixtures/temporal_duration_relative/nanoseconds_and_limits.js"
    ));
}
