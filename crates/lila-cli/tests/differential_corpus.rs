use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static EXECUTION: Mutex<()> = Mutex::new(());
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lila-corpus-cli-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn replay(root: Option<&Path>, output: &Path, worker: &str) -> lila_cli::CliCapture {
    let mut args = vec![
        "differential".to_owned(),
        "replay-corpus".to_owned(),
        "--output-dir".to_owned(),
        output.to_str().unwrap().to_owned(),
        "--oracle".to_owned(),
        "spec-exec".to_owned(),
        "--worker-bin".to_owned(),
        worker.to_owned(),
    ];
    if let Some(root) = root {
        args.extend([
            "--corpus-root".to_owned(),
            root.to_str().unwrap().to_owned(),
        ]);
    }
    lila_cli::run_cli_capture(args)
}
#[cfg(all(unix, feature = "spec-exec-oracle"))]
fn case(
    root: &Path,
    name: &str,
    id: &str,
    protocol: lila_test262::differential::DifferentialProtocol,
    source: &str,
) {
    let input = lila_test262::differential::DifferentialReplayInput::new_script(
        id,
        protocol,
        format!("{id}.js"),
        30_000,
        source,
    )
    .unwrap();
    std::fs::write(root.join(name), input.to_pretty_json().unwrap()).unwrap();
}
#[cfg(all(unix, feature = "spec-exec-oracle"))]
fn read(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn whole_corpus_retains_malformed_shared_failure_and_unsupported_rows() {
    use lila_test262::differential::DifferentialProtocol;
    let _serial = EXECUTION.lock().unwrap();
    let directory = Directory::new();
    let root = directory.0.join("cases");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("00-malformed.json"), b"not JSON\n").unwrap();
    case(
        &root,
        "01-match.json",
        "corpus/match",
        DifferentialProtocol::V2PrimitiveCompletionNoOutput,
        "23;",
    );
    case(
        &root,
        "02-shared-failure.json",
        "corpus/shared-failure",
        DifferentialProtocol::V1SelfCheckingNoOutput,
        "throw 'shared';",
    );
    case(
        &root,
        "03-unsupported.json",
        "corpus/unsupported",
        DifferentialProtocol::V2PrimitiveCompletionNoOutput,
        "({ x: 1 });",
    );
    let output = directory.0.join("reports");
    let captured = replay(Some(&root), &output, env!("CARGO_BIN_EXE_lila"));
    assert_eq!(captured.exit_code, 1);
    let report: serde_json::Value = serde_json::from_slice(&captured.stdout).unwrap();
    assert_eq!(
        (
            report["total"].as_u64(),
            report["completed"].as_u64(),
            report["matched"].as_u64(),
            report["failed"].as_u64()
        ),
        (Some(4), Some(4), Some(1), Some(3))
    );
    assert_eq!(report["verdict"], "contains_failures");
    assert_eq!(report["semantic_equivalence"], "not_established");
    assert_eq!(report, read(&output.join("aggregate.json")));
    assert_eq!(
        std::fs::read(output.join("case-000.input.json")).unwrap(),
        b"not JSON\n"
    );
    assert_eq!(
        read(&output.join("case-000.report.json"))["status"],
        "native_input_rejected"
    );
    for (ordinal, verdict) in [
        (1, "primitive_completions_match"),
        (2, "both_failed"),
        (3, "observation_contract_violated"),
    ] {
        let row = read(&output.join(format!("case-{ordinal:03}.report.json")));
        assert_eq!(row["status"], "compared");
        assert_eq!(row["report"]["verdict"], verdict);
        assert_eq!(
            row["report"]["wasm_aot"]["worker_identity"],
            row["report"]["spec_exec"]["worker_identity"]
        );
        for backend in ["wasm_aot", "spec_exec"] {
            let worker: lila_test262::CompilerProvenance =
                serde_json::from_value(row["report"][backend]["worker_identity"].clone()).unwrap();
            let parent = lila_test262::CompilerProvenance::current().unwrap();
            assert_eq!(
                worker.identity().source_fingerprint(),
                parent.identity().source_fingerprint()
            );
            assert_ne!(
                worker.identity().executable_sha256(),
                parent.identity().executable_sha256()
            );
        }
    }
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn whole_compiled_inventory_keeps_every_worker_failure_and_refuses_output_reuse() {
    let _serial = EXECUTION.lock().unwrap();
    let directory = Directory::new();
    let output = directory.0.join("reports");
    let captured = replay(None, &output, "/bin/false");
    assert_eq!(captured.exit_code, 1);
    let report: serde_json::Value = serde_json::from_slice(&captured.stdout).unwrap();
    assert_eq!(report["total"], 12);
    assert_eq!(report["completed"], 12);
    assert_eq!(report["matched"], 0);
    assert_eq!(report["failed"], 12);
    for ordinal in 0..12 {
        let row = read(&output.join(format!("case-{ordinal:03}.report.json")));
        assert_eq!(row["status"], "compared");
        assert_eq!(row["report"]["verdict"], "worker_failure");
        assert!(row["report"]["mismatch_signature"].is_null());
        for backend in ["wasm_aot", "spec_exec"] {
            assert_eq!(
                row["report"][backend]["execution"]["disposition"],
                "worker_failure"
            );
            assert_eq!(
                row["report"][backend]["output_events"]["availability"],
                "incomplete"
            );
        }
    }
    let bytes = std::fs::read(output.join("aggregate.json")).unwrap();
    let reused = replay(None, &output, "/bin/false");
    assert_eq!(reused.exit_code, 1);
    assert!(reused.stdout.is_empty());
    assert_eq!(std::fs::read(output.join("aggregate.json")).unwrap(), bytes);
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn duplicate_ids_and_worker_admission_errors_remain_individual_red_cases() {
    use lila_test262::differential::DifferentialProtocol;
    let _serial = EXECUTION.lock().unwrap();
    let directory = Directory::new();
    let root = directory.0.join("cases");
    std::fs::create_dir(&root).unwrap();
    case(
        &root,
        "00-original.json",
        "corpus/repeated",
        DifferentialProtocol::V2PrimitiveCompletionNoOutput,
        "1;",
    );
    case(
        &root,
        "01-duplicate.json",
        "corpus/repeated",
        DifferentialProtocol::V2PrimitiveCompletionNoOutput,
        "2;",
    );
    case(
        &root,
        "02-unsealed.json",
        "corpus/unsealed",
        DifferentialProtocol::V2PrimitiveCompletionNoOutput,
        "import('unsealed.mjs');",
    );
    let output = directory.0.join("reports");
    let captured = replay(Some(&root), &output, env!("CARGO_BIN_EXE_lila"));
    assert_eq!(captured.exit_code, 1);
    let report: serde_json::Value = serde_json::from_slice(&captured.stdout).unwrap();
    assert_eq!(report["completed"], 3);
    assert_eq!(report["matched"], 1);
    assert_eq!(report["failed"], 2);
    let duplicate = read(&output.join("case-001.report.json"));
    assert_eq!(duplicate["status"], "native_input_rejected");
    assert!(duplicate["message"].as_str().unwrap().contains("duplicate"));
    let admission = read(&output.join("case-002.report.json"));
    assert_eq!(admission["status"], "replay_rejected");
    for backend in ["wasm_aot", "spec_exec"] {
        assert_eq!(admission[backend]["status"], "rejected");
        assert!(!admission[backend]["message"].as_str().unwrap().is_empty());
    }
}

#[cfg(not(feature = "spec-exec-oracle"))]
#[test]
fn unlinked_oracle_refuses_whole_corpus_before_output_or_worker_lookup() {
    let _serial = EXECUTION.lock().unwrap();
    let directory = Directory::new();
    let output = directory.0.join("reports");
    let captured = replay(
        Some(&directory.0.join("absent-corpus")),
        &output,
        "not-an-executable",
    );
    assert_eq!(captured.exit_code, 1);
    assert!(captured.stdout.is_empty());
    assert!(!output.exists());
    assert!(String::from_utf8(captured.stderr)
        .unwrap()
        .contains("not linked"));
}
