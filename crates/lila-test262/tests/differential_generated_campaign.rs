//! Actual selected-worker campaign controls; authored, not executed in this batch.
use lila_test262::differential::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn negative_campaigns_retain_actual_frontend_phases_outside_the_executable_corpus() {
    for seed in [0, 2, 8, 10] {
        let output = Output::new();
        let generated = NegativeGenerationPlan::new(seed, 3, 2).unwrap();
        let plan = GeneratedCampaignPlan::for_negative_source(
            generated,
            1,
            ArithmeticReductionLimit::new(8).unwrap(),
        )
        .unwrap();
        let aggregate = run_generated_campaign(
            plan,
            SpecExecOracle::explicitly_enabled(),
            &runner(),
            &GeneratedCampaignCancellation::default(),
            &output.0,
        )
        .unwrap();
        assert!(aggregate.is_green());
        assert_eq!(aggregate.completed(), 1);
        assert_eq!(
            std::fs::read_dir(output.0.join("corpus")).unwrap().count(),
            0
        );
        let retained =
            DifferentialReplayInput::load(output.0.join("evidence/case-000.negative.json"))
                .unwrap();
        let expected = generate_negative_case(generated).unwrap();
        assert_eq!(&retained, expected.input());
        let record: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.0.join("evidence/case-000.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(record["outcome"], "frontend_negative");
        let observed = &record["observation"];
        assert_eq!(observed["expected_rejection"], true);
        assert_eq!(observed["report"]["verdict"], "both_failed");
        assert_eq!(
            observed["report"]["wasm_aot"]["execution"]["phase"],
            match expected.expected_phase() {
                NegativeSourcePhase::Parse => "parse",
                NegativeSourcePhase::EarlyError => "early_error",
            }
        );
        assert_eq!(
            observed["report"]["spec_exec"]["execution"]["phase"],
            "spec_exec_entry_syntax"
        );
        assert!(observed["mismatch"].is_null());
    }
}

struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
                "lila-generated-campaign-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            )))
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // A failed assertion must not erase the requests and worker reports
            // needed to diagnose it. Logging must not cause a second panic.
            use std::io::Write as _;
            if self.0.is_dir() {
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "generated campaign evidence retained at: {}",
                    self.0.display()
                );
            }
            return;
        }
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn plan(seed: u64, count: usize) -> Result<GeneratedCampaignPlan, DifferentialError> {
    GeneratedCampaignPlan::new(
        ArithmeticGenerationPlan::new(
            ArithmeticGrammar::IntegerArithmeticV1,
            ArithmeticGenerationSeed::new(seed),
            ArithmeticCheckCount::new(4).unwrap(),
            ArithmeticExpressionDepth::Two,
        ),
        count,
        ArithmeticReductionLimit::new(16).unwrap(),
    )
}

#[test]
fn seed_inventory_is_bounded_and_never_wraps_into_replayed_identity() {
    assert!(plan(1, 0).is_err());
    assert!(plan(1, MAX_GENERATED_CAMPAIGN_CASES + 1).is_err());
    assert!(plan(u64::MAX, 2).is_err());
    assert_eq!(plan(u64::MAX, 1).unwrap().count(), 1);
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
fn runner() -> DifferentialWorkerRunner {
    DifferentialWorkerRunner::new(env!("CARGO_BIN_EXE_lila-differential-worker")).unwrap()
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn consecutive_cases_retain_the_actual_replay_identity_and_fresh_output() {
    let output = Output::new();
    let report = run_generated_campaign(
        plan(1, 2).unwrap(),
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green());
    let first = DifferentialReplayInput::load(output.0.join("corpus/case-000.json")).unwrap();
    let pinned = DifferentialReplayInput::from_json(include_str!(
        "differential/v1/t25-generated-integer-arithmetic-v1-seed-1.json"
    ))
    .unwrap();
    assert_eq!(first.source(), pinned.source());
    let repeated = replay_case(&first, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
    let journal: serde_json::Value = serde_json::from_slice(
        &std::fs::read(output.0.join("evidence/case-000.attempt-000.report.json")).unwrap(),
    )
    .unwrap();
    let replayed: serde_json::Value =
        serde_json::from_str(&repeated.to_pretty_json().unwrap()).unwrap();
    assert_eq!(
        journal["report"]["case_fingerprint"],
        replayed["case_fingerprint"]
    );
    assert!(journal["report"]["case_fingerprint"].as_str().is_some());
    let second = DifferentialReplayInput::load(output.0.join("corpus/case-001.json")).unwrap();
    assert_ne!(first.id(), second.id());
    assert!(run_generated_campaign(
        plan(1, 2).unwrap(),
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0
    )
    .is_err());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn cancellation_before_replay_retains_pending_inventory_without_green_or_cases() {
    let output = Output::new();
    let cancellation = GeneratedCampaignCancellation::default();
    cancellation.cancel();
    let report = run_generated_campaign(
        plan(1, 2).unwrap(),
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &cancellation,
        &output.0,
    )
    .unwrap();
    assert_eq!(report.verdict(), GeneratedCampaignVerdict::Cancelled);
    assert_eq!(report.completed(), 0);
    assert!(!report.is_green());
    assert_eq!(
        std::fs::read_dir(output.0.join("corpus")).unwrap().count(),
        0
    );
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["cases"][0]["state"], "cancelled");
    assert_eq!(aggregate["cases"][1]["state"], "pending");
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn worker_failure_retains_exact_evidence_and_stops_without_corpus_persistence() {
    let output = Output::new();
    let failing_worker = DifferentialWorkerRunner::new(std::env::current_exe().unwrap()).unwrap();
    let report = run_generated_campaign(
        plan(1, 2).unwrap(),
        SpecExecOracle::explicitly_enabled(),
        &failing_worker,
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(!report.is_green());
    assert_eq!(report.completed(), 1);
    assert_eq!(report.failed(), 1);
    assert_eq!(report.verdict(), GeneratedCampaignVerdict::Incomplete);
    assert_eq!(
        std::fs::read_dir(output.0.join("corpus")).unwrap().count(),
        0
    );
    let evidence: serde_json::Value = serde_json::from_slice(
        &std::fs::read(output.0.join("evidence/case-000.attempt-000.report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(evidence["report"]["verdict"], "worker_failure");
    assert!(evidence["report"]["mismatch_signature"].is_null());
    assert!(output
        .0
        .join("evidence/case-000.attempt-000.request.json")
        .is_file());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn object_campaign_persists_actual_v5_cases_and_replays_the_same_graph_observation() {
    let output = Output::new();
    let plan = GeneratedCampaignPlan::for_objects(
        ObjectGenerationPlan::new(9, 2, 3).unwrap(),
        2,
        ArithmeticReductionLimit::new(16).unwrap(),
    )
    .unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green());
    let case = DifferentialReplayInput::load(output.0.join("corpus/case-000.json")).unwrap();
    assert_eq!(
        case,
        generate_object_probe_case(ObjectGenerationPlan::new(9, 2, 3).unwrap()).unwrap()
    );
    let replay = replay_case(&case, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
    assert_eq!(
        replay.verdict(),
        DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
    );
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["grammar"], OBJECT_PROBE_GRAMMAR);
    assert_eq!(aggregate["nodes"], 2);
    assert_eq!(aggregate["properties"], 3);
    assert!(aggregate.get("checks").is_none());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn mutation_campaign_compares_actual_operation_results_and_final_object_graphs() {
    let output = Output::new();
    let generated =
        ObjectGenerationPlan::for_grammar(ObjectGrammar::MutationsV2, 9, 4, 6, 16).unwrap();
    let plan = GeneratedCampaignPlan::for_objects(
        generated,
        2,
        ArithmeticReductionLimit::new(16).unwrap(),
    )
    .unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green());
    let case = DifferentialReplayInput::load(output.0.join("corpus/case-000.json")).unwrap();
    assert_eq!(case, generate_object_probe_case(generated).unwrap());
    let replay = replay_case(&case, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
    assert_eq!(
        replay.verdict(),
        DifferentialVerdict::SelectedObjectProbeAndPrintTranscriptMatch
    );
    let observed: serde_json::Value =
        serde_json::from_str(&replay.to_pretty_json().unwrap()).unwrap();
    assert!(serde_json::to_string(&observed)
        .unwrap()
        .contains("operation_results"));
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["grammar"], OBJECT_MUTATION_GRAMMAR);
    assert_eq!(aggregate["steps"], 16);
    assert_eq!(aggregate["nodes"], 4);
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn module_campaign_persists_the_complete_owner_and_replays_its_real_print_trace() {
    let output = Output::new();
    let plan = GeneratedCampaignPlan::for_modules(
        ModuleGenerationPlan::new(9, 3, 5).unwrap(),
        2,
        ArithmeticReductionLimit::new(16).unwrap(),
    )
    .unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green());
    let case = DifferentialReplayInput::load(output.0.join("corpus/case-000.json")).unwrap();
    assert_eq!(
        case,
        generate_module_graph_case(ModuleGenerationPlan::new(9, 3, 5).unwrap()).unwrap()
    );
    let replay = replay_case(&case, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
    assert_eq!(
        replay.verdict(),
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
    );
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["grammar"], MODULE_GRAPH_GRAMMAR);
    assert_eq!(aggregate["modules"], 3);
    assert_eq!(aggregate["edges"], 5);
    assert!(aggregate.get("checks").is_none());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn async_module_campaign_replays_real_awaits_dynamic_requests_and_exact_attributes() {
    let output = Output::new();
    let generated = ModuleGenerationPlan::for_grammar(ModuleGrammar::AsyncV2, 9, 3, 3).unwrap();
    let plan = GeneratedCampaignPlan::for_modules(
        generated,
        2,
        ArithmeticReductionLimit::new(16).unwrap(),
    )
    .unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green());
    let case = DifferentialReplayInput::load(output.0.join("corpus/case-000.json")).unwrap();
    assert_eq!(case, generate_module_graph_case(generated).unwrap());
    assert!(case.source().contains("await import("));
    assert!(case
        .module_graph()
        .unwrap()
        .resolutions()
        .iter()
        .any(|row| !row.request().attributes().is_empty()));
    let replay = replay_case(&case, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
    assert_eq!(
        replay.verdict(),
        DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
    );
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["grammar"], ASYNC_MODULE_GRAPH_GRAMMAR);
    assert_eq!(aggregate["modules"], 3);
    assert_eq!(aggregate["edges"], 3);
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn control_flow_campaign_retains_all_four_function_protocols_and_actual_traces() {
    let output = Output::new();
    let generated = ControlFlowGenerationPlan::new(4, 4, 3).unwrap();
    let plan = GeneratedCampaignPlan::for_control_flow(
        generated,
        4,
        ArithmeticReductionLimit::new(16).unwrap(),
    )
    .unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green(), "{report:?}");
    for ordinal in 0..4 {
        let case =
            DifferentialReplayInput::load(output.0.join(format!("corpus/case-{ordinal:03}.json")))
                .unwrap();
        assert_eq!(
            case,
            generate_control_flow_case(ControlFlowGenerationPlan::new(4 + ordinal, 4, 3).unwrap())
                .unwrap()
        );
        let replay = replay_case(&case, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
        assert_eq!(
            replay.verdict(),
            DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
        );
        for observation in [replay.wasm_aot(), replay.spec_exec()] {
            let OutputEventsObservation::Captured { events } = &observation.output_events else {
                panic!("selected workers must retain their actual driver transcript");
            };
            assert!(events
                .last()
                .is_some_and(|event| event.starts_with("state:")));
            assert!(events.iter().any(|event| event.starts_with("result:")));
            if matches!(ordinal, 1 | 3) {
                assert!(events.iter().any(|event| event == "request-finally"));
                assert!(events
                    .iter()
                    .any(|event| event.starts_with("request-finally-resume:")));
            }
        }
    }
    let aggregate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("aggregate.json")).unwrap()).unwrap();
    assert_eq!(aggregate["grammar"], CONTROL_FLOW_GRAMMAR);
    assert_eq!(aggregate["steps"], 4);
    assert_eq!(aggregate["depth"], 3);
    assert!(aggregate.get("checks").is_none());
    assert!(aggregate.get("nodes").is_none());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn generator_requests_reach_the_active_finalizer_before_any_generated_early_return() {
    // Return/Throw in both generator protocols; one real request checkpoint
    // precedes every random tree, so this cannot silently become Next-only.
    for seed in [9, 11, 17, 19] {
        let case = generate_control_flow_case(ControlFlowGenerationPlan::new(seed, 1, 1).unwrap())
            .unwrap();
        let report = replay_case(&case, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
        assert_eq!(
            report.verdict(),
            DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
        );
        for observation in [report.wasm_aot(), report.spec_exec()] {
            let OutputEventsObservation::Captured { events } = &observation.output_events else {
                panic!("requests must retain real print observations");
            };
            assert!(events.iter().any(|event| event == "request-finally"));
            assert!(events
                .iter()
                .any(|event| event == "request-finally-resume:3"));
            assert_eq!(events.last().map(String::as_str), Some("state:0"));
            if seed < 16 {
                assert!(events.iter().any(|event| event == "step:true:7"));
                assert!(events.iter().any(|event| event == "result:7"));
            } else {
                assert!(events.iter().any(|event| event == "throw:7"));
                assert!(events.iter().any(|event| event == "result:undefined"));
            }
        }
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn builtin_scenarios_use_actual_selected_workers_for_all_six_stateful_families() {
    let output = Output::new();
    let generated = ScenarioGenerationPlan::new(ScenarioGrammar::StatefulV1, 0, 8).unwrap();
    let plan = GeneratedCampaignPlan::for_scenarios(
        generated,
        6,
        ArithmeticReductionLimit::new(2).unwrap(),
    )
    .unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &runner(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(report.is_green(), "{report:?}");
    for seed in 0..6 {
        let input = DifferentialReplayInput::load(
            output
                .0
                .join(format!("corpus/case-{seed:03}.baseline.json")),
        )
        .unwrap();
        let generated = ScenarioGenerationPlan::new(ScenarioGrammar::StatefulV1, seed, 8).unwrap();
        assert_eq!(
            &input,
            generate_scenario_cases(generated).unwrap().baseline()
        );
        let evidence: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.0.join(format!("evidence/case-{seed:03}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            evidence["observations"]["baseline"]["case"]["source"],
            input.source()
        );
        assert_eq!(
            evidence["observations"]["baseline"]["report"]["verdict"],
            "primitive_completion_and_print_transcript_match"
        );
        assert_eq!(
            evidence["observations"]["metamorphic"]["verdict"],
            "not_requested"
        );
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn realm_and_temporal_campaigns_retain_all_eight_actions_and_actual_paired_observations() {
    // These two checked seeds each select every action exactly once. The
    // private generation control pins that coverage; no runtime trace is
    // predicted here. Sloppy Realm and strict Temporal both transform the
    // complete finite driver, including hooks, allocations and abrupt paths.
    for (seed, family, protocol, event_prefix) in [
        (
            1270,
            ScenarioFamily::CrossRealm,
            DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            "realm-",
        ),
        (
            3215,
            ScenarioFamily::Temporal,
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
            "temporal-",
        ),
    ] {
        let output = Output::new();
        let generated =
            ScenarioGenerationPlan::new(ScenarioGrammar::MetamorphicV2, seed, 8).unwrap();
        assert_eq!(generated.family(), family);
        let plan = GeneratedCampaignPlan::for_scenarios(
            generated,
            1,
            ArithmeticReductionLimit::new(2).unwrap(),
        )
        .unwrap();
        let report = run_generated_campaign(
            plan,
            SpecExecOracle::explicitly_enabled(),
            &runner(),
            &GeneratedCampaignCancellation::default(),
            &output.0,
        )
        .unwrap();
        assert!(report.is_green(), "{family:?}: {report:?}");
        let pair = ScenarioReplayPair::load(output.0.join("evidence/case-000.pair.json")).unwrap();
        assert_eq!(pair, ScenarioReplayPair::generate(generated).unwrap());
        for (variant, request) in [
            ("baseline", pair.cases().baseline()),
            ("transformed", pair.cases().transformed().unwrap()),
        ] {
            assert_eq!(request.protocol(), protocol);
            let input = DifferentialReplayInput::load(
                output.0.join(format!("corpus/case-000.{variant}.json")),
            )
            .unwrap();
            assert_eq!(&input, request);
        }
        let retained: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.0.join("evidence/case-000.json")).unwrap(),
        )
        .unwrap();
        let observations = &retained["observations"];
        assert_eq!(observations["metamorphic"]["verdict"], "observations_match");
        for variant in ["baseline", "transformed"] {
            let actual = &observations[variant]["report"];
            assert_eq!(actual["schema_version"], protocol.schema_version());
            assert_eq!(
                actual["verdict"],
                "primitive_completion_and_print_transcript_match"
            );
            for backend in ["wasm_aot", "spec_exec"] {
                let events = actual[backend]["output_events"]["events"]
                    .as_array()
                    .expect("captured actual print transcript");
                assert!(events.iter().any(|event| event
                    .as_str()
                    .is_some_and(|event| event.starts_with(event_prefix))));
                assert_eq!(
                    events
                        .iter()
                        .filter(|event| event
                            .as_str()
                            .is_some_and(|event| event.starts_with("scenario-step:")))
                        .count(),
                    8
                );
                assert!(events
                    .last()
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .starts_with("scenario-done:"));
            }
        }
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn actual_metamorphic_campaign_retains_a_one_command_replayable_pair_for_each_transform() {
    // Real Promise jobs exercise rename; Array mutation exercises neutral
    // lexical blocks and the equivalent finite loop. No expected trace model.
    for seed in [5, 6, 12] {
        let output = Output::new();
        let generated =
            ScenarioGenerationPlan::new(ScenarioGrammar::MetamorphicV1, seed, 8).unwrap();
        let plan = GeneratedCampaignPlan::for_scenarios(
            generated,
            1,
            ArithmeticReductionLimit::new(2).unwrap(),
        )
        .unwrap();
        let report = run_generated_campaign(
            plan,
            SpecExecOracle::explicitly_enabled(),
            &runner(),
            &GeneratedCampaignCancellation::default(),
            &output.0,
        )
        .unwrap();
        assert!(report.is_green(), "{report:?}");
        let pair = ScenarioReplayPair::load(output.0.join("evidence/case-000.pair.json")).unwrap();
        assert_eq!(pair, ScenarioReplayPair::generate(generated).unwrap());
        assert_eq!(
            pair.cases().baseline(),
            &DifferentialReplayInput::load(output.0.join("corpus/case-000.baseline.json")).unwrap()
        );
        assert_eq!(
            pair.cases().transformed().unwrap(),
            &DifferentialReplayInput::load(output.0.join("corpus/case-000.transformed.json"))
                .unwrap()
        );
        let replay =
            replay_scenario_pair(&pair, SpecExecOracle::explicitly_enabled(), &runner()).unwrap();
        assert!(replay.is_green());
        assert_eq!(replay.metamorphic(), MetamorphicVerdict::ObservationsMatch);
        for actual in [replay.baseline(), replay.transformed().unwrap()] {
            assert_eq!(
                actual.report().verdict(),
                DifferentialVerdict::PrimitiveCompletionAndPrintTranscriptMatch
            );
            for observed in [actual.report().wasm_aot(), actual.report().spec_exec()] {
                let OutputEventsObservation::Captured { events } = &observed.output_events else {
                    panic!("real transcript required");
                };
                assert!(events
                    .last()
                    .is_some_and(|event| event.starts_with("scenario-done:")));
                assert_eq!(
                    events
                        .iter()
                        .filter(|event| event.starts_with("scenario-step:"))
                        .count(),
                    pair.expected_steps()
                );
            }
        }
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn scenario_worker_failure_keeps_the_transformed_source_pending_and_stops_the_campaign() {
    let output = Output::new();
    let generated = ScenarioGenerationPlan::new(ScenarioGrammar::MetamorphicV1, 0, 2).unwrap();
    let plan = GeneratedCampaignPlan::for_scenarios(
        generated,
        2,
        ArithmeticReductionLimit::new(2).unwrap(),
    )
    .unwrap();
    let failing = DifferentialWorkerRunner::new(std::env::current_exe().unwrap()).unwrap();
    let report = run_generated_campaign(
        plan,
        SpecExecOracle::explicitly_enabled(),
        &failing,
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert!(!report.is_green());
    assert_eq!(report.completed(), 1);
    assert!(!output
        .0
        .join("evidence/case-000.attempt-001.request.json")
        .exists());
    assert_eq!(
        std::fs::read_dir(output.0.join("corpus")).unwrap().count(),
        0
    );
    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("evidence/case-000.json")).unwrap())
            .unwrap();
    assert_eq!(
        record["observations"]["baseline"]["report"]["verdict"],
        "worker_failure"
    );
    assert_eq!(
        record["observations"]["metamorphic"]["verdict"],
        "unavailable"
    );
    assert!(record["observations"]["transformed"].is_null());
    assert_eq!(
        record["observations"]["pending_transformed"]["source"],
        generate_scenario_cases(generated)
            .unwrap()
            .transformed()
            .unwrap()
            .source()
    );
    assert!(output.0.join("evidence/case-000.pair.json").is_file());
}
