use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("Array species consumer must compile and execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).to_owned()))
                .collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn flat_and_concat_accept_array_species_carriers_and_proxy_constructors() {
    assert_trace(
        include_str!("fixtures/array_species_consumers/constructor_choices.js"),
        &[
            "flat:constructor,species,construct:0:true,get:0:true:7",
            "concat:constructor,species,construct:0:true,get:0:true:7",
        ],
    );
}

#[test]
fn proxy_array_receiver_observes_constructor_and_species_once() {
    assert_trace(
        include_str!("fixtures/array_species_consumers/proxy_lookup.js"),
        &["flat:1:1:1:true", "concat:1:1:1:true"],
    );
}

#[test]
fn abrupt_species_creation_preserves_throw_identity_and_skips_element_reads() {
    assert_trace(
        include_str!("fixtures/array_species_consumers/abrupt_creation.js"),
        &[
            "flat/constructor:true:0",
            "concat/constructor:true:0",
            "flat/species:true:0",
            "concat/species:true:0",
            "flat/construct:true:0",
            "concat/construct:true:0",
            "flat/nonconstructor:true:0",
            "concat/nonconstructor:true:0",
        ],
    );
}

fn assert_splice_spread_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
            .expect("splice spread must compile and execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.to_owned())],
            "{source}"
        );
    }
}

#[test]
fn splice_spread_preserves_reference_iteration_species_and_property_order() {
    assert_splice_spread_modes(
        include_str!("fixtures/array_splice_spread/ordering_and_iteration.js"),
        "array-splice-spread-order:ok",
    );
}

#[test]
fn splice_spread_preserves_called_realm_and_original_abrupt_completions() {
    assert_splice_spread_modes(
        include_str!("fixtures/array_splice_spread/realms_and_abrupt.js"),
        "array-splice-spread-abrupt:ok",
    );
}
