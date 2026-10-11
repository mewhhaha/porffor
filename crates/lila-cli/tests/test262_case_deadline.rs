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

#[test]
fn reused_worker_requests_have_fresh_realms_and_exact_independent_snapshots() {
    use std::io::Write;
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let root = fixture("reuse");
    for (name, source) in [
        ("failure", "throw 17;"),
        ("first", "globalThis.__caseMarker = 17; Object.prototype.__caseMarker = 23; Array.prototype.push = function () { throw 91; }; Promise.resolve().then(function () { globalThis.__jobMarker = 29; }); new WeakRef({value: 1});"),
        ("second", "if ('__caseMarker' in globalThis || '__caseMarker' in Object.prototype || '__jobMarker' in globalThis) throw 92; var values = []; if (values.push(7) !== 1 || values[0] !== 7) throw 93; var key = {}; var map = new WeakMap([[key, 42]]); if (map.get(key) !== 42) throw 94;"),
    ] {
        fs::write(root.join(format!("suite/test/language/deadline/{name}.js")),
            format!("/*---\nflags: [raw]\n---*/\n{source}\n")).unwrap();
    }
    let output_path = root.join("server.stdout");
    let error_path = root.join("server.stderr");
    let mut server = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "test262", "__case-worker-server-v1"])
        .arg("--suite-root")
        .arg(root.join("suite"))
        .args([
            "--threads",
            "1",
            "--timeout-ms",
            "60000",
            "--case-harness",
            "none",
        ])
        .stdin(Stdio::piped())
        .stdout(fs::File::create(&output_path).unwrap())
        .stderr(fs::File::create(&error_path).unwrap())
        .spawn()
        .unwrap();
    let mut input = server.stdin.take().unwrap();
    for (sequence, name) in [(1, "failure"), (2, "first"), (3, "second")] {
        writeln!(
            input,
            "{}",
            serde_json::json!({
                "sequence": sequence,
                "test_id": format!("raw-script:language/deadline/{name}.js"),
                "snapshot_dir": root.join(name),
                "snapshot_name": name,
            })
        )
        .unwrap();
    }
    drop(input);
    let expires = Instant::now() + Duration::from_secs(180);
    let status = loop {
        if let Some(status) = server.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= expires {
            server.kill().unwrap();
            server.wait().unwrap();
            panic!(
                "sequential worker exceeded the fixture deadline: {}",
                fs::read_to_string(&error_path).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "{}",
        fs::read_to_string(&error_path).unwrap()
    );
    let replies = fs::read_to_string(output_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(replies.len(), 3);
    let identity = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("compiler-identity")
        .output()
        .unwrap();
    assert!(identity.status.success());
    let identity: serde_json::Value = serde_json::from_slice(&identity.stdout).unwrap();
    for (index, name) in ["failure", "first", "second"].into_iter().enumerate() {
        let id = format!("raw-script:language/deadline/{name}.js");
        assert_eq!(replies[index]["sequence"], index + 1);
        assert_eq!(replies[index]["test_id"], id);
        assert_eq!(
            replies[index]["outcome"],
            if index == 0 { "failed" } else { "passed" }
        );
        let snapshot = fs::read_dir(root.join(name))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(snapshot).unwrap()).unwrap();
        assert_eq!(value["total"], 1);
        assert_eq!(value["passed"], usize::from(index != 0));
        assert_eq!(value["completed_test_ids"], serde_json::json!([id]));
        assert_eq!(value["compiler_identity"], identity);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reused_supervisor_preserves_semantic_failures_and_every_remaining_id() {
    let root = fixture("reuse-failure");
    for (name, source) in [("a", "throw 17;"), ("b", "40 + 2;"), ("c", "6 * 7;")] {
        fs::write(
            root.join(format!("suite/test/language/deadline/{name}.js")),
            format!("/*---\nflags: [raw]\n---*/\n{source}\n"),
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "test262", "run", "language/deadline"])
        .arg("--suite-root")
        .arg(root.join("suite"))
        .arg("--snapshot-dir")
        .arg(root.join("snapshots"))
        .args([
            "--snapshot-name",
            "reuse-failure",
            "--threads",
            "1",
            "--reuse-case-workers",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let snapshot = fs::read_dir(root.join("snapshots"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(snapshot).unwrap()).unwrap();
    assert_eq!(value["total"], 4);
    assert_eq!(value["passed"], 3);
    assert_eq!(value["completed_test_ids"].as_array().unwrap().len(), 4);
    assert_eq!(value["failures"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["failures"][0]["test_id"],
        "raw-script:language/deadline/a.js"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reused_supervisor_replaces_workers_at_the_bounded_request_limit() {
    let root = fixture("reuse-limit");
    let source =
        "/*---\nflags: [raw]\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/\nconst = ;\n";
    for index in 0..70 {
        fs::write(
            root.join(format!("suite/test/language/deadline/negative-{index}.js")),
            source,
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "test262", "run", "language/deadline"])
        .arg("--suite-root")
        .arg(root.join("suite"))
        .arg("--snapshot-dir")
        .arg(root.join("snapshots"))
        .args([
            "--snapshot-name",
            "reuse-limit",
            "--threads",
            "1",
            "--timeout-ms",
            "1000",
            "--reuse-case-workers",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let snapshot = fs::read_dir(root.join("snapshots"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(snapshot).unwrap()).unwrap();
    assert_eq!(value["total"], 71);
    assert_eq!(value["passed"], 71);
    let mut expected = (0..70)
        .map(|index| format!("raw-script:language/deadline/negative-{index}.js"))
        .collect::<Vec<_>>();
    expected.push("raw-script:language/deadline/negative.js".into());
    expected.sort();
    assert_eq!(value["completed_test_ids"], serde_json::json!(expected));
    fs::remove_dir_all(root).unwrap();
}
