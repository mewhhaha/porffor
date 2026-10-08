use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "lila-robustness-reduce-{}-{}",
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
        "robustness/reduction",
        RobustnessTarget::Compiler {
            goal: DifferentialGoal::Module,
        },
        2000,
        bytes.to_vec(),
    )
    .unwrap()
}
fn crash(
    input: &RobustnessInput,
    stage: RobustnessStage,
    failure: DifferentialWorkerFailure,
) -> RobustnessObservation {
    let expected = input.target().stages();
    let end = expected.iter().position(|actual| *actual == stage).unwrap() + 1;
    RobustnessObservation {
        input: input.clone(),
        compiler_identity: Some(crate::CompilerProvenance::current().unwrap()),
        stages: expected[..end].to_vec(),
        result: RobustnessResult::WorkerFailure {
            failure,
            cleanup_error: None,
        },
        journal_bytes_hex: "636f6d6d6974746564".into(),
        stderr_bytes_hex: "6372617368".into(),
        journal_error: None,
        interrupted_stage: Some(stage),
    }
}
fn exit(input: &RobustnessInput, code: i32) -> RobustnessObservation {
    crash(
        input,
        RobustnessStage::Lowering,
        DifferentialWorkerFailure::Exit {
            code: Some(code),
            signal: None,
        },
    )
}
fn rejection(input: &RobustnessInput) -> RobustnessObservation {
    let mut result = exit(input, 101);
    result.interrupted_stage = None;
    result.stages = input.target().stages()[..2].to_vec();
    result.result = RobustnessResult::Rejected {
        stage: RobustnessStage::Preparation,
        phase: RobustnessRejectionPhase::Parse,
        message: "actual ordinary parser rejection".into(),
    };
    result
}

#[test]
fn only_committed_interrupted_target_crashes_are_reduction_witnesses() {
    let input = input(b"broken");
    let original = exit(&input, 101);
    let witness = signature(&original, &input, None).unwrap().unwrap();
    assert_eq!(witness.stage, RobustnessStage::Lowering);
    assert_eq!(signature(&rejection(&input), &input, None).unwrap(), None);
    for failure in [
        DifferentialWorkerFailure::Protocol {
            message: "bad journal".into(),
        },
        DifferentialWorkerFailure::Process {
            message: "spawn rejected".into(),
        },
        DifferentialWorkerFailure::ObservationLimit { limit_bytes: 1 },
        DifferentialWorkerFailure::UnsupportedPlatform,
        DifferentialWorkerFailure::Exit {
            code: Some(0),
            signal: None,
        },
        DifferentialWorkerFailure::Timeout { timeout_ms: 1999 },
    ] {
        assert!(signature(
            &crash(&input, RobustnessStage::Lowering, failure),
            &input,
            None
        )
        .is_err());
    }
    let timeout = crash(
        &input,
        RobustnessStage::Lowering,
        DifferentialWorkerFailure::Timeout { timeout_ms: 2000 },
    );
    assert!(matches!(
        signature(&timeout, &input, None)
            .unwrap()
            .unwrap()
            .disposition,
        CrashDisposition::Timeout { timeout_ms: 2000 }
    ));
    for damage in 0..5 {
        let mut damaged = exit(&input, 101);
        match damage {
            0 => damaged.compiler_identity = None,
            1 => damaged.journal_error = Some("foreign frame".into()),
            2 => damaged.interrupted_stage = None, // torn or completed journal
            3 => {
                if let RobustnessResult::WorkerFailure { cleanup_error, .. } = &mut damaged.result {
                    *cleanup_error = Some("reap failed".into());
                }
            }
            4 => damaged.stages.pop().map(|_| ()).unwrap(),
            _ => unreachable!(),
        }
        assert!(signature(&damaged, &input, None).is_err());
    }
    assert!(signature(&original, &self::input(b"foreign"), None).is_err());
}

#[test]
fn exact_byte_candidates_strictly_decrease_and_record_their_actual_mutation() {
    for original in [
        b"\0\xff\x80\x01".as_slice(),
        b"x".as_slice(),
        b"".as_slice(),
    ] {
        let mut cursor = Cursor::new(original);
        let mut seen = std::collections::BTreeSet::new();
        while let Some((bytes, edit)) = cursor.next(original) {
            assert!(ByteComplexity::of(&bytes) < ByteComplexity::of(original));
            let mut expected = original.to_vec();
            match edit {
                ReductionEdit::Delete { start, end } => {
                    expected.drain(start..end);
                }
                ReductionEdit::LowerByte { offset, byte } => expected[offset] = byte,
            }
            assert_eq!(bytes, expected);
            seen.insert(bytes);
        }
        assert!(original.is_empty() || seen.contains(&Vec::new()));
    }
}

#[test]
fn native_ir_reduction_binds_exact_target_bytes_and_interrupted_admission_stage() {
    // Synthetic committed worker observations exercise the reducer protocol;
    // they are not evidence of a crash in the native IR implementation.
    let original = RobustnessInput::new(
        "robustness/native-ir-reduction",
        RobustnessTarget::IrAdmission {},
        2000,
        br#"{ "schema_version":1, "body":[] }"#.to_vec(),
    )
    .unwrap();
    let failure = || DifferentialWorkerFailure::Exit {
        code: Some(101),
        signal: None,
    };
    let baseline = crash(&original, RobustnessStage::IrAdmission, failure());
    assert_eq!(
        signature(&baseline, &original, None)
            .unwrap()
            .unwrap()
            .stage,
        RobustnessStage::IrAdmission
    );
    for foreign in [
        RobustnessInput::new(
            original.id().as_str(),
            RobustnessTarget::Compiler {
                goal: DifferentialGoal::Script,
            },
            original.timeout_ms(),
            original.bytes().to_vec(),
        )
        .unwrap(),
        RobustnessInput::new(
            original.id().as_str(),
            original.target(),
            original.timeout_ms(),
            b"{}".to_vec(),
        )
        .unwrap(),
    ] {
        assert_eq!(
            signature(&baseline, &foreign, None),
            Err(StopReason::ForeignInput)
        );
    }
    let mut missing_input_admission = crash(&original, RobustnessStage::IrAdmission, failure());
    missing_input_admission.stages.remove(1);
    assert_eq!(
        signature(&missing_input_admission, &original, None),
        Err(StopReason::IncompleteJournal)
    );

    let output = Output::new();
    let mut calls = 0;
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(3).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |candidate| {
            let ordinal = calls;
            calls += 1;
            assert_eq!(candidate.target(), original.target());
            assert_eq!(candidate.timeout_ms(), original.timeout_ms());
            assert_eq!(
                RobustnessInput::load(output.0.join(format!("attempt-{ordinal:03}.input.json")))
                    .unwrap(),
                *candidate
            );
            assert_eq!(
                fs::read(output.0.join(format!("attempt-{ordinal:03}.bin"))).unwrap(),
                candidate.bytes()
            );
            let mut observed = crash(candidate, RobustnessStage::IrAdmission, failure());
            if ordinal == 1 {
                observed = crash(candidate, RobustnessStage::IrInput, failure());
            } else if ordinal == 2 {
                observed.interrupted_stage = None;
                observed.result = RobustnessResult::Rejected {
                    stage: RobustnessStage::IrAdmission,
                    phase: RobustnessRejectionPhase::NativeBoundary,
                    message: "completed native admission rejection".into(),
                };
                assert_eq!(signature(&observed, candidate, None).unwrap(), None);
            }
            Ok(observed)
        },
    )
    .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(
        report.state(),
        RobustnessReductionState::ReplayBudgetExhausted
    );
    assert_eq!(report.candidate_replays(), 3);
    assert_eq!(report.retained_candidates(), 1);
    assert_eq!(
        report.witness.as_ref().unwrap().stage,
        RobustnessStage::IrAdmission
    );
    assert_eq!(report.attempts[1].decision, Decision::DifferentOutcome);
    assert_eq!(report.attempts[2].decision, Decision::DifferentOutcome);
    assert_eq!(report.attempts[3].decision, Decision::Retained);
    let retained = report.reproducing_input().unwrap();
    assert!(retained.bytes().len() < original.bytes().len());
    assert_eq!(retained.target(), RobustnessTarget::IrAdmission {});
    assert_eq!(
        RobustnessInput::load(output.0.join("minimized.input.json")).unwrap(),
        *retained
    );
    assert_eq!(
        fs::read(output.0.join("minimized.bin")).unwrap(),
        retained.bytes()
    );
}

#[test]
fn reduction_retains_only_same_disposition_stage_and_policy_with_durable_replay_bytes() {
    let output = Output::new();
    let original = input(b"zzxqq");
    let mut calls = 0;
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(128).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |candidate| {
            let ordinal = calls;
            calls += 1;
            assert_eq!(candidate.target(), original.target());
            assert_eq!(candidate.timeout_ms(), original.timeout_ms());
            assert_eq!(
                RobustnessInput::load(output.0.join(format!("attempt-{ordinal:03}.input.json")))
                    .unwrap(),
                *candidate
            );
            assert_eq!(
                fs::read(output.0.join(format!("attempt-{ordinal:03}.bin"))).unwrap(),
                candidate.bytes()
            );
            Ok(if candidate.bytes().contains(&b'x') {
                exit(candidate, 101)
            } else if candidate.bytes().is_empty() {
                rejection(candidate)
            } else {
                exit(candidate, 102)
            })
        },
    )
    .unwrap();
    assert_eq!(
        report.state(),
        RobustnessReductionState::CandidateSetExhausted
    );
    assert_eq!(report.reproducing_input().unwrap().bytes(), b"x");
    assert_eq!(calls, report.candidate_replays() as usize + 1);
    assert!(report.retained_candidates() > 0);
    let mut previous = ByteComplexity::of(original.bytes());
    for attempt in &report.attempts {
        let observed: serde_json::Value = serde_json::from_slice(
            &fs::read(output.0.join(attempt.observation.as_ref().unwrap())).unwrap(),
        )
        .unwrap();
        assert!(!observed["journal_bytes_hex"].as_str().unwrap().is_empty());
        if !attempt.baseline && attempt.decision == Decision::Retained {
            assert!(attempt.complexity < previous);
            previous = attempt.complexity;
        }
    }
    let minimized = RobustnessInput::load(output.0.join("minimized.input.json")).unwrap();
    assert_eq!(minimized, *report.reproducing_input().unwrap());
    assert_eq!(
        fs::read(output.0.join("minimized.bin")).unwrap(),
        minimized.bytes()
    );
    assert!(report
        .attempts
        .iter()
        .any(|attempt| attempt.decision == Decision::DifferentOutcome));
    assert!(run_with(
        &original,
        ArithmeticReductionLimit::new(1).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |_| panic!("fresh output admission precedes any callback")
    )
    .is_err());
}

#[test]
fn replay_budget_and_cancellation_count_only_actual_worker_callbacks() {
    let output = Output::new();
    let original = input(b"must-stay-whole");
    let mut calls = 0;
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(3).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |candidate| {
            calls += 1;
            Ok(if calls == 1 {
                exit(candidate, 101)
            } else {
                rejection(candidate)
            })
        },
    )
    .unwrap();
    assert_eq!(
        report.state(),
        RobustnessReductionState::ReplayBudgetExhausted
    );
    assert_eq!(calls, 4);
    assert_eq!(report.candidate_replays(), 3);
    assert_eq!(report.retained_candidates(), 0);
    assert_eq!(report.reproducing_input().unwrap(), &original);
    let cancelled = Output::new();
    let cancellation = GeneratedCampaignCancellation::default();
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(3).unwrap(),
        &cancellation,
        &cancelled.0,
        &mut |candidate| {
            cancellation.cancel();
            Ok(exit(candidate, 101))
        },
    )
    .unwrap();
    assert_eq!(report.state(), RobustnessReductionState::Cancelled);
    assert_eq!(report.candidate_replays(), 0);
    assert_eq!(report.attempts.len(), 1);
    assert!(cancelled.0.join("minimized.input.json").exists());
    let before = Output::new();
    let cancellation = GeneratedCampaignCancellation::default();
    cancellation.cancel();
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(3).unwrap(),
        &cancellation,
        &before.0,
        &mut |_| panic!("cancelled baseline must remain pending"),
    )
    .unwrap();
    assert_eq!(report.state(), RobustnessReductionState::Cancelled);
    assert!(report.attempts.is_empty());
    assert!(report.reproducing_input().is_none());
}

#[test]
fn ordinary_rejections_do_not_reduce_and_foreign_or_incomplete_evidence_stops_after_retention() {
    let original = input(b"repro");
    let output = Output::new();
    let mut calls = 0;
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(8).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |candidate| {
            calls += 1;
            Ok(rejection(candidate))
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(report.state(), RobustnessReductionState::NotReducible);
    assert!(report.reproducing_input().is_none());
    for foreign in [false, true] {
        let output = Output::new();
        let mut calls = 0;
        let report = run_with(
            &original,
            ArithmeticReductionLimit::new(8).unwrap(),
            &GeneratedCampaignCancellation::default(),
            &output.0,
            &mut |candidate| {
                calls += 1;
                let mut observed = exit(candidate, 101);
                if calls == 2 {
                    if foreign {
                        observed.input = input(b"not-the-request");
                    } else {
                        observed.interrupted_stage = None;
                    }
                }
                Ok(observed)
            },
        )
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(report.state(), RobustnessReductionState::Stopped);
        assert_eq!(report.candidate_replays(), 1);
        assert_eq!(report.reproducing_input().unwrap(), &original);
        assert!(output.0.join("attempt-001.observation.json").exists());
        assert_eq!(report.attempts[1].decision, Decision::Stopped);
    }
}

#[test]
fn a_timeout_witness_keeps_its_entered_stage_and_worker_disposition() {
    let original = input(b"abcdefgh");
    let output = Output::new();
    let mut calls = 0;
    let report = run_with(
        &original,
        ArithmeticReductionLimit::new(3).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
        &mut |candidate| {
            calls += 1;
            Ok(match calls {
                2 => crash(
                    candidate,
                    RobustnessStage::Preparation,
                    DifferentialWorkerFailure::Timeout { timeout_ms: 2000 },
                ),
                3 => exit(candidate, 101),
                _ => crash(
                    candidate,
                    RobustnessStage::Lowering,
                    DifferentialWorkerFailure::Timeout { timeout_ms: 2000 },
                ),
            })
        },
    )
    .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(
        report.state(),
        RobustnessReductionState::ReplayBudgetExhausted
    );
    assert_eq!(report.attempts[1].decision, Decision::DifferentOutcome);
    assert_eq!(report.attempts[2].decision, Decision::DifferentOutcome);
    assert_eq!(report.attempts[3].decision, Decision::Retained);
    assert_eq!(report.retained_candidates(), 1);
    let witness = report.witness.unwrap();
    assert_eq!(witness.stage, RobustnessStage::Lowering);
    assert_eq!(
        witness.disposition,
        CrashDisposition::Timeout {
            timeout_ms: original.timeout_ms()
        }
    );
}

#[test]
fn changed_worker_provenance_and_controller_errors_are_not_crash_reductions() {
    let original = input(b"repro");
    for worker_error in [false, true] {
        let output = Output::new();
        let mut calls = 0;
        let report = run_with(
            &original,
            ArithmeticReductionLimit::new(8).unwrap(),
            &GeneratedCampaignCancellation::default(),
            &output.0,
            &mut |candidate| {
                calls += 1;
                if calls == 2 && worker_error {
                    return Err(DifferentialError::WorkerConfiguration(
                        "admission failed".into(),
                    ));
                }
                let mut observed = exit(candidate, 101);
                if calls == 2 {
                    let mut identity =
                        serde_json::to_value(observed.compiler_identity.as_ref().unwrap()).unwrap();
                    let replacement = if identity["source_fingerprint"] == "0".repeat(64) {
                        "1"
                    } else {
                        "0"
                    };
                    identity["source_fingerprint"] = replacement.repeat(64).into();
                    observed.compiler_identity = Some(serde_json::from_value(identity).unwrap());
                }
                Ok(observed)
            },
        )
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(report.state(), RobustnessReductionState::Stopped);
        assert_eq!(report.candidate_replays(), 1);
        assert_eq!(report.retained_candidates(), 0);
        assert_eq!(report.reproducing_input().unwrap(), &original);
        assert_eq!(
            report.stop_reason,
            Some(if worker_error {
                StopReason::ReplayError
            } else {
                StopReason::ProvenanceMismatch
            })
        );
        assert!(output
            .0
            .join(if worker_error {
                "attempt-001.error.json"
            } else {
                "attempt-001.observation.json"
            })
            .exists());
    }
}
