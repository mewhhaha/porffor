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

fn assert_display_names_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("DisplayNames control must compile and execute through Wasm AOT");
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

fn assert_display_names_runtime_exception(source: &str, constructor: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(&script, compile_options(), run_options())
            .expect_err("DisplayNames negative must be an actual JavaScript exception");
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
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/metadata_and_resolved.js"),
        "ok metadata_and_resolved",
    );
}

#[test]
fn constructor_observation() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/constructor_observation.js"),
        "ok constructor_observation",
    );
}

#[test]
fn constructor_short_circuits() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/constructor_short_circuits.js"),
        "ok constructor_short_circuits",
    );
}

#[test]
fn of_coercion_and_brand() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/of_coercion_and_brand.js"),
        "ok of_coercion_and_brand",
    );
}

#[test]
fn genuine_names_and_widths() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/genuine_names_and_widths.js"),
        "ok genuine_names_and_widths",
    );
}

#[test]
fn language_dialects_and_aliases() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/language_dialects_and_aliases.js"),
        "ok language_dialects_and_aliases",
    );
}

#[test]
fn canonical_codes_and_fallback() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/canonical_codes_and_fallback.js"),
        "ok canonical_codes_and_fallback",
    );
}

#[test]
fn invalid_code_domains() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/invalid_code_domains.js"),
        "ok invalid_code_domains",
    );
}

#[test]
fn supported_locale_order_and_boxing() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/supported_locale_order_and_boxing.js"),
        "ok supported_locale_order_and_boxing",
    );
}

#[test]
fn called_function_realms() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/called_function_realms.js"),
        "ok called_function_realms",
    );
}

#[test]
fn tagged_newtarget_prototypes() {
    assert_display_names_script(
        include_str!("../fixtures/intl_display_names/tagged_newtarget_prototypes.js"),
        "ok tagged_newtarget_prototypes",
    );
}

#[test]
fn missing_type_is_an_actual_type_error() {
    assert_display_names_runtime_exception("new Intl.DisplayNames('en-US', {});", "TypeError");
}

#[test]
fn invalid_type_is_an_actual_range_error() {
    assert_display_names_runtime_exception(
        "new Intl.DisplayNames('en-US', {type:'wrong'});",
        "RangeError",
    );
}

#[test]
fn invalid_code_is_an_actual_range_error() {
    assert_display_names_runtime_exception(
        "new Intl.DisplayNames('en-US', {type:'region'}).of('USA');",
        "RangeError",
    );
}
