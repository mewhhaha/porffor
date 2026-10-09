use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_zoned_locale(source: &str) {
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
            .unwrap_or_else(|error| panic!("zoned locale bridge must execute: {error}\n{script}"));
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
fn zoned_locale_preserves_floor_milliseconds_and_epoch_zone() {
    assert_zoned_locale(include_str!(
        "../fixtures/temporal_zoned_locale/exact_epoch_and_zone.js"
    ));
}

#[test]
fn zoned_locale_preserves_option_order_abrupt_identity_and_defining_realm() {
    assert_zoned_locale(include_str!(
        "../fixtures/temporal_zoned_locale/options_order_and_errors.js"
    ));
}

#[test]
fn zoned_locale_checks_actual_calendar_after_formatter_initialization() {
    assert_zoned_locale(include_str!(
        "../fixtures/temporal_zoned_locale/calendar_and_initialization.js"
    ));
}

#[test]
fn zoned_locale_defaults_styles_and_canonical_identifiers_follow_intl() {
    assert_zoned_locale(include_str!(
        "../fixtures/temporal_zoned_locale/defaults_styles_and_identifiers.js"
    ));
}
