//! `language` CLI integration tests.

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
fn run_wasm_backend_succeeds_for_spec_to_boolean_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_to_boolean.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_is_constructor_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_is_constructor.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_to_number_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_to_number.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_to_string_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_to_string.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_strict_equality_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_strict_equality.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_same_value_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_same_value.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_get_v_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_get_v.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_spec_has_property_order_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_spec_has_property_order.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_coercive_relational_runtime_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_coercive_relational_runtime.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_succeeds_for_abstract_module_source_host_hook_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_abstract_module_source_host_hook.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_htmldda_host_hook_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_htmldda_host_hook.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
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
fn run_wasm_backend_succeeds_for_json_parse_invalid_literal_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_json_parse_invalid_literal.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_json_parse_reviver_array_getter_throw_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_json_parse_reviver_array_getter_throw.js",
        ))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_json_parse_reviver_forward_modification_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_json_parse_reviver_forward_modification.js",
        ))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_json_parse_reviver_nonconfigurable_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_json_parse_reviver_nonconfigurable.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
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
fn run_wasm_backend_succeeds_for_aggregateerror_cross_realm_newtarget_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_aggregateerror_cross_realm_newtarget.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_uses_async_function_intrinsic_prototype_topology() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_async_function_intrinsics.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_boolean_cross_realm_newtarget_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_boolean_cross_realm_newtarget.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_bigint_minimal_validation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_minimal_validation.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(5"));
}

#[test]
fn run_wasm_backend_observes_bigint_wrapper_ordinary_to_primitive_hooks() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_wrapper_ordinary_to_primitive.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(6"));
}

#[test]
fn run_wasm_backend_preserves_exact_bigint_number_loose_equality() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_loose_equality.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(137"));
}

#[test]
fn run_wasm_backend_preserves_bigint_number_relational_comparison() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_number_relational.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(1"));
}

#[test]
fn run_wasm_backend_parses_arbitrary_precision_bigint_strings() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_string_arbitrary_precision.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(1"));
}

#[test]
fn run_wasm_backend_converts_arbitrary_precision_bigint_strings_to_fixed_width() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_string_fixed_width.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(1"));
}

#[test]
fn run_wasm_backend_truncates_bigints_at_arbitrary_widths() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bigint_as_n_arbitrary_width.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(1"));
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
fn run_wasm_backend_succeeds_for_to_locale_string_boxed_receivers_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_to_locale_string_boxed_receivers.js"))
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
fn run_wasm_backend_routes_abrupt_completions_through_finally() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_abrupt_finally_routing.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_preserves_catch_finally_derived_return_completions() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_catch_finally_derived_return.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_non_callable_catchability_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_non_callable_catchability.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("string(call,construct,error-call,error-construct)"));
}

#[test]
fn run_wasm_backend_succeeds_for_error_tostring_toprimitive_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_tostring_toprimitive.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("string(message,name)"));
}

#[test]
fn run_wasm_backend_succeeds_for_error_iserror_other_realm_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_iserror_other_realm.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains(
        "string(Error,EvalError,RangeError,ReferenceError,SyntaxError,TypeError,URIError,AggregateError)"
    ));
}

#[test]
fn run_wasm_backend_succeeds_for_error_constructor_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_constructor_properties.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_throwtypeerror_intrinsic_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_throwtypeerror_intrinsic_properties.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_reflect_set_core_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_reflect_set_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_reports_uncaught_throw_fixture_error() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_uncaught_throw.js"))
        .output()
        .expect("run command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("uncaught throw: wasm-aot completion: number(1)"));
}

#[test]
fn run_wasm_backend_reports_gc_requires_real_collector() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_gc_requires_real_collector.js"))
        .output()
        .expect("run command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("uncaught throw"));
    assert!(stderr.contains("gc requires a real collector in wasm-aot"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_abrupt_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_abrupt_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
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
fn run_wasm_backend_preserves_number_isnan_global_nan_binding_result() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_number_isnan_global.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_newtarget_prototype_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_aggregateerror_newtarget_prototype.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_iterable_to_list_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_aggregateerror_iterable_to_list.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_constructor_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_aggregateerror_constructor_properties.js",
        ))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_suppressederror_constructor_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_suppressederror_constructor_properties.js",
        ))
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

#[test]
fn run_wasm_backend_succeeds_for_static_exponentiation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_exponentiation_static_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_dynamic_integer_exponentiation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_exponentiation_dynamic_integer_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_dynamic_fractional_exponentiation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_exponentiation_dynamic_fractional_core.js",
        ))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_special_number_exponentiation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_exponentiation_special_numbers_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_bigint_exponentiation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_exponentiation_bigint_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_math_min_max_arity_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_math_min_max_arity.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_annexb_escape_unescape_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_annexb_escape_unescape_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_uri_codecs_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_uri_codecs_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_bound_construct_new_target_identity_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bound_construct_new_target_identity.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_construct_function_realm_fallback_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_construct_function_realm_fallback.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_preserves_async_generator_yield_star_acquisition_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_wrapper_acquisition.js",
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
    assert!(stdout.contains("number(5"), "{stdout}");
}

#[test]
fn run_wasm_backend_preserves_async_generator_yield_star_lexical_initialization() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_lexical_initialization.js",
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
    assert!(
        stdout.contains("async-generator-yield-star-lexical-initialization:11:13"),
        "{stdout}"
    );
}

#[test]
fn run_wasm_backend_validates_async_generator_yield_star_next_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_next_wrapper_validation.js",
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
    assert!(
        stdout.contains("async-generator-next-wrapper-validation:12:false"),
        "{stdout}"
    );
}

#[test]
fn run_wasm_backend_validates_async_generator_yield_star_return_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_return_wrapper_validation.js",
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
    assert!(
        stdout.contains("async-generator-return-wrapper-validation:4"),
        "{stdout}"
    );
}

#[test]
fn run_wasm_backend_validates_async_generator_yield_star_throw_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_throw_wrapper_validation.js",
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
    assert!(
        stdout.contains("async-generator-throw-wrapper-validation:4"),
        "{stdout}"
    );
}
