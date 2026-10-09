use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn original_generator_and_async_resource_scopes_stage_initializers_and_keep_one_lifetime() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for (source, marker) in [
        (
            include_str!("../fixtures/resumable_resource_scopes/generator.js"),
            "resumable-generator-resource-scopes:ok",
        ),
        (
            include_str!("../fixtures/resumable_resource_scopes/async.js"),
            "resumable-async-resource-scopes:ok",
        ),
    ] {
        for strict in [false, true] {
            let source = if strict {
                format!("'use strict';\n{source}")
            } else {
                source.to_owned()
            };
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
                .unwrap();
            assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
            assert!(
                matches!(observed.completion, ObservedCompletion::Normal(_)),
                "{:?}",
                observed.completion
            );
            assert_eq!(
                observed.output_events,
                [HostOutputEvent::PrintLine(marker.into())]
            );
        }
    }
}
