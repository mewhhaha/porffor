use super::*;

fn plan() -> ObjectGenerationPlan {
    ObjectGenerationPlan::new(7, 3, 4).unwrap()
}
fn data(value: Value) -> Property {
    Property {
        key: Key::String("x"),
        descriptor: Descriptor::Data {
            value,
            writable: true,
        },
        enumerable: true,
        configurable: true,
    }
}
fn node(properties: Vec<Property>) -> Node {
    Node {
        kind: Kind::Object,
        prototype: None,
        properties,
        extensible: true,
    }
}
fn root(id: u8) -> Root {
    Root {
        name: id,
        node: NodeId(id),
    }
}
#[test]
fn graph_constructor_refuses_foreign_aliases_and_duplicate_descriptor_keys() {
    assert!(ObjectGenerationPlan::new(0, 0, 1).is_err());
    assert!(ObjectGenerationPlan::new(0, 1, 17).is_err());
    assert!(Program::new(
        vec![node(vec![data(Value::Node(NodeId(1)))])],
        vec![],
        vec![root(0)],
        false
    )
    .is_none());
    assert!(Program::new(
        vec![node(vec![data(Value::Symbol(SymbolId(0)))])],
        vec![],
        vec![root(0)],
        false
    )
    .is_none());
    assert!(Program::new(
        vec![node(vec![data(Value::Null), data(Value::Null)])],
        vec![],
        vec![root(0)],
        false
    )
    .is_none());
    let mut cyclic_prototype = node(vec![]);
    cyclic_prototype.prototype = Some(NodeId(0));
    assert!(Program::new(vec![cyclic_prototype], vec![], vec![root(0)], false).is_none());
    assert!(Program::new(
        vec![node(vec![]), node(vec![])],
        vec![],
        vec![root(0)],
        false
    )
    .is_none());
    let mut array = node(vec![data(Value::Null)]);
    array.kind = Kind::Array;
    array.properties[0].key = Key::String("length");
    assert!(Program::new(vec![array], vec![], vec![root(0)], false).is_none());
}

#[test]
fn mutation_aliases_participate_in_admission_liveness_and_reindexing() {
    let operation = Operation::Set {
        target: NodeId(2),
        key: Key::Symbol(SymbolId(1)),
        value: Value::Node(NodeId(3)),
        receiver: NodeId(1),
    };
    let mut program = Program::checked(
        ObjectGrammar::MutationsV2,
        vec![node(vec![]), node(vec![]), node(vec![]), node(vec![])],
        vec!["unused", "kept"],
        vec![root(0), root(1)],
        false,
        vec![operation],
    )
    .unwrap();
    program.roots.remove(0);
    let pruned = program.prune().unwrap();
    assert_eq!(pruned.nodes.len(), 3);
    assert_eq!(pruned.symbols, vec!["kept"]);
    assert_eq!(
        pruned.operations,
        vec![Operation::Set {
            target: NodeId(1),
            key: Key::Symbol(SymbolId(0)),
            value: Value::Node(NodeId(2)),
            receiver: NodeId(0)
        }]
    );
    assert!(Program::checked(
        ObjectGrammar::StaticV1,
        vec![node(vec![])],
        vec![],
        vec![root(0)],
        false,
        vec![Operation::PreventExtensions { target: NodeId(0) }]
    )
    .is_none());
    for invalid in [
        Operation::Get {
            target: NodeId(0),
            key: Key::String("x"),
            receiver: NodeId(1),
        },
        Operation::Define {
            target: NodeId(0),
            key: Key::Symbol(SymbolId(0)),
            value: Value::Undefined,
            writable: true,
            enumerable: true,
            configurable: true,
        },
        Operation::ArrayLength {
            target: NodeId(0),
            length: 1,
            writable: true,
        },
    ] {
        assert!(Program::checked(
            ObjectGrammar::MutationsV2,
            vec![node(vec![])],
            vec![],
            vec![root(0)],
            false,
            vec![invalid]
        )
        .is_none());
    }
}

#[test]
fn mutation_reductions_keep_observed_aliases_and_the_selected_grammar() {
    let plan = ObjectGenerationPlan::for_grammar(ObjectGrammar::MutationsV2, 9, 4, 6, 16).unwrap();
    let program = generate(plan).unwrap();
    assert_eq!(program.operations.len(), 16);
    assert!(program
        .case(ObjectGenerationPlan::new(9, 4, 6).unwrap())
        .is_err());
    assert!(ObjectGenerationPlan::for_grammar(ObjectGrammar::MutationsV2, 9, 16, 6, 16).is_err());
    assert!(ObjectGenerationPlan::for_grammar(ObjectGrammar::MutationsV2, 9, 4, 6, 0).is_err());
    assert!(ObjectGenerationPlan::for_grammar(ObjectGrammar::StaticV1, 9, 4, 6, 1).is_err());
    let case = program.case(plan).unwrap();
    assert_eq!(case, generate_object_probe_case(plan).unwrap());
    super::super::object_probe::validate_body(case.source()).unwrap();
    let candidates = program.reductions();
    assert!(candidates
        .iter()
        .any(|candidate| candidate.operations.len() < program.operations.len()));
    for candidate in candidates {
        assert!(candidate.complexity() < program.complexity());
        assert_eq!(candidate.grammar, ObjectGrammar::MutationsV2);
        assert!(candidate
            .operations
            .iter()
            .all(|operation| operation.valid(&candidate.nodes, candidate.symbols.len())));
        assert!(candidate.reachable().iter().all(|used| *used));
        super::super::object_probe::validate_body(&candidate.source()).unwrap();
    }
}
#[test]
fn graph_pruning_reindexes_shared_cycle_edges_and_symbols_without_redirecting_them() {
    let program = Program::new(
        vec![
            node(vec![]),
            node(vec![data(Value::Node(NodeId(2)))]),
            node(vec![data(Value::Node(NodeId(1)))]),
        ],
        vec!["unused", "shared"],
        vec![root(0), root(1)],
        false,
    )
    .unwrap();
    let mut program = program;
    program.roots.remove(0);
    let pruned = program.prune().unwrap();
    assert_eq!(pruned.nodes.len(), 2);
    assert_eq!(pruned.roots[0].name, 1);
    assert_eq!(pruned.roots[0].node, NodeId(0));
    assert_eq!(
        pruned.nodes[0].properties[0].descriptor.value(),
        &Value::Node(NodeId(1))
    );
    assert_eq!(
        pruned.nodes[1].properties[0].descriptor.value(),
        &Value::Node(NodeId(0))
    );
    assert!(pruned.symbols.is_empty());
    let program = Program::new(
        vec![node(vec![data(Value::Symbol(SymbolId(1)))])],
        vec!["unused", "shared"],
        vec![root(0)],
        false,
    )
    .unwrap()
    .prune()
    .unwrap();
    assert_eq!(program.symbols, vec!["shared"]);
    assert_eq!(
        program.nodes[0].properties[0].descriptor.value(),
        &Value::Symbol(SymbolId(0))
    );
}
#[test]
fn deterministic_rendering_keeps_plan_identity_and_every_candidate_strictly_smaller() {
    let program = generate(plan()).unwrap();
    let case = program.case(plan()).unwrap();
    assert_eq!(case, generate_object_probe_case(plan()).unwrap());
    assert_eq!(
        case.protocol(),
        DifferentialProtocol::V5SelectedObjectProbePrintTranscript
    );
    assert!(case.source().contains("value:n0"));
    assert!(case.source().contains("Object.defineProperty"));
    assert_ne!(
        case.id(),
        generate_object_probe_case(ObjectGenerationPlan::new(7, 2, 4).unwrap())
            .unwrap()
            .id()
    );
    // The actual existing FunctionBody admission, not a second generator parser.
    super::super::object_probe::validate_body(case.source()).unwrap();
    for candidate in program.reductions() {
        assert!(candidate.complexity() < program.complexity());
        assert!(candidate.reachable().iter().all(|used| *used));
        super::super::object_probe::validate_body(&candidate.source()).unwrap();
    }
}
fn observation(backend: DifferentialBackend, value: bool) -> BackendObservation {
    let graph: SelectedObjectProbeGraph = serde_json::from_value(serde_json::json!({
        "version": 1, "roots": [{"name": "root", "value": {"type": "boolean", "value": value}}],
        "anchors": [], "nodes": [], "symbols": []
    }))
    .unwrap();
    BackendObservation {
        worker_identity: None,
        backend,
        output_events: OutputEventsObservation::Captured { events: vec![] },
        execution: ExecutionObservation::SelectedObjectProbe {
            graph,
            backend_note: "test observation".into(),
        },
    }
}
fn compare(case: &DifferentialReplayInput, differs: bool) -> DifferentialReport {
    super::super::compare_observations(
        case,
        observation(DifferentialBackend::WasmAot, false),
        observation(DifferentialBackend::SpecExec, differs),
    )
}
#[test]
fn reducer_preserves_graph_difference_while_source_fingerprints_change() {
    let first = compare(&generate_object_probe_case(plan()).unwrap(), true);
    let outcome = run_with_replay(
        plan(),
        ArithmeticReductionLimit::new(512).unwrap(),
        |case| {
            Ok(compare(
                case,
                case.source().contains("Object.defineProperty"),
            ))
        },
    )
    .unwrap();
    let Outcome::ReducedMismatch {
        case,
        report,
        reduction,
    } = outcome
    else {
        panic!("expected a retained graph mismatch");
    };
    assert!(case.source().contains("Object.defineProperty"));
    assert!(reduction.accepted_reductions > 0);
    assert_eq!(Witness::from_report(&first), Witness::from_report(&report));
    assert_ne!(first.mismatch_signature(), report.mismatch_signature());
    assert_eq!(report, compare(&case, true));
}
#[test]
fn reducer_rejects_worker_failure_and_does_not_trade_graph_error_for_backend_error() {
    let original = compare(&generate_object_probe_case(plan()).unwrap(), true);
    let mut backend_failure = observation(DifferentialBackend::WasmAot, false);
    backend_failure.execution = ExecutionObservation::EngineFailure {
        phase: FailurePhase::Lowering,
        message: "failed".into(),
    };
    let backend = super::super::compare_observations(
        &generate_object_probe_case(plan()).unwrap(),
        backend_failure,
        observation(DifferentialBackend::SpecExec, true),
    );
    assert_ne!(
        Witness::from_report(&original),
        Witness::from_report(&backend)
    );
    let mut calls = 0;
    let outcome = run_with_replay(plan(), ArithmeticReductionLimit::new(8).unwrap(), |case| {
        calls += 1;
        if calls == 1 {
            return Ok(compare(case, true));
        }
        let mut failed = observation(DifferentialBackend::WasmAot, false);
        failed.execution = ExecutionObservation::WorkerFailure {
            failure: DifferentialWorkerFailure::Protocol {
                message: "bound header missing".into(),
            },
            cleanup_error: None,
        };
        Ok(super::super::compare_observations(
            case,
            failed,
            observation(DifferentialBackend::SpecExec, true),
        ))
    })
    .unwrap();
    let Outcome::Rejected { report } = outcome else {
        panic!("worker failure cannot be an accepted reduction");
    };
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(report.mismatch_signature().is_none());
    assert_eq!(calls, 2);
}
