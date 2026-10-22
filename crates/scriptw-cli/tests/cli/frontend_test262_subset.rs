//! Complete fake Wasm-AOT subset through the real, supervised CLI image.
//! Keep this scope separate in partitioned verification: its two case workers
//! can have a larger peak than an ordinary fixture. Reuse bounds compiler/cache
//! setup while each case still receives a fresh Store/realm and its deadline.
//! All 187 fixtures and their verdict assertions remain part of the check.

use crate::*;

#[test]
fn test262_wasm_backend_runs_supported_fixture_subset() {
    let output = Command::new(env!("CARGO_BIN_EXE_scriptw"))
        .args(["--jobs", "1"])
        .arg("test262")
        .arg("run")
        .arg("language/wasm/pass")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-wasm-fixture")
        .arg("--execution-backend")
        .arg("wasm")
        // Two supervised case workers; reuse immutable compiler/module state,
        // not the JavaScript Store or realm. The Command wrapper retains the
        // existing 900-second whole-child bound for Test262 invocations.
        .arg("--threads")
        .arg("2")
        .arg("--reuse-case-workers")
        .output()
        .expect("test262 wasm run should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: wasm-aot"));
    assert!(stdout.contains("total: 187"));
    assert!(stdout.contains("passed: 187"));
}
