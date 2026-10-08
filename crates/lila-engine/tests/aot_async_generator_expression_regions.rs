use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn mixed_async_generator_expressions_preserve_original_literals_class_references_and_lazy_tails() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for (source, marker, strict) in [
        (
            include_str!("fixtures/async_generator_expressions/literals_and_classes.js"),
            "mixed-async-generator-literals-classes:ok",
            false,
        ),
        (
            include_str!("fixtures/async_generator_expressions/literals_and_classes.js"),
            "mixed-async-generator-literals-classes:ok",
            true,
        ),
        (
            include_str!("fixtures/async_generator_expressions/optional_and_delete.js"),
            "mixed-async-generator-optional-delete:ok",
            false,
        ),
        (
            include_str!("fixtures/async_generator_expressions/optional_and_delete.js"),
            "mixed-async-generator-optional-delete:ok",
            true,
        ),
        (
            include_str!("fixtures/async_generator_expressions/updates_and_special_reads.js"),
            "mixed-async-generator-updates-special-reads:ok",
            false,
        ),
        (
            include_str!("fixtures/async_generator_expressions/updates_and_special_reads.js"),
            "mixed-async-generator-updates-special-reads:ok",
            true,
        ),
        (
            include_str!("fixtures/async_generator_expressions/web_compat_update.js"),
            "mixed-async-generator-annex-b-update:ok",
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
