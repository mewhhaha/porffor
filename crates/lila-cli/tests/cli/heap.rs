//! `heap` CLI integration tests.

use crate::*;

#[test]
fn inspect_reports_phase_nine_heap_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("inspect")
        .arg(fixture_path("wasm_heap_shapes.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("array_lengths=1"));
    assert!(stdout.contains("heap_shapes="));
}

#[test]
fn inspect_reports_phase_seventeen_heap_coercion_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("inspect")
        .arg(fixture_path("wasm_heap_coercions.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("heap_to_primitives="));
    assert!(stdout.contains("heap_loose_equalities="));
    assert!(stdout.contains("heap_coercions="));
}

#[test]
fn build_wasm_succeeds_for_supported_heap_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_shapes.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_heap_coercion_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_coercions.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_heap_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_shapes.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(10"));
}

#[test]
fn run_wasm_backend_succeeds_for_heap_memory_growth_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_memory_growth.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(66"));
}

/// Declared `ignored` in `tests/known-failures.tsv`, owner T05.
///
/// `pub(crate)` so `known_failures.rs` can assert at compile time that this
/// function still exists under this name.
#[test]
#[ignore = "T05 allocation stress; run explicitly with --ignored"]
pub(crate) fn run_wasm_backend_succeeds_for_heap_page_boundary_stress_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_page_boundary_stress.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(81"));
}

#[test]
fn run_wasm_backend_succeeds_for_heap_dynamic_alignment_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_dynamic_alignment.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(38"));
}

#[test]
fn run_wasm_backend_succeeds_for_heap_rooted_closure_exception_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_rooted_closure_exception.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(48"));
}

#[test]
fn run_wasm_backend_succeeds_for_heap_rooted_bound_function_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_rooted_bound_function.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(58"));
}

#[test]
fn run_wasm_backend_succeeds_for_heap_rooted_generator_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_rooted_generator.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(50"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_heap_coercion_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_heap_coercions.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("[object Arguments]"));
}
