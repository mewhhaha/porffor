use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe_resource_fixture(source: &str, marker: &str, strict: bool) {
    lila_engine::configure_compilation_jobs(1).unwrap();
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

#[test]
fn mixed_async_generator_resources_preserve_registration_scope() {
    for strict in [false, true] {
        observe_resource_fixture(
            include_str!("fixtures/async_generator_resources/registration_and_scopes.js"),
            "mixed-async-generator-resource-scopes:ok",
            strict,
        );
    }
}

#[test]
fn mixed_async_generator_resources_preserve_whole_completion_and_references() {
    observe_resource_fixture(
        include_str!("fixtures/async_generator_resources/completions_and_references.js"),
        "mixed-async-generator-resource-completions:ok",
        false,
    );
}
