use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
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
            .expect("object binding fixture compiles and executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())]
        );
    }
}

#[test]
fn ordinary_and_var_bindings_retain_one_get_and_selected_defaults() {
    assert_modes(
        include_str!("fixtures/object_binding_single_get/ordinary_and_var.js"),
        "object-binding-ordinary:ok",
    );
}

#[test]
fn loop_and_async_heads_do_not_repeat_get_or_default_on_resume() {
    assert_modes(
        include_str!("fixtures/object_binding_single_get/loops_and_async.js"),
        "object-binding-loops:ok",
    );
}

#[test]
fn default_and_getter_abrupts_keep_identity_cutoffs_and_tdz() {
    assert_modes(
        include_str!("fixtures/object_binding_single_get/abrupt_and_tdz.js"),
        "object-binding-abrupt:ok",
    );
}
