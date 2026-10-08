use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
const BOXED_UNDEFINED: &str = include_str!(
    "../../../test262/vendor/test262/test/built-ins/String/prototype/concat/S15.5.4.6_A1_T9.js"
);
const LOOKUP_CONTROLS: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_concat_method_lookup.js");

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("concat lookup regression must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn unchanged_pinned_boxed_undefined_concat_executes_in_both_script_modes() {
    for directive in ["", "\"use strict\";\n"] {
        assert_wasm_true(&format!("{directive}{STA}\n{BOXED_UNDEFINED}\ntrue;"));
    }
}

#[test]
fn concat_lookup_overrides_and_abrupt_order_execute_in_both_script_modes() {
    for directive in ["", "\"use strict\";\n"] {
        assert_wasm_true(&format!("{directive}{LOOKUP_CONTROLS}"));
    }
}
