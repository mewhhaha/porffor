use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
    RuntimeUnavailableCapability, WasmExecutionFailureKind,
};

fn options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}

fn run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    }
}

fn assert_normal(source: &str, output: Vec<HostOutputEvent>) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}\n262;");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(&source, options(), run_options())
            .unwrap_or_else(|error| {
                panic!("installed weak intrinsic surface and early errors: {error:?}\n{source}")
            });
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
        );
        assert_eq!(observed.output_events, output, "{source}");
    }
}

#[test]
fn weak_intrinsics_keep_created_realm_descriptors_and_invalid_receiver_errors() {
    for source in [
        include_str!("../../../lila-cli/tests/fixtures/wasm_weak_ref_created_realm.js"),
        include_str!(
            "../../../lila-cli/tests/fixtures/wasm_finalization_registry_created_realm.js"
        ),
        include_str!("../../../lila-cli/tests/fixtures/wasm_weak_collections_created_realm.js"),
    ] {
        assert_normal(source, Vec::new());
    }
}

#[test]
fn early_weak_errors_and_prototype_abrupt_completion_keep_order_and_realm() {
    assert_normal(
        include_str!("../fixtures/weak_unavailable/errors_and_order.js"),
        vec![HostOutputEvent::PrintLine("weak-early-errors:ok".into())],
    );
}

#[test]
fn valid_weak_construction_reports_a_capability_failure_through_aliases_and_catches() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let capability = RuntimeUnavailableCapability::WeakReachability;
    for source in [
        "try { new WeakMap(); } catch (error) {}",
        "try { new WeakSet(); } catch (error) {}",
        "try { new WeakRef({}); } catch (error) {}",
        "try { new FinalizationRegistry(function() {}); } catch (error) {}",
        "const realm = __lilaCreateRealm().global; const C = realm.WeakMap; realm.WeakMap = null; Reflect.construct(C, []);",
        "const realm = __lilaCreateRealm().global; const C = realm.WeakSet; realm.WeakSet = null; Reflect.construct(C, []);",
        "const realm = __lilaCreateRealm().global; const C = realm.WeakRef; realm.WeakRef = null; Reflect.construct(C, [{}]);",
        "const realm = __lilaCreateRealm().global; const C = realm.FinalizationRegistry; realm.FinalizationRegistry = null; Reflect.construct(C, [function() {}]);",
        "new WeakRef(Symbol('unregistered'));",
        "new WeakRef(Symbol.iterator);",
        "new WeakRef(new Proxy({}, {}));",
    ] {
        for directive in ["", "'use strict';\n"] {
            let source = format!("{directive}{source}\n262;");
            let error = Engine::new(RealmBuilder::new().build())
                .observe_script(&source, options(), run_options())
                .expect_err("valid weak construction must reject before a strong record can be returned");
            assert_eq!(error.wasm_execution_failure_kind(), Some(WasmExecutionFailureKind::UnavailableCapability), "{source}: {error:?}");
            assert_eq!(error.runtime_unavailable_capabilities(), vec![capability]);
            assert!(error.runtime_semantic_gaps().is_empty());
            assert!(error.runtime_dynamic_source_operations().is_empty());
            assert_eq!(error.wasm_javascript_exception_constructor_name(), None);
            assert_eq!(error.message(), capability.to_string());
        }
    }
}
