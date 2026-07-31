//! `frontend` CLI integration tests.

use crate::*;

#[test]
fn help_lists_clean_break_commands() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("--help")
        .output()
        .expect("help command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build wasm"));
    assert!(stdout.contains("types [entrypoint]"));
    assert!(stdout.contains("test262 run"));
    assert!(stdout.contains("inspect"));
}

#[test]
fn subprocess_argument_handling_rejects_missing_run_source() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .output()
        .expect("run command should report a missing source");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("run needs a source file"));
}

#[test]
fn types_generates_from_discovered_jsonc_config_and_entrypoint() {
    let project = unique_project_dir("types-jsonc");
    write_project_file(
        &project,
        "src/index.ts",
        r#"
export default {
  fetch(request, env, ctx) {
    return new Response(env.MESSAGE);
  },
  scheduled() {}
};
"#,
    );
    write_project_file(
        &project,
        "wrangler.jsonc",
        r#"{
  // JSONC comments and trailing commas match common Wrangler config files.
  "main": "src/index.ts",
  "compatibility_date": "2026-06-19",
  "kv_namespaces": [{ "binding": "CACHE", "id": "x" }],
  "r2_buckets": [{ "binding": "ASSETS", "bucket_name": "assets" }],
  "vars": { "MESSAGE": "hello", "COUNT": 3 },
  "env": {
    "prod": {
      "vars": { "MESSAGE": "prod" },
      "queues": { "producers": [{ "binding": "JOBS", "queue": "jobs" }] }
    },
  },
}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("types")
        .arg("--cwd")
        .arg(&project)
        .arg("--print")
        .output()
        .expect("types command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("// Config: wrangler.jsonc"));
    assert!(stdout.contains("// Entrypoint: src/index.ts"));
    assert!(stdout.contains("// Detected handlers: fetch, scheduled"));
    assert!(stdout.contains("CACHE: KVNamespace;"));
    assert!(stdout.contains("ASSETS: R2Bucket;"));
    assert!(stdout.contains("COUNT: 3;"));
    assert!(stdout.contains("MESSAGE: \"hello\" | \"prod\";"));
    assert!(stdout.contains("JOBS?: Queue;"));
    assert!(stdout.contains("compatibilityDate: \"2026-06-19\";"));
}

#[test]
fn types_uses_explicit_entrypoint_and_check_for_written_output() {
    let project = unique_project_dir("types-explicit");
    write_project_file(
        &project,
        "src/worker.ts",
        r#"
export default {
  fetch() {
    return new Response("ok");
  }
};
"#,
    );
    write_project_file(
        &project,
        "src/config-main.ts",
        r#"
export default {
  scheduled() {}
};
"#,
    );
    write_project_file(
        &project,
        "wrangler.json",
        r#"{"main":"src/config-main.ts","vars":{"FLAG":true},"services":[{"binding":"API","service":"api"}]}"#,
    );

    let output_path = "types/worker-env.d.ts";
    let write = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("typegen")
        .arg("src/worker.ts")
        .arg(output_path)
        .arg("--cwd")
        .arg(&project)
        .arg("--config")
        .arg("wrangler.json")
        .output()
        .expect("typegen command should write");

    assert!(
        write.status.success(),
        "{}",
        String::from_utf8_lossy(&write.stderr)
    );

    let check = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("types")
        .arg("src/worker.ts")
        .arg(output_path)
        .arg("--cwd")
        .arg(&project)
        .arg("--config")
        .arg("wrangler.json")
        .arg("--check")
        .output()
        .expect("types --check should run");

    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let written = fs::read_to_string(project.join(output_path)).expect("types output should exist");
    assert!(written.contains("FLAG: true;"));
    assert!(written.contains("API: Fetcher;"));
    assert!(written.contains("entrypoint: \"src/worker.ts\";"));
    assert!(written.contains("syntax: \"module\";"));
    assert!(written.contains("// Detected handlers: fetch"));
    assert!(!written.contains("// Detected handlers: scheduled"));
}

#[test]
fn types_reads_toml_env_and_can_omit_runtime_declarations() {
    let project = unique_project_dir("types-toml");
    write_project_file(
        &project,
        "src/worker.ts",
        r#"
addEventListener("fetch", event => event.respondWith(new Response("ok")));
"#,
    );
    write_project_file(
        &project,
        "wrangler.toml",
        r#"
main = "src/worker.ts"
compatibility_date = "2026-06-19"

[vars]
MODE = "dev"

[[d1_databases]]
binding = "DB"
database_name = "main"

[env.preview.vars]
MODE = "preview"
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("typegen")
        .arg("--cwd")
        .arg(&project)
        .arg("--env")
        .arg("preview")
        .arg("--env-interface")
        .arg("WorkerEnv")
        .arg("--include-runtime=false")
        .arg("--print")
        .output()
        .expect("typegen toml command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("interface WorkerEnv"));
    assert!(stdout.contains("MODE: \"preview\";"));
    assert!(stdout.contains("DB: D1Database;"));
    assert!(stdout.contains("syntax: \"service-worker\";"));
    assert!(!stdout.contains("ExecutionContext"));
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
    assert!(stdout.contains("stages: parsed-source, ast-reparsed, script-ir-built, wasm-ready"));
}

#[test]
fn inspect_reports_phase_five_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_switch.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ir: script statements="));
    assert!(stdout.contains("switches=1"));
    assert!(stdout.contains("labels=1"));
    assert!(stdout.contains("debuggers=1"));
}

#[test]
fn inspect_reports_phase_six_var_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_var.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vars=3"));
}

#[test]
fn inspect_reports_phase_seven_function_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_functions.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("functions=2"));
    assert!(stdout.contains("calls=3"));
    assert!(stdout.contains("returns=2"));
}

#[test]
fn inspect_reports_phase_eight_object_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_objects.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("objects=1"));
    assert!(stdout.contains("arrays=1"));
    assert!(stdout.contains("property_reads=1"));
    assert!(stdout.contains("property_writes=1"));
}

#[test]
fn inspect_reports_phase_ten_callable_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_callables.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("function_values="));
    assert!(stdout.contains("indirect_calls="));
    assert!(stdout.contains("method_calls="));
    assert!(stdout.contains("this_reads="));
}

#[test]
fn inspect_reports_phase_eleven_closure_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_closures.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nested_functions=1"));
    assert!(stdout.contains("function_exprs=2"));
    assert!(stdout.contains("closures="));
    assert!(stdout.contains("captures="));
}

#[test]
fn inspect_reports_phase_fourteen_param_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_params.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("default_params=4"));
    assert!(stdout.contains("rest_params=2"));
    assert!(stdout.contains("arguments_uses=1"));
    assert!(stdout.contains("lexical_arguments_captures=1"));
}

#[test]
fn inspect_reports_phase_sixteen_coercion_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_coercions.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("loose_equalities=1"));
    assert!(stdout.contains("coercive_numeric_ops=1"));
    assert!(stdout.contains("coercive_relational_ops=1"));
    assert!(stdout.contains("void_uses=1"));
    assert!(stdout.contains("comma_ops=1"));
}

#[test]
fn inspect_reports_phase_eighteen_global_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_globals.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("global_bindings=64"));
    assert!(stdout.contains("global_this_uses=4"));
    assert!(stdout.contains("top_level_this_uses=1"));
    assert!(stdout.contains("global_default_this_calls=2"));
}

#[test]
fn inspect_reports_phase_twenty_one_constructor_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_constructors.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("constructs="));
    assert!(stdout.contains("instanceofs="));
    assert!(stdout.contains("prototype_reads="));
    assert!(stdout.contains("prototype_writes="));
}

#[test]
fn inspect_reports_phase_twenty_three_exception_ir_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("inspect")
        .arg(fixture_path("wasm_exceptions.js"))
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("throws=1"));
    assert!(stdout.contains("try_catches=2"));
}

#[test]
fn build_wasm_succeeds_for_supported_fixture() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_var.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_function_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_functions.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_object_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_objects.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_callable_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_callables.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_closure_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_closures.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_param_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_params.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_coercion_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_coercions.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_global_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_globals.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_host_output_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("hello.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_constructor_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_constructors.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn build_wasm_succeeds_for_supported_exception_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("build")
        .arg("wasm")
        .arg(fixture_path("wasm_exceptions.js"))
        .output()
        .expect("build command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("built Wasm artifact"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_fixture() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_var.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(6"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_function_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_functions.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(3"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_object_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_objects.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(2"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_callable_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_callables.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(18"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_closure_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_closures.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(5"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_param_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_params.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(2"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_coercion_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_coercions.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(2"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_global_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_globals.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true"));
}

#[test]
fn run_wasm_backend_succeeds_for_host_output_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("hello.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("grug"));
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("undefined"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_constructor_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_constructors.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(19"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_exception_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_exceptions.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("string(ReferenceError)"));
}

#[test]
fn run_wasm_backend_handles_many_choice_free_regexp_captures() {
    let root = unique_project_dir("wasm-regexp-many-captures");
    let capture_count = 500;
    let source = format!(
        "var match = /{}/.exec(\"{}\"); if (match.length !== {} || match[1] !== \"a\" || match[{}] !== \"a\") throw \"capture endpoints\"; true;",
        "(a)".repeat(capture_count),
        "a".repeat(capture_count),
        capture_count + 1,
        capture_count,
    );
    write_project_file(&root, "many-captures.js", &source);
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(root.join("many-captures.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_handles_many_one_shot_regexp_choices() {
    let root = unique_project_dir("wasm-regexp-many-one-shot-choices");
    let atom_count = 2_000;
    let source = format!(
        "var match = /{}/.exec(\"{}\"); if (match === null || match[0].length !== {}) throw \"one-shot choice match\"; true;",
        "a?".repeat(atom_count),
        "a".repeat(100_000),
        atom_count,
    );
    write_project_file(&root, "many-one-shot-choices.js", &source);
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(root.join("many-one-shot-choices.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("boolean(true)"));
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
    assert!(stdout.contains("count: 190"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_run_writes_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("run")
        .arg("--execution-backend")
        .arg("spec-exec")
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
    assert!(stdout.contains("total: 190"));
    assert!(stdout.contains("passed: 190"));
    assert!(stdout.contains("Unsupported: 0"));
}

#[test]
fn test262_run_exits_unsuccessfully_when_a_case_fails() {
    let suite_root = unique_project_dir("test262-failing-run");
    let test_dir = suite_root.join("test/language/fail");
    fs::create_dir_all(&test_dir).expect("failing test262 directory should be created");
    fs::write(
        test_dir.join("throws.js"),
        "/*---\nflags: [raw]\n---*/\nthrow new Error('intentional failure');\n",
    )
    .expect("failing test262 case should write");

    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("run")
        .arg("--suite-root")
        .arg(&suite_root)
        .arg("--snapshot-dir")
        .arg(unique_snapshot_dir("failing-run"))
        .arg("--snapshot-name")
        .arg("cli-failing-run")
        .output()
        .expect("failing test262 run should complete");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("total: 1"));
    assert!(stdout.contains("passed: 0"));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("test262 run failed: 1 of 1 cases did not pass"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_report_groups_failures_by_bucket() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("report")
        .arg("--execution-backend")
        .arg("spec-exec")
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
    assert!(stdout.contains("passed: 190"));
    assert!(stdout.contains("failed: 0"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_report_all_aggregates_fixture_suite() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("report-all")
        .arg("--execution-backend")
        .arg("spec-exec")
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
    assert!(stdout.contains("total: 190"));
    assert!(stdout.contains("passed: 190"));
    assert!(stdout.contains("targets:"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_backlog_and_snapshot_compare_read_completed_matrix_snapshots() {
    let snapshot_dir = unique_snapshot_dir("backlog-compare");
    let suite_root = copied_suite_root("backlog-compare");
    let base_snapshot_name = "cli-backlog-compare-base";
    let candidate_snapshot_name = "cli-backlog-compare-candidate";
    for snapshot_name in [base_snapshot_name, candidate_snapshot_name] {
        let output = Command::new(env!("CARGO_BIN_EXE_porf"))
            .arg("test262")
            .arg("report-all")
            .arg("--execution-backend")
            .arg("spec-exec")
            .arg("--suite-root")
            .arg(&suite_root)
            .arg("--snapshot-dir")
            .arg(&snapshot_dir)
            .arg("--snapshot-name")
            .arg(snapshot_name)
            .output()
            .expect("test262 report-all should run");

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let backlog = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("generate-backlog")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(&suite_root)
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(candidate_snapshot_name)
        .output()
        .expect("test262 generate-backlog should run");

    assert!(
        backlog.status.success(),
        "{}",
        String::from_utf8_lossy(&backlog.stderr)
    );
    let stdout = String::from_utf8_lossy(&backlog.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("total: 190"));
    assert!(stdout.contains("records: 0"));
    assert!(stdout.contains("backlog_json:"));
    assert!(stdout.contains("backlog_txt:"));

    let compare = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("compare-snapshots")
        .arg(base_snapshot_name)
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(&suite_root)
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(candidate_snapshot_name)
        .output()
        .expect("test262 compare-snapshots should run");

    assert!(
        compare.status.success(),
        "{}",
        String::from_utf8_lossy(&compare.stderr)
    );
    let stdout = String::from_utf8_lossy(&compare.stdout);
    assert!(stdout.contains("base_snapshot: cli-backlog-compare-base"));
    assert!(stdout.contains("candidate_snapshot: cli-backlog-compare-candidate"));
    assert!(stdout.contains("added_passes: 0"));
    assert!(stdout.contains("regressions: 0"));
    assert!(stdout.contains("changed_failure_hashes: 0"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_publish_status_updates_readme_and_writes_artifacts() {
    let readme_path = temp_readme_path("publish-status-spec");
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("publish-status")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-publish-status")
        .arg("--readme-path")
        .arg(&readme_path)
        .output()
        .expect("test262 publish-status should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("total: 190"));
    assert!(stdout.contains("passed: 190"));
    assert!(stdout.contains("manifest_hash:"));
    assert!(stdout.contains("snapshot_json:"));
    assert!(stdout.contains("status_json:"));
    assert!(stdout.contains("outcome_Success: 190"));
    assert!(stdout.contains("outcome_NotImplemented: 0"));
    assert!(stdout.contains("readme_path:"));

    let readme = std::fs::read_to_string(&readme_path).expect("updated readme should read");
    assert!(readme.contains("Fake wasm-safe Test262 subset: `187/187` green"));
    assert!(readme.contains("Fake full Rust rewrite suite: `190/190` green"));
    assert!(readme.contains("Pinned real Test262 baseline (`spec-exec`"));
    assert!(readme.contains("Pinned revisions: `ecma262=ecma262-current-draft`"));
    assert!(readme
        .contains("Current real outcomes: `Success=190`, `NotImplemented=0`, `Crash=0`, `Bug=0`"));
    assert!(readme.contains(
        "Real Test262 goal: Success=190/190 (100.0%); burn down NotImplemented=0, Crash=0, Bug=0 to zero"
    ));
    assert!(readme.contains("Published status artifacts: `"));
    assert!(
        readme.contains("./scripts/publish-real-status-low-ram.sh spec-exec codex-published-real")
    );
    assert!(readme.contains("## Design"));

    let status_json_line = stdout
        .lines()
        .find(|line| line.starts_with("status_json: "))
        .expect("stdout should include status_json path");
    let status_json_path = status_json_line
        .strip_prefix("status_json: ")
        .expect("status_json line should have prefix");
    let status_json = std::fs::read_to_string(status_json_path).expect("status json should read");
    assert!(status_json.contains("\"counts_per_outcome\""));
    assert!(status_json.contains("\"label\": \"Success\""));
    let status: serde_json::Value =
        serde_json::from_str(&status_json).expect("status json should parse");
    assert_eq!(
        status["real_suite"]["goal"]["name"],
        "Full pinned Test262 green"
    );
    assert_eq!(
        status["real_suite"]["goal"]["denominator"],
        "pinned-suite-total"
    );
    assert_eq!(status["real_suite"]["goal"]["target_total"], 190);
    assert_eq!(status["real_suite"]["goal"]["current_success"], 190);
    assert_eq!(status["real_suite"]["goal"]["remaining_to_green"], 0);
    assert_eq!(status["real_suite"]["goal"]["pass_rate"], "100.0%");

    let status_txt_line = stdout
        .lines()
        .find(|line| line.starts_with("status_txt: "))
        .expect("stdout should include status_txt path");
    let status_txt_path = status_txt_line
        .strip_prefix("status_txt: ")
        .expect("status_txt line should have prefix");
    let status_txt = std::fs::read_to_string(status_txt_path).expect("status txt should read");
    assert!(status_txt.contains("goal=Full pinned Test262 green"));
    assert!(status_txt.contains("progress=190/190"));
    assert!(status_txt.contains("remaining_to_green=0"));
    assert!(status_txt.contains("burn_down: NotImplemented=0 Crash=0 Bug=0"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_publish_status_is_stable_on_second_run() {
    let readme_path = temp_readme_path("publish-status-stable");
    let snapshot_dir = snapshot_dir();
    let command = || {
        Command::new(env!("CARGO_BIN_EXE_porf"))
            .arg("test262")
            .arg("publish-status")
            .arg("--execution-backend")
            .arg("spec-exec")
            .arg("--suite-root")
            .arg(suite_root())
            .arg("--snapshot-dir")
            .arg(&snapshot_dir)
            .arg("--snapshot-name")
            .arg("cli-publish-status-stable")
            .arg("--readme-path")
            .arg(&readme_path)
            .output()
            .expect("publish-status should run")
    };

    let first = command();
    assert!(first.status.success());
    let after_first = std::fs::read_to_string(&readme_path).expect("first readme should read");

    let second = command();
    assert!(second.status.success());
    let after_second = std::fs::read_to_string(&readme_path).expect("second readme should read");

    assert_eq!(after_first, after_second);
}

#[test]
fn test262_publish_status_supports_wasm_backend() {
    let readme_path = temp_readme_path("publish-status-wasm");
    let suite_root = tiny_wasm_suite_root("publish-status-wasm");
    let snapshot_dir = unique_snapshot_dir("publish-status-wasm");
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("publish-status")
        .arg("--suite-root")
        .arg(&suite_root)
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg("cli-publish-status-wasm")
        .arg("--execution-backend")
        .arg("wasm")
        .arg("--readme-path")
        .arg(&readme_path)
        .output()
        .expect("test262 wasm publish-status should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: wasm-aot"));
    assert!(stdout.contains("total: 1"));
    assert!(stdout.contains("passed: 1"));
    assert!(stdout.contains("outcome_Success: 1"));
    assert!(stdout.contains("status_json:"));
    assert!(stdout.contains("status_txt:"));
    assert!(stdout.contains("snapshot_json:"));
    assert!(stdout.contains("snapshot_txt:"));

    let readme = std::fs::read_to_string(&readme_path).expect("wasm readme should read");
    assert!(readme.contains("Pinned real Test262 baseline (`wasm-aot`"));
    assert!(readme.contains("): `1/1` green"));
    assert!(readme.contains("Current real outcomes:"));
    assert!(
        readme.contains("./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real")
    );

    let status_json_line = stdout
        .lines()
        .find(|line| line.starts_with("status_json: "))
        .expect("stdout should include status_json path");
    let status_json_path = status_json_line
        .strip_prefix("status_json: ")
        .expect("status_json line should have prefix");
    assert!(std::path::Path::new(status_json_path).exists());

    let status_txt_line = stdout
        .lines()
        .find(|line| line.starts_with("status_txt: "))
        .expect("stdout should include status_txt path");
    let status_txt_path = status_txt_line
        .strip_prefix("status_txt: ")
        .expect("status_txt line should have prefix");
    assert!(std::path::Path::new(status_txt_path).exists());

    let status_json = std::fs::read_to_string(status_json_path).expect("status json should read");
    let status: serde_json::Value =
        serde_json::from_str(&status_json).expect("status json should parse");
    assert_eq!(status["real_suite"]["backend"], "wasm-aot");
    assert_eq!(status["real_suite"]["total"], 1);
    assert_eq!(status["real_suite"]["passed"], 1);
}

#[test]
fn test262_publish_status_rejects_max_matrix_nodes() {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("publish-status")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(snapshot_dir())
        .arg("--snapshot-name")
        .arg("cli-publish-status-reject-limit")
        .arg("--max-matrix-nodes")
        .arg("1")
        .output()
        .expect("test262 publish-status should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("publish-status does not allow --max-matrix-nodes"));
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_progress_status_reports_incomplete_aggregate_without_publishing() {
    let snapshot_dir = std::env::temp_dir()
        .join(format!(
            "porffor-cli-test262-progress-status-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
        .display()
        .to_string();
    let snapshot_name = "cli-progress-status";
    let seed = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("report-all")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(snapshot_name)
        .arg("--max-matrix-nodes")
        .arg("1")
        .output()
        .expect("partial report-all should run");

    assert!(seed.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("progress-status")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(snapshot_name)
        .output()
        .expect("test262 progress-status should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("complete=false"));
    assert!(stdout.contains("matrix_nodes_completed: 1"));
    assert!(stdout.contains("matrix_nodes_total:"));
    assert!(stdout.contains("observed_total:"));
    assert!(stdout.contains("target_total: 190"));
    assert!(stdout.contains("unobserved_total:"));
    assert!(stdout.contains("current_success:"));
    assert!(stdout.contains("current_success_full:"));
    assert!(stdout.contains("remaining_observed_failures:"));
    assert!(stdout.contains("remaining_to_green:"));
    assert!(stdout.contains("outcomes:"));
    assert!(stdout.contains("  Success:"));
    assert!(stdout.contains("  NotImplemented:"));
    assert!(stdout.contains("  Crash:"));
    assert!(stdout.contains("  Bug:"));
    assert!(stdout.contains("burn_down: NotImplemented="));
    assert!(stdout.contains("not_run:"));

    assert!(!std::path::Path::new(&snapshot_dir)
        .join("published-status-spec-exec.json")
        .exists());
    assert!(!std::path::Path::new(&snapshot_dir)
        .join("published-status-spec-exec.txt")
        .exists());
}

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn test262_triage_and_failure_details_read_completed_matrix_snapshots() {
    let snapshot_dir = std::env::temp_dir()
        .join(format!(
            "porffor-cli-test262-triage-status-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
        .display()
        .to_string();
    let snapshot_name = "cli-triage-status";
    let seed = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("report-all")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(snapshot_name)
        .output()
        .expect("report-all should run");

    assert!(seed.status.success());

    let triage = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("triage-status")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(snapshot_name)
        .output()
        .expect("triage-status should run");

    assert!(triage.status.success());
    let stdout = String::from_utf8_lossy(&triage.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("failing_nodes:"));
    assert!(stdout.contains("ranking: Crash,Bug,NotImplemented,failed"));

    let details = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("test262")
        .arg("failure-details")
        .arg("language/wasm")
        .arg("--execution-backend")
        .arg("spec-exec")
        .arg("--suite-root")
        .arg(suite_root())
        .arg("--snapshot-dir")
        .arg(&snapshot_dir)
        .arg("--snapshot-name")
        .arg(snapshot_name)
        .output()
        .expect("failure-details should run");

    assert!(details.status.success());
    let stdout = String::from_utf8_lossy(&details.stdout);
    assert!(stdout.contains("execution_backend: spec-exec"));
    assert!(stdout.contains("node_id: language/wasm"));
    assert!(stdout.contains("filter: language/wasm"));
    assert!(stdout.contains("detail_groups: 0"));
}

#[test]
fn test262_wasm_backend_runs_supported_fixture_subset() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_porf"))
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
        .output()
        .expect("test262 wasm run should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execution_backend: wasm-aot"));
    assert!(stdout.contains("total: 187"));
    assert!(stdout.contains("passed: 187"));
}
