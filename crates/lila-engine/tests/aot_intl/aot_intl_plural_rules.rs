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

fn assert_plural_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("PluralRules control must compile and execute through Wasm AOT");
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

fn assert_plural_runtime_exception(source: &str, constructor: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(&script, compile_options(), run_options())
            .expect_err("PluralRules negative must be an actual JavaScript exception");
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
fn intrinsic_family() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/intrinsic_family.js"),
        "ok intrinsic_family",
    );
}

#[test]
fn constructor_observation() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/constructor_observation.js"),
        "ok constructor_observation",
    );
}

#[test]
fn digit_lifecycle() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/digit_lifecycle.js"),
        "ok digit_lifecycle",
    );
}

#[test]
fn scalar_categories() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/scalar_categories.js"),
        "ok scalar_categories",
    );
}

#[test]
fn exact_mathematical_values() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/exact_mathematical_values.js"),
        "ok exact_mathematical_values",
    );
}

#[test]
fn scalar_coercion_and_brand() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/scalar_coercion_and_brand.js"),
        "ok scalar_coercion_and_brand",
    );
}

#[test]
fn rounding_modes_and_visible_digits() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/rounding_modes_and_visible_digits.js"),
        "ok rounding_modes_and_visible_digits",
    );
}

#[test]
fn range_observation() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/range_observation.js"),
        "ok range_observation",
    );
}

#[test]
fn range_bare_identity() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/range_bare_identity.js"),
        "ok range_bare_identity",
    );
}

#[test]
fn notation_and_resolved_options() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/notation_and_resolved_options.js"),
        "ok notation_and_resolved_options",
    );
}

#[test]
fn locale_support_and_primitive_options() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/locale_support_and_primitive_options.js"),
        "ok locale_support_and_primitive_options",
    );
}

#[test]
fn called_function_and_constructor_realms() {
    assert_plural_script(
        include_str!("../fixtures/intl_plural_rules/called_function_and_constructor_realms.js"),
        "ok called_function_and_constructor_realms",
    );
}

#[test]
fn scalar_symbol_is_an_actual_type_error() {
    assert_plural_runtime_exception(
        "new Intl.PluralRules('en').select(Symbol('number'));",
        "TypeError",
    );
}

#[test]
fn range_nan_is_an_actual_range_error() {
    assert_plural_runtime_exception(
        "new Intl.PluralRules('en').selectRange(NaN, 1);",
        "RangeError",
    );
}
