use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest
        .join("../..")
        .canonicalize()
        .expect("Lila workspace root should resolve");
    let compiler_inputs = [
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
        workspace.join("crates/lila-front/Cargo.toml"),
        workspace.join("crates/lila-front/src"),
        workspace.join("crates/lila-ir/Cargo.toml"),
        workspace.join("crates/lila-ir/src"),
        workspace.join("crates/lila-aot-wasm/Cargo.toml"),
        workspace.join("crates/lila-aot-wasm/src"),
    ];
    println!("cargo:rerun-if-changed=build.rs");
    let mut files = Vec::new();
    for input in &compiler_inputs {
        println!("cargo:rerun-if-changed={}", input.display());
        collect_files(input, &mut files);
    }
    files.sort();
    for path in &files {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let mut digest = Sha256::new();
    digest.update(b"lila-program-cache-compiler-v2");
    for path in files {
        let relative = path.strip_prefix(&workspace).unwrap_or_else(|_| {
            panic!(
                "compiler input {} is outside workspace {}",
                path.display(),
                workspace.display()
            )
        });
        let label = relative
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let bytes = fs::read(&path)
            .unwrap_or_else(|err| panic!("failed to fingerprint {}: {err}", path.display()));
        digest.update((label.len() as u64).to_le_bytes());
        digest.update(label.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    println!(
        "cargo:rustc-env=LILA_COMPILER_FINGERPRINT={:x}",
        digest.finalize()
    );
}

fn collect_files(path: &Path, output: &mut Vec<PathBuf>) {
    if path.is_file() {
        output.push(path.to_path_buf());
        return;
    }
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        .map(|entry| {
            entry
                .expect("compiler input directory entry should read")
                .path()
        })
        .collect::<Vec<_>>();
    entries.sort();
    for entry in entries {
        if entry.is_dir() || entry.extension().is_some_and(|extension| extension == "rs") {
            collect_files(&entry, output);
        }
    }
}
