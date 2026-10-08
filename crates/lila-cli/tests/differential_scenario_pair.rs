//! The relation is replayed by one real CLI command, not inferred from filenames.
use lila_test262::differential::{ScenarioGenerationPlan, ScenarioGrammar, ScenarioReplayPair};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct PairFile(PathBuf);
impl PairFile {
    fn new(json: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lila-cli-scenario-pair-{}-{}.json",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, json).unwrap();
        Self(path)
    }
}
impl Drop for PairFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn pair() -> ScenarioReplayPair {
    ScenarioReplayPair::generate(
        ScenarioGenerationPlan::new(ScenarioGrammar::MetamorphicV1, 12, 2).unwrap(),
    )
    .unwrap()
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn one_cli_command_reports_both_actual_backend_pairs_and_the_metamorphic_relation() {
    let pair = pair();
    let file = PairFile::new(&pair.to_pretty_json().unwrap());
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["differential", "replay-scenario-pair"])
        .arg(&file.0)
        .args(["--oracle", "spec-exec"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["metamorphic"]["verdict"], "observations_match");
    assert_eq!(
        report["baseline"]["case"]["source"],
        pair.cases().baseline().source()
    );
    assert_eq!(
        report["transformed"]["case"]["source"],
        pair.cases().transformed().unwrap().source()
    );
    for variant in ["baseline", "transformed"] {
        assert_eq!(
            report[variant]["report"]["verdict"],
            "primitive_completion_and_print_transcript_match"
        );
        assert!(report[variant]["report"]["wasm_aot"]["worker_identity"].is_object());
        assert!(report[variant]["report"]["spec_exec"]["worker_identity"].is_object());
    }
}

#[test]
fn forged_pair_checkpoints_are_rejected_before_selecting_a_worker() {
    let mut wire: serde_json::Value =
        serde_json::from_str(&pair().to_pretty_json().unwrap()).unwrap();
    wire["expected_steps"] = 1.into();
    let file = PairFile::new(&serde_json::to_string(&wire).unwrap());
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["differential", "replay-scenario-pair"])
        .arg(&file.0)
        .args([
            "--oracle",
            "spec-exec",
            "--worker-bin",
            "unavailable-worker-must-not-be-opened",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("progress count does not match"));
}
