//! CLI dispatch, selected-image provenance and campaign persistence share the
//! actual Cargo-built worker entry, including when the parent is an embedder.

use lila_test262::differential::{DifferentialProtocol, DifferentialReplayInput};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static EXECUTION: Mutex<()> = Mutex::new(());

struct CaseDirectory(PathBuf);

impl CaseDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lila-cli-differential-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn case(&self, source: &str) -> PathBuf {
        let input = DifferentialReplayInput::new_script(
            "cli/selected-worker",
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
            "cli/selected-worker.js",
            30_000,
            source,
        )
        .unwrap();
        let path = self.0.join("case.json");
        std::fs::write(&path, input.to_pretty_json().unwrap()).unwrap();
        path
    }
}

impl Drop for CaseDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn replay(path: &Path, worker: &str) -> lila_cli::CliCapture {
    lila_cli::run_cli_capture([
        "differential".to_string(),
        "replay".to_string(),
        path.to_str().unwrap().to_string(),
        "--oracle".to_string(),
        "spec-exec".to_string(),
        "--worker-bin".to_string(),
        worker.to_string(),
    ])
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn embedded_cli_replay_uses_the_selected_cli_image_and_worker_only_admission() {
    let _serial = EXECUTION.lock().unwrap();
    let directory = CaseDirectory::new();
    let output = replay(
        &directory.case("print('cli-worker'); 3;"),
        env!("CARGO_BIN_EXE_lila"),
    );
    assert_eq!(output.exit_code, 0, "{:?}", output.stderr);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["verdict"],
        "primitive_completion_and_print_transcript_match"
    );
    assert!(report["mismatch_signature"].is_null());
    for backend in ["wasm_aot", "spec_exec"] {
        assert_eq!(
            report[backend]["output_events"]["events"],
            serde_json::json!(["cli-worker"])
        );
        let worker: lila_test262::CompilerProvenance =
            serde_json::from_value(report[backend]["worker_identity"].clone()).unwrap();
        let current = lila_test262::CompilerProvenance::current().unwrap();
        assert_eq!(
            worker.identity().source_fingerprint(),
            current.identity().source_fingerprint()
        );
        assert_eq!(
            worker.identity().source_revision(),
            current.identity().source_revision()
        );
        assert_ne!(
            worker.identity().executable_sha256(),
            current.identity().executable_sha256(),
            "the selected CLI image must not be replaced by this integration-test image"
        );
        assert_eq!(
            report["wasm_aot"]["worker_identity"],
            report["spec_exec"]["worker_identity"]
        );
    }

    let invalid = replay(
        &directory.case("import('unsealed.mjs');"),
        env!("CARGO_BIN_EXE_lila"),
    );
    assert_eq!(invalid.exit_code, 1);
    assert!(
        invalid.stdout.is_empty(),
        "admission rejection is not a completed report"
    );
    assert!(String::from_utf8(invalid.stderr)
        .unwrap()
        .contains("outer dynamic import requires an embedded module graph"));
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn generated_cli_campaign_worker_failure_is_red_and_never_persists_a_case() {
    let _serial = EXECUTION.lock().unwrap();
    let directory = CaseDirectory::new();
    let path = directory.0.join("generated.json");
    let output = lila_cli::run_cli_capture([
        "differential",
        "generate-arithmetic",
        path.to_str().unwrap(),
        "--seed",
        "1",
        "--checks",
        "1",
        "--depth",
        "1",
        "--max-replays",
        "1",
        "--oracle",
        "spec-exec",
        "--worker-bin",
        "/bin/false",
    ]);
    assert_eq!(output.exit_code, 1);
    assert!(!path.exists());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["verdict"], "worker_failure");
    assert!(report["mismatch_signature"].is_null());
    for backend in ["wasm_aot", "spec_exec"] {
        assert_eq!(
            report[backend]["execution"]["disposition"],
            "worker_failure"
        );
        assert_eq!(
            report[backend]["output_events"]["availability"],
            "incomplete"
        );
    }
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("worker failure"));
}

#[cfg(not(feature = "spec-exec-oracle"))]
#[test]
fn default_cli_replay_refuses_the_unlinked_oracle_before_worker_execution() {
    let _serial = EXECUTION.lock().unwrap();
    let directory = CaseDirectory::new();
    let output = replay(
        &directory.case("print('must-not-run'); 3;"),
        "not-an-executable",
    );
    assert_eq!(output.exit_code, 1);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("not linked"));
}
