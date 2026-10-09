use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
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
            .expect("logical assignment awaits execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn declarative_values_skip_jobs_and_selected_writes_retain_original_binding() {
    assert_modes(
        include_str!("../fixtures/logical_assignment_await/values_and_jobs.js"),
        "logical-assignment-values:ok",
    );
}

#[test]
fn property_get_and_put_share_receiver_and_normalized_key_across_await() {
    assert_modes(
        include_str!("../fixtures/logical_assignment_await/property_references.js"),
        "logical-assignment-properties:ok",
    );
}

#[test]
fn get_key_rhs_and_put_errors_preserve_identity_precedence_and_finally() {
    assert_modes(
        include_str!("../fixtures/logical_assignment_await/abrupt_realms.js"),
        "logical-assignment-abrupt:ok",
    );
}

#[test]
fn awaited_left_private_and_super_references_use_one_original_get_and_selected_put() {
    assert_modes(
        include_str!("../fixtures/logical_assignment_await/complete_references.js"),
        "logical-assignment-complete-references:ok",
    );
}

#[test]
fn suspended_reference_operands_preserve_plain_compound_update_and_delete_order() {
    assert_modes(
        include_str!("../fixtures/logical_assignment_await/reference_operations.js"),
        "async-reference-operations:ok",
    );
}
