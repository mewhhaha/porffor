use super::*;

fn plan(seed: u64, grammar: ScenarioGrammar) -> ScenarioGenerationPlan {
    ScenarioGenerationPlan::new(grammar, seed, 8).unwrap()
}
fn all_operations(family: ScenarioFamily) -> Program {
    Program::new(
        family,
        false,
        CHOICES
            .iter()
            .enumerate()
            .map(|(index, choice)| Action {
                operation: Operation::selected(family, *choice),
                slot: Slot(index as u8),
                value: SmallValue(index as i8 - 4),
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn checked_scenarios_reject_foreign_operations_and_unbounded_operands() {
    for steps in [0, MAX_CONTROL_FLOW_STEPS + 1, usize::MAX] {
        assert!(ScenarioGenerationPlan::new(ScenarioGrammar::StatefulV1, 0, steps).is_err());
    }
    assert!(Program::new(ScenarioFamily::ArrayAndBuffer, false, vec![]).is_none());
    let action = Action {
        operation: Operation::Array(ArrayOperation::Push),
        slot: Slot(0),
        value: SmallValue(1),
    };
    assert!(Program::new(ScenarioFamily::Collections, false, vec![action]).is_none());
    assert!(Program::new(
        ScenarioFamily::ArrayAndBuffer,
        false,
        vec![Action {
            slot: Slot(8),
            ..action
        }]
    )
    .is_none());
    assert!(Program::new(
        ScenarioFamily::ArrayAndBuffer,
        false,
        vec![Action {
            value: SmallValue(-17),
            ..action
        }]
    )
    .is_none());
    assert!(Program::new(
        ScenarioFamily::ArrayAndBuffer,
        false,
        vec![action; MAX_CONTROL_FLOW_STEPS + 1]
    )
    .is_none());
    assert_eq!(
        ScenarioGrammar::from_name(BUILTIN_STATEFUL_GRAMMAR),
        Some(ScenarioGrammar::StatefulV1)
    );
    assert_eq!(
        ScenarioGrammar::from_name(METAMORPHIC_STATEFUL_GRAMMAR),
        Some(ScenarioGrammar::MetamorphicV1)
    );
    assert_eq!(ScenarioGrammar::from_name("realm-or-temporal-v1"), None);
    assert_eq!(
        ScenarioGrammar::from_name(BUILTIN_STATEFUL_GRAMMAR_V2),
        Some(ScenarioGrammar::StatefulV2)
    );
    assert_eq!(
        ScenarioGrammar::from_name(METAMORPHIC_STATEFUL_GRAMMAR_V2),
        Some(ScenarioGrammar::MetamorphicV2)
    );
}

#[test]
fn v2_services_do_not_reinterpret_retained_v1_plan_authority() {
    let old = plan(6, ScenarioGrammar::MetamorphicV1);
    let new = plan(6, ScenarioGrammar::MetamorphicV2);
    assert_eq!(old.family(), ScenarioFamily::ArrayAndBuffer);
    assert_eq!(
        old.transformation(),
        Some(MetamorphicTransformation::NeutralBlocks)
    );
    assert_eq!(new.family(), ScenarioFamily::CrossRealm);
    assert_eq!(
        new.transformation(),
        Some(MetamorphicTransformation::BindingRename)
    );
    assert!(plan(18, ScenarioGrammar::MetamorphicV1).strict());
    assert!(!plan(18, ScenarioGrammar::MetamorphicV2).strict());
    for (seed, family) in [
        (1270, ScenarioFamily::CrossRealm),
        (3215, ScenarioFamily::Temporal),
    ] {
        let plan = plan(seed, ScenarioGrammar::MetamorphicV2);
        let program = generate(plan).unwrap();
        assert_eq!(plan.family(), family);
        assert!(
            CHOICES.iter().all(|choice| program
                .actions
                .iter()
                .any(|action| action.operation == Operation::selected(family, *choice))),
            "the focused real-worker fixture must exercise all eight actions"
        );
        let pair = ScenarioReplayPair::from_program(program, plan).unwrap();
        let original = serde_json::to_value(&pair).unwrap();
        for mutation in 0..4 {
            let mut wire = original.clone();
            match mutation {
                0 => wire["plan"]["grammar"] = METAMORPHIC_STATEFUL_GRAMMAR.into(),
                1 => {
                    wire["baseline"]["schema_version"] = if family == ScenarioFamily::CrossRealm {
                        3.into()
                    } else {
                        6.into()
                    }
                }
                2 => {
                    wire["transformed"]["schema_version"] = if family == ScenarioFamily::CrossRealm
                    {
                        3.into()
                    } else {
                        6.into()
                    }
                }
                3 => wire["program"]["family"] = "array_and_buffer".into(),
                _ => unreachable!(),
            }
            assert!(ScenarioReplayPair::from_json(&wire.to_string()).is_err());
        }
    }
}

#[test]
fn every_builtin_action_and_metamorphic_variant_is_real_parsable_source() {
    for (grammar, count) in [
        (ScenarioGrammar::MetamorphicV1, 18),
        (ScenarioGrammar::MetamorphicV2, 24),
    ] {
        for seed in 0..count {
            let plan = plan(seed, grammar);
            let program = all_operations(plan.family());
            let cases = program.cases(plan).unwrap();
            assert_ne!(cases.baseline(), cases.transformed().unwrap());
            for case in [cases.baseline(), cases.transformed().unwrap()] {
                assert_eq!(case.protocol(), plan.family().protocol());
                assert_eq!(
                    DifferentialReplayInput::from_json(&case.to_pretty_json().unwrap()).unwrap(),
                    *case
                );
                lila_front::parse(case.source(), lila_front::ParseOptions::script()).unwrap();
            }
            match cases.transformation().unwrap() {
                MetamorphicTransformation::BindingRename => {
                    assert!(cases.baseline().source().contains("let scenario_state=0"));
                    assert!(cases
                        .transformed()
                        .unwrap()
                        .source()
                        .contains("let renamed_state=0"));
                    assert!(!cases
                        .transformed()
                        .unwrap()
                        .source()
                        .contains("scenario_state"));
                }
                MetamorphicTransformation::NeutralBlocks => assert!(cases
                    .transformed()
                    .unwrap()
                    .source()
                    .contains("case 0:{\n{\n")),
                MetamorphicTransformation::EquivalentFiniteLoop => {
                    assert!(cases
                        .baseline()
                        .source()
                        .contains("for(let scenario_index=0;"));
                    assert!(cases
                        .transformed()
                        .unwrap()
                        .source()
                        .contains("let scenario_index=0;while(scenario_index<8)"));
                    assert!(cases
                        .transformed()
                        .unwrap()
                        .source()
                        .contains("scenario_index+=1;"));
                }
            }
        }
    }
}

#[test]
fn generation_and_reduction_preserve_family_strictness_relation_and_source_budget() {
    for seed in 0..72 {
        for grammar in [
            ScenarioGrammar::StatefulV1,
            ScenarioGrammar::MetamorphicV1,
            ScenarioGrammar::StatefulV2,
            ScenarioGrammar::MetamorphicV2,
        ] {
            let plan = plan(seed, grammar);
            let program = generate(plan).unwrap();
            assert_eq!(
                program.cases(plan).unwrap(),
                generate_scenario_cases(plan).unwrap()
            );
            assert_eq!(
                generate_scenario_cases(plan).unwrap(),
                generate_scenario_cases(plan).unwrap()
            );
            for candidate in program.reductions() {
                assert!(candidate.complexity() < program.complexity());
                assert_eq!(candidate.family, program.family);
                assert_eq!(candidate.strict, program.strict);
                assert_eq!(
                    candidate.cases(plan).unwrap().transformation(),
                    plan.transformation()
                );
                assert!(candidate.cases(plan.with_seed(seed + 1)).is_err());
            }
            let too_small = ScenarioGenerationPlan::new(grammar, seed, 1).unwrap();
            assert!(program.cases(too_small).is_err());
        }
    }
}

fn completion(kind: CompletionKindObservation) -> PrimitiveCompletionObservation {
    match kind {
        CompletionKindObservation::Normal => PrimitiveCompletionObservation::Normal {
            value: PrimitiveValueObservation::Undefined,
        },
        CompletionKindObservation::Throw => PrimitiveCompletionObservation::Throw {
            value: PrimitiveValueObservation::Number {
                bits: "3ff0000000000000".into(),
            },
        },
    }
}
fn observation(backend: DifferentialBackend, events: Vec<String>) -> BackendObservation {
    BackendObservation {
        worker_identity: None,
        backend,
        output_events: OutputEventsObservation::Captured { events },
        execution: ExecutionObservation::PrimitiveCompletion {
            completion: completion(CompletionKindObservation::Normal),
            backend_note: "synthetic protocol unit input".into(),
        },
    }
}
fn structural_events(case: &DifferentialReplayInput) -> Vec<String> {
    // This is an adversarial observer fixture for progress admission, not a
    // scenario interpreter or a prediction of any builtin/hook values.
    let mut events: Vec<_> = (0..case.source().matches("case ").count())
        .map(|index| format!("scenario-step:{index}"))
        .collect();
    events.push("scenario-done:unit-progress".into());
    events
}
fn compared(
    case: &DifferentialReplayInput,
    wasm: BackendObservation,
    spec: BackendObservation,
) -> DifferentialReport {
    super::super::compare_observations(case, wasm, spec)
}
fn matching(case: &DifferentialReplayInput) -> DifferentialReport {
    let events = structural_events(case);
    compared(
        case,
        observation(DifferentialBackend::WasmAot, events.clone()),
        observation(DifferentialBackend::SpecExec, events),
    )
}

#[test]
fn matched_failed_or_unfinished_drivers_cannot_publish_green() {
    for seed in 0..6 {
        let plan = plan(seed, ScenarioGrammar::MetamorphicV1);
        for malformed in 0..4 {
            let outcome =
                run_with_replay(plan, ArithmeticReductionLimit::new(4).unwrap(), |case| {
                    let mut events = structural_events(case);
                    match malformed {
                        0 => events.clear(),
                        1 => {
                            events.pop();
                        }
                        2 => {
                            events.remove(0);
                        }
                        3 => {}
                        _ => unreachable!(),
                    }
                    let mut wasm = observation(DifferentialBackend::WasmAot, events.clone());
                    let mut spec = observation(DifferentialBackend::SpecExec, events);
                    if malformed == 3 {
                        wasm.execution = ExecutionObservation::EngineFailure {
                            phase: FailurePhase::Lowering,
                            message: "driver not executable".into(),
                        };
                        spec.execution = wasm.execution.clone();
                    }
                    Ok(compared(case, wasm, spec))
                })
                .unwrap();
            let Outcome::Rejected { observations } = outcome else {
                panic!("unavailable/unfinished observations must stay red");
            };
            assert!(!observations.is_green());
            assert!(observations.transformed.is_some());
        }
    }
}

#[test]
fn actual_metamorphic_difference_is_red_even_when_each_backend_pair_matches() {
    let plan = plan(12, ScenarioGrammar::MetamorphicV1);
    let initial = generate_scenario_cases(plan).unwrap();
    let outcome = run_with_replay(plan, ArithmeticReductionLimit::new(32).unwrap(), |case| {
        let mut events = structural_events(case);
        events.insert(
            0,
            if case.source().contains("while(scenario_index") {
                "variant-hook"
            } else {
                "baseline-hook"
            }
            .into(),
        );
        Ok(compared(
            case,
            observation(DifferentialBackend::WasmAot, events.clone()),
            observation(DifferentialBackend::SpecExec, events),
        ))
    })
    .unwrap();
    let Outcome::ReducedMismatch {
        observations,
        reduction,
    } = outcome
    else {
        panic!("the real cross-source difference is reducible");
    };
    assert!(reduction.accepted_reductions() > 0);
    assert_eq!(reduction.attempted_replays() % 2, 0);
    assert!(observations.baseline.case.source().len() < initial.baseline().source().len());
    assert_eq!(
        observations.baseline.report.verdict(),
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
    );
    assert_eq!(
        observations.transformed.as_ref().unwrap().report.verdict(),
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
    );
    assert!(!observations.is_green());
    assert!(matches!(
        observations.metamorphic,
        MetamorphicVerdict::Mismatch {
            wasm_aot: ScenarioDifference {
                completion: false,
                print: true,
                ..
            },
            spec_exec: ScenarioDifference {
                completion: false,
                print: true,
                ..
            }
        }
    ));
}

#[test]
fn reducer_keeps_phase_opposite_completion_and_each_mismatch_dimension() {
    let plan = plan(3, ScenarioGrammar::MetamorphicV1);
    let case = generate_scenario_cases(plan).unwrap().baseline;
    let events = structural_events(&case);
    let mut failed = observation(DifferentialBackend::WasmAot, events.clone());
    failed.execution = ExecutionObservation::EngineFailure {
        phase: FailurePhase::Lowering,
        message: "original phase".into(),
    };
    let original = DifferentialWitness::from_report(&compared(
        &case,
        failed.clone(),
        observation(DifferentialBackend::SpecExec, events.clone()),
    ))
    .unwrap();
    let mut different_phase = failed.clone();
    different_phase.execution = ExecutionObservation::EngineFailure {
        phase: FailurePhase::Parse,
        message: "foreign phase".into(),
    };
    assert_ne!(
        Some(original),
        DifferentialWitness::from_report(&compared(
            &case,
            different_phase,
            observation(DifferentialBackend::SpecExec, events.clone())
        ))
    );
    let mut thrown = observation(DifferentialBackend::SpecExec, events.clone());
    thrown.execution = ExecutionObservation::PrimitiveCompletion {
        completion: completion(CompletionKindObservation::Throw),
        backend_note: "opposite kind changed".into(),
    };
    assert_ne!(
        Some(original),
        DifferentialWitness::from_report(&compared(&case, failed, thrown))
    );
    let mut different_events = events.clone();
    different_events.insert(0, "actual-print-difference".into());
    let print_only = DifferentialWitness::from_report(&compared(
        &case,
        observation(DifferentialBackend::WasmAot, events.clone()),
        observation(DifferentialBackend::SpecExec, different_events.clone()),
    ))
    .unwrap();
    let mut changed_value = observation(DifferentialBackend::SpecExec, different_events);
    changed_value.execution = ExecutionObservation::PrimitiveCompletion {
        completion: PrimitiveCompletionObservation::Normal {
            value: PrimitiveValueObservation::Null,
        },
        backend_note: "extra dimension".into(),
    };
    assert_ne!(
        Some(print_only),
        DifferentialWitness::from_report(&compared(
            &case,
            observation(DifferentialBackend::WasmAot, events),
            changed_value
        ))
    );
}

#[test]
fn cancellation_between_variants_and_worker_failure_during_reduction_remain_red() {
    let plan = plan(2, ScenarioGrammar::MetamorphicV1);
    let mut calls = 0;
    let cancelled = run_with_replay(plan, ArithmeticReductionLimit::new(4).unwrap(), |case| {
        calls += 1;
        if calls == 2 {
            Err(DifferentialError::CampaignCancelled)
        } else {
            Ok(matching(case))
        }
    });
    assert!(matches!(
        cancelled,
        Err(DifferentialError::CampaignCancelled)
    ));
    assert_eq!(calls, 2);
    let mut calls = 0;
    let outcome = run_with_replay(plan, ArithmeticReductionLimit::new(4).unwrap(), |case| {
        calls += 1;
        let events = structural_events(case);
        let mut wasm = observation(DifferentialBackend::WasmAot, events.clone());
        let mut spec = observation(DifferentialBackend::SpecExec, events);
        if calls <= 2 {
            let OutputEventsObservation::Captured { events } = &mut spec.output_events else {
                unreachable!()
            };
            events.insert(0, "original-print-mismatch".into());
        } else {
            wasm.execution = ExecutionObservation::WorkerFailure {
                failure: DifferentialWorkerFailure::Protocol {
                    message: "missing worker header".into(),
                },
                cleanup_error: None,
            };
        }
        Ok(compared(case, wasm, spec))
    })
    .unwrap();
    let Outcome::Rejected { observations } = outcome else {
        panic!("worker failure must never yield a reduced successful case");
    };
    assert_eq!(
        calls, 3,
        "a failed baseline must not launch its transformed worker pair"
    );
    assert!(!observations.is_green());
    assert_eq!(
        observations.baseline.report.verdict(),
        DifferentialVerdict::WorkerFailure
    );
    assert!(observations.transformed.is_none());
    assert!(observations.pending_transformed().is_some());
    assert_eq!(observations.metamorphic(), MetamorphicVerdict::Unavailable);
}

#[test]
fn replay_identity_and_backend_relation_cannot_be_substituted() {
    let plan = plan(0, ScenarioGrammar::MetamorphicV1);
    let foreign = generate_scenario_cases(plan.with_seed(1)).unwrap();
    assert!(matches!(
        run_with_replay(plan, ArithmeticReductionLimit::new(2).unwrap(), |_| Ok(
            matching(foreign.baseline())
        )),
        Err(DifferentialError::GeneratorInvariant(_))
    ));
    let left = observation(DifferentialBackend::WasmAot, vec![]);
    let right = observation(DifferentialBackend::SpecExec, vec![]);
    assert!(difference(&left, &right).is_none());
}

#[test]
fn paired_wire_checks_actual_source_count_transform_domains_and_unknown_fields() {
    let plan = plan(12, ScenarioGrammar::MetamorphicV1);
    let pair = ScenarioReplayPair::generate(plan).unwrap();
    let json = pair.to_pretty_json().unwrap();
    assert_eq!(ScenarioReplayPair::from_json(&json).unwrap(), pair);
    assert!(ScenarioReplayPair::generate(plan.with_seed(0)).is_ok());
    assert!(ScenarioReplayPair::generate(
        ScenarioGenerationPlan::new(ScenarioGrammar::StatefulV1, 0, 8).unwrap()
    )
    .is_err());
    let original: serde_json::Value = serde_json::from_str(&json).unwrap();
    for mutation in 0..9 {
        let mut wire = original.clone();
        match mutation {
            0 => wire["schema_version"] = 2.into(),
            1 => wire["expected_steps"] = 1.into(),
            2 => wire["transformation"] = "binding_rename".into(),
            3 => wire["program"]["actions"][0]["slot"] = 8.into(),
            4 => wire["program"]["actions"][0]["value"] = 17.into(),
            5 => wire["program"]["strict"] = true.into(),
            6 => wire["program"]["extra"] = true.into(),
            7 => wire["baseline"]["source"] = "print('scenario-done:pretend');void 0;".into(),
            8 => wire["transformed"]["schema_version"] = 2.into(),
            _ => unreachable!(),
        }
        assert!(
            ScenarioReplayPair::from_json(&serde_json::to_string(&wire).unwrap()).is_err(),
            "mutation {mutation}"
        );
    }
    let candidate = generate(plan)
        .unwrap()
        .reductions()
        .into_iter()
        .find(|candidate| candidate.actions.len() < usize::from(plan.steps()))
        .unwrap();
    let reduced = ScenarioReplayPair::from_program(candidate, plan).unwrap();
    assert!(reduced.expected_steps() < usize::from(plan.steps()));
    assert_eq!(
        ScenarioReplayPair::from_json(&reduced.to_pretty_json().unwrap()).unwrap(),
        reduced
    );
}

#[test]
fn paired_replay_record_retains_both_actual_requests_and_untouched_reports() {
    let pair = ScenarioReplayPair::generate(plan(5, ScenarioGrammar::MetamorphicV1)).unwrap();
    let observations = pair.replay_with(&mut |case| Ok(matching(case))).unwrap();
    assert!(observations.is_green());
    assert_eq!(observations.replay_pair(), Some(&pair));
    let record = serde_json::to_value(&observations).unwrap();
    assert_eq!(
        record["baseline"]["case"]["source"],
        pair.cases().baseline().source()
    );
    assert_eq!(
        record["transformed"]["case"]["source"],
        pair.cases().transformed().unwrap().source()
    );
    assert_eq!(
        record["baseline"]["report"],
        serde_json::to_value(observations.baseline().report()).unwrap()
    );
    assert_eq!(
        record["transformed"]["report"],
        serde_json::to_value(observations.transformed().unwrap().report()).unwrap()
    );
    assert_eq!(record["metamorphic"]["verdict"], "observations_match");
}

#[test]
fn v2_reductions_preserve_service_authority_and_exact_pair() {
    for seed in [6, 7] {
        let plan = plan(seed, ScenarioGrammar::MetamorphicV2);
        let outcome = run_with_replay(plan, ArithmeticReductionLimit::new(16).unwrap(), |case| {
            assert_eq!(case.protocol(), plan.family().protocol());
            // Synthetic observed difference exercises the reducer contract;
            // the real builtin outputs are tested by the selected workers.
            let wasm_events = structural_events(case);
            let mut spec_events = wasm_events.clone();
            spec_events.insert(0, "synthetic-service-difference".into());
            Ok(compared(
                case,
                observation(DifferentialBackend::WasmAot, wasm_events),
                observation(DifferentialBackend::SpecExec, spec_events),
            ))
        })
        .unwrap();
        let Outcome::ReducedMismatch {
            observations,
            reduction,
        } = outcome
        else {
            panic!("a retained service print difference must enter actual reduction");
        };
        assert!(reduction.accepted_reductions() > 0);
        assert!(!observations.is_green());
        let pair = observations.replay_pair().unwrap();
        assert!(pair.expected_steps() < usize::from(plan.steps()));
        assert_eq!(pair.plan(), plan);
        assert_eq!(pair.cases().baseline().protocol(), plan.family().protocol());
        assert_eq!(
            pair.cases().transformed().unwrap().protocol(),
            plan.family().protocol()
        );
        assert_eq!(
            ScenarioReplayPair::from_json(&pair.to_pretty_json().unwrap()).unwrap(),
            *pair
        );
    }
}
