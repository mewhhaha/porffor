//! Real selected worker controls; authored and unrun with this source batch.
#![cfg(all(unix, feature = "spec-exec-oracle"))]
use lila_test262::differential::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "lila-robustness-worker-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[cfg(all(unix, feature = "spec-exec-oracle"))]
fn runner() -> DifferentialWorkerRunner {
    DifferentialWorkerRunner::new(env!("CARGO_BIN_EXE_lila-differential-worker")).unwrap()
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn actual_product_compilation_and_parser_rejections_retain_distinct_stage_evidence() {
    let oracle = SpecExecOracle::explicitly_enabled();
    let runner = runner();
    for (goal, source) in [
        (
            DifferentialGoal::Script,
            "function twice(x) { return x + x; } twice(3);",
        ),
        (DifferentialGoal::Module, "export const value = 3;"),
    ] {
        let input = RobustnessInput::new(
            "robustness/product",
            RobustnessTarget::Compiler { goal },
            300_000,
            source.as_bytes().to_vec(),
        )
        .unwrap();
        let observation = replay_robustness(&input, oracle, &runner).unwrap();
        assert!(observation.completed_without_failure(), "{observation:?}");
        assert!(
            matches!(observation.result(), RobustnessResult::Accepted { stage: RobustnessStage::Validation, artifact_bytes: Some(size) } if *size > 8)
        );
        assert_eq!(
            observation.stages(),
            &[
                RobustnessStage::Decode,
                RobustnessStage::Preparation,
                RobustnessStage::Lowering,
                RobustnessStage::Emission,
                RobustnessStage::RuntimeSetup,
                RobustnessStage::Validation
            ]
        );
    }
    let invalid = RobustnessInput::new(
        "robustness/rejected",
        RobustnessTarget::Compiler {
            goal: DifferentialGoal::Script,
        },
        300_000,
        b"let value = ;".to_vec(),
    )
    .unwrap();
    let rejected = replay_robustness(&invalid, oracle, &runner).unwrap();
    assert!(matches!(
        rejected.result(),
        RobustnessResult::Rejected {
            stage: RobustnessStage::Preparation,
            phase: RobustnessRejectionPhase::Parse,
            ..
        }
    ));
    assert_eq!(rejected.input().bytes(), invalid.bytes());
    assert_eq!(
        rejected.stages(),
        &[RobustnessStage::Decode, RobustnessStage::Preparation]
    );
    let encoding =
        RobustnessInput::new("robustness/encoding", invalid.target(), 1000, vec![0xff]).unwrap();
    assert!(matches!(
        replay_robustness(&encoding, oracle, &runner)
            .unwrap()
            .result(),
        RobustnessResult::Rejected {
            stage: RobustnessStage::Decode,
            phase: RobustnessRejectionPhase::Encoding,
            ..
        }
    ));
}

#[test]
fn native_ir_worker_admits_one_real_artifact_and_rejects_invalid_native_states() {
    let oracle = SpecExecOracle::explicitly_enabled();
    let runner = runner();
    let native = RobustnessInput::new(
        "robustness/native-ir-artifact",
        RobustnessTarget::IrAdmission {},
        300_000,
        br#"{"schema_version":1,"body":[
            {"op":"value","value":{"kind":"number","bits":"4008000000000000"}},
            {"op":"define_property","target":{"kind":"object"},"key":"answer","value":{"kind":"number","bits":"4045000000000000"}},
            {"op":"delete_optional","target":{"kind":"null"},"chain":[{"op":"property","key":"answer","shorted":true}],"strict":true},
            {"op":"if","condition":{"kind":"boolean","value":true},"then":[{"op":"block","body":[{"op":"value","value":{"kind":"string","value":"kept"}}]}],"else":[{"op":"empty"}]}
        ]}"#.to_vec(),
    )
    .unwrap();
    let replay = RobustnessInput::from_json(&native.to_pretty_json().unwrap()).unwrap();
    assert_eq!(replay, native);
    let accepted = replay_robustness(&replay, oracle, &runner).unwrap();
    assert_eq!(accepted.input(), &native);
    assert!(accepted.compiler_identity().is_some());
    assert!(accepted.completed_without_failure(), "{accepted:?}");
    assert!(
        matches!(accepted.result(), RobustnessResult::Accepted {
            stage: RobustnessStage::Validation,
            artifact_bytes: Some(bytes),
        } if *bytes > 8),
        "{accepted:?}"
    );
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
    assert_eq!(accepted.stages(), &stages);

    for (bytes, stage, prefix) in [
        (
            br#"{"schema_version":2,"body":[]}"#.as_slice(),
            RobustnessStage::IrInput,
            2,
        ),
        (
            br#"{"schema_version":1,"body":[{"op":"define_property","target":{"kind":"null"},"key":"x","value":{"kind":"undefined"}}]}"#.as_slice(),
            RobustnessStage::IrAdmission,
            5,
        ),
        (
            br#"{"schema_version":1,"body":[{"op":"if","condition":{"kind":"boolean","value":false},"then":[{"op":"await","value":{"kind":"null"}}],"else":[]}] }"#.as_slice(),
            RobustnessStage::IrAdmission,
            5,
        ),
    ] {
        let input = RobustnessInput::new(
            "robustness/native-ir-rejection",
            RobustnessTarget::IrAdmission {},
            300_000,
            bytes.to_vec(),
        ).unwrap();
        let rejected = replay_robustness(&input, oracle, &runner).unwrap();
        assert_eq!(rejected.input(), &input);
        assert_eq!(rejected.stages(), &stages[..prefix]);
        assert!(rejected.compiler_identity().is_some());
        assert!(rejected.completed_without_failure(), "{rejected:?}");
        assert!(
            matches!(rejected.result(), RobustnessResult::Rejected {
                stage: actual,
                phase: RobustnessRejectionPhase::NativeBoundary,
                message,
            } if *actual == stage && !message.is_empty()),
            "{rejected:?}"
        );
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn native_metadata_and_actual_builtin_parser_calls_keep_rejections_and_throws() {
    let oracle = SpecExecOracle::explicitly_enabled();
    let runner = runner();
    for (target, bytes) in [
        (
            RobustnessTarget::Frontmatter {},
            b"/*---\nflags: [onlyStrict, noStrict]\n---*/\n0;".as_slice(),
        ),
        (
            RobustnessTarget::Snapshot {},
            b"{\"version\":999}".as_slice(),
        ),
        (RobustnessTarget::Corpus {}, b"{}".as_slice()),
        (RobustnessTarget::ModuleGraph {}, b"{}".as_slice()),
        (RobustnessTarget::ReportObservation {}, b"{}".as_slice()),
    ] {
        let input =
            RobustnessInput::new("robustness/native", target, 300_000, bytes.to_vec()).unwrap();
        let observed = replay_robustness(&input, oracle, &runner).unwrap();
        assert!(
            matches!(
                observed.result(),
                RobustnessResult::Rejected {
                    phase: RobustnessRejectionPhase::NativeBoundary,
                    ..
                }
            ),
            "{observed:?}"
        );
    }
    for (parser, data) in [
        (BuiltinParserTarget::Json, "{\"x\":1}"),
        (BuiltinParserTarget::Json, "{"),
        (BuiltinParserTarget::RegExp, "["),
        (BuiltinParserTarget::Number, " 0x10 "),
        (BuiltinParserTarget::BigInt, "123"),
    ] {
        let input = RobustnessInput::new(
            "robustness/builtin",
            RobustnessTarget::Builtin { parser },
            300_000,
            data.as_bytes().to_vec(),
        )
        .unwrap();
        let observed = replay_robustness(&input, oracle, &runner).unwrap();
        assert!(
            matches!(observed.result(), RobustnessResult::Executed { .. }),
            "{observed:?}"
        );
        if data == "{" || data == "[" {
            assert!(matches!(
                observed.result(),
                RobustnessResult::Executed {
                    completion: RobustnessCompletionKind::Throw,
                    ..
                }
            ));
        }
    }
}

#[test]
fn report_observation_uses_the_native_terminal_frame_and_canonical_value_domain() {
    let oracle = SpecExecOracle::explicitly_enabled();
    let runner = runner();
    let terminal = serde_json::json!({
        "kind": "terminal", "print_count": 0,
        "execution": {
            "disposition": "primitive_completion",
            "completion": {"kind": "normal", "value": {"type": "number", "bits": "0000000000000000"}},
            "backend_note": "native frame fixture"
        }
    });
    let mut noncanonical = terminal.clone();
    noncanonical["execution"]["completion"]["value"]["bits"] =
        serde_json::json!("7ff0000000000001");
    let mut foreign = terminal.clone();
    foreign["worker_identity"] = serde_json::json!({});
    for (wire, accepts) in [
        (terminal, true),
        (noncanonical, false),
        (foreign, false),
        (
            serde_json::json!({"kind": "print_line", "sequence": 0, "text": "terminal"}),
            false,
        ),
    ] {
        let input = RobustnessInput::new(
            "robustness/native-terminal",
            RobustnessTarget::ReportObservation {},
            300_000,
            serde_json::to_vec(&wire).unwrap(),
        )
        .unwrap();
        let observed = replay_robustness(&input, oracle, &runner).unwrap();
        if accepts {
            assert!(
                matches!(
                    observed.result(),
                    RobustnessResult::Accepted {
                        stage: RobustnessStage::ReportObservation,
                        artifact_bytes: None,
                    }
                ),
                "{observed:?}"
            );
        } else {
            assert!(
                matches!(
                    observed.result(),
                    RobustnessResult::Rejected {
                        stage: RobustnessStage::ReportObservation,
                        phase: RobustnessRejectionPhase::NativeBoundary,
                        ..
                    }
                ),
                "{observed:?}"
            );
        }
    }
}

#[test]
fn original_prelude_and_filesystem_boundaries_replay_exact_payloads_and_closed_stages() {
    let oracle = SpecExecOracle::explicitly_enabled();
    let runner = runner();
    let prelude = serde_json::json!({
        "schema_version":1, "source":"1;", "execution_mode":"strict-script",
        "harness_profile":"none", "merged_harness":null,
        "files":[{"name":"assert.js","contents":"/* fixture assertion */"}], "overrides":[]
    });
    let mut missing_host = prelude.clone();
    missing_host["source"] = serde_json::json!("$262.createRealm();");
    let filesystem = serde_json::json!({
        "schema_version":1, "operation":"resolve_and_load", "referrer":"entry",
        "layout":"plain", "specifier":"./dep.js", "attributes":[]
    });
    let mut outside = filesystem.clone();
    outside["specifier"] = serde_json::json!("../outside/dep.js");
    for (target, wire, final_stage, accepted) in [
        (
            RobustnessTarget::Prelude {},
            prelude,
            RobustnessStage::PreludeMaterialization,
            true,
        ),
        (
            RobustnessTarget::Prelude {},
            missing_host,
            RobustnessStage::PreludeMaterialization,
            false,
        ),
        (
            RobustnessTarget::FilesystemResolver {},
            filesystem,
            RobustnessStage::ModuleLoading,
            true,
        ),
        (
            RobustnessTarget::FilesystemResolver {},
            outside,
            RobustnessStage::ModuleResolution,
            false,
        ),
    ] {
        let bytes = serde_json::to_vec(&wire).unwrap();
        let input = RobustnessInput::new("robustness/owned-native", target, 30_000, bytes).unwrap();
        let observed = replay_robustness(&input, oracle, &runner).unwrap();
        assert_eq!(observed.input(), &input);
        assert_eq!(observed.stages().last(), Some(&final_stage));
        assert!(observed.completed_without_failure(), "{observed:?}");
        if accepted {
            assert!(
                matches!(observed.result(), RobustnessResult::Accepted { stage, artifact_bytes:None } if *stage == final_stage),
                "{observed:?}"
            );
        } else {
            assert!(
                matches!(observed.result(), RobustnessResult::Rejected { stage, phase:RobustnessRejectionPhase::NativeBoundary, .. } if *stage == final_stage),
                "{observed:?}"
            );
        }
        let stages: &[RobustnessStage] = match target {
            RobustnessTarget::Prelude {} => &[
                RobustnessStage::Decode,
                RobustnessStage::PreludeInput,
                RobustnessStage::PreludeLoad,
                RobustnessStage::PreludeMaterialization,
            ],
            RobustnessTarget::FilesystemResolver {} if accepted => &[
                RobustnessStage::Decode,
                RobustnessStage::FilesystemInput,
                RobustnessStage::FilesystemSetup,
                RobustnessStage::ModuleResolution,
                RobustnessStage::ModuleLoading,
            ],
            RobustnessTarget::FilesystemResolver {} => &[
                RobustnessStage::Decode,
                RobustnessStage::FilesystemInput,
                RobustnessStage::FilesystemSetup,
                RobustnessStage::ModuleResolution,
            ],
            _ => unreachable!("closed native fixture targets"),
        };
        assert_eq!(observed.stages(), stages);
    }
}

#[test]
fn uri_worker_executes_original_utf16_data_and_keeps_lone_surrogate_throw() {
    let oracle = SpecExecOracle::explicitly_enabled();
    let runner = runner();
    for (parser, units, expected) in [
        (
            BuiltinParserTarget::DecodeUriComponent,
            "%2F".encode_utf16().collect::<Vec<_>>(),
            RobustnessCompletionKind::Normal,
        ),
        (
            BuiltinParserTarget::EncodeUri,
            vec![0xd800],
            RobustnessCompletionKind::Throw,
        ),
    ] {
        let bytes =
            serde_json::to_vec(&serde_json::json!({"schema_version":1,"units":units})).unwrap();
        let input = RobustnessInput::new(
            "robustness/uri",
            RobustnessTarget::Builtin { parser },
            300_000,
            bytes,
        )
        .unwrap();
        let observed = replay_robustness(&input, oracle, &runner).unwrap();
        assert_eq!(observed.input(), &input);
        let RobustnessResult::Executed { completion, value } = observed.result() else {
            panic!("{observed:?}");
        };
        assert_eq!(*completion, expected);
        if expected == RobustnessCompletionKind::Normal {
            assert_eq!(value, "Normal(String([47]))");
        }
        assert_eq!(
            observed.stages(),
            &[
                RobustnessStage::Decode,
                RobustnessStage::BuiltinInput,
                RobustnessStage::Preparation,
                RobustnessStage::Lowering,
                RobustnessStage::Emission,
                RobustnessStage::RuntimeSetup,
                RobustnessStage::Validation,
                RobustnessStage::BuiltinExecution,
            ]
        );
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn actual_failed_worker_stops_mutations_and_retains_raw_input_and_diagnostics() {
    use std::os::unix::fs::PermissionsExt;
    let output = Output::new();
    let script = output.0.with_extension("sh");
    std::fs::write(
        &script,
        "#!/bin/sh\necho 'deliberate worker crash' >&2\nexit 71\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
    let input = RobustnessInput::new(
        "robustness/crash",
        RobustnessTarget::Compiler {
            goal: DifferentialGoal::Script,
        },
        1000,
        vec![0, 255, 1],
    )
    .unwrap();
    let plan = RobustnessCampaignPlan::new(input, 9, 4).unwrap();
    let report = run_robustness_campaign(
        &plan,
        SpecExecOracle::explicitly_enabled(),
        &DifferentialWorkerRunner::new(&script).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &output.0,
    )
    .unwrap();
    assert_eq!(report.state(), RobustnessCampaignState::Failed);
    assert_eq!(report.completed(), 1);
    assert!(!report.completed_without_failure());
    let observation: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.0.join("case-000.observation.json")).unwrap())
            .unwrap();
    assert_eq!(observation["result"]["failure"]["code"], 71);
    assert!(!observation["stderr_bytes_hex"].as_str().unwrap().is_empty());
    assert_eq!(
        std::fs::read(output.0.join("case-000.bin")).unwrap(),
        vec![0, 255, 1]
    );
    assert!(!output.0.join("case-001.input.json").exists());
    // An actual selected executable dying before its admitted header is not a
    // target crash proof and must not start byte reduction.
    let reduction = Output::new();
    let report = minimize_robustness(
        plan.input(),
        ArithmeticReductionLimit::new(4).unwrap(),
        SpecExecOracle::explicitly_enabled(),
        &DifferentialWorkerRunner::new(&script).unwrap(),
        &GeneratedCampaignCancellation::default(),
        &reduction.0,
    )
    .unwrap();
    assert_eq!(report.state(), RobustnessReductionState::Stopped);
    assert_eq!(report.candidate_replays(), 0);
    assert!(report.reproducing_input().is_none());
    assert!(reduction.0.join("attempt-000.observation.json").exists());
    assert!(!reduction.0.join("attempt-001.input.json").exists());
    std::fs::remove_file(script).unwrap();
}
