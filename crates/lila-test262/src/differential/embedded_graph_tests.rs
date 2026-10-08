use super::*;
use lila_engine::ObservedNumber;

const CASES: [(&str, &str, &[&str]); 7] = [
    (
        "cycles",
        include_str!("../../tests/differential/v4/t25-module-cycles-and-metadata.json"),
        &["embedded-cycle:3"],
    ),
    (
        "self",
        include_str!("../../tests/differential/v4/t25-script-self-import.json"),
        &["embedded-script-self:2"],
    ),
    (
        "attributes",
        include_str!("../../tests/differential/v4/t25-computed-exact-attributes.json"),
        &["embedded-attributes:1,2"],
    ),
    (
        "undeclared",
        include_str!("../../tests/differential/v4/t25-undeclared-and-unused.json"),
        &["embedded-undeclared:denied"],
    ),
    (
        "parse",
        include_str!("../../tests/differential/v4/t25-dynamic-parse-failure.json"),
        &["embedded-parse:rejected"],
    ),
    (
        "defer",
        include_str!("../../tests/differential/v4/t25-defer-order.json"),
        &[
            "embedded-defer:before",
            "embedded-defer:body",
            "embedded-defer:value=7",
        ],
    ),
    (
        "source",
        include_str!("../../tests/differential/v4/t25-source-phase.json"),
        &["embedded-source:rejected"],
    ),
];

fn decode(json: &str) -> DifferentialCase {
    DifferentialCase::from_json(json).expect("authored complete graph is admissible")
}

fn mutated(json: &str, change: impl FnOnce(&mut serde_json::Value)) -> DifferentialCase {
    let mut wire: serde_json::Value = serde_json::from_str(json).unwrap();
    change(&mut wire);
    decode(&serde_json::to_string(&wire).unwrap())
}

#[test]
fn v4_roundtrip_couples_the_exact_entry_and_shared_execution_policy() {
    for (name, json, _) in CASES {
        let case = decode(json);
        let graph = case.module_graph().unwrap();
        assert_eq!(case.source(), graph.entry().source(), "{name}");
        assert_eq!(case.filename(), graph.entry().identity(), "{name}");
        assert_eq!(
            case.protocol(),
            DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
        );
        let roundtrip = decode(&case.to_pretty_json().unwrap());
        assert_eq!(case, roundtrip, "{name}");
        assert_eq!(case_fingerprint(&case), case_fingerprint(&roundtrip));
        match compile_options_for_case(&case).module_loading_policy {
            ModuleLoadingPolicy::Embedded(owner) => assert!(Arc::ptr_eq(graph, &owner)),
            ModuleLoadingPolicy::Filesystem | ModuleLoadingPolicy::RejectAll => {
                panic!("v4 lost its exact graph owner")
            }
        }
        let wire: serde_json::Value =
            serde_json::from_str(&case.to_pretty_json().unwrap()).unwrap();
        for absent in ["goal", "source", "filename"] {
            assert!(wire.get(absent).is_none(), "{name}: duplicated {absent}");
        }
    }
    let module_case = decode(CASES[0].1);
    assert_eq!(module_case.goal(), DifferentialGoal::Module);
    assert_eq!(
        module_case
            .module_graph()
            .unwrap()
            .modules()
            .iter()
            .filter(|module| module.identity() == "entry.js")
            .count(),
        1,
    );
    let script_case = decode(CASES[1].1);
    assert_eq!(script_case.goal(), DifferentialGoal::Script);
    let graph = script_case.module_graph().unwrap();
    assert_ne!(
        graph.entry().source(),
        graph.module("entry.js").unwrap().source()
    );
}

#[test]
fn typed_json_records_survive_worker_projection_and_bind_case_identity() {
    let wire = serde_json::json!({
        "schema_version": 4,
        "id": "t25/json-record",
        "observation_contract": "primitive_completion_print_transcript",
        "timeout_ms": 5000,
        "module_graph": {
            "entry": {"goal":"script", "identity":"entry.js", "source":"262;", "meta_url":"lila://entry"},
            "modules": [{"kind":"json", "identity":"data.json", "source":"42", "meta_url":"lila://data"}],
            "resolutions": []
        }
    });
    let json = decode(&wire.to_string());
    let projected = json.to_pretty_json().unwrap();
    let roundtrip = decode(&projected);
    assert_eq!(json, roundtrip);
    assert_eq!(case_fingerprint(&json), case_fingerprint(&roundtrip));
    assert_eq!(
        roundtrip
            .module_graph()
            .unwrap()
            .module("data.json")
            .unwrap()
            .kind(),
        lila_engine::EmbeddedModuleKind::Json
    );
    let parsed: serde_json::Value = serde_json::from_str(&projected).unwrap();
    assert_eq!(parsed["module_graph"]["modules"][0]["kind"], "json");

    let mut source_wire = wire.clone();
    source_wire["module_graph"]["modules"][0]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    let source = decode(&source_wire.to_string());
    assert_ne!(case_fingerprint(&json), case_fingerprint(&source));
    let source_roundtrip: serde_json::Value =
        serde_json::from_str(&source.to_pretty_json().unwrap()).unwrap();
    assert!(source_roundtrip["module_graph"]["modules"][0]
        .get("kind")
        .is_none());
    for invalid in [
        serde_json::json!("unknown"),
        serde_json::json!({"kind":"json"}),
        serde_json::Value::Null,
    ] {
        let mut damaged = wire.clone();
        damaged["module_graph"]["modules"][0]["kind"] = invalid;
        assert!(DifferentialCase::from_json(&damaged.to_string()).is_err());
    }
}

#[test]
fn schema_v4_rejects_uncoupled_or_invalid_graph_inputs_once() {
    assert!(DifferentialCase::new(
        "t25/invalid",
        DifferentialGoal::Script,
        DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript,
        "entry.js",
        30_000,
        "262;"
    )
    .is_err());
    for field in ["goal", "source", "filename"] {
        let mut wire: serde_json::Value = serde_json::from_str(CASES[0].1).unwrap();
        wire[field] = serde_json::json!("independently changed");
        assert!(
            DifferentialCase::from_json(&wire.to_string()).is_err(),
            "{field}"
        );
    }
    let changes: [fn(&mut serde_json::Value); 5] = [
        |wire: &mut serde_json::Value| {
            wire["schema_version"] = 3.into();
        },
        |wire: &mut serde_json::Value| {
            wire["timeout_ms"] = 0.into();
        },
        |wire: &mut serde_json::Value| {
            wire["module_graph"]["entry"]["identity"] = "../entry.js".into();
        },
        |wire: &mut serde_json::Value| {
            wire["module_graph"]["resolutions"][0]["target"] = "absent.js".into();
        },
        |wire: &mut serde_json::Value| {
            wire["module_graph"]["resolutions"][0]["referrer"]["identity"] = "absent.js".into();
        },
    ];
    for change in changes {
        let mut wire: serde_json::Value = serde_json::from_str(CASES[0].1).unwrap();
        change(&mut wire);
        assert!(DifferentialCase::from_json(&wire.to_string()).is_err());
    }
}

#[test]
fn full_graph_case_identity_is_canonical_and_captures_unused_policy_drift() {
    let base = decode(CASES[0].1);
    let reordered = mutated(CASES[0].1, |wire| {
        wire["module_graph"]["modules"]
            .as_array_mut()
            .unwrap()
            .reverse();
        wire["module_graph"]["resolutions"]
            .as_array_mut()
            .unwrap()
            .reverse();
    });
    assert_eq!(case_fingerprint(&base), case_fingerprint(&reordered));
    let source = mutated(CASES[0].1, |wire| {
        wire["module_graph"]["modules"][2]["source"] = "export const unused = 100;".into();
    });
    let metadata = mutated(CASES[0].1, |wire| {
        wire["module_graph"]["modules"][2]["meta_url"] = "lila://changed/unused.js".into();
    });
    let unused_edge = mutated(CASES[0].1, |wire| {
        wire["module_graph"]["resolutions"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "referrer": {"kind":"module","identity":"unused.js"},
                "specifier":"never-used", "attributes":[], "target":"right.js"
            }));
    });
    for changed in [source, metadata, unused_edge] {
        assert_ne!(case_fingerprint(&base), case_fingerprint(&changed));
    }
    let attributes = decode(CASES[2].1);
    let reordered_attributes = mutated(CASES[2].1, |wire| {
        wire["module_graph"]["resolutions"][0]["attributes"]
            .as_array_mut()
            .unwrap()
            .reverse();
    });
    assert_eq!(
        case_fingerprint(&attributes),
        case_fingerprint(&reordered_attributes)
    );
    let target = mutated(CASES[2].1, |wire| {
        wire["module_graph"]["resolutions"][0]["target"] = "red.js".into();
    });
    let attribute_value = mutated(CASES[2].1, |wire| {
        wire["module_graph"]["resolutions"][0]["attributes"][1][1] = "green".into();
    });
    for changed in [target, attribute_value] {
        assert_ne!(case_fingerprint(&attributes), case_fingerprint(&changed));
    }
    assert!(case_fingerprint(&base).as_str().contains(":graph-sha256:"));
}

fn completed(backend: DifferentialBackend, value: f64, output: &[&str]) -> BackendExecution {
    BackendExecution {
        backend,
        output_events: OutputEventsObservation::Captured {
            events: output.iter().map(|line| (*line).into()).collect(),
        },
        result: BackendExecutionResult::Completion {
            completion: ObservedCompletion::Normal(ObservedJsValue::Number(
                ObservedNumber::from_f64(value),
            )),
            backend_note: "finite observation".into(),
        },
    }
}

#[test]
fn v4_reports_keep_the_declared_primitive_print_contract_and_own_mismatch_domain() {
    let case = decode(CASES[1].1);
    let matching = compare_executions(
        &case,
        completed(DifferentialBackend::WasmAot, 262.0, &["first", "second"]),
        completed(DifferentialBackend::SpecExec, 262.0, &["first", "second"]),
    );
    assert_eq!(
        matching.verdict(),
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
    );
    assert_eq!(
        matching.semantic_equivalence(),
        SemanticEquivalence::NotEstablished
    );
    assert_eq!(matching.compared_dimensions(), &COMPARED_DIMENSIONS_V3);
    let wire: serde_json::Value =
        serde_json::from_str(&matching.to_pretty_json().unwrap()).unwrap();
    assert_eq!(wire["schema_version"], 4);
    for (value, output) in [
        (263.0, &["first", "second"][..]),
        (262.0, &["second", "first"][..]),
    ] {
        let mismatch = compare_executions(
            &case,
            completed(DifferentialBackend::WasmAot, 262.0, &["first", "second"]),
            completed(DifferentialBackend::SpecExec, value, output),
        );
        assert_eq!(mismatch.verdict(), DifferentialVerdict::Mismatch);
        let signature = mismatch.mismatch_signature().unwrap().as_str();
        assert!(signature.starts_with("lila-diff-v4:"));
        assert!(signature.contains(":graph-sha256:"));
    }
}
