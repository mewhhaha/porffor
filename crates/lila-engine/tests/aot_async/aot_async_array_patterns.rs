use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn async_array_patterns_preserve_iterator_records_original_targets_and_close_through_await() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for directive in ["", "'use strict';\n"] {
        for (fixture, label) in [
            (
                include_str!("../fixtures/async_array_patterns/bindings.js"),
                "async-array-bindings:ok",
            ),
            (
                include_str!("../fixtures/async_array_patterns/assignments.js"),
                "async-array-assignments:ok",
            ),
        ] {
            let observed = Engine::new(RealmBuilder::new().build())
                .observe_script(
                    &format!("{directive}{fixture}"),
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
                .unwrap();
            assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
            assert!(
                matches!(observed.completion, ObservedCompletion::Normal(_)),
                "{:?}",
                observed.completion
            );
            assert_eq!(
                observed.output_events,
                vec![HostOutputEvent::PrintLine(label.into())]
            );
        }
    }
}
