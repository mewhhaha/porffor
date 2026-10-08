use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_duration_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
            .expect("Duration control must compile and execute through Wasm AOT");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            observation.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{script}"
        );
    }
}

#[test]
fn constructor_order_and_strict_options() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/constructor_order_and_strict_options.js"),
        "ok constructor_order_and_strict_options",
    );
}

#[test]
fn effective_options_and_abrupt_order() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/effective_options_and_abrupt_order.js"),
        "ok effective_options_and_abrupt_order",
    );
}

#[test]
fn duration_field_sequence_and_coercions() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/duration_field_sequence_and_coercions.js"),
        "ok duration_field_sequence_and_coercions",
    );
}

#[test]
fn abrupt_integrality_stops_next_get() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/abrupt_integrality_stops_next_get.js"),
        "ok abrupt_integrality_stops_next_get",
    );
}

#[test]
fn completed_aggregate_sign_and_bounds() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/completed_aggregate_sign_and_bounds.js"),
        "ok completed_aggregate_sign_and_bounds",
    );
}

#[test]
fn stored_temporal_duration_and_strings() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/stored_temporal_duration_and_strings.js"),
        "ok stored_temporal_duration_and_strings",
    );
}

#[test]
fn native_number_list_parts_and_units() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/native_number_list_parts_and_units.js"),
        "ok native_number_list_parts_and_units",
    );
}

#[test]
fn sign_zero_and_exact_fractions() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/sign_zero_and_exact_fractions.js"),
        "ok sign_zero_and_exact_fractions",
    );
}

#[test]
fn locale_numbering_and_serbian_data() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/locale_numbering_and_serbian_data.js"),
        "ok locale_numbering_and_serbian_data",
    );
}

#[test]
fn fresh_results_and_descriptors() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/fresh_results_and_descriptors.js"),
        "ok fresh_results_and_descriptors",
    );
}

#[test]
fn cross_realm_and_new_target() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/cross_realm_and_new_target.js"),
        "ok cross_realm_and_new_target",
    );
}

#[test]
fn zero_partition_and_numeric_display_bridge() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/zero_partition_and_numeric_display_bridge.js"),
        "ok zero_partition_and_numeric_display_bridge",
    );
}

#[test]
fn temporal_locale_string_uses_intrinsic_and_stored_fields() {
    assert_duration_script(
        include_str!("fixtures/intl_duration_format/temporal_locale_string_uses_intrinsic_and_stored_fields.js"),
        "ok temporal_locale_string_uses_intrinsic_and_stored_fields",
    );
}

#[test]
fn temporal_locale_string_brand_and_option_order() {
    assert_duration_script(
        include_str!(
            "fixtures/intl_duration_format/temporal_locale_string_brand_and_option_order.js"
        ),
        "ok temporal_locale_string_brand_and_option_order",
    );
}
