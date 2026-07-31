//! Byte-identity golden capture for the Wasm backend.
//!
//! The T02 module split (descriptor tables, `intrinsics/` extraction, flattening
//! the `compile_standard_builtin` dispatch) is a sequence of pure refactors. The
//! existing suites are a weak proof for that class of change: they are output
//! string assertions, so a refactor that perturbs emission order, function index
//! assignment, or property installation order can leave every one of them green
//! while changing the emitted module.
//!
//! This capture closes that gap. It walks the CLI fixture corpus, runs the real
//! `parse -> lower -> emit` pipeline over each file, and records the emitted byte
//! length and a content hash per fixture, plus the backend's own `debug_dump`.
//! Run it before and after a refactor and `diff -r` the two output directories:
//! an empty diff is proof of byte identity, and a non-empty one localises the
//! divergence to a named fixture and then, through the dump, to a named builtin.
//!
//! It is inert unless `PORFFOR_GOLDEN_OUT` names a directory, so it costs
//! nothing in an ordinary `cargo test` run.
//!
//! ```sh
//! git stash
//! PORFFOR_GOLDEN_OUT=$PWD/target/golden/before cargo test -p porffor-aot-wasm --test emit_golden
//! git stash pop
//! PORFFOR_GOLDEN_OUT=$PWD/target/golden/after  cargo test -p porffor-aot-wasm --test emit_golden
//! diff -r target/golden/before target/golden/after   # must be empty
//! ```
//!
//! Write captures under `target/` (gitignored), not `/tmp`. A capture costs ten
//! minutes and is worthless without the other side to compare against; a `/tmp`
//! cleaner removing the baseline mid-session means paying for it twice.
//!
//! Fixtures that fail to parse, lower or emit are recorded as such rather than
//! skipped. A refactor that turns an emitting fixture into a failing one — or the
//! reverse — is exactly the kind of regression this exists to catch, so the
//! failure text is part of the golden record.

use std::fs;
use std::path::{Path, PathBuf};

use porffor_aot_wasm::emit;
use porffor_front::{parse, ParseOptions};
use porffor_ir::lower;

const OUTPUT_DIR_ENV: &str = "PORFFOR_GOLDEN_OUT";

/// Matches the worker stack the engine and Test262 runner already use.
const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

/// FNV-1a, 64-bit. Hand-rolled to keep this test dependency-free — the crate has
/// no hashing dependency and this needs no cryptographic property, only a stable
/// content fingerprint that is identical across runs and machines.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../porffor-cli/tests/fixtures")
        .canonicalize()
        .expect("CLI fixture corpus should resolve")
}

/// Sorted so the manifest ordering is stable regardless of directory iteration
/// order, which is not guaranteed across filesystems.
fn fixture_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(root)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
        .map(|entry| entry.expect("fixture directory entry should read").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "js"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

/// One line of the manifest. Either the emitted size and hash, or the stage that
/// refused the fixture — both are part of the golden record.
fn capture(source_text: &str) -> (String, Option<String>) {
    let unit = match parse(source_text, ParseOptions::script()) {
        Ok(unit) => unit,
        Err(err) => return (format!("parse-error {err}"), None),
    };
    let program = lower(&unit);
    match emit(&program) {
        Ok(artifact) => (
            format!(
                "bytes={} hash={:016x}",
                artifact.bytes.len(),
                fnv1a(&artifact.bytes)
            ),
            Some(artifact.debug_dump),
        ),
        Err(err) => (format!("emit-error {err}"), None),
    }
}

#[test]
fn capture_emit_golden() {
    let Some(output_root) = std::env::var_os(OUTPUT_DIR_ENV) else {
        eprintln!("{OUTPUT_DIR_ENV} unset, skipping golden capture");
        return;
    };
    let output_root = PathBuf::from(output_root);
    let dumps = output_root.join("dumps");
    fs::create_dir_all(&dumps)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", dumps.display()));

    let root = fixture_dir();
    let paths = fixture_paths(&root);
    assert!(
        !paths.is_empty(),
        "no .js fixtures found under {}",
        root.display()
    );

    // Each fixture emits a full bootstrap module, measured at ~12 s apiece, so a
    // serial walk of the corpus takes over an hour — too slow to run twice around
    // every refactor step. The fixtures are independent, so fan them across the
    // machine. `thread::scope` keeps the borrows of `paths` safe without an
    // owned-data dance and without a threadpool dependency.
    let worker_count = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(4)
        .min(paths.len());

    let lines = std::thread::scope(|scope| {
        let workers = (0..worker_count)
            .map(|worker| {
                let paths = &paths;
                let dumps = &dumps;
                // Lowering and emission recurse over the AST deeply enough to
                // blow the 2 MiB default thread stack on real fixtures. The
                // engine and the Test262 runner both give their workers large
                // stacks for the same reason; match that here rather than
                // relying on the caller to set RUST_MIN_STACK.
                std::thread::Builder::new()
                    .stack_size(WORKER_STACK_BYTES)
                    .spawn_scoped(scope, move || {
                        // Stride rather than chunk: fixtures are sorted by name and
                        // cost correlates with name (whole families are expensive),
                        // so contiguous chunks would leave workers badly unbalanced.
                        paths
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| index % worker_count == worker)
                            .map(|(index, path)| {
                                let name = path
                                    .file_name()
                                    .expect("fixture path should have a file name")
                                    .to_string_lossy()
                                    .into_owned();
                                let source_text = fs::read_to_string(path).unwrap_or_else(|err| {
                                    panic!("failed to read {}: {err}", path.display())
                                });

                                let (summary, debug_dump) = capture(&source_text);
                                if let Some(dump) = debug_dump {
                                    fs::write(dumps.join(format!("{name}.dump")), dump)
                                        .unwrap_or_else(|err| {
                                            panic!("failed to write dump for {name}: {err}")
                                        });
                                }
                                (index, format!("{name} {summary}\n"))
                            })
                            .collect::<Vec<_>>()
                    })
                    .expect("golden capture worker should spawn")
            })
            .collect::<Vec<_>>();

        let mut lines = workers
            .into_iter()
            .flat_map(|worker| {
                worker
                    .join()
                    .expect("golden capture worker should not panic")
            })
            .collect::<Vec<_>>();
        // Restore fixture order so the manifest is byte-comparable across runs
        // regardless of how work happened to be distributed.
        lines.sort_by_key(|(index, _)| *index);
        lines
    });

    let manifest = lines.into_iter().map(|(_, line)| line).collect::<String>();

    let manifest_path = output_root.join("manifest.txt");
    fs::write(&manifest_path, &manifest)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", manifest_path.display()));
    eprintln!(
        "golden capture: {} fixtures -> {}",
        paths.len(),
        output_root.display()
    );
}
