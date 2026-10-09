use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_var_binding_order(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let result = Engine::new(RealmBuilder::new().build())
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
        .expect("var BindingInitialization executes through emitted Wasm");
    assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        result.completion,
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
        "{source}"
    );
}

#[test]
fn var_destructuring_computed_key_order() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/computed_key_order.js"
    ));
}

#[test]
fn var_destructuring_static_selected_object() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/static_selected_object.js"
    ));
}

#[test]
fn var_destructuring_repeated_targets() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/repeated_targets.js"
    ));
}

#[test]
fn var_destructuring_array_and_rest_order() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/array_and_rest_order.js"
    ));
}

#[test]
fn var_destructuring_abrupt_resolution() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/abrupt_resolution.js"
    ));
}

#[test]
fn var_destructuring_unscopables_and_nested_fallback() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/unscopables_and_nested_fallback.js"
    ));
}

#[test]
fn var_destructuring_lexical_and_function_cutoff() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/lexical_and_function_cutoff.js"
    ));
}

#[test]
fn var_destructuring_var_loop_heads() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/var_loop_heads.js"
    ));
}

#[test]
fn var_destructuring_unscopables_abrupt() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/unscopables_abrupt.js"
    ));
}

#[test]
fn var_destructuring_direct_eval_reference() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/direct_eval_reference.js"
    ));
}

#[test]
fn var_destructuring_conditional_rest_fallback_facts() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/conditional_rest_fallback_facts.js"
    ));
}
