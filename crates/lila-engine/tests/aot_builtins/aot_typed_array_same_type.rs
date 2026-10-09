use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
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
            .expect("same-type TypedArray fixture compiles and executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn same_type_copy_methods_use_original_defining_realm_intrinsics_for_every_kind() {
    assert_modes(
        include_str!("../fixtures/typed_array_same_type/constructors_and_realms.js"),
        "typed-array-same-type-realms:ok",
    );
}

#[test]
fn with_captures_length_then_coerces_and_checks_the_fresh_integer_index() {
    assert_modes(
        include_str!("../fixtures/typed_array_same_type/ordering_and_views.js"),
        "typed-array-same-type-views:ok",
    );
}

#[test]
fn sorting_preserves_snapshots_abrupt_completion_and_fresh_writeback() {
    assert_modes(
        include_str!("../fixtures/typed_array_same_type/sorting_and_abrupt.js"),
        "typed-array-same-type-sort:ok",
    );
}
