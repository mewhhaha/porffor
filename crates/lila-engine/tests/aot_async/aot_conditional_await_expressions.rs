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
            .expect("conditional await executes through Wasm AOT");
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
fn selected_branch_and_original_invocation_reference_survive_resumption() {
    assert_modes(
        include_str!("../fixtures/conditional_await_expressions/invocation_order.js"),
        "invocation:ok",
    );
}

#[test]
fn nested_joins_own_retained_values_facts_and_lexical_initialization() {
    assert_modes(
        include_str!("../fixtures/conditional_await_expressions/nested_values.js"),
        "nested:ok",
    );
}

#[test]
fn rejection_and_synchronous_branch_throw_keep_identity_and_realm_through_finally() {
    assert_modes(
        include_str!("../fixtures/conditional_await_expressions/abrupt_realms.js"),
        "abrupt:ok",
    );
}
