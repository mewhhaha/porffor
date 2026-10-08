use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn mixed_assignment_preserves_reference_order_and_skipped_or_abrupt_completions() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for (source, expected, strict_allowed) in [
        (
            include_str!("fixtures/async_generator_assignment/references.js"),
            "mixed-assignment-references:ok",
            true,
        ),
        (
            include_str!("fixtures/async_generator_assignment/super.js"),
            "mixed-super-assignment:ok",
            true,
        ),
        (
            include_str!("fixtures/async_generator_assignment/web_compat.js"),
            "mixed-call-assignment:ok",
            false,
        ),
    ] {
        let mut sources = vec![source.to_owned()];
        if strict_allowed {
            sources.push(format!("'use strict';\n{source}"));
        }
        for source in sources {
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
}
