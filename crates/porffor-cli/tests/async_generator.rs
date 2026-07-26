use std::path::PathBuf;

#[test]
fn wasm_backend_resumes_async_generator_loops_for_zero_one_and_many_iterations() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wasm_async_generator_resumable_loop.js")
        .display()
        .to_string();
    let output = porffor_cli::run_cli_capture([
        "run".to_string(),
        "--execution-backend".to_string(),
        "wasm".to_string(),
        fixture,
    ]);

    assert_eq!(
        output.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            "async-generator-resumable-loop:9:false:true:0:false:9:false:true:0:2:4:9:false:true:0:false:true:tdz:false:tdz:false:true:7:false:true:true"
        ),
        "{stdout}"
    );
}
