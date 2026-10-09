use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostClock, HostOutputEvent, HostSurfacePolicy,
    MonotonicClockInstant, ObservedCompletion, ObservedJsValue, ObservedNumber, ObservedRunOutcome,
    Realm, RealmBuilder, RunOptions, UtcEpochMilliseconds,
};
use lila_intl::ConfiguredSystemTimeZone;

const COLD_EXECUTION_TIMEOUT_MS: u64 = 120_000;

struct FixedClock(AtomicU64);

impl HostClock for FixedClock {
    fn utc_epoch_milliseconds(&self) -> UtcEpochMilliseconds {
        UtcEpochMilliseconds::new(1234).expect("fixed valid Date clock")
    }

    fn monotonic_instant(&self) -> MonotonicClockInstant {
        MonotonicClockInstant::new(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

fn configured_realm(identifier: &str) -> Realm {
    RealmBuilder::new()
        .with_system_time_zone(
            ConfiguredSystemTimeZone::resolve(identifier).expect("control zone must be admitted"),
        )
        .with_host_clock(Box::new(FixedClock(AtomicU64::new(0))))
        .build()
}

fn options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}

fn run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(COLD_EXECUTION_TIMEOUT_MS),
        ..RunOptions::default()
    }
}

fn assert_observation(observation: ObservedRunOutcome, output: &str, source: &str) {
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        observation.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
        "{source}",
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine(output.into())],
        "{source}",
    );
}

fn assert_date(source: &str, identifier: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    // Clones retain the admitted immutable choice and injected clock. Each
    // mode receives a fresh Wasm instance while the Rust Realm proof is shared.
    let realm = configured_realm(identifier);
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(realm.clone())
            .observe_script(&script, options(), run_options())
            .unwrap_or_else(|error| panic!("Date control must execute: {error}\n{script}"));
        assert_observation(observation, "ok", &script);
    }
}

#[test]
fn date_private_value_brand_and_direct_clone_ignore_public_hooks() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/private_brand_and_clone.js"),
        "UTC",
    );
}

#[test]
fn date_constructor_order_and_make_full_year_use_selected_zone() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/constructor_order_and_full_year.js"),
        "+01:00",
    );
}

#[test]
fn date_component_setters_capture_value_before_argument_coercion() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/captured_setter_value.js"),
        "+01:00",
    );
}

#[test]
fn date_invalid_setter_returns_and_recovery_preserve_spec_order() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/invalid_setter_recovery.js"),
        "+01:00",
    );
}

#[test]
fn date_utc_operations_are_independent_of_selected_named_zone() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/utc_isolation.js"),
        "America/New_York",
    );
}

#[test]
fn date_named_gap_fold_and_missing_zone_parse_choose_compatible_epoch() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/named_gap_and_fold.js"),
        "America/New_York",
    );
}

#[test]
fn date_half_hour_gap_and_fold_use_actual_transition_offsets() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/half_hour_transition.js"),
        "Australia/Lord_Howe",
    );
}

#[test]
fn date_historical_seconds_survive_projection_inverse_and_display() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/historical_second_offset.js"),
        "Europe/Paris",
    );
}

#[test]
fn date_quarter_hour_gap_preserves_real_displacement() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/quarter_hour_transition.js"),
        "Asia/Kathmandu",
    );
}

#[test]
fn date_positive_offset_selects_raw_epoch_before_time_clip() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/time_clip_positive_offset.js"),
        "+01:00",
    );
}

#[test]
fn date_negative_offset_selects_raw_epoch_before_time_clip() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/time_clip_negative_offset.js"),
        "-01:00",
    );
}

#[test]
fn date_owned_display_round_trips_and_rejects_malformed_suffix() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/display_round_trip_and_rejection.js"),
        "Europe/Paris",
    );
}

#[test]
fn date_now_intl_and_locale_methods_share_primary_default_zone() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/default_consumer_agreement.js"),
        "US/Eastern",
    );
}

#[test]
fn date_created_realm_shares_default_and_keeps_intrinsic_errors() {
    assert_date(
        include_str!("../fixtures/date_system_time_zone/created_realm_default_and_errors.js"),
        "US/Eastern",
    );
}

#[test]
fn date_worker_inherits_primary_zone_and_injected_clock() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = include_str!("../fixtures/date_system_time_zone/worker_inherits_default.js");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let outcome = Engine::new(configured_realm("US/Eastern"))
            .run_wasm_aot_script_with_agents(
                &script,
                options(),
                Some(COLD_EXECUTION_TIMEOUT_MS),
                true,
                String::new(),
            )
            .unwrap_or_else(|error| panic!("Date worker control must execute: {error}\n{script}"));
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(outcome.note, "wasm-aot completion: number(262)");
    }
}
