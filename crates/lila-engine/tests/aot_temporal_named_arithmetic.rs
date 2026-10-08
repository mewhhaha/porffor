//! Zoned calendar arithmetic and exact elapsed time through compiled Wasm.
use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_arithmetic(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
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
                    timeout_ms: Some(120_000),
                    ..RunOptions::default()
                },
            )
            .expect("zoned arithmetic must execute through Wasm AOT");
        assert_eq!(
            observation.backend_used,
            ExecutionBackend::WasmAot,
            "{script}"
        );
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
fn calendar_days_and_elapsed_time_cross_hour_and_half_hour_transitions() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/calendar_and_elapsed.js"
    ));
}

#[test]
fn zero_date_duration_preserves_the_later_fold_and_signed_fraction() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/zero_and_later_fold.js"
    ));
}

#[test]
fn calendar_addition_retains_the_calendar_and_applies_overflow() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/calendar_and_overflow.js"
    ));
}

#[test]
fn addition_reads_duration_before_options_and_uses_branded_slots() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/add_observable_order.js"
    ));
}

#[test]
fn differences_keep_calendar_days_and_round_exact_dst_midpoints() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/difference_across_dst.js"
    ));
}

#[test]
fn differences_validate_calendar_and_primary_zone_before_zero() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/difference_guards_and_order.js"
    ));
}

#[test]
fn skipped_dates_use_bounded_compatible_difference_probes() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/skipped_date_corrections.js"
    ));
}

#[test]
fn epoch_arithmetic_checks_range_and_equal_epochs_skip_contextual_windows() {
    assert_arithmetic(include_str!(
        "fixtures/temporal_named_arithmetic/exact_range_and_zero.js"
    ));
}
