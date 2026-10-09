use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_relative_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
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
            .expect("RelativeTimeFormat control must use real Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{script}"
        );
    }
}

#[test]
fn metadata_and_resolved_options() {
    assert_relative_script(
        include_str!("../fixtures/intl_relative_time_format/metadata_and_resolved_options.js"),
        "ok metadata_and_resolved_options",
    );
}
#[test]
fn constructor_observation_and_short_circuits() {
    assert_relative_script(
        include_str!(
            "../fixtures/intl_relative_time_format/constructor_observation_and_short_circuits.js"
        ),
        "ok constructor_observation_and_short_circuits",
    );
}
#[test]
fn scalar_coercion_order_and_finite_validation() {
    assert_relative_script(
        include_str!(
            "../fixtures/intl_relative_time_format/scalar_coercion_order_and_finite_validation.js"
        ),
        "ok scalar_coercion_order_and_finite_validation",
    );
}
#[test]
fn eight_units_parts_rounding_and_signed_zero() {
    assert_relative_script(
        include_str!(
            "../fixtures/intl_relative_time_format/eight_units_parts_rounding_and_signed_zero.js"
        ),
        "ok eight_units_parts_rounding_and_signed_zero",
    );
}
#[test]
fn genuine_auto_style_and_numbering_data() {
    assert_relative_script(
        include_str!(
            "../fixtures/intl_relative_time_format/genuine_auto_style_and_numbering_data.js"
        ),
        "ok genuine_auto_style_and_numbering_data",
    );
}
#[test]
fn supported_locales_order_boxing_and_fresh_arrays() {
    assert_relative_script(
        include_str!(
            "../fixtures/intl_relative_time_format/supported_locales_order_boxing_and_fresh_arrays.js"
        ),
        "ok supported_locales_order_boxing_and_fresh_arrays",
    );
}
#[test]
fn receiver_brand_and_called_realm_outputs() {
    assert_relative_script(
        include_str!(
            "../fixtures/intl_relative_time_format/receiver_brand_and_called_realm_outputs.js"
        ),
        "ok receiver_brand_and_called_realm_outputs",
    );
}
#[test]
fn tagged_newtarget_prototypes_and_foreign_fallback() {
    assert_relative_script(include_str!("../fixtures/intl_relative_time_format/tagged_newtarget_prototypes_and_foreign_fallback.js"), "ok tagged_newtarget_prototypes_and_foreign_fallback");
}
