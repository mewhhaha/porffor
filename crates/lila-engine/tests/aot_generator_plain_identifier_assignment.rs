use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe_plain_identifier_assignment(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("write-only Identifier Reference executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine(expected.into())]
    );
}

#[test]
fn plain_generator_writes_delay_assignment_failures_and_preserve_rhs_values_and_abrupts() {
    for (directive, strict) in [("", false), ("'use strict';\n", true)] {
        observe_plain_identifier_assignment(
            &format!(
                "{directive}var strictPlainFixture = {strict};\n{}",
                include_str!("fixtures/generator_plain_identifier_assignment/bindings.js"),
            ),
            "generator-plain-bindings:ok",
        );
    }
}

#[test]
fn plain_generator_with_writes_retain_selected_references_without_getting_the_binding() {
    observe_plain_identifier_assignment(
        include_str!("fixtures/generator_plain_identifier_assignment/with_reference.js"),
        "generator-plain-with:ok",
    );
}
