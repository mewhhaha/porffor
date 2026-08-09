//! Wall-clock acceptance benchmarks for the Wasmtime-AOT path.
//!
//! All three are `#[ignore]`d and all three are declared in
//! `tests/known-failures.tsv` with owner T25.
//! `known_failures::every_ignored_test_is_declared` scans this file, so an
//! ignore attribute added here without a ledger row fails the `cli` suite.
//!
//! They stay ignored on purpose. These are timing gates, not correctness
//! tests: a loaded or shared machine makes the measurement meaningless rather
//! than failing it honestly, and `chunk_cases` hard-panics unless the
//! machine-local `benchmarks/wasm-aot-20.txt` exists. The goal of the ledger
//! rows is declaration with an owner, not execution by default. Run them
//! deliberately:
//!
//! ```sh
//! cargo test -p porffor-cli --test perf -- --ignored --nocapture --test-threads=1
//! ```

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

fn chunk_cases() -> Vec<String> {
    // `benchmarks/wasm-aot-20.txt` is machine-local (gitignored via `*.txt`), so
    // it must be read at run time: `include_str!` made every fresh clone fail
    // `cargo check --all-targets` even though this test is `#[ignore]`d.
    let manifest =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/wasm-aot-20.txt");
    let contents = std::fs::read_to_string(&manifest).unwrap_or_else(|err| {
        panic!(
            "benchmark manifest {} is required to run this benchmark: {err}",
            manifest.display()
        )
    });
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

#[test]
#[ignore = "T25 performance acceptance benchmark; run explicitly on an idle machine"]
fn warm_exact_wasmtime_aot_is_sub_second() {
    run_fixture("wasm_functions.js");
    let started = Instant::now();
    run_fixture("wasm_functions.js");
    let elapsed = started.elapsed();
    eprintln!("warm exact Wasmtime-AOT: {elapsed:?}");
    assert!(elapsed <= EXACT_LIMIT, "warm exact took {elapsed:?}");
}

#[test]
#[ignore = "T25 performance acceptance benchmark; warms twenty large fixtures first"]
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
#[ignore = "T25 performance acceptance benchmark; intentionally fails until the runtime/program split lands"]
fn cold_exact_after_cache_prune_is_under_five_seconds() {
    let prune = Command::new(env!("CARGO_BIN_EXE_porf"))
        .args(["cache", "prune"])
        .output()
        .expect("cache prune process should run");
    assert!(
        prune.status.success(),
        "{}",
        String::from_utf8_lossy(&prune.stderr)
    );

    let started = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .args(["run", fixture_path("wasm_host_output.js").as_str()])
        .output()
        .expect("cold exact process should run");
    let elapsed = started.elapsed();
    eprintln!("cold exact Wasmtime-AOT after cache prune: {elapsed:?}");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(elapsed <= COLD_LIMIT, "cold exact took {elapsed:?}");
}
