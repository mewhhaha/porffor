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

fn assert_segmenter_script(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("Segmenter control must compile and execute through Wasm AOT");
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

fn assert_segmenter_runtime_exception(source: &str, constructor: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(&script, compile_options(), run_options())
            .expect_err("Segmenter negative must be an actual JavaScript exception");
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
fn branding_coercion_and_abrupt_order() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/branding_coercion_and_abrupt_order.js"),
        "ok branding_coercion_and_abrupt_order",
    );
}

#[test]
fn constructor_order_and_strict_options() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/constructor_order_and_strict_options.js"),
        "ok constructor_order_and_strict_options",
    );
}

#[test]
fn containing_tagged_storage_and_retained_owners() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/containing_tagged_storage_and_retained_owners.js"),
        "ok containing_tagged_storage_and_retained_owners",
    );
}

#[test]
fn cross_realm_and_newtarget() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/cross_realm_and_newtarget.js"),
        "ok cross_realm_and_newtarget",
    );
}

#[test]
fn fresh_objects_and_hidden_partition() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/fresh_objects_and_hidden_partition.js"),
        "ok fresh_objects_and_hidden_partition",
    );
}

#[test]
fn iterator_metadata_completion_and_lifetimes() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/iterator_metadata_completion_and_lifetimes.js"),
        "ok iterator_metadata_completion_and_lifetimes",
    );
}

#[test]
fn metadata_and_resolved() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/metadata_and_resolved.js"),
        "ok metadata_and_resolved",
    );
}

#[test]
fn supported_locales_and_option_boxing() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/supported_locales_and_option_boxing.js"),
        "ok supported_locales_and_option_boxing",
    );
}

#[test]
fn utf16_graphemes_and_containing() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/utf16_graphemes_and_containing.js"),
        "ok utf16_graphemes_and_containing",
    );
}

#[test]
fn word_status_and_genuine_tailoring() {
    assert_segmenter_script(
        include_str!("fixtures/intl_segmenter/word_status_and_genuine_tailoring.js"),
        "ok word_status_and_genuine_tailoring",
    );
}

#[test]
fn segment_symbol_throws_typeerror() {
    assert_segmenter_runtime_exception(
        "new Intl.Segmenter('en').segment(Symbol('input'));",
        "TypeError",
    );
}

#[test]
fn containing_bigint_throws_typeerror() {
    assert_segmenter_runtime_exception(
        "new Intl.Segmenter('en').segment('A').containing(1n);",
        "TypeError",
    );
}

#[test]
fn supported_null_options_throws_typeerror() {
    assert_segmenter_runtime_exception(
        "Intl.Segmenter.supportedLocalesOf(['en'], null);",
        "TypeError",
    );
}

#[test]
fn invalid_granularity_throws_rangeerror() {
    assert_segmenter_runtime_exception(
        "new Intl.Segmenter('en', { granularity: 'character' });",
        "RangeError",
    );
}
