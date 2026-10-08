use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_tols_script(source: &str) {
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
                    timeout_ms: Some(120_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| {
                panic!("Tolong Siki numbering must execute through Wasm AOT: {error}\n{script}")
            });
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
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
fn tols_number_parts_and_exact_inputs() {
    assert_tols_script(include_str!(
        "fixtures/intl_numbering_tols/number_parts_and_exact_inputs.js"
    ));
}

#[test]
fn tols_date_time_and_buddhist() {
    assert_tols_script(include_str!(
        "fixtures/intl_numbering_tols/date_time_and_buddhist.js"
    ));
}

#[test]
fn tols_selection_and_measurement_text() {
    assert_tols_script(include_str!(
        "fixtures/intl_numbering_tols/selection_and_measurement_text.js"
    ));
}

#[test]
fn tols_enumeration_and_called_realm() {
    assert_tols_script(include_str!(
        "fixtures/intl_numbering_tols/enumeration_and_called_realm.js"
    ));
}
