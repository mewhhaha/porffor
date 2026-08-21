//! Explicit-resource-management CLI integration tests.

use crate::*;

#[test]
fn wasm_disposable_stack_constructor_surface() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_disposable_stack_constructor_surface.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(
        stdout.contains("disposable-stack-async-brand:true"),
        "{stdout}"
    );
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn wasm_disposable_stack_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_disposable_stack_lifecycle.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn wasm_using_synchronous_scope_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_using_synchronous_scope.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn wasm_using_plain_generator_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_using_plain_generator_lifecycle.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn wasm_using_plain_async_function_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_using_plain_async_function_lifecycle.js"))
        .output()
        .expect("run command should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(
        stdout.contains("using-plain-async-function:true"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("using-plain-async-function:FAILED")
            && !stdout.contains("uncaught throw")
            && !stderr.contains("uncaught throw"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn wasm_await_using_plain_async_function_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_await_using_plain_async_function_lifecycle.js",
        ))
        .output()
        .expect("run command should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("await-using-plain-async:true"), "{stdout}");
    assert!(
        !stdout.contains("await-using-plain-async:FAILED")
            && !stdout.contains("uncaught throw")
            && !stderr.contains("uncaught throw"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn wasm_await_using_async_generator_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_await_using_async_generator_lifecycle.js",
        ))
        .output()
        .expect("run command should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(
        stdout.contains("await-using-async-generator:true"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("await-using-async-generator:FAILED")
            && !stdout.contains("uncaught throw")
            && !stderr.contains("uncaught throw"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn wasm_using_async_generator_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_using_async_generator_lifecycle.js"))
        .output()
        .expect("run command should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("using-async-generator:true"), "{stdout}");
    assert!(
        !stdout.contains("using-async-generator:FAILED")
            && !stdout.contains("uncaught throw")
            && !stderr.contains("uncaught throw"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn wasm_using_classic_for_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_using_classic_for_head.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn wasm_await_using_classic_for_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_await_using_classic_for_lifecycle.js"))
        .output()
        .expect("run command should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("await-using-classic-for:true"), "{stdout}");
    assert!(
        !stdout.contains("await-using-classic-for:FAILED")
            && !stdout.contains("uncaught throw")
            && !stderr.contains("uncaught throw"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn wasm_using_for_of_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_using_for_of_lifecycle.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}
