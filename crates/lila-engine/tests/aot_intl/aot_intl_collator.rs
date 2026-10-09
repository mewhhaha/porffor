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

fn assert_collator_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("Collator control must compile and execute through Wasm AOT");
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

fn assert_collator_runtime_exception(source: &str, constructor: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(&script, compile_options(), run_options())
            .expect_err("Collator negative must be an actual JavaScript exception");
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
    assert_collator_script(
        include_str!("../fixtures/intl_collator/metadata_and_resolved.js"),
        "ok metadata_and_resolved",
    );
}

#[test]
fn constructor_observation() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/constructor_observation.js"),
        "ok constructor_observation",
    );
}

#[test]
fn constructor_short_circuits() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/constructor_short_circuits.js"),
        "ok constructor_short_circuits",
    );
}

#[test]
fn option_boxing_and_supported() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/option_boxing_and_supported.js"),
        "ok option_boxing_and_supported",
    );
}

#[test]
fn brand_and_cache() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/brand_and_cache.js"),
        "ok brand_and_cache",
    );
}

#[test]
fn ordered_argument_conversion() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/ordered_argument_conversion.js"),
        "ok ordered_argument_conversion",
    );
}

#[test]
fn sensitivity_numeric_case() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/sensitivity_numeric_case.js"),
        "ok sensitivity_numeric_case",
    );
}

#[test]
fn search_and_data_defaults() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/search_and_data_defaults.js"),
        "ok search_and_data_defaults",
    );
}

#[test]
fn utf16_and_canonical() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/utf16_and_canonical.js"),
        "ok utf16_and_canonical",
    );
}

#[test]
fn locale_compare_observation() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/locale_compare_observation.js"),
        "ok locale_compare_observation",
    );
}

#[test]
fn locale_compare_taint() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/locale_compare_taint.js"),
        "ok locale_compare_taint",
    );
}

#[test]
fn called_function_realms() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/called_function_realms.js"),
        "ok called_function_realms",
    );
}

#[test]
fn tagged_newtarget_prototypes() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/tagged_newtarget_prototypes.js"),
        "ok tagged_newtarget_prototypes",
    );
}

#[test]
fn extension_option_resolution() {
    assert_collator_script(
        include_str!("../fixtures/intl_collator/extension_option_resolution.js"),
        "ok extension_option_resolution",
    );
}

#[test]
fn symbol_compare_is_an_actual_type_error() {
    assert_collator_runtime_exception(
        "new Intl.Collator('en-US').compare(Symbol('x'), 'a');",
        "TypeError",
    );
}

#[test]
fn malformed_collation_is_an_actual_range_error() {
    assert_collator_runtime_exception(
        "new Intl.Collator('en-US', {collation: 'a'});",
        "RangeError",
    );
}
