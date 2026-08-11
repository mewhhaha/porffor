//! `language` CLI integration tests: pipeline shape, bindings, scoping,
//! closures, `for-in` and TDZ.
//!
//! # Why this module is three modules
//!
//! Not taste — memory, measured. As one 105-test libtest process this module
//! could not be run at all on the 4-CPU / 15.7 GiB container: three attempts
//! (batch 6, 22:43Z / 23:29Z / 00:30Z) were each SIGKILLed by the OOM killer at
//! t+1200 s after 66, 75 and 75 tests, with `avail` falling MONOTONICALLY from
//! 8.5 GiB at ~7 tests to 3.56 GiB at ~49 and 1.14 GiB two minutes before the
//! kill. That trajectory is cumulative growth across the process, not a
//! three-in-flight plateau (contrast `frontend_test262_subset`, whose 5.55 GiB
//! is flat), so fewer tests per process was the lever this split reached for:
//!
//!   * per-tier cache limits at 256/64/64 MiB were tried and changed nothing —
//!     they bound bytes on disk, not RSS (`porffor-engine/src/cache.rs`);
//!   * `PORFFOR_CPU_PERCENT` is overridden inside `run_chunk`
//!     (`scripts/rung1c-chunks.sh`);
//!   * `--test-threads` below 3 is banned by that script's property 1 — libtest
//!     then names every worker thread `main`, `known_failures::execution_path`
//!     cannot route on the per-test name, and all ~600 tests fall back to
//!     spawning a cold `porf` child.
//!
//! # What the growth actually is, and why "the only lever" was the wrong claim
//!
//! An earlier version of this comment called the split "the only lever left".
//! That list is three *environment knobs*; it never examined in-process
//! retention, and the accumulation has a named mechanism there:
//! `WASM_MODULE_MEMORY_CACHE_ENTRIES` (`porffor-engine/src/lib.rs`) bounds a
//! `VecDeque` LRU of fully compiled Wasmtime modules **by entry count and by
//! nothing else** — 64 entries, no byte ceiling. The in-process path these
//! tests take retains into it (`WasmModuleMemoryCachePolicy::Retain`), so it
//! holds one native module per distinct fixture. That is why the three disk
//! knobs did nothing, and it is why `frontend_test262_subset` is flat: it is
//! ONE test, so it caches ONE module.
//!
//! It also corrects the sizing model below. Growth is capped at 64 entries, so
//! `avail` cannot fall linearly forever — it plateaus, and the right unit is
//! cached modules, not tests. Twelve of the 13 "cheap" tests kept here cache
//! nothing at all: every `build_wasm_succeeds_for_*` and
//! `inspect_reports_phase_*` runs `Command::new(env!("CARGO_BIN_EXE_porf"))`, a
//! CHILD process, so it touches neither this process's RSS nor its module
//! cache. Only `in_process_module_reuse_*` does. In those units the three fatal
//! runs reached 53, 62 and 62 cached modules — just short of the 64 cap — a
//! two-way split at ~52 heavy tests would land at ~52 cached modules, i.e.
//! INSIDE that fatal band, and this three-way split lands at roughly 33/29/31,
//! about half of it. The linear extrapolation below reaches the same answer,
//! but for a reason that is wrong in the only regime that matters.
//!
//! `PORFFOR_MODULE_MEMORY_CACHE_ENTRIES` now overrides that bound, so a
//! memory-constrained run has a lever that needs no code change; bounding the
//! deque by bytes, as the disk tiers already are, is the standing follow-up.
//!
//! Splitting by libtest FILTER is not available either:
//! `known_failures::rung_1c_chunks` asserts each chunk's second argument is
//! exactly `<name>::` and that anything further is `--skip <other>::`, and
//! `rung_1c_chunks_cover_every_cli_area_module` asserts a bijection between
//! chunk names, `tests/cli/*.rs` stems and `mod` lines in `main.rs`. An
//! `--exact` name list is rejected at rung 0. So the split has to be by module
//! file, which is what [`crate::language_errors`] and [`crate::language_numerics`]
//! are.
//!
//! Sizing came from the measurements, not from taste. 75 tests in 1200 s is
//! 16.0 s/test and the standalone 30-test tail did 30 in 498.2 s (16.6 s/test),
//! so per-test cost is uniform enough to size by count; the `avail` trajectory
//! (8.5 GiB @ ~7 tests, 3.56 @ ~49, 1.14 @ ~67) is ~0.118 GiB/test. Thirty-odd
//! heavy tests lands near 5 GiB `avail`; a two-way split at ~52 heavy tests
//! lands near 3.2 GiB, past where the third attempt was already in trouble.
//! Hence three, at 32 / 29 / 31 heavy tests. This module keeps the 13 cheap
//! ones (`in_process_module_reuse_*`, six `inspect_reports_phase_*`, six
//! `build_wasm_succeeds_for_*`) as well, since they cost almost nothing.
//!
//! THE THREE STEMS MUST NOT BECOME `::`-SUFFIXES OF ONE ANOTHER. The overlap
//! rule in `rung_1c_chunks_cover_every_cli_area_module` fires when
//! `format!("{other}::").ends_with(&format!("{chunk}::"))`, and
//! `"language_errors::".ends_with("language::")` is false — which is exactly why
//! these three need no `--skip` while `array` needs `--skip typed_array::`. By
//! the same token libtest's substring filter `language::` does not select
//! `language_errors::…`, so the three chunks run in three SEPARATE processes,
//! which is the entire point: the accumulation is per-process.

use crate::*;

#[test]
fn in_process_module_reuse_keeps_host_output_in_fresh_realms() {
    let path = fixture_path("wasm_host_output.js");
    let args = ["run", "--execution-backend", "wasm", path.as_str()];
    for _ in 0..2 {
        let output = porffor_cli::run_cli_capture(args.map(str::to_string));
        assert_eq!(
            output.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.matches("root\n").count(), 1);
        assert_eq!(stdout.matches("alias\n").count(), 1);
        assert_eq!(stdout.matches("method\n").count(), 1);
    }
}

#[test]
fn inspect_reports_phase_nineteen_global_resolution_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_global_resolution.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("global_property_reads="));
    assert!(stdout.contains("global_property_writes="));
    assert!(stdout.contains("implicit_globals="));
}

#[test]
fn inspect_reports_phase_twenty_host_output_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_host_output.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("host_globals=1"));
    assert!(stdout.contains("host_builtin_calls=3"));
}

#[test]
fn inspect_reports_phase_twenty_four_abrupt_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_abrupt_core.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("try_finallys=2"));
    assert!(stdout.contains("deletes=2"));
    assert!(stdout.contains("spec_operations=7"));
    assert!(stdout.contains("in_ops=0"));
    assert!(stdout.contains("new_target_uses=3"));
}

#[test]
fn inspect_reports_phase_twenty_five_builtin_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_builtin_globals.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("builtin_globals=51"));
    assert!(stdout.contains("builtin_ctor_calls="));
    assert!(stdout.contains("builtin_static_calls="));
    assert!(stdout.contains("error_builtin_calls="));
}

#[test]
fn inspect_reports_phase_twenty_nine_delete_global_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_delete_globals.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("deletes="));
    assert!(stdout.contains("identifier_deletes="));
    assert!(stdout.contains("global_deletes="));
}

#[test]
fn inspect_reports_phase_thirty_null_heritage_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_null_heritage.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("null_heritage_classes="));
}

#[test]
fn build_wasm_succeeds_for_dynamic_fractional_exponentiation_fixture() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_exponentiation_dynamic_fractional_core.js",
        ))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_global_resolution_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_global_resolution.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_abrupt_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_abrupt_core.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_builtin_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_builtin_globals.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_delete_global_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_delete_globals.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_null_heritage_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_null_heritage.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn run_wasm_backend_preserves_var_parameter_bindings() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_var_parameter_bindings.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_preserves_outer_bindings_during_recursion() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_recursive_function_outer_binding.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_supports_annex_b_block_functions() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_annexb_block_functions.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_captures_annex_b_block_function_bindings() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_annexb_block_capture_aliases.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_preserves_function_values_after_for_lexical_initializers() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_lexical_function_value.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(9)"));
}

#[test]
fn run_wasm_backend_preserves_captured_block_lexical_environments() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_block_lexical_environments.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_for_in_array_key_capture_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_array_key_capture.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_script_global_var_nested_update_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_script_global_var_nested_update.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_optional_property_chain_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_optional_property_chain.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_optional_private_property_chain_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_optional_private_property_chain.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_supported_strict_this_calls_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_strict_this_calls.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_lexical_super_home_object_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_lexical_super_home_object.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_block_function_declaration_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_block_function_declaration.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_lexical_shadowing_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_lexical_shadowing.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_script_lexical_capture_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_script_lexical_capture.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_for_in_let_closure_capture_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_let_closure_capture.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_shadowed_for_in_let_closure_capture_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_shadowed_let_closure_capture.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_preserves_supported_binding_pattern_capture_storage_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_binding_pattern_capture_storage_contract.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_for_in_order_simple_object_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_order_simple_object.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_for_in_prototype_order_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_prototype_order.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_for_in_array_define_property_order_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_array_define_property_order.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_for_in_head_tdz_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_for_in_head_tdz.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_enforces_runtime_lexical_tdz_for_pattern_initializers() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_runtime_lexical_tdz_patterns.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_uses_iterators_for_call_argument_spread() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_call_argument_spread_iterators.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_preserves_depth_two_const_array_capture_immutability() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_depth_two_const_array_capture.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_missing_arguments_shadowing_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_missing_arguments_shadowing.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_global_resolution_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_global_resolution.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(4"));
}

#[test]
fn run_wasm_backend_succeeds_for_global_constant_descriptors_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_global_constant_descriptors.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_builtin_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_builtin_globals.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_delete_global_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_delete_globals.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_null_heritage_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_null_heritage.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

/// A hoisted function's `const` capture must not be typed from the hoist-time
/// TDZ placeholder.
///
/// Function declarations are lowered before the statement list, so when
/// `function fb() { return B; }` captures the top-level `const B`, `B` is still
/// the uninitialized placeholder whose kind is `Undefined`. Publishing that as
/// the capture's proven value propagates into `signature.return_kind`, and then
/// `typeof fb()` constant-folds to `"undefined"` without ever calling `fb`.
///
/// Both fields are plain observable JavaScript, so this stays a black-box
/// check rather than an assertion about inferred kinds. The fixture documents
/// the wider const-capture operator-selection defect that this test
/// deliberately does not cover.
#[test]
fn run_wasm_backend_types_a_hoisted_functions_const_capture_from_its_initializer() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_const_capture_return_kind.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(
        stdout.contains("const-capture-return-kind:object:1"),
        "{stdout}"
    );
}
