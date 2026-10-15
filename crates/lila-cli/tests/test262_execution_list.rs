//! Exact cohorts use native supervised execution and native resume evidence.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "lila-execution-list-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("suite/test/language/list")).unwrap();
    for name in ["a", "b"] {
        fs::write(root.join(format!("suite/test/language/list/{name}.js")),
            "/*---\n---*/\nif (typeof previousCase !== 'undefined') throw new Error('Realm leaked'); var previousCase = 1;\n").unwrap();
    }
    fs::write(
        root.join("suite/test/language/list/failure.js"),
        "/*---\nflags: [raw]\n---*/\nthrow new Error('original failure');\n",
    )
    .unwrap();
    root
}

fn run(root: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "test262", "run-list"])
        .arg(root.join("executions"))
        .arg("--suite-root")
        .arg(root.join("suite"))
        .arg("--snapshot-dir")
        .arg(root.join("snapshots"))
        .args([
            "--snapshot-name",
            "cohort",
            "--threads",
            "1",
            "--timeout-ms",
            "1000",
            "--reuse-case-workers",
        ])
        .args(extra)
        .env("LILA_CACHE_DIR", root.join("cache"))
        .env("LILA_FUNCTION_CACHE_LIMIT_BYTES", "67108864")
        .env("LILA_MODULE_CACHE_LIMIT_BYTES", "67108864")
        .env("LILA_PROGRAM_CACHE_LIMIT_BYTES", "67108864")
        .output()
        .unwrap()
}

fn snapshot(root: &Path) -> serde_json::Value {
    let path = fs::read_dir(root.join("snapshots"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .unwrap();
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn execution_list_preserves_modes_and_fresh_realms_with_bounded_worker_reuse() {
    let root = fixture();
    let ids = [
        "strict-script:language/list/a.js",
        "sloppy-script:language/list/b.js",
    ];
    fs::write(root.join("executions"), ids.join("\n")).unwrap();
    let result = run(&root, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value = snapshot(&root);
    assert_eq!(value["total"], 2);
    assert_eq!(value["passed"], 2);
    assert_eq!(value["completed_test_ids"], serde_json::json!(ids));
    let inventory = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["test262", "list-all", "--suite-root"])
        .arg(root.join("suite"))
        .output()
        .unwrap();
    assert!(inventory.status.success());
    let text = String::from_utf8(inventory.stdout).unwrap();
    assert!(text.starts_with("count: 5\n"));
    assert_eq!(
        text.lines().count(),
        6,
        "list-all cannot truncate the selected inventory"
    );
    assert!(text.contains("sloppy-script:language/list/a.js\n"));
    assert!(text.contains("raw-script:language/list/failure.js\n"));
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
fn execution_list_keeps_actual_failure_and_resume_red_without_repeating_cases() {
    let root = fixture();
    fs::write(
        root.join("executions"),
        "strict-script:language/list/a.js\nraw-script:language/list/failure.js\n",
    )
    .unwrap();
    let first = run(&root, &[]);
    assert_eq!(first.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&first.stderr).contains("test262 run-list failed: 1 of 2"));
    let before = snapshot(&root);
    assert_eq!(before["passed"], 1);
    assert_eq!(before["counts_per_outcome"]["Bug"], 1);
    assert_eq!(
        before["failures"][0]["test_id"],
        "raw-script:language/list/failure.js"
    );
    assert!(before["failures"][0]["duration_ms"].as_u64().is_some());
    assert_eq!(before["slowest_tests"].as_array().unwrap().len(), 2);
    let resumed = run(&root, &["--resume"]);
    assert_eq!(resumed.status.code(), Some(1));
    assert_eq!(
        snapshot(&root),
        before,
        "completed native evidence must be reused exactly"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn execution_list_missing_file_or_mode_cannot_publish_partial_passing_evidence() {
    for missing in [
        "raw-script:language/list/a.js",
        "strict-script:language/list/absent.js",
    ] {
        let root = fixture();
        fs::write(
            root.join("executions"),
            format!("sloppy-script:language/list/a.js\n{missing}\n"),
        )
        .unwrap();
        let result = run(&root, &[]);
        assert_eq!(result.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&result.stderr).contains(missing));
        assert!(!root.join("snapshots").exists());
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn four_case_workers_keep_exact_native_shards_realms_and_literal_failures() {
    use std::collections::BTreeMap;
    let root = fixture();
    for name in ["c", "d", "e", "f"] {
        fs::write(root.join(format!("suite/test/language/list/{name}.js")),
            "/*---\n---*/\nif (typeof previousCase !== 'undefined') throw new Error('Realm leaked'); var previousCase = 1;\n").unwrap();
    }
    let execute = |stage: &str, shard: Option<&str>, reuse: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lila"));
        command.args([
            "--jobs",
            "1",
            "test262",
            if shard.is_some() { "shard" } else { "run" },
        ]);
        if let Some(shard) = shard {
            command.arg(shard);
        }
        command
            .arg("language/list")
            .arg("--suite-root")
            .arg(root.join("suite"))
            .arg("--snapshot-dir")
            .arg(root.join(stage).join("snapshots"))
            .args([
                "--snapshot-name",
                "cohort",
                "--threads",
                if shard.is_some() { "4" } else { "1" },
                "--timeout-ms",
                "1000",
            ])
            .env("LILA_CACHE_DIR", root.join("cache"))
            .env("LILA_FUNCTION_CACHE_LIMIT_BYTES", "67108864")
            .env("LILA_MODULE_CACHE_LIMIT_BYTES", "67108864")
            .env("LILA_PROGRAM_CACHE_LIMIT_BYTES", "67108864");
        if reuse {
            command.arg("--reuse-case-workers");
        }
        let output = command.output().unwrap();
        assert!(
            matches!(output.status.code(), Some(0 | 1)),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = snapshot(&root.join(stage));
        let mut results = BTreeMap::new();
        for id in value["completed_test_ids"].as_array().unwrap() {
            assert!(results
                .insert(id.as_str().unwrap().to_owned(), "Success".to_owned())
                .is_none());
        }
        for failure in value["failures"].as_array().unwrap() {
            *results
                .get_mut(failure["test_id"].as_str().unwrap())
                .unwrap() = failure["outcome"].as_str().unwrap().to_owned();
        }
        assert_eq!(
            output.status.success(),
            results.values().all(|value| value == "Success")
        );
        (value["compiler_identity"].clone(), results)
    };
    let (compiler, serial) = execute("serial", None, false);
    assert_eq!(serial.len(), 13);
    assert_eq!(serial["raw-script:language/list/failure.js"], "Bug");
    let mut union = BTreeMap::new();
    for (stage, shard, reuse) in [("first", "1/2", false), ("second", "2/2", true)] {
        let (identity, rows) = execute(stage, Some(shard), reuse);
        assert_eq!(identity, compiler);
        for (id, outcome) in rows {
            assert!(union.insert(id, outcome).is_none());
        }
    }
    assert_eq!(union, serial);
    fs::remove_dir_all(root).unwrap();
}
