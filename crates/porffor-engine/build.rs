use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let compiler_inputs = [
        manifest.join("../../Cargo.lock"),
        manifest.join("Cargo.toml"),
        manifest.join("src"),
        manifest.join("../porffor-front/Cargo.toml"),
        manifest.join("../porffor-front/src"),
        manifest.join("../porffor-ir/Cargo.toml"),
        manifest.join("../porffor-ir/src"),
        manifest.join("../porffor-aot-wasm/Cargo.toml"),
        manifest.join("../porffor-aot-wasm/src"),
    ];
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
    digest.update(b"porffor-program-cache-compiler-v1");
    for path in files {
        digest.update(path.to_string_lossy().as_bytes());
        match fs::read(&path) {
            Ok(bytes) => digest.update(bytes),
            Err(err) => panic!("failed to fingerprint {}: {err}", path.display()),
        }
    }
    println!(
        "cargo:rustc-env=PORFFOR_COMPILER_FINGERPRINT={:x}",
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
