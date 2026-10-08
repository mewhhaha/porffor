use super::*;
use lila_runtime::rooted_snapshot::{SnapshotBudgetDimension, SnapshotExotic};
use serde_json::json;

fn input(limits: SnapshotLimits) -> DifferentialReplayInput {
    DifferentialReplayInput::new_snapshot_script(
        "t25/rooted/native",
        "differential/rooted-native.js",
        5_000,
        "const value = Object.create(null); value.self = value; value;",
        limits,
    )
    .unwrap()
}

fn graph(limits: SnapshotLimits, writable: bool) -> RootedSnapshotGraph {
    serde_json::from_value(json!({
        "version": 1, "limits": limits, "root": {"type":"object","id":0},
        "nodes": [{"id":0,"kind":{"kind":"ordinary"},"extensible":true,
            "prototype":{"type":"null"},"anchors":[],"properties":[{
                "key":{"type":"string","units":[115,101,108,102]},
                "descriptor":{"kind":"data","value":{"type":"object","id":0},
                    "writable":writable,"enumerable":true,"configurable":true}
            }]}], "symbols":[], "realm_count":1
    }))
    .unwrap()
}

fn observed(
    backend: DifferentialBackend,
    completion: SnapshotCompletion,
    note: &str,
) -> BackendObservation {
    BackendObservation {
        worker_identity: None,
        backend,
        output_events: OutputEventsObservation::Captured {
            events: vec!["after-jobs".into()],
        },
        execution: ExecutionObservation::RootedCompletionGraph {
            completion,
            backend_note: note.into(),
        },
    }
}

fn captured(limits: SnapshotLimits, writable: bool) -> SnapshotCompletion {
    SnapshotCompletion {
        kind: SnapshotCompletionKind::Normal,
        outcome: SnapshotOutcome::Captured {
            graph: graph(limits, writable),
        },
    }
}

#[test]
fn rooted_wire_retains_source_and_explicit_authority_with_all_limits_bound() {
    let original = input(SnapshotLimits::default());
    let encoded = original.to_pretty_json().unwrap();
    let decoded = DifferentialReplayInput::from_json(&encoded).unwrap();
    assert_eq!(decoded, original);
    let case = DifferentialCase::from_json(&encoded).unwrap();
    assert_eq!(case.source(), original.source());
    assert_eq!(DifferentialReplayInput::from(&case), original);
    assert_eq!(
        compile_options_for_case(&case).host_surface_policy,
        lila_ir::HostSurfacePolicy::Test262
    );
    assert_eq!(
        compile_options_for_case(&case).module_loading_policy,
        ModuleLoadingPolicy::RejectAll
    );
    let value = serde_json::to_value(&original).unwrap();
    assert_eq!(value["host_profile"], "test262");
    assert_eq!(value["program"]["source"], original.source());
    assert!(value.get("source").is_none());
    assert!(value.get("goal").is_none());
    for key in [
        "nodes",
        "symbols",
        "realms",
        "properties",
        "utf16_units",
        "bigint_digits",
        "work",
        "depth",
    ] {
        let mut wire = value.clone();
        wire["snapshot_limits"][key] = (wire["snapshot_limits"][key].as_u64().unwrap() - 1).into();
        let changed = DifferentialReplayInput::from_json(&wire.to_string()).unwrap();
        assert_ne!(
            input_fingerprint(&original),
            input_fingerprint(&changed),
            "{key}"
        );
    }
    for key in ["host_profile", "snapshot_limits"] {
        let mut wire = value.clone();
        wire.as_object_mut().unwrap().remove(key);
        assert!(DifferentialReplayInput::from_json(&wire.to_string()).is_err());
    }
    for (key, replacement) in [
        ("host_profile", json!("product")),
        ("schema_version", json!(6)),
        ("goal", json!("script")),
    ] {
        let mut wire = value.clone();
        wire[key] = replacement;
        assert!(DifferentialReplayInput::from_json(&wire.to_string()).is_err());
    }
    for invalid in [json!(0), json!(u32::MAX)] {
        let mut wire = value.clone();
        wire["snapshot_limits"]["nodes"] = invalid;
        assert!(DifferentialReplayInput::from_json(&wire.to_string()).is_err());
    }
    let mut foreign = value.clone();
    foreign["program"]["goal"] = json!("module");
    assert!(DifferentialReplayInput::from_json(&foreign.to_string()).is_err());
    assert!(
        DifferentialReplayInput::new_script("t25/rooted", PROTOCOL, "rooted.js", 1, "0").is_err()
    );
    assert!(DifferentialCase::new_snapshot_script(
        "t25/rooted",
        "rooted.js",
        1,
        "import('ambient.mjs')",
        SnapshotLimits::default()
    )
    .is_err());
}

#[test]
fn rooted_embedded_wire_owns_typed_graph_without_duplicate_source_authority() {
    let old: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/differential/v4/t25-module-cycles-and-metadata.json"
    ))
    .unwrap();
    let mut wire = serde_json::to_value(input(SnapshotLimits::default())).unwrap();
    wire["program"] = json!({"kind":"embedded_graph", "module_graph":old["module_graph"]});
    let native = DifferentialReplayInput::from_json(&wire.to_string()).unwrap();
    let admitted = DifferentialCase::from_json(&wire.to_string()).unwrap();
    assert_eq!(native.goal(), DifferentialGoal::Module);
    assert_eq!(
        native.source(),
        native.module_graph().unwrap().entry().source()
    );
    assert_eq!(native, DifferentialReplayInput::from(&admitted));
    assert_eq!(
        native,
        DifferentialReplayInput::from_json(&native.to_pretty_json().unwrap()).unwrap()
    );
    match compile_options_for_case(&admitted).module_loading_policy {
        ModuleLoadingPolicy::Embedded(graph) => {
            assert!(Arc::ptr_eq(&graph, admitted.module_graph().unwrap()))
        }
        ModuleLoadingPolicy::Filesystem | ModuleLoadingPolicy::RejectAll => {
            panic!("v7 lost its admitted graph")
        }
    }
    let original = input_fingerprint(&native);
    wire["program"]["module_graph"]["entry"]["source"] = json!("void 0;");
    let changed = DifferentialReplayInput::from_json(&wire.to_string()).unwrap();
    assert_ne!(original, input_fingerprint(&changed));
    wire["program"]["source"] = json!("forged source");
    assert!(DifferentialReplayInput::from_json(&wire.to_string()).is_err());
}

#[test]
fn rooted_comparison_retains_descriptors_kind_prints_and_ignores_diagnostic_handles() {
    let limits = SnapshotLimits::default();
    let input = input(limits);
    let wasm = observed(
        DifferentialBackend::WasmAot,
        captured(limits, true),
        "diagnostic-only native handle",
    );
    let spec = observed(
        DifferentialBackend::SpecExec,
        captured(limits, true),
        "different diagnostic-only handle",
    );
    let report = compare_observations(&input, wasm.clone(), spec.clone());
    assert!(report.is_green());
    assert_eq!(
        report.verdict(),
        DifferentialVerdict::RootedCompletionGraphAndPrintTranscriptMatch
    );
    let wire = serde_json::to_value(&report).unwrap();
    assert_eq!(wire["schema_version"], 7);
    assert_eq!(wire["host_profile"], "test262");
    assert_eq!(
        wire["snapshot_limits"],
        serde_json::to_value(limits).unwrap()
    );
    assert_eq!(wire["semantic_equivalence"], "not_established");
    for changed in [
        captured(limits, false),
        SnapshotCompletion {
            kind: SnapshotCompletionKind::Throw,
            ..captured(limits, true)
        },
    ] {
        let changed = observed(DifferentialBackend::SpecExec, changed, "irrelevant");
        let mismatch = compare_observations(&input, wasm.clone(), changed.clone());
        assert_eq!(mismatch.verdict(), DifferentialVerdict::Mismatch);
        let mut without_notes = changed;
        if let ExecutionObservation::RootedCompletionGraph { backend_note, .. } =
            &mut without_notes.execution
        {
            backend_note.clear();
        }
        assert_eq!(
            mismatch.mismatch_signature(),
            compare_observations(&input, wasm.clone(), without_notes).mismatch_signature()
        );
    }
    let mut changed_print = spec;
    changed_print.output_events = OutputEventsObservation::Captured {
        events: vec!["before-jobs".into()],
    };
    assert_eq!(
        compare_observations(&input, wasm, changed_print).verdict(),
        DifferentialVerdict::Mismatch
    );
}

#[test]
fn rooted_rejections_and_foreign_budgets_never_form_a_green_match() {
    let limits = SnapshotLimits::default();
    let input = input(limits);
    for reason in [
        SnapshotRejection::BudgetExceeded {
            dimension: SnapshotBudgetDimension::Nodes,
        },
        SnapshotRejection::UnsupportedExotic {
            exotic: SnapshotExotic::Proxy,
        },
        SnapshotRejection::InvalidGraph {
            detail: "invalid witness".into(),
        },
        SnapshotRejection::BackendInvariant {
            detail: "missing retained root".into(),
        },
    ] {
        let completion = SnapshotCompletion {
            kind: SnapshotCompletionKind::Throw,
            outcome: SnapshotOutcome::Rejected { reason },
        };
        let report = compare_observations(
            &input,
            observed(DifferentialBackend::WasmAot, completion.clone(), ""),
            observed(DifferentialBackend::SpecExec, completion, ""),
        );
        assert_eq!(
            report.verdict(),
            DifferentialVerdict::ObservationContractViolated
        );
        assert!(!report.is_green());
        assert!(report.mismatch_signature().is_none());
    }
    let wrong = observed(
        DifferentialBackend::WasmAot,
        captured(SnapshotLimits::HARD_MAX, true),
        "",
    );
    assert!(!terminal_limits_match(&input, &wrong.execution));
    assert_eq!(
        compare_observations(&input, wrong.clone(), wrong).verdict(),
        DifferentialVerdict::ObservationContractViolated
    );
    let mut malformed = serde_json::to_value(
        observed(DifferentialBackend::WasmAot, captured(limits, true), "").execution,
    )
    .unwrap();
    malformed["completion"]["outcome"]["graph"]["nodes"][0]["prototype"] =
        json!({"type":"object","id":4});
    assert!(serde_json::from_value::<ExecutionObservation>(malformed).is_err());
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn rooted_terminal_shape_is_exclusive_to_v7() {
    let execution = observed(
        DifferentialBackend::WasmAot,
        captured(SnapshotLimits::default(), true),
        "",
    )
    .execution;
    assert!(worker_process::valid_terminal(PROTOCOL, &execution));
    for protocol in [
        DifferentialProtocol::V1SelfCheckingNoOutput,
        DifferentialProtocol::V2PrimitiveCompletionNoOutput,
        DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
        DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript,
        DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
        DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
    ] {
        assert!(!worker_process::valid_terminal(protocol, &execution));
    }
    assert!(!worker_process::valid_terminal(
        PROTOCOL,
        &ExecutionObservation::Normal {
            backend_note: "unit type label".into()
        }
    ));
}
