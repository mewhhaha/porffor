//! `Intl` CLI integration tests.

use crate::*;

/// Pins ECMA-402 `Intl.DateTimeFormat` construction order: the tagged result
/// is reserved through NewTarget.prototype before locale/options observation,
/// then published only after its record and brand are complete.
#[test]
fn run_wasm_intl_date_time_format_construction_order_fixture_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_intl_date_time_format_construction_order.js",
        ))
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
    assert!(stdout.contains("number(262"), "{stdout}");
}

/// Pins ECMA-402 `Intl.Locale` construction order: NewTarget prototype
/// resolution reserves an unreachable object before tag/options observation,
/// and only the fully initialized Locale state is published.
#[test]
fn run_wasm_intl_locale_construction_order_fixture_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_intl_locale_construction_order.js"))
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
    assert!(stdout.contains("number(262"), "{stdout}");
}

/// Pins the distinct canonical tag/component result roles consumed by
/// `Intl.Locale`, `Intl.getCanonicalLocales`, and `Intl.DateTimeFormat`.
#[test]
fn run_wasm_intl_canonical_locale_tag_roles_fixture_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_intl_canonical_locale_tag_roles.js"))
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
    assert!(stdout.contains("number(262"), "{stdout}");
}
