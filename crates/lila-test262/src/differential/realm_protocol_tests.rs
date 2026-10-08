use super::*;

fn input(protocol: DifferentialProtocol) -> DifferentialReplayInput {
    // Identical source, id and filename deliberately exercise authority from
    // the native protocol only, including a misleading ordinary case name.
    DifferentialReplayInput::new_script(
        "t25/product-probe",
        protocol,
        "differential/product-probe.js",
        5_000,
        "print(typeof __lilaCreateRealm); void 0;",
    )
    .unwrap()
}

#[test]
fn realm_authority_is_explicit_native_wire_and_fingerprint_bound() {
    let product = input(DifferentialProtocol::V3PrimitiveCompletionPrintTranscript);
    let realm = input(DifferentialProtocol::V6Test262HostPrimitivePrintTranscript);
    assert_ne!(input_fingerprint(&product), input_fingerprint(&realm));
    for (input, policy) in [
        (product, lila_ir::HostSurfacePolicy::Product),
        (realm, lila_ir::HostSurfacePolicy::Test262),
    ] {
        let wire = input.to_pretty_json().unwrap();
        assert_eq!(DifferentialReplayInput::from_json(&wire).unwrap(), input);
        let admitted = DifferentialCase::from_json(&wire).unwrap();
        let options = compile_options_for_case(&admitted);
        assert_eq!(options.host_surface_policy, policy);
        assert_eq!(
            options.module_loading_policy,
            ModuleLoadingPolicy::RejectAll
        );
        assert_eq!(DifferentialReplayInput::from(admitted), input);
    }
    // Observation-contract shorthand continues to grant only Product.
    assert_eq!(
        DifferentialProtocol::from(ObservationContract::PrimitiveCompletionPrintTranscript),
        DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
    );
}

#[test]
fn realm_wire_rejects_foreign_fields_contracts_and_ambient_dependencies() {
    let original = serde_json::to_value(input(
        DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
    ))
    .unwrap();
    for (field, value) in [
        ("host_surface_policy", serde_json::json!("product")),
        (
            "observation_contract",
            serde_json::json!("self_checking_no_output"),
        ),
        ("goal", serde_json::json!("module")),
        ("schema_version", serde_json::json!(7)),
    ] {
        let mut wire = original.clone();
        wire[field] = value;
        assert!(DifferentialReplayInput::from_json(&wire.to_string()).is_err());
        assert!(DifferentialCase::from_json(&wire.to_string()).is_err());
    }
    for source in ["import('ambient.mjs')", "import("] {
        let mut wire = original.clone();
        wire["source"] = source.into();
        assert!(DifferentialCase::from_json(&wire.to_string()).is_err());
    }
}

fn observed(backend: DifferentialBackend, events: &[&str]) -> BackendExecution {
    BackendExecution {
        backend,
        output_events: OutputEventsObservation::Captured {
            events: events.iter().map(|event| (*event).into()).collect(),
        },
        result: BackendExecutionResult::Completion {
            completion: ObservedCompletion::Normal(ObservedJsValue::Undefined),
            backend_note: "synthetic protocol input; no Realm execution claim".into(),
        },
    }
}

#[test]
fn realm_reports_keep_actual_primitive_and_ordered_print_dimensions() {
    let native = input(DifferentialProtocol::V6Test262HostPrimitivePrintTranscript);
    let case = DifferentialCase::from_json(&native.to_pretty_json().unwrap()).unwrap();
    for (events, verdict) in [
        (
            vec!["prototype:true", "marker:true"],
            DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch,
        ),
        (
            vec!["marker:true", "prototype:true"],
            DifferentialVerdict::Mismatch,
        ),
    ] {
        let report = compare_executions(
            &case,
            observed(
                DifferentialBackend::WasmAot,
                &["prototype:true", "marker:true"],
            ),
            observed(DifferentialBackend::SpecExec, &events),
        );
        assert_eq!(report.verdict(), verdict);
        assert_eq!(report.protocol(), native.protocol());
        let wire = serde_json::to_value(&report).unwrap();
        assert_eq!(wire["schema_version"], 6);
        if verdict == DifferentialVerdict::Mismatch {
            assert!(wire["mismatch_signature"]
                .as_str()
                .unwrap()
                .starts_with("lila-diff-v6:test262-host-primitive-print:"));
        }
    }
}
