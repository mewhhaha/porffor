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
            .expect("branch condition executes through Wasm AOT");
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
fn nested_condition_branches_repeat_effects_and_only_selected_awaits_schedule() {
    assert_modes(
        include_str!("../fixtures/async_while_branch_condition/branches_and_scheduling.js"),
        "while-branches:ok",
    );
}

#[test]
fn optional_reads_and_call_receivers_survive_reentry_and_erased_awaits_repeat() {
    assert_modes(
        include_str!("../fixtures/async_while_branch_condition/optional_and_receiver.js"),
        "while-references:ok",
    );
}

#[test]
fn condition_throws_and_rejections_keep_identity_and_finally_completions() {
    assert_modes(
        include_str!("../fixtures/async_while_branch_condition/abrupt_and_finally.js"),
        "while-abrupt:ok",
    );
}
