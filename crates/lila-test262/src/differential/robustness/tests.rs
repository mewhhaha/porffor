use super::*;
mod native_targets;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "lila-robustness-unit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn input(bytes: &[u8]) -> RobustnessInput {
    RobustnessInput::new(
        "robustness/control",
        RobustnessTarget::Compiler {
            goal: DifferentialGoal::Script,
        },
        1000,
        bytes.into(),
    )
    .unwrap()
}
fn observed(
    input: &RobustnessInput,
    result: RobustnessResult,
    stages: Vec<RobustnessStage>,
) -> RobustnessObservation {
    RobustnessObservation {
        input: input.clone(),
        compiler_identity: Some(crate::CompilerProvenance::current().unwrap()),
        result,
        stages,
        journal_bytes_hex: String::new(),
        stderr_bytes_hex: String::new(),
        journal_error: None,
        interrupted_stage: None,
    }
}

#[test]
fn wire_preserves_every_byte_and_rejects_foreign_domains_or_noncanonical_encoding() {
    let input = input(&[0, 255, 128, b'\n', b'\0']);
    assert_eq!(
        RobustnessInput::from_json(&input.to_pretty_json().unwrap()).unwrap(),
        input
    );
    let base: serde_json::Value = serde_json::from_str(&input.to_pretty_json().unwrap()).unwrap();
    for (field, value) in [
        ("schema_version", serde_json::json!(2)),
        ("bytes_hex", serde_json::json!("FF")),
        ("bytes_hex", serde_json::json!("0")),
        ("timeout_ms", serde_json::json!(0)),
        ("unexpected", serde_json::json!(true)),
    ] {
        let mut damaged = base.clone();
        damaged[field] = value;
        assert!(RobustnessInput::from_json(&damaged.to_string()).is_err());
    }
    assert!(RobustnessTarget::from_name("uri").is_none());
    assert_ne!(
        input.fingerprint(),
        super::input::RobustnessInput::new(
            "robustness/control",
            RobustnessTarget::Frontmatter {},
            1000,
            input.bytes().to_vec()
        )
        .unwrap()
        .fingerprint()
    );
}

#[test]
fn every_native_target_rejects_foreign_fields_without_changing_its_existing_wire() {
    for (kind, target) in [
        ("ir_admission", RobustnessTarget::IrAdmission {}),
        ("prelude", RobustnessTarget::Prelude {}),
        (
            "filesystem_resolver",
            RobustnessTarget::FilesystemResolver {},
        ),
        ("frontmatter", RobustnessTarget::Frontmatter {}),
        ("snapshot", RobustnessTarget::Snapshot {}),
        ("corpus", RobustnessTarget::Corpus {}),
        ("module_graph", RobustnessTarget::ModuleGraph {}),
        ("report_observation", RobustnessTarget::ReportObservation {}),
    ] {
        let input =
            RobustnessInput::new("robustness/closed-target", target, 1000, vec![0xff]).unwrap();
        let original: serde_json::Value =
            serde_json::from_str(&input.to_pretty_json().unwrap()).unwrap();
        assert_eq!(original["target"], serde_json::json!({ "kind": kind }));
        assert_eq!(
            RobustnessInput::from_json(&original.to_string()).unwrap(),
            input
        );
        for (field, value) in [
            ("goal", serde_json::json!("script")),
            ("parser", serde_json::json!("json")),
            ("foreign", serde_json::Value::Null),
        ] {
            let mut damaged = original.clone();
            damaged["target"][field] = value;
            assert!(
                RobustnessInput::from_json(&damaged.to_string()).is_err(),
                "{damaged}"
            );
        }
    }
}

#[test]
fn deterministic_byte_mutation_retains_target_timeout_and_checked_bounds() {
    let original = input(b"let value = [1, 2]; value[0];");
    let plan =
        RobustnessCampaignPlan::new(original.clone(), 0, MAX_GENERATED_CAMPAIGN_CASES).unwrap();
    assert_eq!(plan.case(0).unwrap().0.bytes(), original.bytes());
    assert!(RobustnessCampaignPlan::new(original.clone(), u64::MAX, 2).is_err());
    assert!(RobustnessCampaignPlan::new(original.clone(), 0, 0).is_err());
    for ordinal in 0..plan.count() {
        let first = plan.case(ordinal).unwrap();
        assert_eq!(first, plan.case(ordinal).unwrap());
        assert_eq!(first.0.target(), original.target());
        assert_eq!(first.0.timeout_ms(), original.timeout_ms());
        assert!(first.0.bytes().len() <= super::input::MAX_INPUT_BYTES);
        assert_eq!(
            RobustnessInput::from_json(&first.0.to_pretty_json().unwrap()).unwrap(),
            first.0
        );
    }
    let empty = RobustnessCampaignPlan::new(input(&[]), 4, 2).unwrap();
    assert_eq!(empty.case(1).unwrap().0.bytes().len(), 1);
}

#[test]
fn stages_cannot_claim_acceptance_before_validation_or_hide_a_native_rejection_phase() {
    let target = RobustnessTarget::Compiler {
        goal: DifferentialGoal::Script,
    };
    let accepted = RobustnessResult::Accepted {
        stage: RobustnessStage::Validation,
        artifact_bytes: Some(8),
    };
    assert!(accepted.valid_terminal(target, target.stages()));
    assert!(!accepted.valid_terminal(target, &target.stages()[..3]));
    let rejected = RobustnessResult::Rejected {
        stage: RobustnessStage::Preparation,
        phase: RobustnessRejectionPhase::Parse,
        message: "actual syntax rejection".into(),
    };
    assert!(rejected.valid_terminal(target, &target.stages()[..2]));
    assert!(rejected.completed_without_failure()); // rejection remains a rejection in the retained result
    assert!(!rejected.valid_terminal(
        RobustnessTarget::Snapshot {},
        RobustnessTarget::Snapshot {}.stages()
    ));
    assert!(!RobustnessResult::InvalidWasm {
        message: "bad module".into()
    }
    .completed_without_failure());
    assert!(!RobustnessResult::Unsupported {
        stage: RobustnessStage::Preparation,
        message: "parser aborted".into()
    }
    .completed_without_failure());
}

#[test]
fn native_ir_mutations_replay_exact_bytes_without_acquiring_a_source_goal() {
    let bytes = br#"{ "schema_version":1, "body":[{"op":"value","value":{"kind":"number","bits":"4008000000000000"}}] }"#;
    let original = RobustnessInput::new(
        "robustness/native-ir",
        RobustnessTarget::IrAdmission {},
        1000,
        bytes.to_vec(),
    )
    .unwrap();
    assert_eq!(
        RobustnessTarget::from_name("ir-admission"),
        Some(original.target())
    );
    assert_ne!(original.fingerprint(), input(bytes).fingerprint());
    let mut wire: serde_json::Value =
        serde_json::from_str(&original.to_pretty_json().unwrap()).unwrap();
    assert_eq!(wire["target"], serde_json::json!({"kind":"ir_admission"}));
    wire["target"]["goal"] = serde_json::json!("script");
    assert!(RobustnessInput::from_json(&wire.to_string()).is_err());

    let plan = RobustnessCampaignPlan::new(original.clone(), 19, 8).unwrap();
    for ordinal in 0..plan.count() {
        let (candidate, mutation) = plan.case(ordinal).unwrap();
        let mut expected = original.bytes().to_vec();
        match mutation {
            RobustnessMutation::Original => {}
            RobustnessMutation::Delete { start, end } => {
                expected.drain(start..end);
            }
            RobustnessMutation::Truncate { end } => expected.truncate(end),
            RobustnessMutation::Replace { offset, byte } => expected[offset] = byte,
            RobustnessMutation::Insert { offset, byte } => expected.insert(offset, byte),
            RobustnessMutation::Duplicate { start, end } => {
                let copied = original.bytes()[start..end].to_vec();
                expected.splice(end..end, copied);
            }
        }
        assert_eq!(candidate.bytes(), expected);
        assert_eq!(candidate.target(), RobustnessTarget::IrAdmission {});
        assert_eq!(candidate.timeout_ms(), original.timeout_ms());
        assert_eq!(
            RobustnessInput::from_json(&candidate.to_pretty_json().unwrap()).unwrap(),
            candidate
        );
        assert_eq!((candidate, mutation), plan.case(ordinal).unwrap());
    }
}

#[test]
fn native_ir_terminal_evidence_requires_both_admissions_and_real_artifact_validation() {
    let target = RobustnessTarget::IrAdmission {};
    let stages = [
        RobustnessStage::Decode,
        RobustnessStage::IrInput,
        RobustnessStage::Preparation,
        RobustnessStage::Lowering,
        RobustnessStage::IrAdmission,
        RobustnessStage::Emission,
        RobustnessStage::RuntimeSetup,
        RobustnessStage::Validation,
    ];
    let accepted = RobustnessResult::Accepted {
        stage: RobustnessStage::Validation,
        artifact_bytes: Some(8),
    };
    assert!(accepted.valid_terminal(target, &stages));
    for length in 0..stages.len() {
        assert!(!accepted.valid_terminal(target, &stages[..length]));
    }
    for omitted in [RobustnessStage::IrInput, RobustnessStage::IrAdmission] {
        let damaged: Vec<_> = stages
            .iter()
            .copied()
            .filter(|stage| *stage != omitted)
            .collect();
        assert!(!accepted.valid_terminal(target, &damaged));
    }
    for artifact_bytes in [None, Some(7)] {
        assert!(!RobustnessResult::Accepted {
            stage: RobustnessStage::Validation,
            artifact_bytes,
        }
        .valid_terminal(target, &stages));
    }
    for (stage, end) in [
        (RobustnessStage::IrInput, 2),
        (RobustnessStage::IrAdmission, 5),
    ] {
        let rejected = RobustnessResult::Rejected {
            stage,
            phase: RobustnessRejectionPhase::NativeBoundary,
            message: "checked native boundary rejection".into(),
        };
        assert!(rejected.valid_terminal(target, &stages[..end]));
        assert!(rejected.completed_without_failure());
        assert!(!rejected.valid_terminal(target, &stages));
    }
    assert!(!RobustnessResult::Executed {
        completion: RobustnessCompletionKind::Normal,
        value: "3".into(),
    }
    .valid_terminal(target, &stages));
}

#[test]
fn exact_inputs_survive_rejection_and_worker_failure_stops_with_pending_inventory() {
    let output = Output::new();
    let plan = RobustnessCampaignPlan::new(input(b"let = ;"), 7, 3).unwrap();
    let mut calls = 0;
    let report = campaign::run_with(
        &plan,
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |input| {
            calls += 1;
            Ok(if calls == 1 {
                observed(
                    input,
                    RobustnessResult::Rejected {
                        stage: RobustnessStage::Preparation,
                        phase: RobustnessRejectionPhase::Parse,
                        message: "actual parser diagnostic".into(),
                    },
                    input.target().stages()[..2].to_vec(),
                )
            } else {
                observed(
                    input,
                    RobustnessResult::WorkerFailure {
                        failure: DifferentialWorkerFailure::Timeout { timeout_ms: 1000 },
                        cleanup_error: None,
                    },
                    input.target().stages()[..3].to_vec(),
                )
            })
        },
    )
    .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(report.completed(), 2);
    assert_eq!(report.state(), RobustnessCampaignState::Failed);
    assert!(!report.completed_without_failure());
    for ordinal in 0..2 {
        let retained =
            RobustnessInput::load(output.0.join(format!("case-{ordinal:03}.input.json"))).unwrap();
        assert_eq!(
            fs::read(output.0.join(format!("case-{ordinal:03}.bin"))).unwrap(),
            retained.bytes()
        );
    }
    assert!(!output.0.join("case-002.input.json").exists());
    let aggregate: serde_json::Value =
        serde_json::from_slice(&fs::read(output.0.join("robustness.json")).unwrap()).unwrap();
    assert_eq!(aggregate["state"], "failed");
    assert!(aggregate["cases"][2]["observation"].is_null());
    assert!(campaign::run_with(
        &plan,
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |_| panic!("output reuse must fail before target replay")
    )
    .is_err());
}

#[test]
fn cancellation_and_foreign_worker_result_cannot_publish_a_finished_report() {
    let output = Output::new();
    let plan = RobustnessCampaignPlan::new(input(b"1;"), 0, 2).unwrap();
    let cancellation = GeneratedCampaignCancellation::default();
    cancellation.cancel();
    let report = campaign::run_with(&plan, &cancellation, &output.0, &mut |_| {
        panic!("cancelled targets must stay pending")
    })
    .unwrap();
    assert_eq!(report.state(), RobustnessCampaignState::Cancelled);
    assert_eq!(report.completed(), 0);
    assert!(!report.completed_without_failure());
    let foreign = Output::new();
    let result = campaign::run_with(
        &plan,
        &GeneratedCampaignCancellation::default(),
        &foreign.0,
        &mut |_| {
            Ok(observed(
                &input(b"foreign"),
                RobustnessResult::Accepted {
                    stage: RobustnessStage::Validation,
                    artifact_bytes: Some(8),
                },
                input(b"foreign").target().stages().to_vec(),
            ))
        },
    );
    assert!(matches!(
        result,
        Err(DifferentialError::GeneratorInvariant(_))
    ));
    let aggregate: serde_json::Value =
        serde_json::from_slice(&fs::read(foreign.0.join("robustness.json")).unwrap()).unwrap();
    assert_eq!(aggregate["state"], "running");
}
