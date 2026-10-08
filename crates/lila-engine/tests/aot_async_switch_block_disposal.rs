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
            .unwrap_or_else(|error| {
                panic!("async switch disposal control failed: {error}\n{source}")
            });
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
fn async_switch_disposal_preserves_selection_shared_cells_and_null_awaits() {
    assert_modes(
        include_str!("fixtures/async_switch_block_disposal/selection_and_cells.js"),
        "async-switch-selection-and-cells:ok",
    );
}

#[test]
fn async_switch_disposal_preserves_lifo_and_finalizer_selected_completion() {
    assert_modes(
        include_str!("fixtures/async_switch_block_disposal/completion_and_disposal.js"),
        "async-switch-completion-and-disposal:ok",
    );
}

#[test]
fn async_switch_disposal_preserves_abrupt_identity_suppression_and_realms() {
    assert_modes(
        include_str!("fixtures/async_switch_block_disposal/errors_and_realms.js"),
        "async-switch-errors-and-realms:ok",
    );
}
