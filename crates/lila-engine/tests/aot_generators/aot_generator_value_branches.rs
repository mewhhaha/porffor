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
            .expect("generator value branches execute through Wasm AOT");
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
fn selected_yields_capture_received_values_and_skipped_branches_keep_original_values() {
    assert_modes(
        include_str!("../fixtures/generator_value_branches/values_and_skips.js"),
        "generator-values:ok",
    );
}

#[test]
fn yielded_optional_keys_keep_the_base_and_complete_suffix_and_outer_reference_in_order() {
    assert_modes(
        include_str!("../fixtures/generator_value_branches/property_values.js"),
        "generator-properties:ok",
    );
}

#[test]
fn injected_throw_return_and_eager_finally_preserve_identity_and_skip_publication() {
    assert_modes(
        include_str!("../fixtures/generator_value_branches/abrupt_realms.js"),
        "generator-abrupt:ok",
    );
}

#[test]
fn complete_value_regions_keep_lazy_selection_delegation_and_abrupt_identity() {
    assert_modes(
        include_str!("../fixtures/generator_value_branches/complete_regions.js"),
        "generator-complete-regions:ok",
    );
}
