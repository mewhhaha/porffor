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
            .expect("Proxy ownKeys fixture compiles and executes through Wasm AOT");
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
fn proxy_own_keys_observes_live_gets_then_publishes_an_independent_snapshot() {
    assert_modes(
        include_str!("../fixtures/proxy_own_keys/live_get_and_snapshot.js"),
        "proxy-own-keys-live:ok",
    );
}

#[test]
fn proxy_own_keys_validates_types_then_duplicates_then_target_invariants() {
    assert_modes(
        include_str!("../fixtures/proxy_own_keys/validation_order.js"),
        "proxy-own-keys-validation:ok",
    );
}

#[test]
fn proxy_own_keys_preserves_abrupt_identity_and_borrowed_builtin_realms() {
    assert_modes(
        include_str!("../fixtures/proxy_own_keys/abrupt_and_realms.js"),
        "proxy-own-keys-abrupt:ok",
    );
}

#[test]
fn proxy_own_keys_finishes_real_target_operations_before_membership_checks() {
    assert_modes(
        include_str!("../fixtures/proxy_own_keys/target_operations.js"),
        "proxy-own-keys-target-operations:ok",
    );
}

#[test]
fn proxy_own_keys_checks_exotics_and_retains_the_outer_builtin_realm() {
    assert_modes(
        include_str!("../fixtures/proxy_own_keys/target_exotics_and_realms.js"),
        "proxy-own-keys-target-realms:ok",
    );
}
