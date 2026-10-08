use super::*;

fn plan(seed: u64) -> NegativeGenerationPlan {
    NegativeGenerationPlan::new(seed, 4, 3).unwrap()
}
fn phase(program: &Program) {
    let options = match program.goal {
        DifferentialGoal::Script => lila_front::ParseOptions::script(),
        DifferentialGoal::Module => lila_front::ParseOptions::module(),
    };
    let error = lila_front::parse(&program.source(), options)
        .expect_err("generated negative must be rejected");
    assert_eq!(
        error.diagnostic().error_type(),
        Some("SyntaxError"),
        "{}: {error:?}",
        program.source()
    );
    let actual = match error.diagnostic().phase() {
        lila_front::ParseDiagnosticPhase::Parse => NegativeSourcePhase::Parse,
        lila_front::ParseDiagnosticPhase::Early => NegativeSourcePhase::EarlyError,
    };
    assert_eq!(
        actual,
        program.family.phase(),
        "{}: {error:?}",
        program.source()
    );
}

#[test]
fn generated_and_reduced_negatives_keep_the_actual_goal_strictness_and_rejection_phase() {
    for seed in 0..32 {
        let plan = plan(seed);
        let program = generate(plan).unwrap();
        phase(&program);
        let case = generate_negative_case(plan).unwrap();
        assert_eq!(case.input(), program.case(plan).unwrap().input());
        assert_eq!(
            DifferentialReplayInput::from_json(&case.input().to_pretty_json().unwrap()).unwrap(),
            *case.input()
        );
        for candidate in program.reductions() {
            assert_eq!(candidate.family, program.family);
            assert_eq!(candidate.goal, program.goal);
            assert_eq!(candidate.strict, program.strict);
            assert!(candidate.complexity() < program.complexity());
            phase(&candidate);
            candidate.case(plan).unwrap();
        }
        assert!(program.case(plan.with_seed(seed ^ 1)).is_err());
    }
}

#[test]
fn source_owner_rejects_broken_context_and_size_claims_before_rendering() {
    for (steps, depth) in [(0, 1), (33, 1), (1, 0), (1, 5)] {
        assert!(NegativeGenerationPlan::new(0, steps, depth).is_err());
    }
    assert!(Program::new(
        NegativeSourceFamily::StrictDelete,
        DifferentialGoal::Script,
        false,
        1,
        0,
        0,
        vec![]
    )
    .is_none());
    assert!(Program::new(
        NegativeSourceFamily::DuplicateLexical,
        DifferentialGoal::Module,
        false,
        1,
        0,
        0,
        vec![]
    )
    .is_none());
    assert!(Program::new(
        NegativeSourceFamily::DuplicateLexical,
        DifferentialGoal::Script,
        false,
        1,
        32,
        0,
        vec![]
    )
    .is_none());
    let program = generate(plan(2)).unwrap();
    assert!(program
        .case(NegativeGenerationPlan::new(2, 1, 1).unwrap())
        .is_err());
}

fn observed(backend: DifferentialBackend, phase: FailurePhase) -> BackendObservation {
    BackendObservation {
        worker_identity: None,
        backend,
        output_events: OutputEventsObservation::Captured { events: vec![] },
        execution: ExecutionObservation::EngineFailure {
            phase,
            message: "actual test error".into(),
        },
    }
}
fn report(
    case: &DifferentialReplayInput,
    product: FailurePhase,
    oracle: FailurePhase,
) -> DifferentialReport {
    compare_observations(
        case,
        observed(DifferentialBackend::WasmAot, product),
        observed(DifferentialBackend::SpecExec, oracle),
    )
}

#[test]
fn expected_source_rejection_keeps_the_unmodified_both_failed_report() {
    for seed in [0, 2, 8, 10] {
        let generated = plan(seed);
        let expected = generated.family().phase().failure();
        let Outcome::Verified { observation } = run_with_replay(
            generated,
            ArithmeticReductionLimit::new(8).unwrap(),
            |case| Ok(report(case, expected, FailurePhase::SpecExecEntrySyntax)),
        )
        .unwrap() else {
            panic!("expected both entry frontends to reject");
        };
        assert!(observation.is_green());
        assert_eq!(
            observation.report.verdict(),
            DifferentialVerdict::BothFailed
        );
        assert!(!observation.report.is_green());
        assert!(observation.mismatch.is_none());
    }
}

#[test]
fn runtime_failure_capability_gap_and_accepted_source_do_not_pass_as_syntax_errors() {
    let case = generate_negative_case(plan(0)).unwrap();
    for product in [
        FailurePhase::FrontendCapability,
        FailurePhase::Lowering,
        FailurePhase::WasmRuntimeOrBackend,
    ] {
        let observed = observation(
            case.clone(),
            report(case.input(), product, FailurePhase::SpecExecEntrySyntax),
        )
        .unwrap();
        assert!(!observed.is_green());
        assert!(observed.mismatch.is_some());
    }
    let retained = observation(
        case.clone(),
        report(
            case.input(),
            FailurePhase::Parse,
            FailurePhase::SpecExecExecution,
        ),
    )
    .unwrap();
    assert!(!retained.is_green());
    let mut accepted = observed(DifferentialBackend::WasmAot, FailurePhase::Parse);
    accepted.execution = ExecutionObservation::PrimitiveCompletion {
        completion: PrimitiveCompletionObservation::Normal {
            value: PrimitiveValueObservation::Undefined,
        },
        backend_note: "unexpected acceptance".into(),
    };
    let compared = compare_observations(
        case.input(),
        accepted,
        observed(
            DifferentialBackend::SpecExec,
            FailurePhase::SpecExecEntrySyntax,
        ),
    );
    assert!(!observation(case, compared).unwrap().is_green());
}

#[test]
fn reduction_retains_both_actual_phases_and_the_reduced_source_identity() {
    let original = generate_negative_case(plan(2)).unwrap();
    let Outcome::ReducedMismatch {
        observation,
        reduction,
    } = run_with_replay(
        plan(2),
        ArithmeticReductionLimit::new(64).unwrap(),
        |case| {
            Ok(report(
                case,
                FailurePhase::Lowering,
                FailurePhase::SpecExecEntrySyntax,
            ))
        },
    )
    .unwrap()
    else {
        panic!("expected original lowering-phase mismatch");
    };
    assert!(reduction.accepted_reductions > 0);
    assert!(observation.case.source().len() < original.input().source().len());
    assert_eq!(
        observation.report,
        report(
            &observation.case,
            FailurePhase::Lowering,
            FailurePhase::SpecExecEntrySyntax
        )
    );
    assert!(observation
        .case
        .source()
        .contains("let binding_0; let binding_0;"));
    let witness = observation.mismatch.unwrap();
    assert_ne!(
        Some(witness),
        NegativeMismatchSignature::from_report(
            NegativeSourcePhase::EarlyError,
            &report(
                &observation.case,
                FailurePhase::WasmRuntimeOrBackend,
                FailurePhase::SpecExecEntrySyntax
            )
        )
    );
}

#[test]
fn foreign_reports_and_worker_failure_cannot_complete_or_minimize_a_negative_case() {
    let foreign = generate_negative_case(plan(9)).unwrap();
    assert!(matches!(
        run_with_replay(plan(0), ArithmeticReductionLimit::new(8).unwrap(), |_| Ok(
            report(
                foreign.input(),
                FailurePhase::Parse,
                FailurePhase::SpecExecEntrySyntax
            )
        )),
        Err(DifferentialError::GeneratorInvariant(_))
    ));
    let mut attempts = 0;
    let Outcome::Rejected { observation } =
        run_with_replay(plan(0), ArithmeticReductionLimit::new(8).unwrap(), |case| {
            attempts += 1;
            if attempts == 1 {
                return Ok(report(
                    case,
                    FailurePhase::Lowering,
                    FailurePhase::SpecExecEntrySyntax,
                ));
            }
            let mut wasm = observed(DifferentialBackend::WasmAot, FailurePhase::Lowering);
            wasm.execution = ExecutionObservation::WorkerFailure {
                failure: DifferentialWorkerFailure::Timeout { timeout_ms: 5000 },
                cleanup_error: None,
            };
            Ok(compare_observations(
                case,
                wasm,
                observed(
                    DifferentialBackend::SpecExec,
                    FailurePhase::SpecExecEntrySyntax,
                ),
            ))
        })
        .unwrap()
    else {
        panic!("worker failure must terminate reduction");
    };
    assert_eq!(attempts, 2);
    assert!(observation.worker_failed());
    assert!(!observation.is_green());
    assert!(observation.mismatch.is_none());
}
