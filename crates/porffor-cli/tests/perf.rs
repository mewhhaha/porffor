use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

const EXACT_LIMIT: Duration = Duration::from_secs(1);
const CHUNK_LIMIT: Duration = Duration::from_secs(5);
const COLD_LIMIT: Duration = Duration::from_secs(5);

fn fixture_path(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .display()
        .to_string()
}

fn run_fixture(name: &str) {
    let output = porffor_cli::run_cli_capture([
        "run".to_string(),
        "--execution-backend".to_string(),
        "wasm".to_string(),
        fixture_path(name),
    ]);
    assert_eq!(
        output.exit_code,
        0,
        "{name} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn chunk_cases() -> Vec<&'static str> {
    include_str!("../../../benchmarks/wasm-aot-20.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

#[test]
#[ignore = "performance acceptance benchmark; run explicitly on an idle machine"]
fn warm_exact_wasmtime_aot_is_sub_second() {
    run_fixture("wasm_functions.js");
    let started = Instant::now();
    run_fixture("wasm_functions.js");
    let elapsed = started.elapsed();
    eprintln!("warm exact Wasmtime-AOT: {elapsed:?}");
    assert!(elapsed <= EXACT_LIMIT, "warm exact took {elapsed:?}");
}

#[test]
#[ignore = "performance acceptance benchmark; warms twenty large fixtures first"]
fn warmed_twenty_case_chunk_is_under_five_seconds() {
    let cases = chunk_cases();
    assert_eq!(cases.len(), 20, "benchmark manifest must stay at 20 cases");
    for case in &cases {
        run_fixture(case);
    }
    let started = Instant::now();
    for case in &cases {
        run_fixture(case);
    }
    let elapsed = started.elapsed();
    eprintln!("warmed 20-case Wasmtime-AOT chunk: {elapsed:?}");
    assert!(elapsed <= CHUNK_LIMIT, "warmed chunk took {elapsed:?}");
}

#[test]
#[ignore = "performance acceptance benchmark; intentionally fails until the runtime/program split lands"]
fn cold_exact_after_cache_prune_is_under_five_seconds() {
    let prune = Command::new(env!("CARGO_BIN_EXE_porf"))
        .args(["cache", "prune"])
        .output()
        .expect("cache prune process should run");
    assert!(prune.status.success(), "{}", String::from_utf8_lossy(&prune.stderr));

    let started = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .args(["run", fixture_path("wasm_host_output.js").as_str()])
        .output()
        .expect("cold exact process should run");
    let elapsed = started.elapsed();
    eprintln!("cold exact Wasmtime-AOT after cache prune: {elapsed:?}");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(elapsed <= COLD_LIMIT, "cold exact took {elapsed:?}");
}
