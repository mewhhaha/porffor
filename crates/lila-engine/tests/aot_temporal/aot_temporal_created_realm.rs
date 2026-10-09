use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostClock, HostOutputEvent, HostSurfacePolicy,
    MonotonicClockInstant, ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder,
    RunOptions, UtcEpochMilliseconds,
};

struct FixedClock {
    next_monotonic_nanoseconds: AtomicU64,
}

impl HostClock for FixedClock {
    fn utc_epoch_milliseconds(&self) -> UtcEpochMilliseconds {
        UtcEpochMilliseconds::new(1234).expect("fixed clock is within the UTC time-value domain")
    }

    fn monotonic_instant(&self) -> MonotonicClockInstant {
        MonotonicClockInstant::new(
            self.next_monotonic_nanoseconds
                .fetch_add(1, Ordering::Relaxed),
        )
    }
}

fn assert_created_realm(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let realm = RealmBuilder::new()
            .with_host_clock(Box::new(FixedClock {
                next_monotonic_nanoseconds: AtomicU64::new(0),
            }))
            .build();
        let observation = Engine::new(realm)
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
                panic!("created-Realm control must execute: {error}\n{script}")
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
fn temporal_family_publication_and_descriptors() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/family_publication.js"
    ));
}

#[test]
fn temporal_constructor_defaults_follow_foreign_new_target() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/primitive_new_target.js"
    ));
}

#[test]
fn temporal_constructor_object_prototypes_keep_tags_and_inherited_access() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/object_prototype_tags.js"
    ));
}

#[test]
fn temporal_results_use_called_function_realm() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/called_realm_results.js"
    ));
}

#[test]
fn temporal_now_namespace_methods_keep_intrinsic_results() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/now_namespace.js"
    ));
}

#[test]
fn temporal_nested_hooks_preserve_intrinsic_error_realm() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/nested_error_realm.js"
    ));
}

#[test]
fn shared_plain_date_conversion_keeps_called_realm_slots_order_and_errors() {
    assert_created_realm(include_str!(
        "../fixtures/temporal_created_realm/plain_date_conversion.js"
    ));
}
