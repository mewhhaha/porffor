use super::*;

fn plan(seed: u64) -> ControlFlowGenerationPlan {
    ControlFlowGenerationPlan::new(seed, 6, 4).unwrap()
}
fn leaf(value: Leaf) -> Node {
    Node::Leaf(value)
}
fn checked(steps: Vec<Node>) -> Option<Program> {
    Program::new(Execution::Generator, true, Request::Next, steps)
}

#[test]
fn actual_source_owner_refuses_foreign_suspensions_and_orphaned_abrupt_targets() {
    for budgets in [(0, 1), (33, 1), (1, 0), (1, 5)] {
        assert!(ControlFlowGenerationPlan::new(0, budgets.0, budgets.1).is_err());
    }
    assert!(checked(vec![leaf(Leaf::Suspend(Suspension::Await, 0))]).is_none());
    assert!(Program::new(
        Execution::Async,
        false,
        Request::Next,
        vec![leaf(Leaf::Suspend(Suspension::Yield, 0))]
    )
    .is_none());
    assert!(checked(vec![leaf(Leaf::Break(LoopTarget::Current))]).is_none());
    assert!(checked(vec![Node::Loop {
        iterations: 1,
        body: Box::new(leaf(Leaf::Continue(LoopTarget::Outer)))
    }])
    .is_none());
    assert!(checked(vec![Node::Loop {
        iterations: 4,
        body: Box::new(leaf(Leaf::Observe))
    }])
    .is_none());
    let nested = Node::Loop {
        iterations: 2,
        body: Box::new(Node::Loop {
            iterations: 3,
            body: Box::new(Node::TryFinally {
                body: Box::new(leaf(Leaf::Continue(LoopTarget::Outer))),
                finalizer: Leaf::Suspend(Suspension::Yield, 1),
            }),
        }),
    };
    let program = checked(vec![nested]).unwrap();
    lila_front::parse(&program.source(), lila_front::ParseOptions::script()).unwrap();
    for candidate in program.reductions() {
        assert!(candidate.complexity() < program.complexity());
        lila_front::parse(&candidate.source(), lila_front::ParseOptions::script()).unwrap();
    }
    let deep = Node::Block(Box::new(Node::Block(Box::new(Node::Block(Box::new(
        Node::Block(Box::new(leaf(Leaf::Observe))),
    ))))));
    assert!(checked(vec![deep]).is_none());
    let shallow = ControlFlowGenerationPlan::new(1, 1, 1).unwrap();
    let two_steps = Program::new(
        Execution::Generator,
        false,
        Request::Next,
        vec![leaf(Leaf::Observe), leaf(Leaf::Observe)],
    )
    .unwrap();
    assert!(two_steps.case(shallow).is_err());
    let too_deep = Program::new(
        Execution::Generator,
        false,
        Request::Next,
        vec![Node::Block(Box::new(leaf(Leaf::Observe)))],
    )
    .unwrap();
    assert!(too_deep.case(shallow).is_err());
    let reduced = Program::new(
        Execution::Generator,
        false,
        Request::Next,
        vec![leaf(Leaf::Observe)],
    )
    .unwrap();
    assert!(reduced.case(shallow).is_ok());
}

#[test]
fn generated_and_reduced_programs_retain_strictness_protocol_and_real_request_schedule() {
    // All four execution kinds, both strictness modes and all request schedules.
    for seed in 0..24 {
        let plan = plan(seed);
        let program = generate(plan).unwrap();
        let case = generate_control_flow_case(plan).unwrap();
        assert_eq!(case, program.case(plan).unwrap());
        assert_eq!(
            case.protocol(),
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
        );
        assert_eq!(
            DifferentialReplayInput::from_json(&case.to_pretty_json().unwrap()).unwrap(),
            case
        );
        lila_front::parse(case.source(), lila_front::ParseOptions::script()).unwrap();
        if matches!(
            program.execution,
            Execution::Generator | Execution::AsyncGenerator
        ) {
            assert!(case.source().contains("try{yield state;"));
            assert!(case
                .source()
                .contains("print('request-finally');const received=yield state;"));
        }
        for candidate in program.reductions() {
            assert_eq!(candidate.execution, program.execution);
            assert_eq!(candidate.strict, program.strict);
            assert_eq!(candidate.request, program.request);
            assert!(candidate.complexity() < program.complexity());
            let case = candidate.case(plan).unwrap();
            lila_front::parse(case.source(), lila_front::ParseOptions::script()).unwrap();
        }
        let other_protocol = ControlFlowGenerationPlan::new(seed ^ 1, 6, 4).unwrap();
        assert!(program.case(other_protocol).is_err());
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
fn reduction_preserves_actual_print_difference_and_updates_the_replayed_fingerprint() {
    let plan = plan(7);
    let original = generate_control_flow_case(plan).unwrap();
    let original_report = compare(&original, true);
    let Outcome::ReducedMismatch {
        case,
        report,
        reduction,
    } = run_with_replay(plan, ArithmeticReductionLimit::new(512).unwrap(), |case| {
        Ok(compare(case, true))
    })
    .unwrap()
    else {
        panic!("expected a retained print mismatch");
    };
    assert!(reduction.accepted_reductions > 0);
    assert!(case.source().len() < original.source().len());
    assert!(case.source().starts_with("'use strict';"));
    assert!(case.source().contains("async function* subject()"));
    assert_eq!(
        Witness::from_report(&original_report),
        Witness::from_report(&report)
    );
    assert_ne!(
        original_report.mismatch_signature(),
        report.mismatch_signature()
    );
    assert_eq!(report, compare(&case, true));
}

#[test]
fn a_print_only_witness_does_not_accept_a_different_completion_or_backend_failure() {
    let case = generate_control_flow_case(plan(1)).unwrap();
    let original = Witness::from_report(&compare(&case, true)).unwrap();
    let mut foreign = observation(DifferentialBackend::SpecExec, true);
    foreign.execution = ExecutionObservation::PrimitiveCompletion {
        completion: PrimitiveCompletionObservation::Throw {
            value: PrimitiveValueObservation::Undefined,
        },
        backend_note: "changed completion".into(),
    };
    let report = super::super::compare_observations(
        &case,
        observation(DifferentialBackend::WasmAot, false),
        foreign.clone(),
    );
    assert_ne!(Witness::from_report(&report), Some(original));

    let mut failed = observation(DifferentialBackend::WasmAot, false);
    failed.execution = ExecutionObservation::EngineFailure {
        phase: FailurePhase::Lowering,
        message: "actual lowering failure".into(),
    };
    let normal = super::super::compare_observations(
        &case,
        failed.clone(),
        observation(DifferentialBackend::SpecExec, false),
    );
    let thrown = super::super::compare_observations(&case, failed, foreign);
    assert_ne!(Witness::from_report(&normal), Witness::from_report(&thrown));
}

#[test]
fn matching_missing_driver_or_finalizer_transcripts_remain_red() {
    for seed in 0..4 {
        assert!(run_with_replay(
            plan(seed),
            ArithmeticReductionLimit::new(8).unwrap(),
            |case| Ok(compare(case, false))
        )
        .is_err());
    }
    let make = |backend, events: Vec<String>| BackendObservation {
        backend,
        output_events: OutputEventsObservation::Captured { events },
        ..observation(backend, false)
    };
    let program = generate(plan(1)).unwrap();
    let case = program.case(plan(1)).unwrap();
    let events = vec!["result:7".into(), "state:0".into()];
    let missing = super::super::compare_observations(
        &case,
        make(DifferentialBackend::WasmAot, events.clone()),
        make(DifferentialBackend::SpecExec, events),
    );
    assert!(!has_finished_trace(&program, &missing));
    // This tests the admission guard's structural contract, not an expected
    // program trace or a production oracle.
    let events = vec![
        "request-finally".into(),
        "request-finally-resume:3".into(),
        "result:7".into(),
        "state:0".into(),
    ];
    let finished = super::super::compare_observations(
        &case,
        make(DifferentialBackend::WasmAot, events.clone()),
        make(DifferentialBackend::SpecExec, events),
    );
    assert!(has_finished_trace(&program, &finished));
}

#[test]
fn cancellation_during_reduction_never_publishes_the_earlier_accepted_case() {
    let mut calls = 0;
    let outcome = run_with_replay(plan(7), ArithmeticReductionLimit::new(8).unwrap(), |case| {
        calls += 1;
        if calls == 2 {
            return Err(DifferentialError::CampaignCancelled);
        }
        Ok(compare(case, true))
    });
    assert!(matches!(outcome, Err(DifferentialError::CampaignCancelled)));
    assert_eq!(calls, 2);
}

#[test]
fn worker_failure_during_reduction_keeps_red_evidence_and_no_persistable_case() {
    let mut calls = 0;
    let outcome = run_with_replay(plan(1), ArithmeticReductionLimit::new(8).unwrap(), |case| {
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
        panic!("a failed worker cannot publish a reduced case");
    };
    assert_eq!(calls, 2);
    assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
    assert!(report.mismatch_signature().is_none());
}
