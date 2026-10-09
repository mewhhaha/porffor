use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

#[test]
fn borrowed_array_sort_skips_absent_typed_array_indices_before_bigint_comparison_and_writeback() {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let fixture = include_str!("../fixtures/array_sort_typed_array_presence.js");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{fixture}");
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
            .expect("borrowed Array.sort fixture compiles and executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(
                "array-sort-typed-array-presence:ok".into()
            )],
            "{source}"
        );
    }
}
