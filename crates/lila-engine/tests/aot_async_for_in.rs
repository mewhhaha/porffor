use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn whole_async_for_in_preserves_native_enumeration_original_cells_and_abrupt_cleanup() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let enumeration = include_str!("fixtures/async_for_in/enumeration.js");
    let references = include_str!("fixtures/async_for_in/references_and_completions.js");
    for (source, expected) in [
        (enumeration.to_string(), "async-for-in-enumeration:ok"),
        (
            format!("'use strict';\n{enumeration}"),
            "async-for-in-enumeration:ok",
        ),
        (references.to_string(), "async-for-in-references:ok"),
        (
            include_str!("fixtures/async_for_in/lexical_cells.js").to_string(),
            "async-for-in-lexical-cells:ok",
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
            vec![HostOutputEvent::PrintLine(expected.into())]
        );
    }
}
