use lila_test262::differential::{DifferentialGoal, RobustnessInput, RobustnessTarget};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct InputFile(PathBuf);
impl InputFile {
    fn new(json: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lila-robustness-cli-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, json).unwrap();
        Self(path)
    }
}
impl Drop for InputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn forged_native_byte_wire_is_rejected_before_worker_selection() {
    let input = RobustnessInput::new(
        "robustness/cli",
        RobustnessTarget::Compiler {
            goal: DifferentialGoal::Script,
        },
        1000,
        vec![0xff],
    )
    .unwrap();
    let mut wire: serde_json::Value =
        serde_json::from_str(&input.to_pretty_json().unwrap()).unwrap();
    wire["bytes_hex"] = "FF".into();
    let file = InputFile::new(&wire.to_string());
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["differential", "replay-robustness"])
        .arg(&file.0)
        .args([
            "--oracle",
            "spec-exec",
            "--worker-bin",
            "this-worker-must-not-be-opened",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("canonical lowercase hex"));
    let reduction = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["differential", "minimize-robustness"])
        .arg(&file.0)
        .args([
            "--replays",
            "1",
            "--output-dir",
            "this-output-must-not-be-created",
            "--oracle",
            "spec-exec",
            "--worker-bin",
            "this-worker-must-not-be-opened",
        ])
        .output()
        .unwrap();
    assert!(!reduction.status.success());
    assert!(reduction.stdout.is_empty());
    assert!(String::from_utf8_lossy(&reduction.stderr).contains("canonical lowercase hex"));
}
#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn one_real_cli_command_replays_exact_rejected_source_without_claiming_acceptance() {
    let input = RobustnessInput::new(
        "robustness/cli",
        RobustnessTarget::Compiler {
            goal: DifferentialGoal::Script,
        },
        300_000,
        b"let value = ;".to_vec(),
    )
    .unwrap();
    let file = InputFile::new(&input.to_pretty_json().unwrap());
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["differential", "replay-robustness"])
        .arg(&file.0)
        .args(["--oracle", "spec-exec"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(observed["result"]["kind"], "rejected");
    assert_eq!(observed["result"]["phase"], "parse");
    assert_eq!(
        observed["input"]["bytes_hex"],
        serde_json::from_str::<serde_json::Value>(&input.to_pretty_json().unwrap()).unwrap()
            ["bytes_hex"]
    );
    assert!(observed["compiler_identity"].is_object());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn native_ir_cli_replay_retains_native_target_bytes_and_input_rejection_phase() {
    let input = RobustnessInput::new(
        "robustness/cli-native-ir",
        RobustnessTarget::IrAdmission {},
        300_000,
        br#"{ "schema_version":2, "body":[] }"#.to_vec(),
    )
    .unwrap();
    let original: serde_json::Value =
        serde_json::from_str(&input.to_pretty_json().unwrap()).unwrap();
    let file = InputFile::new(&input.to_pretty_json().unwrap());
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["differential", "replay-robustness"])
        .arg(&file.0)
        .args(["--oracle", "spec-exec"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(observed["input"], original);
    assert_eq!(
        observed["input"]["target"],
        serde_json::json!({"kind":"ir_admission"})
    );
    assert_eq!(
        observed["stages"],
        serde_json::json!(["decode", "ir_input"])
    );
    assert_eq!(observed["result"]["kind"], "rejected");
    assert_eq!(observed["result"]["stage"], "ir_input");
    assert_eq!(observed["result"]["phase"], "native_boundary");
    assert!(observed["result"]["message"]
        .as_str()
        .is_some_and(|message| !message.is_empty()));
    assert!(observed["compiler_identity"].is_object());
}

#[cfg(all(unix, feature = "spec-exec-oracle"))]
#[test]
fn prelude_and_filesystem_cli_replay_keep_native_bytes_and_real_boundary_results() {
    for (target, payload, final_stage, kind) in [
        (
            RobustnessTarget::Prelude {},
            serde_json::json!({
                "schema_version":1, "source":"1;", "execution_mode":"sloppy-script",
                "harness_profile":"none", "merged_harness":null,
                "files":[{"name":"assert.js","contents":"/* fixture */"}], "overrides":[]
            }),
            "prelude_materialization",
            "accepted",
        ),
        (
            RobustnessTarget::FilesystemResolver {},
            serde_json::json!({
                "schema_version":1, "operation":"load_direct", "referrer":"none", "layout":"plain",
                "specifier":"../outside/dep.js", "attributes":[]
            }),
            "module_loading",
            "rejected",
        ),
    ] {
        let input = RobustnessInput::new(
            "robustness/cli-owned",
            target,
            30_000,
            serde_json::to_vec(&payload).unwrap(),
        )
        .unwrap();
        let original: serde_json::Value =
            serde_json::from_str(&input.to_pretty_json().unwrap()).unwrap();
        let file = InputFile::new(&input.to_pretty_json().unwrap());
        let output = Command::new(env!("CARGO_BIN_EXE_lila"))
            .args(["differential", "replay-robustness"])
            .arg(&file.0)
            .args(["--oracle", "spec-exec"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let observed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(observed["input"], original);
        assert_eq!(observed["result"]["kind"], kind);
        assert_eq!(observed["result"]["stage"], final_stage);
        assert_eq!(
            observed["stages"].as_array().unwrap().last().unwrap(),
            final_stage
        );
        if kind == "accepted" {
            assert!(observed["result"]["artifact_bytes"].is_null());
        } else {
            assert_eq!(observed["result"]["phase"], "native_boundary");
        }
    }
}
