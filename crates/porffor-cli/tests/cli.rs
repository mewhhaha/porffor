use std::process::Command;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

fn suite_root() -> String {
    format!(
        "{}/../porffor-test262/tests/fixtures/fake_test262/vendor/test262",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn snapshot_dir() -> String {
    std::env::temp_dir()
        .join(format!("porffor-cli-test262-{}", std::process::id()))
        .display()
        .to_string()
}

#[test]
fn help_lists_clean_break_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("--help")
        .output()
        .expect("help command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build wasm"));
    assert!(stdout.contains("test262 run"));
    assert!(stdout.contains("inspect"));
}

#[test]
fn inspect_reports_pipeline_invariants() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("hello.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("goal: Script"));
    assert!(stdout.contains("direct-js-to-wasm-only"));
}

#[test]
fn build_wasm_failure_repeats_product_rule() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("hello.js"))
        .output()
        .expect("build command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("compile JavaScript directly to Wasm"));
    assert!(stderr.contains("interpreter-in-Wasm"));
}

#[test]
fn test262_list_works_with_fixture_suite() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("list")
        .arg("--suite-root")
        .arg(suite_root())
        .output()
        .expect("test262 list should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("count: 3"));
}

#[test]
fn test262_run_writes_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("run")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-fixture")
        .output()
        .expect("test262 run should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("total: 3"));
    assert!(stdout.contains("passed: 3"));
    assert!(stdout.contains("Unsupported: 0"));
}

#[test]
fn test262_report_groups_failures_by_bucket() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("report")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-report")
        .output()
        .expect("test262 report should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("passed: 3"));
    assert!(stdout.contains("failed: 0"));
}

#[test]
fn test262_report_all_aggregates_fixture_suite() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("report-all")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-report-all")
        .output()
        .expect("test262 report-all should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("total: 3"));
    assert!(stdout.contains("passed: 3"));
    assert!(stdout.contains("targets:"));
}
