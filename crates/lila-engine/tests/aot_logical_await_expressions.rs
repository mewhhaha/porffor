use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
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
            .expect("logical await executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn skipped_values_get_value_once_and_only_selected_rhs_schedules_jobs() {
    assert_modes(
        include_str!("fixtures/logical_await_expressions/values_and_scheduling.js"),
        "logical-values:ok",
    );
}

#[test]
fn nested_joins_keep_results_and_original_call_constructor_and_tag_references() {
    assert_modes(
        include_str!("fixtures/logical_await_expressions/nested_invocation.js"),
        "logical-invocation:ok",
    );
}

#[test]
fn abrupt_joins_keep_marker_identity_realm_and_finally_completion() {
    assert_modes(
        include_str!("fixtures/logical_await_expressions/abrupt_realms.js"),
        "logical-abrupt:ok",
    );
}
