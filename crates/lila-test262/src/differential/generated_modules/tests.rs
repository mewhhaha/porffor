use super::*;
fn module(label: u8, targets: &[u8]) -> Module {
    Module {
        label,
        initial: 7,
        metadata: true,
        await_before: false,
        edges: targets
            .iter()
            .map(|target| Edge {
                target: ModuleId(*target),
                load: EdgeLoad::Static {
                    namespace: true,
                    reexport: true,
                },
                increment: true,
            })
            .collect(),
    }
}
fn plan() -> ModuleGenerationPlan {
    ModuleGenerationPlan::new(9, 3, 5).unwrap()
}
#[test]
fn graph_constructor_refuses_absent_targets_duplicate_rows_and_unreachable_source() {
    assert!(ModuleGenerationPlan::new(0, 0, 0).is_err());
    assert!(ModuleGenerationPlan::new(0, 4, 2).is_err());
    assert!(ModuleGenerationPlan::new(0, 4, 17).is_err());
    assert!(Program::new(ModuleGrammar::StaticV1, vec![module(0, &[1])]).is_none());
    assert!(Program::new(ModuleGrammar::StaticV1, vec![module(0, &[0, 0])]).is_none());
    assert!(Program::new(
        ModuleGrammar::StaticV1,
        vec![module(0, &[]), module(1, &[])]
    )
    .is_none());
    assert!(Program::new(
        ModuleGrammar::StaticV1,
        vec![module(0, &[1]), module(0, &[0])]
    )
    .is_none());
    assert!(Program::new(
        ModuleGrammar::StaticV1,
        vec![module(0, &[1]), module(1, &[0])]
    )
    .is_some());
}
#[test]
fn dependency_pruning_reindexes_edges_and_preserves_real_source_identities() {
    let mut program = Program::new(
        ModuleGrammar::StaticV1,
        vec![module(0, &[1, 2]), module(1, &[]), module(2, &[0])],
    )
    .unwrap();
    program.modules[0].edges.remove(0);
    let pruned = program.prune().unwrap();
    assert_eq!(pruned.modules.len(), 2);
    assert_eq!(pruned.modules[1].label, 2);
    assert_eq!(pruned.modules[0].edges[0].target, ModuleId(1));
    assert_eq!(pruned.modules[1].edges[0].target, ModuleId(0));
    let case = pruned.case(plan()).unwrap();
    let graph = case.module_graph().unwrap();
    assert!(graph.module("module-1.mjs").is_none());
    assert!(graph.module("module-2.mjs").is_some());
    assert_eq!(graph.resolutions().len(), 2);
    assert!(case.source().contains("'./module-2.mjs'"));
    assert_eq!(
        DifferentialReplayInput::from_json(&case.to_pretty_json().unwrap()).unwrap(),
        case
    );
}
#[test]
fn generation_and_every_reduction_publish_one_complete_exact_graph() {
    let program = generate(plan()).unwrap();
    let case = generate_module_graph_case(plan()).unwrap();
    assert_eq!(case, program.case(plan()).unwrap());
    assert_eq!(
        case.protocol(),
        DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
    );
    let graph = case.module_graph().unwrap();
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(graph.resolutions().len(), 5);
    assert_eq!(case.source(), graph.entry().source());
    assert!(case.source().contains("import.meta.url"));
    for row in graph.resolutions() {
        assert!(graph.module(row.target()).is_some());
        let EmbeddedModuleReferrer::Module(referrer) = row.referrer() else {
            panic!("all requests have actual module origins");
        };
        assert!(graph
            .module(referrer)
            .unwrap()
            .source()
            .contains(row.request().specifier()));
    }
    for reduced in program.reductions() {
        assert!(reduced.complexity() < program.complexity());
        let case = reduced.case(plan()).unwrap();
        assert_eq!(
            DifferentialReplayInput::from_json(&case.to_pretty_json().unwrap()).unwrap(),
            case
        );
    }
}
fn observation(backend: DifferentialBackend, differs: bool) -> BackendObservation {
    BackendObservation {
        worker_identity: None,
        backend,
        output_events: OutputEventsObservation::Captured {
            events: vec![if differs { "different" } else { "same" }.into()],
        },
        execution: ExecutionObservation::PrimitiveCompletion {
            completion: PrimitiveCompletionObservation::Normal {
                value: PrimitiveValueObservation::Undefined,
            },
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
fn reduction_retains_v4_print_difference_with_complete_changed_graph_fingerprints() {
    let original = compare(&generate_module_graph_case(plan()).unwrap(), true);
    let outcome = run_with_replay(
        plan(),
        ArithmeticReductionLimit::new(512).unwrap(),
        |case| Ok(compare(case, case.source().contains("live-"))),
    )
    .unwrap();
    let Outcome::ReducedMismatch {
        case,
        report,
        reduction,
    } = outcome
    else {
        panic!("expected retained print mismatch");
    };
    assert!(reduction.accepted_reductions > 0);
    assert!(case.source().contains("live-"));
    assert_eq!(
        Witness::from_report(&original),
        Witness::from_report(&report)
    );
    assert_ne!(original.mismatch_signature(), report.mismatch_signature());
    assert_eq!(report, compare(&case, true));
    let graph = case.module_graph().unwrap();
    for row in graph.resolutions() {
        assert!(graph.module(row.target()).is_some());
    }
}
#[test]
fn worker_failure_stops_module_reduction_without_persistable_case() {
    let mut calls = 0;
    let outcome = run_with_replay(plan(), ArithmeticReductionLimit::new(8).unwrap(), |case| {
        calls += 1;
        if calls == 1 {
            return Ok(compare(case, true));
        }
        let mut wasm = observation(DifferentialBackend::WasmAot, false);
        wasm.execution = ExecutionObservation::WorkerFailure {
            failure: DifferentialWorkerFailure::Protocol {
                message: "missing header".into(),
            },
            cleanup_error: None,
        };
        Ok(super::super::compare_observations(
            case,
            wasm,
            observation(DifferentialBackend::SpecExec, true),
        ))
    })
    .unwrap();
    let Outcome::Rejected { report } = outcome else {
        panic!("worker failure cannot be reduced corpus");
    };
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(report.mismatch_signature().is_none());
    assert_eq!(calls, 2);
}

#[test]
fn async_graph_authority_refuses_wait_cycles_and_foreign_grammar_features() {
    assert!(ModuleGenerationPlan::for_grammar(ModuleGrammar::AsyncV2, 0, 4, 7).is_err());
    assert!(ModuleGenerationPlan::for_grammar(ModuleGrammar::AsyncV2, 0, 4, 6).is_ok());
    assert!(ModuleGenerationPlan::for_grammar(ModuleGrammar::AsyncV2, 0, 1, 0).is_ok());
    assert!(Program::new(
        ModuleGrammar::AsyncV2,
        vec![module(0, &[1]), module(1, &[0])]
    )
    .is_none());
    assert!(Program::new(ModuleGrammar::AsyncV2, vec![module(0, &[0])]).is_none());
    let mut modules = vec![module(0, &[1]), module(1, &[])];
    modules[0].edges[0].load = EdgeLoad::Dynamic {
        computed_specifier: true,
        attributes: ImportAttributes::Blue,
    };
    assert!(Program::new(ModuleGrammar::StaticV1, modules.clone()).is_none());
    let program = Program::new(ModuleGrammar::AsyncV2, modules).unwrap();
    assert!(
        program.case(plan()).is_err(),
        "a static filename cannot publish an async grammar"
    );
    let mut eager = module(0, &[]);
    eager.await_before = true;
    assert!(Program::new(ModuleGrammar::StaticV1, vec![eager]).is_none());
}

#[test]
fn async_reductions_preserve_declared_requests_attributes_and_topological_ownership() {
    let plan = ModuleGenerationPlan::for_grammar(ModuleGrammar::AsyncV2, 9, 4, 5).unwrap();
    let original = generate(plan).unwrap();
    let first = original.case(plan).unwrap();
    assert!(first.source().contains("await Promise.resolve()"));
    assert!(first
        .source()
        .contains("await import(('./'+'module-1.mjs'),{with:{kind:'probe',flavor:'blue'}})"));
    let request = first
        .module_graph()
        .unwrap()
        .resolutions()
        .iter()
        .find(|row| {
            row.referrer() == &EmbeddedModuleReferrer::Module("module-0.mjs".into())
                && row.target() == "module-1.mjs"
        })
        .unwrap();
    assert_eq!(
        request.request().attributes(),
        [
            ("flavor".into(), "blue".into()),
            ("kind".into(), "probe".into())
        ]
    );
    for candidate in original.reductions() {
        assert_eq!(candidate.grammar, ModuleGrammar::AsyncV2);
        assert!(candidate.complexity() < original.complexity());
        for (index, module) in candidate.modules.iter().enumerate() {
            assert!(module
                .edges
                .iter()
                .all(|edge| usize::from(edge.target.0) > index));
        }
        let case = candidate.case(plan).unwrap();
        assert_eq!(
            DifferentialReplayInput::from_json(&case.to_pretty_json().unwrap()).unwrap(),
            case
        );
        for row in case.module_graph().unwrap().resolutions() {
            assert!(case.module_graph().unwrap().module(row.target()).is_some());
        }
    }
    let Outcome::ReducedMismatch {
        case,
        report,
        reduction,
    } = run_with_replay(plan, ArithmeticReductionLimit::new(512).unwrap(), |case| {
        Ok(compare(case, case.source().contains("await import(")))
    })
    .unwrap()
    else {
        panic!("dynamic request keeps the original print mismatch domain");
    };
    assert!(reduction.accepted_reductions > 0);
    assert!(case.source().contains("await import("));
    assert_eq!(report, compare(&case, true));
}
