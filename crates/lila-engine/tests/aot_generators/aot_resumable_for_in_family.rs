use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe(source: &str, strict: bool) {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let source = if strict {
        format!("\"use strict\";\n{source}")
    } else {
        source.to_string()
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
        [HostOutputEvent::PrintLine("ok".into())]
    );
}
#[test]
fn ordinary_for_in_retains_initialization_references_cells_and_abrupt_pattern_close() {
    for strict in [false, true] {
        observe(
            include_str!("../fixtures/resumable_for_in_family/generator.js"),
            strict,
        );
    }
}
#[test]
fn async_for_in_retains_key_during_await_and_unwinds_pattern_before_outer_finally() {
    for strict in [false, true] {
        observe(
            include_str!("../fixtures/resumable_for_in_family/async.js"),
            strict,
        );
    }
}
#[test]
fn annex_b_for_in_prefix_owns_original_var_write_before_enumeration_head() {
    observe(
        include_str!("../fixtures/resumable_for_in_family/annex_b.js"),
        false,
    );
}
