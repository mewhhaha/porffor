//! Execute the same ordinary JavaScript inputs used by the artifact validator.
//! Decodable/valid Wasm, a successful process, or an expected literal appearing
//! somewhere in output are not substitutes for an exact execution transcript.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[path = "../../lila-aot-wasm/tests/fixtures/product_programs.rs"]
mod product_programs;

const CASE_TIMEOUT: Duration = Duration::from_secs(180);
const WASM_COMPLETION: &str = "run outcome: RunOutcome { backend_used: WasmAot, note: \"wasm-aot completion: undefined(undefined)\" }\n";

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "lila-product-execution-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("product execution fixture directory should be created");
        Self(path)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn wait_bounded(child: &mut Child, name: &str) -> ExitStatus {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) if started.elapsed() < CASE_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(25));
            }
            result => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{name}: CLI did not finish within {CASE_TIMEOUT:?}: {result:?}");
            }
        }
    }
}

fn execution_matches(success: bool, stdout: &str, expected_stdout: &str) -> bool {
    success
        && !expected_stdout.is_empty()
        && stdout == format!("{expected_stdout}{WASM_COMPLETION}")
}

#[test]
fn representative_artifacts_execute_with_the_expected_transcripts() {
    assert!(!product_programs::CASES.is_empty(), "fixture inventory must not be empty");
    let directory = Directory::new();
    let mut names = BTreeSet::new();
    for fixture in product_programs::CASES {
        assert!(names.insert(fixture.name), "duplicate fixture {}", fixture.name);
        assert!(!fixture.source.is_empty(), "{}: source is empty", fixture.name);
        let source = directory.0.join(format!("{}.js", fixture.name));
        let stdout_path = directory.0.join(format!("{}.stdout", fixture.name));
        let stderr_path = directory.0.join(format!("{}.stderr", fixture.name));
        fs::write(&source, fixture.source).expect("fixture source should be written");
        let mut child = Command::new(env!("CARGO_BIN_EXE_lila"))
            .args([
                "run",
                "--execution-backend",
                "wasm",
                "--host-surface",
                "product",
                "--jobs",
                "1",
            ])
            .arg(&source)
            .env("LILA_CACHE_DIR", directory.0.join("cache"))
            .env("LILA_MODULE_MEMORY_CACHE_ENTRIES", "2")
            .stdin(Stdio::null())
            .stdout(fs::File::create(&stdout_path).expect("stdout file should be created"))
            .stderr(fs::File::create(&stderr_path).expect("stderr file should be created"))
            .spawn()
            .expect("the compiled product CLI should start");
        let status = wait_bounded(&mut child, fixture.name);
        let stdout = fs::read_to_string(stdout_path).expect("CLI stdout should be readable UTF-8");
        let stderr = fs::read_to_string(stderr_path).expect("CLI stderr should be readable UTF-8");
        assert!(
            execution_matches(status.success(), &stdout, fixture.expected_stdout),
            "{}: unexpected execution\nstatus: {status}\nexpected program output: {:?}\nactual stdout: {stdout:?}\nstderr: {stderr}",
            fixture.name,
            fixture.expected_stdout
        );
    }
    assert_eq!(names.len(), product_programs::CASES.len());
}

#[test]
fn execution_gate_rejects_wrong_results_extra_output_and_oracle_fallback() {
    let expected = "9\n";
    let valid = format!("{expected}{WASM_COMPLETION}");
    assert!(execution_matches(true, &valid, expected));
    for invalid in [
        format!("8\n{WASM_COMPLETION}"),
        format!("extra\n{valid}"),
        format!("{valid}extra\n"),
        expected.to_owned(),
        valid.replace("WasmAot", "SpecExec"),
        valid.replace("undefined(undefined)", "number(9)"),
    ] {
        assert!(!execution_matches(true, &invalid, expected), "accepted {invalid:?}");
    }
    assert!(!execution_matches(false, &valid, expected));
    assert!(!execution_matches(true, WASM_COMPLETION, ""));
}
