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

pub(crate) const COMPILER_INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "crates/lila-front",
    "crates/lila-ir",
    "crates/lila-aot-wasm",
    "crates/lila-runtime",
    "crates/lila-intl",
    "crates/lila-engine/Cargo.toml",
    "crates/lila-engine/src",
    "crates/lila-engine/build.rs",
    "crates/lila-engine/compiler_fingerprint.rs",
    "vendor",
];

pub(crate) fn fingerprint(workspace: &Path) -> io::Result<String> {
    let mut files = Vec::new();
    for input in COMPILER_INPUTS {
        collect_files(&workspace.join(input), &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut digest = Sha256::new();
    digest.update(b"lila-program-cache-compiler-v3");
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

fn collect_files(path: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
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
        collect_files(&entry.path(), output)?;
    }
    Ok(())
}
