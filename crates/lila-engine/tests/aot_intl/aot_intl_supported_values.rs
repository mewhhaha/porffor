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

fn assert_supported_values_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("Intl.supportedValuesOf control must compile and execute through Wasm AOT");
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

fn assert_supported_values_runtime_exception(source: &str, constructor: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(&script, compile_options(), run_options())
            .expect_err("Intl.supportedValuesOf negative must be an actual JavaScript exception");
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
fn metadata_and_coercion() {
    assert_supported_values_script(
        include_str!("../fixtures/intl_supported_values/metadata_and_coercion.js"),
        "ok metadata_and_coercion",
    );
}

#[test]
fn abrupt_and_reentrant_coercion() {
    assert_supported_values_script(
        include_str!("../fixtures/intl_supported_values/abrupt_and_reentrant_coercion.js"),
        "ok abrupt_and_reentrant_coercion",
    );
}

#[test]
fn six_keys_sorted_and_fresh() {
    assert_supported_values_script(
        include_str!("../fixtures/intl_supported_values/six_keys_sorted_and_fresh.js"),
        "ok six_keys_sorted_and_fresh",
    );
}

#[test]
fn called_function_realms() {
    assert_supported_values_script(
        include_str!("../fixtures/intl_supported_values/called_function_realms.js"),
        "ok called_function_realms",
    );
}

#[test]
fn uncaught_symbol_key_type_error() {
    assert_supported_values_runtime_exception("Intl.supportedValuesOf(Symbol());", "TypeError");
}

#[test]
fn uncaught_invalid_key_range_error() {
    assert_supported_values_runtime_exception("Intl.supportedValuesOf(\"invalid\");", "RangeError");
}
