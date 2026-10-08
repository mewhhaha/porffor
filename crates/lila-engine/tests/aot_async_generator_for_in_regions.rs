use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn mixed_async_generator_for_in_preserves_enumeration_references_and_completions() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for (source, marker, strict) in [
        (
            include_str!("fixtures/async_generator_for_in/enumeration.js"),
            "mixed-async-generator-for-in-enumeration:ok",
            false,
        ),
        (
            include_str!("fixtures/async_generator_for_in/enumeration.js"),
            "mixed-async-generator-for-in-enumeration:ok",
            true,
        ),
        (
            include_str!("fixtures/async_generator_for_in/references_and_completions.js"),
            "mixed-async-generator-for-in-references:ok",
            false,
        ),
    ] {
        let source = if strict {
            format!("'use strict';\n{source}")
        } else {
            source.to_string()
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
