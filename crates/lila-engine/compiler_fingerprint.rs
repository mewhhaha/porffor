//! Inputs that can change the emitted program-Wasm cache entry.
//!
//! A Cargo.lock checksum does not cover an edited path dependency or a
//! [patch.crates-io] source. Keep those bytes in the fingerprint too. Include
//! non-Rust resources: include_str!/include_bytes! are compiler inputs, not
//! merely documentation. Cache invalidation is deliberately conservative.

use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) const FINGERPRINT_SCHEME: &str = "lila-program-cache-compiler-v3";
pub(crate) const NATIVE_FINGERPRINT_SCHEME: &str = "lila-native-compiler-v1";

// Native codegen dependencies have a separate identity from Lila's complete
// program compiler. Cargo.lock binds registry dependencies; all vendored source
// binds patched dependencies. Feature/profile manifests also invalidate it.
// This does not replace or narrow COMPILER_INPUTS or raw R admission.
pub(crate) const NATIVE_COMPILER_INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "vendor",
    "crates/lila-engine/Cargo.toml",
    "crates/lila-runtime-image/Cargo.toml",
];

pub(crate) const COMPILER_INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "crates/lila-front",
    "crates/lila-ir",
    "crates/lila-aot-wasm",
    "crates/lila-runtime",
    "crates/lila-runtime-image", // Cargo producer and immutable image accessors.
    "crates/lila-intl",
    "crates/lila-engine/Cargo.toml",
    "crates/lila-engine/src",
    "crates/lila-engine/build.rs",
    "crates/lila-engine/compiler_fingerprint.rs",
    "vendor",
];

// These exact project paths are outside the product dependency closure: Cargo
// integration tests, the IR's cfg(test) module tree, and explicit cfg(test)-only
// Engine modules. Do not generalize this to every path named "tests": vendored
// sources and production resources retain conservative coverage.
const TEST_ONLY_INPUTS: &[&str] = &[
    "crates/lila-front/tests",
    "crates/lila-ir/tests",
    "crates/lila-ir/src/tests",
    "crates/lila-aot-wasm/tests",
    "crates/lila-runtime/tests",
    "crates/lila-intl/tests",
    "crates/lila-engine/src/embedded_runtime/compile_tests.rs",
    "crates/lila-engine/src/embedded_runtime/tests.rs",
    "crates/lila-engine/src/regexp_program_boundary_tests.rs",
    "crates/lila-engine/src/runtime_regexp_compiler_tests.rs",
    "crates/lila-engine/src/array_present_index_tests.rs",
    "crates/lila-engine/src/module_async_runtime_tests.rs",
    "crates/lila-engine/src/native_function_compiler_tests.rs",
    "crates/lila-engine/src/native_function_compiler_tests",
    "crates/lila-engine/src/wasm_native_function_compiler/observer.rs",
    "crates/lila-engine/src/wasm_native_function_compiler/tests.rs",
];

pub(crate) fn fingerprint(workspace: &Path) -> io::Result<String> {
    let files = input_files(workspace, COMPILER_INPUTS, TEST_ONLY_INPUTS)?;
    fingerprint_files(workspace, FINGERPRINT_SCHEME, files)
}

// Cargo recursively watches directory rerun inputs. Descend mixed source/test
// roots instead of watching their ancestor; keep wholly production subtrees
// recursive so adding or removing data files still triggers fingerprinting.
// A new module in a mixed Rust root also edits a watched module declaration.
pub(crate) fn compiler_rerun_inputs(workspace: &Path) -> io::Result<Vec<PathBuf>> {
    let mut inputs = Vec::new();
    for input in COMPILER_INPUTS {
        collect_rerun_inputs(workspace, &workspace.join(input), &mut inputs)?;
    }
    inputs.sort();
    inputs.dedup();
    Ok(inputs)
}

pub(crate) fn native_fingerprint(workspace: &Path) -> io::Result<String> {
    let files = input_files(workspace, NATIVE_COMPILER_INPUTS, &[])?;
    fingerprint_files(workspace, NATIVE_FINGERPRINT_SCHEME, files)
}

fn input_files(workspace: &Path, inputs: &[&str], excluded: &[&str]) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for input in inputs {
        collect_files(workspace, &workspace.join(input), excluded, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn fingerprint_files(workspace: &Path, scheme: &str, files: Vec<PathBuf>) -> io::Result<String> {
    let mut digest = Sha256::new();
    digest.update(scheme.as_bytes());
    for path in files {
        let relative = path.strip_prefix(workspace).map_err(io::Error::other)?;
        let label = relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let bytes = fs::read(&path)?;
        digest.update((label.len() as u64).to_le_bytes());
        digest.update(label.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn excluded_input(workspace: &Path, path: &Path, excluded: &[&str]) -> bool {
    excluded
        .iter()
        .any(|input| path.starts_with(workspace.join(input)))
}

fn collect_files(
    workspace: &Path,
    path: &Path,
    excluded: &[&str],
    output: &mut Vec<PathBuf>,
) -> io::Result<()> {
    if excluded_input(workspace, path, excluded) {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() {
        output.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "compiler fingerprint input is not a regular file or directory: {}",
                path.display()
            ),
        ));
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        // A local rebuild of a vendored/path crate must not hash its own
        // output recursively. Git administrative state is not source input.
        if entry.file_type()?.is_dir()
            && (entry.file_name() == "target" || entry.file_name() == ".git")
        {
            continue;
        }
        collect_files(workspace, &entry.path(), excluded, output)?;
    }
    Ok(())
}

fn collect_rerun_inputs(
    workspace: &Path,
    path: &Path,
    output: &mut Vec<PathBuf>,
) -> io::Result<()> {
    if excluded_input(workspace, path, TEST_ONLY_INPUTS) {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file()
        || (metadata.is_dir()
            && !TEST_ONLY_INPUTS
                .iter()
                .any(|input| workspace.join(input).starts_with(path)))
    {
        output.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "compiler rerun input is not a regular file or directory: {}",
                path.display()
            ),
        ));
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && (entry.file_name() == "target" || entry.file_name() == ".git")
        {
            continue;
        }
        collect_rerun_inputs(workspace, &entry.path(), output)?;
    }
    Ok(())
}
