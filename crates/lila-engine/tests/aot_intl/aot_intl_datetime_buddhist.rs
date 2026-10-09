use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_buddhist_script(source: &str) {
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
                panic!("Buddhist DateTimeFormat must execute through Wasm AOT: {error}\n{script}")
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
fn buddhist_localized_styles() {
    assert_buddhist_script(include_str!(
        "../fixtures/intl_datetime_buddhist/localized_styles.js"
    ));
}

#[test]
fn buddhist_keyword_and_signed_years() {
    assert_buddhist_script(include_str!(
        "../fixtures/intl_datetime_buddhist/keyword_and_signed_years.js"
    ));
}

#[test]
fn buddhist_exact_instants_and_limits() {
    assert_buddhist_script(include_str!(
        "../fixtures/intl_datetime_buddhist/exact_instants_and_limits.js"
    ));
}

#[test]
fn buddhist_range_and_called_realm() {
    assert_buddhist_script(include_str!(
        "../fixtures/intl_datetime_buddhist/range_and_called_realm.js"
    ));
}

#[test]
fn buddhist_actual_temporal_calendar() {
    assert_buddhist_script(include_str!(
        "../fixtures/intl_datetime_buddhist/actual_temporal_calendar.js"
    ));
}

#[test]
fn buddhist_checked_enumeration_and_neighbors() {
    assert_buddhist_script(include_str!(
        "../fixtures/intl_datetime_buddhist/checked_enumeration_and_neighbors.js"
    ));
}
