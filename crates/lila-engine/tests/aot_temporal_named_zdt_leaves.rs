//! Actual compiled named-zone leaf semantics, integrated with the T22 authority batch.
use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_named_leaf_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &script,
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
            .expect("named-zone leaf control must execute through Wasm AOT");
        assert_eq!(
            observation.backend_used,
            ExecutionBackend::WasmAot,
            "{script}"
        );
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            observation.output_events,
            vec![HostOutputEvent::PrintLine("ok".into())],
            "{script}"
        );
    }
}

#[test]
fn day_boundaries_and_fold() {
    assert_named_leaf_script(include_str!(
        "fixtures/temporal_named_zdt_leaves/day_boundaries_and_fold.js"
    ));
}

#[test]
fn day_and_subday_rounding() {
    assert_named_leaf_script(include_str!(
        "fixtures/temporal_named_zdt_leaves/day_and_subday_rounding.js"
    ));
}

#[test]
fn with_offset_and_disambiguation() {
    assert_named_leaf_script(include_str!(
        "fixtures/temporal_named_zdt_leaves/with_offset_and_disambiguation.js"
    ));
}

#[test]
fn with_observable_order() {
    assert_named_leaf_script(include_str!(
        "fixtures/temporal_named_zdt_leaves/with_observable_order.js"
    ));
}

#[test]
fn strict_transitions() {
    assert_named_leaf_script(include_str!(
        "fixtures/temporal_named_zdt_leaves/strict_transitions.js"
    ));
}

#[test]
fn rounded_format_and_identity() {
    assert_named_leaf_script(include_str!(
        "fixtures/temporal_named_zdt_leaves/rounded_format_and_identity.js"
    ));
}
