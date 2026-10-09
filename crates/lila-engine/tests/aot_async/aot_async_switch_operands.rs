use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn complete_async_switch_operands_preserve_selection_references_case_cells_and_cleanup() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for strict in [false, true] {
        let fixture = include_str!("../fixtures/async_switch_operands.js");
        let source = if strict {
            format!("'use strict';\n{fixture}")
        } else {
            fixture.to_owned()
        };
        let result = Engine::new(RealmBuilder::new().build())
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
            .unwrap();
        assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(result.completion, ObservedCompletion::Normal(_)),
            "{:?}",
            result.completion
        );
        assert_eq!(
            result.output_events,
            [HostOutputEvent::PrintLine(
                "async-switch-operands:ok".into()
            )]
        );
    }
}
