use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn mixed_async_generator_complete_regions_preserve_phases_references_and_whole_completions() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let phases = include_str!("fixtures/async_generator_classic_regions/phases_and_branches.js");
    let completions =
        include_str!("fixtures/async_generator_classic_regions/completions_and_environments.js");
    for (source, expected) in [
        (phases.to_owned(), "mixed-async-generator-phases:ok"),
        (
            format!("'use strict';\n{phases}"),
            "mixed-async-generator-phases:ok",
        ),
        (
            completions.to_owned(),
            "mixed-async-generator-completions:ok",
        ),
        (
            format!("'use strict';\n{completions}"),
            "mixed-async-generator-completions:ok",
        ),
    ] {
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
            [HostOutputEvent::PrintLine(expected.into())]
        );
    }
}
