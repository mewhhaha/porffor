use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_wasm_string_write(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("boxed String assignment compiles and executes through Wasm AOT");
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("ok".to_string())],
        "{source}"
    );
}

#[test]
fn sloppy_object_environment_length_writes_preserve_rhs_and_update_results() {
    assert_wasm_string_write(include_str!(
        "../fixtures/boxed_string_environment_writes/sloppy-length.js"
    ));
}

#[test]
fn strict_captured_object_environment_length_writes_throw_after_rhs() {
    assert_wasm_string_write(include_str!(
        "../fixtures/boxed_string_environment_writes/strict-captured-length.js"
    ));
}

#[test]
fn dynamic_boxed_string_writes_obey_virtual_and_ordinary_descriptors() {
    assert_wasm_string_write(include_str!(
        "../fixtures/boxed_string_environment_writes/dynamic-properties.js"
    ));
}
