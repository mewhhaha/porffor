//! Exercise the production disk-cache provider in separate Engine processes.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, ObservedNumber,
    RealmBuilder, RunOptions,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime};

const ROLE: &str = "LILA_RUNTIME_CACHE_PROCESS_TEST_ROLE";
const TEST: &str = "raw_runtime_artifacts_are_reused_by_fresh_engine_processes";
const NATIVE_TEST: &str = "native_runtime_is_reused_by_executables_with_different_mtimes";

struct CacheDirectory(PathBuf);

impl Drop for CacheDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn child(executable: &Path, test: &str, role: &str, directory: &Path) -> Output {
    let output = Command::new(executable)
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(ROLE, role)
        .env("LILA_CACHE_DIR", directory)
        .env("LILA_WASM_TRACE", "1")
        .output()
        .expect("fresh Engine test process starts");
    assert!(
        output.status.success(),
        "{role} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn result(output: &Output) -> (String, String) {
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let line = stdout
        .lines()
        .find_map(|line| {
            line.split_once("runtime-cache-fixture ")
                .map(|(_, result)| result)
        })
        .expect("child reports its actual emitted artifact digests");
    let (runtime, program) = line.split_once(' ').unwrap();
    (runtime.to_owned(), program.to_owned())
}

#[test]
fn raw_runtime_artifacts_are_reused_by_fresh_engine_processes() {
    if let Ok(role) = std::env::var(ROLE) {
        let engine = Engine::new(RealmBuilder::new().build());
        let source = match role.as_str() {
            "producer" => "const values = ['producer']; values.length;",
            "consumer" => "const values = ['consumer', 'different source']; values.length;",
            _ => panic!("unknown test role"),
        };
        let unit = engine
            .compile_script(source, CompileOptions::default())
            .unwrap();
        let artifact = engine.emit_wasm(&unit).expect("program and runtime emit");
        let runtime = artifact.runtime.as_ref().expect("heap program has R");
        println!(
            "runtime-cache-fixture {:x} {:x}",
            Sha256::digest(runtime.bytes()),
            Sha256::digest(&artifact.bytes)
        );
        return;
    }

    let directory = CacheDirectory(std::env::temp_dir().join(format!(
        "lila-runtime-process-cache-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    fs::create_dir(&directory.0).unwrap();
    let executable = std::env::current_exe().unwrap();
    let producer = child(&executable, TEST, "producer", &directory.0);
    let consumer = child(&executable, TEST, "consumer", &directory.0);
    assert!(String::from_utf8_lossy(&producer.stderr).contains("runtime-cache miss:"));
    let consumer_trace = String::from_utf8_lossy(&consumer.stderr);
    assert!(
        consumer_trace.contains("runtime-cache hit:"),
        "{consumer_trace}"
    );
    assert!(
        !consumer_trace.contains("runtime-cache miss:"),
        "{consumer_trace}"
    );
    let (first_runtime, first_program) = result(&producer);
    let (second_runtime, second_program) = result(&consumer);
    assert_eq!(
        first_runtime, second_runtime,
        "fresh process reloads exact R bytes"
    );
    assert_ne!(
        first_program, second_program,
        "each process compiles its own source to P"
    );
}

fn executable_digest(path: &Path) -> [u8; 32] {
    let mut file = fs::File::open(path).expect("test executable is readable");
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).expect("test executable reads");
        if read == 0 {
            return hash.finalize().into();
        }
        hash.update(&buffer[..read]);
    }
}

fn assert_native_cache_counts(output: &Output, hits: usize, misses: usize) {
    let trace = String::from_utf8_lossy(&output.stderr);
    let counts: Vec<_> = trace
        .lines()
        .filter(|line| line.starts_with("lila wasm trace: module-cache "))
        .collect();
    assert_eq!(counts.len(), 1, "one native module-loading result: {trace}");
    assert!(
        counts[0].starts_with(&format!(
            "lila wasm trace: module-cache hits={hits} misses={misses} during "
        )),
        "expected native hits={hits} misses={misses}: {trace}"
    );
}

#[test]
fn native_runtime_is_reused_by_executables_with_different_mtimes() {
    if let Ok(role) = std::env::var(ROLE) {
        lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
        let engine = Engine::new(RealmBuilder::new().build());
        let (source, expected) = match role.as_str() {
            "producer" => ("const values = ['producer']; values.length;", 1.0),
            "consumer" => (
                "const values = ['consumer', 'different source']; values.length;",
                2.0,
            ),
            _ => panic!("unknown native test role"),
        };
        let unit = engine
            .compile_script(source, CompileOptions::default())
            .expect("native cache fixture compiles");
        let artifact = engine.emit_wasm(&unit).expect("program and runtime emit");
        let runtime = artifact.runtime.as_ref().expect("heap program has R");
        println!(
            "runtime-cache-fixture {:x} {:x}",
            Sha256::digest(runtime.bytes()),
            Sha256::digest(&artifact.bytes)
        );
        drop(artifact);
        drop(unit);

        let observed = engine
            .observe_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("native cache fixture executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(expected))),
            "{role}: {observed:?}"
        );
        assert!(observed.output_events.is_empty(), "{role}: {observed:?}");
        return;
    }

    let directory = CacheDirectory(std::env::temp_dir().join(format!(
        "lila-native-runtime-process-cache-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    fs::create_dir(&directory.0).unwrap();
    let cache = directory.0.join("cache");
    let executable = std::env::current_exe().unwrap();
    let original_digest = executable_digest(&executable);
    let producer_executable = directory
        .0
        .join(format!("producer{}", std::env::consts::EXE_SUFFIX));
    let consumer_executable = directory
        .0
        .join(format!("consumer{}", std::env::consts::EXE_SUFFIX));

    // Different binaries built from the same Wasmtime release must share R.
    // Identical executable bytes keep Lila's compiler identity unchanged; only
    // the mtime used by Wasmtime's erroneous Git-build namespace is different.
    for (copy, seconds) in [(&producer_executable, 10), (&consumer_executable, 20)] {
        fs::copy(&executable, copy).expect("copy the test executable");
        fs::File::options()
            .write(true)
            .open(copy)
            .expect("copied executable permits timestamp changes")
            .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
            .expect("set deliberately different executable mtimes");
        assert_eq!(
            executable_digest(copy),
            original_digest,
            "binary contents must match the original test executable"
        );
    }
    assert_ne!(
        producer_executable.metadata().unwrap().modified().unwrap(),
        consumer_executable.metadata().unwrap().modified().unwrap(),
        "the filesystem must retain distinct executable mtimes"
    );

    let producer = child(&producer_executable, NATIVE_TEST, "producer", &cache);
    let consumer = child(&consumer_executable, NATIVE_TEST, "consumer", &cache);
    let (first_runtime, first_program) = result(&producer);
    let (second_runtime, second_program) = result(&consumer);
    assert_eq!(
        first_runtime, second_runtime,
        "both programs use identical R"
    );
    assert_ne!(first_program, second_program, "each source has its own P");
    assert_native_cache_counts(&producer, 0, 2);
    assert_native_cache_counts(&consumer, 1, 1);
}
