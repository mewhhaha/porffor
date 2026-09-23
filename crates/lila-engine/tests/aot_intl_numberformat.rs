use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_number_format_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("NumberFormat must compile and execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine(expected.to_owned())],
        "{source}"
    );
}

#[test]
fn intrinsic_family() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/01-intrinsic-family.js"),
        "ok intrinsic family",
    );
}

#[test]
fn locale_delegate_options() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/02-locale-delegate-options.js"),
        "ok locale delegates",
    );
}

#[test]
fn constructor_observation() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/03-constructor-observation.js"),
        "ok constructor observation",
    );
}

#[test]
fn validated_options_and_bound_format() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/04-validated-options-and-bound-format.js"),
        "ok options and bound format",
    );
}

#[test]
fn exact_mathematical_values() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/05-exact-mathematical-values.js"),
        "ok exact mathematical values",
    );
}

#[test]
fn rounding_and_parts() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/06-rounding-and-parts.js"),
        "ok rounding and parts",
    );
}

#[test]
fn range_observation() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/07-range-observation.js"),
        "ok range observation",
    );
}

#[test]
fn locale_style_data() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/08-locale-style-data.js"),
        "ok locale style data",
    );
}

#[test]
fn newtarget_observation() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/09-newtarget-observation.js"),
        "ok newTarget observation",
    );
}

#[test]
fn digit_coercion_boundary() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/10-digit-coercion-boundary.js"),
        "ok digit coercion boundary",
    );
}

#[test]
fn range_abrupt_order() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/11-range-abrupt-order.js"),
        "ok range abrupt order",
    );
}

#[test]
fn supported_options_and_ordinary_construction() {
    assert_number_format_script(
        include_str!(
            "fixtures/intl-numberformat/12-supported-options-and-ordinary-construction.js"
        ),
        "ok supported options and ordinary construction",
    );
}

#[test]
fn parts_descriptors_and_huge_inputs() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/13-parts-descriptors-and-huge-inputs.js"),
        "ok parts descriptors and huge inputs",
    );
}

#[test]
fn called_function_realm() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/14-called-function-realm.js"),
        "ok called-function Realm",
    );
}

#[test]
fn resolved_option_order_and_shared_currency_defaults() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/15-resolved-option-order.js"),
        "ok resolved option order",
    );
}

#[test]
fn primitive_options_abrupt_identity_and_utf16_transport() {
    assert_number_format_script(
        include_str!("fixtures/intl-numberformat/16-primitive-options-and-abrupts.js"),
        "ok primitive options and abrupts",
    );
}
