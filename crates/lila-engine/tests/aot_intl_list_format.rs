use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
    WasmExecutionFailureKind,
};

fn compile_options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}

fn run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(120_000),
        ..RunOptions::default()
    }
}

fn assert_list_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("ListFormat control must compile and execute through Wasm AOT");
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

fn assert_list_runtime_exception(source: &str, constructor: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(&script, compile_options(), run_options())
            .expect_err("ListFormat negative must be an actual JavaScript exception");
        assert_eq!(
            error.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::JavaScriptException),
            "{error}\n{script}"
        );
        assert_eq!(
            error.wasm_javascript_exception_constructor_name(),
            Some(constructor),
            "{error}\n{script}"
        );
        assert!(
            error.runtime_semantic_gaps().is_empty(),
            "{error}\n{script}"
        );
        assert!(error.parse_diagnostic().is_none(), "{error}\n{script}");
        assert!(error.ir_diagnostic().is_none(), "{error}\n{script}");
    }
}

#[test]
fn metadata_and_resolved() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/metadata_and_resolved.js"),
        "ok metadata_and_resolved",
    );
}

#[test]
fn constructor_observation() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/constructor_observation.js"),
        "ok constructor_observation",
    );
}

#[test]
fn constructor_short_circuits() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/constructor_short_circuits.js"),
        "ok constructor_short_circuits",
    );
}

#[test]
fn supported_locale_order_and_boxing() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/supported_locale_order_and_boxing.js"),
        "ok supported_locale_order_and_boxing",
    );
}

#[test]
fn iterator_observation() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/iterator_observation.js"),
        "ok iterator_observation",
    );
}

#[test]
fn iterator_abrupts_do_not_close() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/iterator_abrupts_do_not_close.js"),
        "ok iterator_abrupts_do_not_close",
    );
}

#[test]
fn nonstring_close_precedence() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/nonstring_close_precedence.js"),
        "ok nonstring_close_precedence",
    );
}

#[test]
fn brand_before_iterable() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/brand_before_iterable.js"),
        "ok brand_before_iterable",
    );
}

#[test]
fn english_templates_and_parts() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/english_templates_and_parts.js"),
        "ok english_templates_and_parts",
    );
}

#[test]
fn utf16_and_empty_elements() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/utf16_and_empty_elements.js"),
        "ok utf16_and_empty_elements",
    );
}

#[test]
fn conditional_spanish_hebrew() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/conditional_spanish_hebrew.js"),
        "ok conditional_spanish_hebrew",
    );
}

#[test]
fn parts_property_identity() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/parts_property_identity.js"),
        "ok parts_property_identity",
    );
}

#[test]
fn called_function_realms() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/called_function_realms.js"),
        "ok called_function_realms",
    );
}

#[test]
fn tagged_newtarget_prototypes() {
    assert_list_script(
        include_str!("fixtures/intl_list_format/tagged_newtarget_prototypes.js"),
        "ok tagged_newtarget_prototypes",
    );
}

#[test]
fn nonstring_element_is_an_actual_type_error() {
    assert_list_runtime_exception("new Intl.ListFormat('en-US').format([1]);", "TypeError");
}

#[test]
fn invalid_style_is_an_actual_range_error() {
    assert_list_runtime_exception(
        "new Intl.ListFormat('en-US', { style: 'invalid' });",
        "RangeError",
    );
}
