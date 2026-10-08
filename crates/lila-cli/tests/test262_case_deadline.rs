//! Actual CLI supervision and exact child snapshot contracts, not suite counts.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "lila-case-deadline-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("suite/test/language/deadline")).unwrap();
    fs::write(
        root.join("suite/test/language/deadline/negative.js"),
        "/*---\nflags: [raw]\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/\nconst = ;\n",
    )
    .unwrap();
    root
}

#[test]
fn default_supervisor_ignores_legacy_bypass_and_preserves_exact_compiler_identity() {
    let root = fixture("default");
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args([
            "--jobs",
            "1",
            "test262",
            "run",
            "raw-script:language/deadline/negative.js",
        ])
        .arg("--suite-root")
        .arg(root.join("suite"))
        .arg("--snapshot-dir")
        .arg(root.join("snapshots"))
        .args([
            "--snapshot-name",
            "deadline-default",
            "--threads",
            "1",
            "--timeout-ms",
            "1000",
        ])
        .env("LILA_TEST262_DISABLE_CASE_RUNNER", "1")
        .env_remove("LILA_TEST262_FORCE_CASE_RUNNER")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("total: 1") && text.contains("passed: 1"),
        "{text}"
    );
    let snapshot = fs::read_dir(root.join("snapshots"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(snapshot).unwrap()).unwrap();
    assert_eq!(
        value["completed_test_ids"],
        serde_json::json!(["raw-script:language/deadline/negative.js"])
    );
    let identity = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("compiler-identity")
        .output()
        .unwrap();
    assert!(identity.status.success());
    assert_eq!(
        value["compiler_identity"],
        serde_json::from_slice::<serde_json::Value>(&identity.stdout).unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hidden_worker_rejects_directory_selection_without_writing_execution_evidence() {
    let root = fixture("invalid-selection");
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args([
            "--jobs",
            "1",
            "test262",
            "__case-worker",
            "language/deadline",
        ])
        .arg("--suite-root")
        .arg(root.join("suite"))
        .arg("--snapshot-dir")
        .arg(root.join("snapshots"))
        .args(["--threads", "1", "--case-harness", "none"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execution"));
    assert!(!root.join("snapshots").exists());
    fs::remove_dir_all(root).unwrap();
}
