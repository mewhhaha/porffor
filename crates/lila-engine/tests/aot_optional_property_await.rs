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
            .expect("optional property await executes through Wasm AOT");
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
fn nullish_skip_and_selected_key_conversion_retain_base_and_proxy_receiver() {
    assert_modes(
        include_str!("fixtures/optional_property_await/values_and_keys.js"),
        "optional-values:ok",
    );
}

#[test]
fn nested_shortened_links_guard_the_whole_suffix_and_each_get_precedes_later_keys() {
    assert_modes(
        include_str!("fixtures/optional_property_await/nested_suffixes.js"),
        "optional-suffixes:ok",
    );
}

#[test]
fn abrupt_key_and_property_operations_preserve_identity_realm_and_finally() {
    assert_modes(
        include_str!("fixtures/optional_property_await/abrupt_realms.js"),
        "optional-abrupt:ok",
    );
}

#[test]
fn private_and_super_optional_references_and_delete_keep_their_original_final_operation() {
    assert_modes(
        include_str!("fixtures/optional_property_await/complete_references.js"),
        "optional-references:ok",
    );
}
